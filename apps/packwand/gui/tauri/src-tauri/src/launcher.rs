//! In-process launcher adapter over the shared Rust core
//! (`packwand-devboot` -> `packwand-instance`/`packwand-launch`, and now
//! `packwand-msa` for real sign-in). No Go sidecar, no loopback HTTP to the
//! Go backend for this path — see `packwandrs.md`.
//!
//! "Boot a pack" is for dev testing, not real play, but it now uses a real
//! signed-in Microsoft account's session when one is available, falling
//! back to an offline session (see `packwand-auth`) otherwise. Either way,
//! the launch plan's game directory points straight at the pack's own
//! subdir so the mods/config/resourcepacks Minecraft loads are exactly
//! what's checked into the pack, live, with no copying.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use packwand_auth::Session;
use packwand_devboot::{boot_pack, InstallProgress};
use packwand_instance::{FsInstanceRepository, InstanceRepository, ListEntry};
use packwand_launch::{
    build_launch_plan, launch, CancellationToken, LaunchEvent, LaunchOptions, LaunchPlan,
};
use packwand_msa::{KeyringTokenStore, MsaConfig};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::open_with_os_handler;
use crate::window_dock;

/// Tracks cancellation tokens for in-flight boot sessions, keyed by session
/// id. A session only becomes cancellable once the game process has
/// actually started — the install/download phase has no cancellation hook
/// today (a known v1 limitation, not an oversight).
#[derive(Default)]
pub struct LauncherState {
    sessions: Mutex<HashMap<String, CancellationToken>>,
    next_id: AtomicU64,
}

impl LauncherState {
    pub fn new() -> Self {
        Self::default()
    }

    fn new_session_id(&self) -> String {
        format!("boot-{}", self.next_id.fetch_add(1, Ordering::SeqCst))
    }

    fn register(&self, session_id: String, token: CancellationToken) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(session_id, token);
        }
    }

    fn take(&self, session_id: &str) -> Option<CancellationToken> {
        self.sessions
            .lock()
            .ok()
            .and_then(|mut sessions| sessions.remove(session_id))
    }
}

/// The current Microsoft account session, if signed in, plus the OS
/// credential store for the refresh token. One active account at a time
/// (v1 scope, matching `packwand-msa`'s own `KeyringTokenStore` design).
pub struct AuthState {
    session: Mutex<Option<Session>>,
    store: KeyringTokenStore,
    /// Set after the first `auth_status` call attempts a silent refresh
    /// from a stored refresh token, so repeated status polls don't repeat
    /// the network round-trip.
    refresh_attempted: AtomicBool,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(None),
            store: KeyringTokenStore::new(),
            refresh_attempted: AtomicBool::new(false),
        }
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

/// Reads the Azure app client ID this build signs in as. Not configured
/// until a real Azure AD app registration exists (see `packwandrs.md`) —
/// callers must treat an unset/empty value as "sign-in isn't set up yet",
/// not attempt a request with an empty client ID.
fn msa_config() -> Option<MsaConfig> {
    let client_id = std::env::var("PACKWAND_MSA_CLIENT_ID").ok()?;
    if client_id.trim().is_empty() {
        return None;
    }
    Some(MsaConfig { client_id })
}

#[derive(Clone, Serialize)]
pub struct AuthStatus {
    signed_in: bool,
    /// Empty when not signed in — `signed_in` is the field that says so,
    /// keeping this always a plain string (no `null` for the frontend's
    /// JSON decoding to special-case).
    username: String,
}

/// Reports the current sign-in state. On its first call, also attempts a
/// silent refresh from a stored refresh token (no browser) so a returning
/// user doesn't have to sign in again every app start.
#[tauri::command]
pub fn auth_status(state: State<'_, AuthState>) -> Result<AuthStatus, String> {
    if !state.refresh_attempted.swap(true, Ordering::SeqCst) {
        if let Some(config) = msa_config() {
            if let Ok(Some(session)) = packwand_msa::refresh(&config, &state.store) {
                if let Ok(mut guard) = state.session.lock() {
                    *guard = Some(session);
                }
            }
        }
    }
    let guard = state
        .session
        .lock()
        .map_err(|_| "auth state poisoned".to_string())?;
    Ok(AuthStatus {
        signed_in: guard.is_some(),
        username: guard
            .as_ref()
            .map(|s| s.username.clone())
            .unwrap_or_default(),
    })
}

