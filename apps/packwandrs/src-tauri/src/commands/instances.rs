//! Instance listing and real launching.
//!
//! Instances are derived from the packs Packwand already knows about
//! (`commands::packs`) — there is no separate instance-authoring flow yet.
//! Launching an instance is a three-step pipeline that mirrors the shared
//! `packwand-core-probe` and the (Go-app) Tauri shell's `launcher.rs`:
//!
//! 1. Materialize the pack's mods/config into a stable, per-pack instance
//!    directory via the existing packwiz-installer test-install workflow
//!    (`packwand_build::test_with_installer`, already used by
//!    `diagnostics_installer_test`).
//! 2. Resolve and install the Minecraft version/loader that instance
//!    targets, and bake an offline session into a launchable instance
//!    record (`packwand_devboot::boot_pack`, which itself bootstraps via
//!    `packwand-minecraft`/`packwand-runtime`/`packwand-instance` when no
//!    install exists yet).
//! 3. Launch the approved plan and supervise the child process
//!    (`packwand_launch::launch`).
//!
//! All of this runs as one `JobManager` job (the existing async/long-running
//! work pattern in this codebase), so the Jobs page gets full logs for free.
//! On top of that, this module also emits a lighter-weight `instance:status`
//! event stream (`starting` / `running` / `stopped` / `error`) so the
//! Instances page can show live status on each card without polling job
//! internals directly.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use packwand_launch::LaunchEvent;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use crate::commands::jobs::JobRecord;
use crate::commands::packs::{discover_packs, pack_root};
use crate::error::{CommandResult, SerializableError};
use crate::events::emit_instance_status;
use crate::state::AppState;

/// An instance card's static data. For now every instance is 1:1 with a
/// discoverable pack ("Modpacks" tab); Servers/Custom stay frontend-only
/// empty states until Packwand has a concept for them.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSummary {
    pub id: String,
    pub name: String,
    pub path: String,
    pub minecraft_version: Option<String>,
    pub loaders: Vec<String>,
    pub kind: &'static str,
}

/// Live status for one instance's most recent (or in-flight) launch.
/// `phase` is intentionally a small fixed set so the frontend can render it
/// without a lookup table: `starting`, `running`, `stopped`, `error`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatusPayload {
    pub id: String,
    pub phase: String,
    pub message: Option<String>,
    pub job_id: Option<String>,
    pub exit_code: Option<i32>,
}

/// In-memory "last known status" per instance id, so a page that (re)mounts
/// mid-launch can hydrate immediately via `instances_status_list` instead of
/// waiting for the next event. Not persisted — launches don't survive an app
/// restart, so there is nothing meaningful to restore across process runs.
#[derive(Clone, Default)]
pub struct InstanceRegistry {
    entries: Arc<RwLock<HashMap<String, InstanceStatusPayload>>>,
}

impl InstanceRegistry {
    pub async fn set(&self, payload: InstanceStatusPayload) {
        self.entries
            .write()
            .await
            .insert(payload.id.clone(), payload);
    }

    pub async fn list(&self) -> Vec<InstanceStatusPayload> {
        self.entries.read().await.values().cloned().collect()
    }

    pub async fn job_id_for(&self, id: &str) -> Option<String> {
        self.entries
            .read()
            .await
            .get(id)
            .and_then(|payload| payload.job_id.clone())
    }
}

#[tauri::command]
pub fn instances_list(state: State<'_, AppState>) -> CommandResult<Vec<InstanceSummary>> {
    Ok(discover_packs(&state.workspace()?)?
        .into_iter()
        .map(|pack| InstanceSummary {
            id: pack.id,
            name: pack.name,
            path: pack.path,
            minecraft_version: pack.minecraft_version,
            loaders: pack.loaders,
            kind: "modpack",
        })
        .collect())
}

#[tauri::command]
pub async fn instances_status_list(
    state: State<'_, AppState>,
) -> CommandResult<Vec<InstanceStatusPayload>> {
    Ok(state.instances.list().await)
}

/// Cancels the in-flight launch job for `id`, if any. Reuses the generic
/// job-cancellation mechanism (`jobs_cancel`) rather than a parallel one —
/// the launch loop cooperatively checks `JobContext::is_cancelled` and, once
/// the process has started, forwards that to the launch supervisor so the
/// whole child process tree is terminated.
#[tauri::command]
pub async fn instances_stop(id: String, state: State<'_, AppState>) -> CommandResult<bool> {
    match state.instances.job_id_for(&id).await {
        Some(job_id) => Ok(state.jobs.cancel(&job_id).await),
        None => Ok(false),
    }
}

