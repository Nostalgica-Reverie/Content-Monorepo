//! Native desktop shell for the Packwand GUI.
//!
//! Architecture (inspired by the Modrinth App): the webview renders the
//! existing Gleam frontend, while this Rust backend acts as the privileged
//! bridge. Its only system-level responsibility is managing the `packwand gui`
//! HTTP server as a child process. The bundled boot page may select a workspace
//! folder and request the backend URL, then navigates to the local server. The
//! server's own pages get no Tauri IPC access at all.
//!
//! The system tray and job-completion notifications below preserve that
//! boundary: both are driven entirely from this backend polling the
//! packwand HTTP API directly (`/api/v1/version`, `/api/v1/jobs`), never by
//! the webview invoking a Tauri command - so no new capability grants are
//! needed for the server's pages.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;

#[cfg(feature = "launcher-spike")]
mod launcher_spike;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, RunEvent, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;

struct Backend(Mutex<Option<BackendProcess>>);

struct BackendProcess {
    child: Child,
    url: String,
}

/// Returns the currently running backend's base URL, if any.
fn current_backend_url(state: &Backend) -> Option<String> {
    state.0.lock().ok()?.as_ref().map(|b| b.url.clone())
}

/// Locates the packwand executable: PACKWAND_BIN, then next to the app
/// executable, then PATH.
fn find_packwand() -> Result<PathBuf, String> {
    if let Ok(bin) = std::env::var("PACKWAND_BIN") {
        let path = PathBuf::from(bin);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "PACKWAND_BIN is set but does not exist: {}",
            path.display()
        ));
    }
    let name = if cfg!(windows) {
        "packwand.exe"
    } else {
        "packwand"
    };
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join(name);
            if sibling.is_file() {
                return Ok(sibling);
            }
        }
    }
    // Fall back to PATH resolution by the OS.
    Ok(PathBuf::from(name))
}

/// Spawns `packwand gui --no-open --port 0` and reads the bound URL from its
/// startup banner ("packwand gui running at http://127.0.0.1:PORT/").
fn spawn_backend(workspace: Option<&Path>) -> Result<BackendProcess, String> {
    let bin = find_packwand()?;
    let port_file = std::env::temp_dir().join(format!("packwand-gui-{}.url", std::process::id()));
    let _ = fs::remove_file(&port_file);
    let mut command = Command::new(&bin);
    command
        .args(["gui", "--no-open", "--port", "0", "--print-port-file"])
        .arg(&port_file)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(workspace) = workspace {
        command.current_dir(workspace);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("failed to start {} gui: {e}", bin.display()))?;

    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        if let Ok(value) = fs::read_to_string(&port_file) {
            let url = value.trim().to_string();
            let _ = fs::remove_file(&port_file);
            if url.starts_with("http://127.0.0.1:") {
                return Ok(BackendProcess { child, url });
            }
            let _ = child.kill();
            return Err(format!("unexpected packwand gui address: {url}"));
        }
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("packwand gui exited before startup: {status}"));
        }
        thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    Err("timed out waiting for packwand gui to start".into())
}
/// Opens a native folder picker and returns the selected workspace path.
#[tauri::command]
async fn select_workspace(app: AppHandle) -> Result<Option<String>, String> {
    let selected =
        tauri::async_runtime::spawn_blocking(move || app.dialog().file().blocking_pick_folder())
            .await
            .map_err(|e| format!("workspace picker failed: {e}"))?;
    selected
        .map(|path| {
            path.into_path()
                .map(|path| path.to_string_lossy().into_owned())
                .map_err(|_| "selected folder is not a local filesystem path".to_string())
        })
        .transpose()
}

/// Ensures the local backend is running and returns its URL for navigation.
#[tauri::command]
fn backend_url(workspace: Option<String>, state: State<'_, Backend>) -> Result<String, String> {
    let mut guard = state.0.lock().map_err(|_| "backend state poisoned")?;
    if let Some(backend) = guard.as_mut() {
        match backend.child.try_wait() {
            Ok(None) => return Ok(backend.url.clone()), // still running
            _ => *guard = None,                         // exited; respawn below
        }
    }
    let workspace = workspace
        .map(PathBuf::from)
        .map(|path| {
            path.canonicalize()
                .map_err(|e| format!("invalid workspace {}: {e}", path.display()))
        })
        .transpose()?;
    if workspace.as_ref().is_some_and(|path| !path.is_dir()) {
        return Err("selected workspace is not a directory".into());
    }
    let backend = spawn_backend(workspace.as_deref())?;
    let url = backend.url.clone();
    *guard = Some(backend);
    Ok(url)
}

