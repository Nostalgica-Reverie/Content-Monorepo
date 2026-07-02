//! Native desktop shell for the Packwand GUI.
//!
//! Architecture (inspired by the Modrinth App): the webview renders the
//! existing Gleam frontend, while this Rust backend acts as the privileged
//! bridge. Its only system-level responsibility is managing the `packwand gui`
//! HTTP server as a child process; a single validated IPC command
//! (`backend_url`) is exposed to the bundled boot page, which then navigates
//! to the local server. The server's own pages get no Tauri IPC access at all.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;

use tauri::{Manager, RunEvent, State};

struct Backend(Mutex<Option<BackendProcess>>);

struct BackendProcess {
    child: Child,
    url: String,
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
    let name = if cfg!(windows) { "packwand.exe" } else { "packwand" };
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
fn spawn_backend() -> Result<BackendProcess, String> {
    let bin = find_packwand()?;
    let mut child = Command::new(&bin)
        .args(["gui", "--no-open", "--port", "0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to start {} gui: {e}", bin.display()))?;

    let stdout = child.stdout.take().ok_or("failed to capture packwand stdout")?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            if let Some(url) = line
                .strip_prefix("packwand gui running at ")
                .map(str::trim)
            {
                let _ = tx.send(url.to_string());
            }
            // Keep draining stdout so the child never blocks on a full pipe.
        }
    });

    match rx.recv_timeout(Duration::from_secs(15)) {
        Ok(url) if url.starts_with("http://127.0.0.1:") => Ok(BackendProcess { child, url }),
        Ok(url) => {
            let _ = child.kill();
            Err(format!("unexpected packwand gui address: {url}"))
        }
        Err(_) => {
            let _ = child.kill();
            Err("timed out waiting for packwand gui to start".into())
        }
    }
}

/// The only command exposed to the boot page: ensure the local backend is
/// running and return its URL for navigation.
#[tauri::command]
fn backend_url(state: State<'_, Backend>) -> Result<String, String> {
    let mut guard = state.0.lock().map_err(|_| "backend state poisoned")?;
    if let Some(backend) = guard.as_mut() {
        match backend.child.try_wait() {
            Ok(None) => return Ok(backend.url.clone()), // still running
            _ => *guard = None,                         // exited; respawn below
        }
    }
    let backend = spawn_backend()?;
    let url = backend.url.clone();
    *guard = Some(backend);
    Ok(url)
}

pub fn run() {
    tauri::Builder::default()
        .manage(Backend(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![backend_url])
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