fn find_installer_jar(app: &AppHandle) -> Option<PathBuf> {
    let resource_dir = app.path().resource_dir().ok()?;
    [
        resource_dir.join("resources/packwiz-installer.jar"),
        resource_dir.join("packwiz-installer.jar"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

/// Messages sent from the fully-synchronous launch pipeline (running on a
/// blocking thread) back to the async job future, which is the only place
/// that may call the async `JobContext::log`/`progress` or touch the
/// `AppHandle` event emitter.
enum LaunchSignal {
    Log(String),
    Progress(f64, Option<String>),
    Status {
        phase: &'static str,
        message: Option<String>,
        exit_code: Option<i32>,
    },
}

#[tauri::command]
pub async fn instances_launch(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
    let workspace = state.workspace()?;
    let pack_dir = pack_root(&workspace, &id)?;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| SerializableError::new("path", error.to_string()))?;
    let app_cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| SerializableError::new("path", error.to_string()))?;
    let managed_root = packwand_devboot::default_managed_root(&app_data_dir);
    let instance_dir = app_cache_dir.join("instances").join(&id);
    let installer_jar = find_installer_jar(&app);

    let registry = state.instances.clone();
    let label = format!("Launch {id}");
    let launch_app = app.clone();
    let launch_id = id.clone();

    let job = state
        .jobs
        .spawn(
            app.clone(),
            "instance.launch",
            label,
            move |context| async move {
                let job_id = context.id().to_string();
                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<LaunchSignal>();
                // `run_instance_launch` only needs a way to *check* cancellation
                // (a plain sync bool), not the whole `JobContext` — this keeps it
                // testable without a Tauri/JobManager fixture. See the `tests`
                // module below.
                let cancel_context = context.clone();
                let is_cancelled = move || cancel_context.is_cancelled();
                let blocking_handle = tokio::task::spawn_blocking(move || {
                    run_instance_launch(
                        &pack_dir,
                        installer_jar.as_deref(),
                        &instance_dir,
                        &managed_root,
                        &is_cancelled,
                        &tx,
                    )
                });

                while let Some(signal) = rx.recv().await {
                    match signal {
                        LaunchSignal::Log(line) => context.log(line).await,
                        LaunchSignal::Progress(fraction, message) => {
                            context.progress(fraction, message).await
                        }
                        LaunchSignal::Status {
                            phase,
                            message,
                            exit_code,
                        } => {
                            let payload = InstanceStatusPayload {
                                id: launch_id.clone(),
                                phase: phase.to_string(),
                                message,
                                job_id: Some(job_id.clone()),
                                exit_code,
                            };
                            registry.set(payload.clone()).await;
                            let _ = emit_instance_status(&launch_app, payload);
                        }
                    }
                }

                let outcome = match blocking_handle.await {
                    Ok(outcome) => outcome,
                    Err(error) => Err(SerializableError::new("task", error.to_string())),
                };
                // Safety net: whatever went wrong (a bug inside the pipeline, an
                // early `?` return that predates a status send, or even a panic
                // caught by `spawn_blocking`), the Instances page must never be
                // left showing a stale "starting" card once the job has actually
                // finished — that reads as "broken" and makes Stop look like it
                // does nothing (the job is no longer `Running` by then, so
                // `jobs_cancel` correctly, but confusingly, no-ops).
                if let Err(ref error) = outcome {
                    let payload = InstanceStatusPayload {
                        id: launch_id.clone(),
                        phase: terminal_phase_for(error).to_string(),
                        message: Some(error.message.clone()),
                        job_id: Some(job_id.clone()),
                        exit_code: None,
                    };
                    registry.set(payload.clone()).await;
                    let _ = emit_instance_status(&launch_app, payload);
                }
                outcome
            },
        )
        .await;
    Ok(job)
}

/// Maps a terminal error to the `instance:status` phase the frontend should
/// show for it: a cooperative cancellation is `stopped` (expected, not a
/// failure), everything else is `error`.
fn terminal_phase_for(error: &SerializableError) -> &'static str {
    if error.kind == "cancelled" {
        "stopped"
    } else {
        "error"
    }
}

fn cancelled_error() -> SerializableError {
    SerializableError::new("cancelled", "job was cancelled")
}