/// Opens `path` in the OS's file manager.
fn open_in_file_manager(path: &str) {
    let result = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(path).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(path).spawn()
    } else {
        Command::new("xdg-open").arg(path).spawn()
    };
    if let Err(e) = result {
        eprintln!("failed to open {path} in the file manager: {e}");
    }
}

/// Fetches the workspace root from the running backend's `/api/v1/version`
/// and opens it in the OS file manager. No-op (with a stderr note) if the
/// backend isn't up yet.
fn open_workspace_folder(app: &AppHandle) {
    let Some(url) = app
        .try_state::<Backend>()
        .and_then(|s| current_backend_url(&s))
    else {
        eprintln!("open workspace folder: backend is not running");
        return;
    };
    #[derive(Deserialize)]
    struct VersionResponse {
        root: String,
    }
    match ureq::get(&format!("{url}/api/v1/version")).call() {
        Ok(response) => match response.into_json::<VersionResponse>() {
            Ok(v) if !v.root.is_empty() => open_in_file_manager(&v.root),
            Ok(_) => eprintln!("open workspace folder: empty root in /api/v1/version"),
            Err(e) => eprintln!("open workspace folder: failed to parse /api/v1/version: {e}"),
        },
        Err(e) => eprintln!("open workspace folder: failed to reach backend: {e}"),
    }
}

#[derive(Deserialize)]
struct JobLite {
    id: String,
    action: String,
    status: String,
}

/// Polls `/api/v1/jobs` every 2s and fires a native notification for any job
/// that finishes (completed or failed) while the main window is unfocused.
/// Runs for the lifetime of the app on a dedicated background thread.
fn spawn_job_watcher(app: AppHandle) {
    thread::spawn(move || {
        let mut last_status: HashMap<String, String> = HashMap::new();
        loop {
            thread::sleep(Duration::from_secs(2));
            let Some(url) = app
                .try_state::<Backend>()
                .and_then(|s| current_backend_url(&s))
            else {
                continue;
            };
            let jobs: Vec<JobLite> = match ureq::get(&format!("{url}/api/v1/jobs")).call() {
                Ok(response) => response.into_json().unwrap_or_default(),
                Err(_) => continue,
            };
            let focused = app
                .get_webview_window("main")
                .and_then(|w| w.is_focused().ok())
                .unwrap_or(true);
            for job in &jobs {
                let previous = last_status.insert(job.id.clone(), job.status.clone());
                let just_finished = previous.as_deref() == Some("running")
                    && (job.status == "completed" || job.status == "failed");
                if just_finished && !focused {
                    let title = if job.status == "failed" {
                        "Packwand job failed"
                    } else {
                        "Packwand job finished"
                    };
                    let _ = app
                        .notification()
                        .builder()
                        .title(title)
                        .body(format!("packwand {}", job.action))
                        .show();
                }
            }
            last_status.retain(|id, _| jobs.iter().any(|j| &j.id == id));
        }
    });
}

pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Backend(Mutex::new(None)));
    #[cfg(not(feature = "launcher-spike"))]
    let builder = builder.invoke_handler(tauri::generate_handler![backend_url, select_workspace]);
    #[cfg(feature = "launcher-spike")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        backend_url,
        select_workspace,
        launcher_spike::core_list_instances,
        launcher_spike::core_plan_launch
    ]);
    builder
        .setup(|app| {
            let handle = app.handle().clone();
            spawn_job_watcher(handle);

            let open_item = MenuItem::with_id(app, "open", "Open Packwand", true, None::<&str>)?;
            let open_folder_item = MenuItem::with_id(
                app,
                "open_folder",
                "Open Workspace Folder",
                true,
                None::<&str>,
            )?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_item, &open_folder_item, &quit_item])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(
                    app.default_window_icon()
                        .cloned()
                        .expect("default window icon"),
                )
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "open_folder" => open_workspace_folder(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Packwand GUI")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                // Terminate the packwand gui server with the app.
                if let Some(state) = app.try_state::<Backend>() {
                    if let Ok(mut guard) = state.0.lock() {
                        if let Some(backend) = guard.as_mut() {
                            let _ = backend.child.kill();
                            let _ = backend.child.wait();
                        }
                        *guard = None;
                    }
                }
            }
        });
}