#[derive(Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum AuthEvent {
    SignedIn { username: String },
    Failed { error: String },
}

fn emit_auth_event(app: &AppHandle, event: AuthEvent) {
    let _ = app.emit("auth://event", event);
}

/// Starts an interactive Microsoft sign-in: opens the system's default
/// browser to the Microsoft login page (deliberately not an embedded
/// webview — Microsoft's own guidance discourages that for credential
/// entry) and waits for the redirect on a background thread. Returns
/// immediately; the outcome arrives later via `auth://event`.
#[tauri::command]
pub fn auth_login(app: AppHandle) -> Result<(), String> {
    let config = msa_config().ok_or_else(|| {
        "Microsoft sign-in isn't configured yet (no PACKWAND_MSA_CLIENT_ID) — register an Azure AD app and set that env var (see packwandrs.md)".to_string()
    })?;
    let login_session = packwand_msa::begin_login(&config).map_err(|e| e.to_string())?;
    open_with_os_handler(&login_session.authorize_url);

    let thread_app = app.clone();
    thread::spawn(move || {
        let Some(auth_state) = thread_app.try_state::<AuthState>() else {
            return;
        };
        match packwand_msa::await_login(
            login_session,
            Duration::from_secs(300),
            &config,
            &auth_state.store,
        ) {
            Ok(session) => {
                let username = session.username.clone();
                if let Ok(mut guard) = auth_state.session.lock() {
                    *guard = Some(session);
                }
                emit_auth_event(&thread_app, AuthEvent::SignedIn { username });
            }
            Err(e) => emit_auth_event(
                &thread_app,
                AuthEvent::Failed {
                    error: e.to_string(),
                },
            ),
        }
    });
    Ok(())
}

/// Signs out: clears both the in-memory session and the stored refresh
/// token, so future boots fall back to the offline dev-testing session.
#[tauri::command]
pub fn auth_logout(state: State<'_, AuthState>) -> Result<(), String> {
    packwand_msa::logout(&state.store).map_err(|e| e.to_string())?;
    *state
        .session
        .lock()
        .map_err(|_| "auth state poisoned".to_string())? = None;
    Ok(())
}

/// Lists the instances stored under `root`, including error entries for
/// corrupt or future-schema records.
#[tauri::command]
pub fn core_list_instances(root: String) -> Result<Vec<ListEntry>, String> {
    FsInstanceRepository::new(PathBuf::from(root))
        .list()
        .map_err(|e| e.to_string())
}

/// Builds the deterministic launch plan for one instance under `root`.
#[tauri::command]
pub fn core_plan_launch(root: String, instance: String) -> Result<LaunchPlan, String> {
    let repo = FsInstanceRepository::new(PathBuf::from(root));
    let record = repo.get(&instance).map_err(|e| e.to_string())?;
    Ok(build_launch_plan(&record, &repo.instance_paths(&record.id)))
}

fn managed_root(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve the app data directory: {e}"))?;
    Ok(packwand_devboot::default_managed_root(&data_dir))
}

#[derive(Clone, Serialize)]
struct ProgressPayload {
    session_id: String,
    finished_downloads: usize,
    total_downloads: usize,
    downloaded_bytes: u64,
    // Omitted entirely (not `null`) when unknown, so the frontend's
    // "field absent -> default" JSON decoding doesn't have to special-case
    // `null` on top of a missing key.
    #[serde(skip_serializing_if = "Option::is_none")]
    total_bytes: Option<u64>,
}

#[derive(Clone, Serialize)]
struct EventPayload {
    session_id: String,
    #[serde(flatten)]
    event: LaunchEvent,
}

fn emit_event(app: &AppHandle, session_id: &str, event: LaunchEvent) {
    let _ = app.emit(
        "launcher://event",
        EventPayload {
            session_id: session_id.to_string(),
            event,
        },
    );
}