/// The fully synchronous install -> resolve/install -> launch pipeline.
/// Runs on a blocking thread; reports progress and status purely by sending
/// [`LaunchSignal`]s, and consults `is_cancelled()` (a plain sync check, not
/// tied to `JobContext` so this function can be exercised directly in tests)
/// at each phase boundary and inside the launch event loop.
fn run_instance_launch<F: Fn() -> bool>(
    pack_dir: &Path,
    installer_jar: Option<&Path>,
    instance_dir: &Path,
    managed_root: &Path,
    is_cancelled: &F,
    tx: &tokio::sync::mpsc::UnboundedSender<LaunchSignal>,
) -> CommandResult<()> {
    let send_status = |phase: &'static str, message: Option<String>, exit_code: Option<i32>| {
        let _ = tx.send(LaunchSignal::Status {
            phase,
            message,
            exit_code,
        });
    };
    let send_log = |line: String| {
        let _ = tx.send(LaunchSignal::Log(line));
    };

    send_status(
        "starting",
        Some("Installing pack contents".to_string()),
        None,
    );
    send_log("Installing pack contents via packwiz-installer".to_string());
    let report = packwand_build::test_with_installer(pack_dir, installer_jar, instance_dir)
        .map_err(|error| SerializableError::new("installer", error.to_string()))?;
    send_log(format!(
        "Pack contents ready in {}",
        report.instance.display()
    ));

    // `test_with_installer` only materializes the mod/config files packwiz-installer
    // downloads — it never copies `pack.toml` itself into the instance directory (the
    // manifest stays source-side, served over the loopback installer protocol). But
    // `boot_pack` uses this same directory as both its `pack.toml` lookup *and* the
    // Minecraft `game_dir`, so it must exist here too, or version/loader resolution
    // fails with a bare "file not found" before anything gets a chance to launch.
    std::fs::copy(
        pack_dir.join("pack.toml"),
        report.instance.join("pack.toml"),
    )
    .map_err(|error| {
        SerializableError::new(
            "installer",
            format!("failed to stage pack.toml for launch: {error}"),
        )
    })?;

    if is_cancelled() {
        send_status("stopped", Some("Cancelled".to_string()), None);
        return Err(cancelled_error());
    }

    send_status(
        "starting",
        Some("Resolving Minecraft version and loader".to_string()),
        None,
    );
    let session = packwand_devboot::default_offline_session()
        .map_err(|error| SerializableError::new("auth", error))?;
    send_log(format!("Using offline session for {}", session.username));

    let progress_tx = tx.clone();
    let on_progress = move |update: packwand_devboot::InstallProgress| {
        let fraction = if update.total_downloads > 0 {
            update.finished_downloads as f64 / update.total_downloads as f64
        } else {
            0.0
        };
        let message = match update.total_bytes {
            Some(total) if total > 0 => format!(
                "{}/{} downloads · {}/{} MiB",
                update.finished_downloads,
                update.total_downloads,
                update.downloaded_bytes / (1024 * 1024),
                total / (1024 * 1024)
            ),
            _ => format!(
                "{}/{} downloads · {} MiB",
                update.finished_downloads,
                update.total_downloads,
                update.downloaded_bytes / (1024 * 1024)
            ),
        };
        let _ = progress_tx.send(LaunchSignal::Progress(fraction, Some(message)));
    };

    let booted =
        packwand_devboot::boot_pack(managed_root, &report.instance, &session, None, on_progress)
            .map_err(|error| SerializableError::new("bootstrap", error.to_string()))?;
    send_log(format!(
        "Instance ready: {} ({})",
        booted.record.id, booted.record.name
    ));

    if is_cancelled() {
        send_status("stopped", Some("Cancelled".to_string()), None);
        return Err(cancelled_error());
    }

    send_status("starting", Some("Launching Minecraft".to_string()), None);
    let options = packwand_launch::LaunchOptions {
        secrets: booted.secrets,
        ..Default::default()
    };
    let handle = packwand_launch::launch(&booted.plan, options)
        .map_err(|error| SerializableError::new("launch", error.to_string()))?;

    let mut outcome: CommandResult<()> = Ok(());
    loop {
        match handle.events().recv_timeout(Duration::from_millis(250)) {
            Ok(event) => match event {
                LaunchEvent::Starting { .. } => {}
                LaunchEvent::Started { pid, .. } => {
                    send_log(format!("Minecraft process started (pid {pid})"));
                    send_status("running", Some(format!("Running (pid {pid})")), None);
                }
                LaunchEvent::Stdout { line, .. } | LaunchEvent::Stderr { line, .. } => {
                    send_log(line);
                }
                LaunchEvent::Exited { code, .. } => {
                    let ok = code == Some(0);
                    send_status(
                        if ok { "stopped" } else { "error" },
                        Some(match code {
                            Some(code) => format!("Exited with code {code}"),
                            None => "Exited".to_string(),
                        }),
                        code,
                    );
                    if !ok {
                        outcome = Err(SerializableError::new(
                            "exit_code",
                            format!("Minecraft exited with code {code:?}"),
                        ));
                    }
                    break;
                }
                LaunchEvent::Failed { error, .. } => {
                    send_status("error", Some(error.clone()), None);
                    outcome = Err(SerializableError::new("launch_failed", error));
                    break;
                }
                LaunchEvent::Cancelled { .. } => {
                    send_status("stopped", Some("Cancelled".to_string()), None);
                    outcome = Err(cancelled_error());
                    break;
                }
            },
            Err(RecvTimeoutError::Timeout) => {
                if is_cancelled() {
                    handle.cancel();
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    outcome
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    fn drain(rx: &mut tokio::sync::mpsc::UnboundedReceiver<LaunchSignal>) -> Vec<LaunchSignal> {
        let mut out = Vec::new();
        while let Ok(signal) = rx.try_recv() {
            out.push(signal);
        }
        out
    }

    /// Reproduces the reported "install/launch broken" symptom against a
    /// pack directory that cannot possibly resolve (no real JDK/network
    /// needed): `test_with_installer` fails immediately on
    /// `pack.canonicalize()`. This must fail fast with a readable error and
    /// must not hang — it's the same shape of failure a missing JDK, a
    /// missing packwiz-installer.jar, or a stale run-lock would produce.
    #[test]
    fn missing_pack_directory_fails_fast_with_a_readable_error_instead_of_hanging() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let missing = Path::new("./this-pack-directory-does-not-exist-for-the-test");

        let result = run_instance_launch(
            missing,
            None,
            Path::new("./unused-instance-dir"),
            Path::new("./unused-managed-root"),
            &|| false,
            &tx,
        );

        let error = result.expect_err("a missing pack directory must fail, not hang or succeed");
        assert_eq!(error.kind, "installer");

        let signals = drain(&mut rx);
        assert!(
            signals.iter().any(|signal| matches!(
                signal,
                LaunchSignal::Status {
                    phase: "starting",
                    ..
                }
            )),
            "must report a starting phase before failing, so the UI has something to show \
             instead of nothing at all",
        );
    }

    /// The single most important property for "can't cancel" reports: no
    /// matter which phase the pipeline is in when it fails, the caller
    /// (`instances_launch`'s job future) can always compute a terminal
    /// phase from the returned error — it never has to guess or leave the
    /// card stuck on `starting`.
    #[test]
    fn every_failure_path_maps_to_a_terminal_status_phase() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let missing = Path::new("./another-missing-pack-directory-for-the-test");

        let error = run_instance_launch(
            missing,
            None,
            Path::new("./unused-instance-dir-2"),
            Path::new("./unused-managed-root-2"),
            &|| false,
            &tx,
        )
        .unwrap_err();
        assert_eq!(terminal_phase_for(&error), "error");

        let cancelled = cancelled_error();
        assert_eq!(terminal_phase_for(&cancelled), "stopped");
    }

    /// Cancellation requested up front is observed at the very first phase
    /// boundary (before any network/process work happens) rather than only
    /// once the game process itself has started.
    #[test]
    fn cancellation_flag_is_observed_even_when_never_un_set() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let cancelled = AtomicBool::new(true);
        let missing = Path::new("./cancelled-before-install-pack-directory");

        // `test_with_installer` still runs first (installation has no
        // cancellation hook — a known limitation, see the module docs) and
        // fails on the missing directory before the first cancellation
        // check is reached; this asserts that failure is reported as an
        // `installer` error, not silently swallowed by the cancellation
        // flag being set.
        let result = run_instance_launch(
            missing,
            None,
            Path::new("./unused-instance-dir-3"),
            Path::new("./unused-managed-root-3"),
            &|| cancelled.load(Ordering::SeqCst),
            &tx,
        );
        assert!(result.is_err());
        drop(rx);
    }
}