/// Boots `pack_dir` for dev testing: ensures the shared Minecraft+loader
/// install exists (bootstrapping it if not, reporting progress via
/// `launcher://progress`), then launches it pointed at the pack's own
/// directory, forwarding every lifecycle event via `launcher://event` until
/// a terminal event. Returns a session id immediately; the boot itself runs
/// on a background thread.
///
/// Uses the signed-in Microsoft account's session when one is available,
/// falling back to the offline dev-testing session otherwise — signing in
/// is optional, not a requirement to try packwand at all.
///
/// `dock`: when true, once the game window appears it's repositioned (not
/// resized — the player's configured resolution is left alone) flush beside
/// the Packwand window, "docked" rather than embedded. Windows-only for now
/// (see `window_dock`); a no-op elsewhere.
#[tauri::command]
pub fn launcher_boot(
    app: AppHandle,
    state: State<'_, LauncherState>,
    auth_state: State<'_, AuthState>,
    pack_dir: String,
    dock: bool,
) -> Result<String, String> {
    let session_id = state.new_session_id();
    let root = managed_root(&app)?;
    let pack_path = PathBuf::from(pack_dir);
    let account_session = {
        let guard = auth_state
            .session
            .lock()
            .map_err(|_| "auth state poisoned".to_string())?;
        match guard.clone() {
            Some(session) => session,
            None => packwand_devboot::default_offline_session()?,
        }
    };

    let thread_app = app.clone();
    let thread_session = session_id.clone();
    thread::spawn(move || {
        let progress_app = thread_app.clone();
        let progress_session = thread_session.clone();
        let on_progress = move |update: InstallProgress| {
            let _ = progress_app.emit(
                "launcher://progress",
                ProgressPayload {
                    session_id: progress_session.clone(),
                    finished_downloads: update.finished_downloads,
                    total_downloads: update.total_downloads,
                    downloaded_bytes: update.downloaded_bytes,
                    total_bytes: update.total_bytes,
                },
            );
        };

        let booted = match boot_pack(&root, &pack_path, &account_session, on_progress) {
            Ok(booted) => booted,
            Err(e) => {
                emit_event(
                    &thread_app,
                    &thread_session,
                    LaunchEvent::Failed {
                        instance_id: pack_path.display().to_string(),
                        error: e.to_string(),
                    },
                );
                return;
            }
        };

        let options = LaunchOptions {
            secrets: booted.secrets,
            ..LaunchOptions::default()
        };
        let handle = match launch(&booted.plan, options) {
            Ok(handle) => handle,
            Err(e) => {
                emit_event(
                    &thread_app,
                    &thread_session,
                    LaunchEvent::Failed {
                        instance_id: booted.record.id.clone(),
                        error: e.to_string(),
                    },
                );
                return;
            }
        };

        if let Some(state) = thread_app.try_state::<LauncherState>() {
            state.register(thread_session.clone(), handle.cancel_token());
        }

        for event in handle.events() {
            if dock {
                if let LaunchEvent::Started { pid, .. } = &event {
                    let dock_app = thread_app.clone();
                    let dock_pid = *pid;
                    thread::spawn(move || {
                        window_dock::dock_game_window(
                            &dock_app,
                            dock_pid,
                            Duration::from_secs(300),
                        );
                    });
                }
            }
            emit_event(&thread_app, &thread_session, event);
        }
        handle.wait();

        if let Some(state) = thread_app.try_state::<LauncherState>() {
            state.take(&thread_session);
        }
    });

    Ok(session_id)
}

/// Cancels an in-flight boot session. Returns an error if the session isn't
/// cancellable yet (still installing) or has already finished.
#[tauri::command]
pub fn launcher_cancel(state: State<'_, LauncherState>, session_id: String) -> Result<(), String> {
    match state.take(&session_id) {
        Some(token) => {
            token.cancel();
            Ok(())
        }
        None => Err(format!(
            "session {session_id:?} is not cancellable (still installing, or already finished)"
        )),
    }
}
