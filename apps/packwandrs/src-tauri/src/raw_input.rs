use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{CommandResult, SerializableError};

const IDLE_DELAY: Duration = Duration::from_millis(50);
const ACTIVE_DELAY: Duration = Duration::from_millis(2);
const MAX_BATCH: usize = 256;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawInputPayload {
    pub kind: &'static str,
    pub timestamp_ms: u32,
    pub make_code: u16,
    pub flags: u16,
    pub virtual_key: u16,
    pub button_flags: u16,
    pub delta_x: i32,
    pub delta_y: i32,
    pub wheel_delta: i16,
}

impl From<packwandc::RawInputEvent> for RawInputPayload {
    fn from(event: packwandc::RawInputEvent) -> Self {
        Self {
            kind: match event.kind {
                packwandc::RawInputKind::Keyboard => "keyboard",
                packwandc::RawInputKind::Mouse => "mouse",
            },
            timestamp_ms: event.timestamp_ms,
            make_code: event.make_code,
            flags: event.flags,
            virtual_key: event.virtual_key,
            button_flags: event.button_flags,
            delta_x: event.delta_x,
            delta_y: event.delta_y,
            wheel_delta: event.wheel_delta,
        }
    }
}

pub struct RawInputState {
    enabled: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
}

impl Drop for RawInputState {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.enabled.store(false, Ordering::Release);
        packwandc::raw_input_stop();
    }
}

pub fn start(app: &AppHandle) -> CommandResult<()> {
    let enabled = Arc::new(AtomicBool::new(false));
    let shutdown = Arc::new(AtomicBool::new(false));
    app.manage(RawInputState {
        enabled: Arc::clone(&enabled),
        shutdown: Arc::clone(&shutdown),
    });

    let pump_app = app.clone();
    std::thread::Builder::new()
        .name("packwand-raw-input".into())
        .spawn(move || pump(pump_app, enabled, shutdown))
        .map_err(|error| SerializableError::new("raw_input_thread", error.to_string()))?;
    Ok(())
}

fn pump(app: AppHandle, enabled: Arc<AtomicBool>, shutdown: Arc<AtomicBool>) {
    let mut last_dropped = 0;
    let mut batch = Vec::with_capacity(MAX_BATCH);
    while !shutdown.load(Ordering::Acquire) {
        if !enabled.load(Ordering::Acquire) {
            std::thread::sleep(IDLE_DELAY);
            continue;
        }

        batch.clear();
        while batch.len() < MAX_BATCH {
            match packwandc::raw_input_read() {
                Ok(Some(event)) => batch.push(RawInputPayload::from(event)),
                Ok(None) => break,
                Err(_) => {
                    enabled.store(false, Ordering::Release);
                    break;
                }
            }
        }
        if !batch.is_empty() {
            let _ = app.emit("raw-input:batch", &batch);
        }
        if let Ok(dropped) = packwandc::raw_input_dropped()
            && dropped > last_dropped
        {
            let _ = app.emit("raw-input:dropped", dropped - last_dropped);
            last_dropped = dropped;
        }
        std::thread::sleep(ACTIVE_DELAY);
    }
}

pub fn set_enabled(app: &AppHandle, enabled: bool) -> CommandResult<bool> {
    let state = app.state::<RawInputState>();
    if state.enabled.load(Ordering::Acquire) == enabled {
        return Ok(enabled);
    }
    if enabled {
        #[cfg(windows)]
        {
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| SerializableError::new("raw_input", "main window is unavailable"))?;
            let hwnd = window
                .hwnd()
                .map_err(|error| SerializableError::new("raw_input", error.to_string()))?;
            packwandc::raw_input_start(hwnd.0 as usize)
                .map_err(|error| SerializableError::new("raw_input", error.to_string()))?;
        }
        #[cfg(not(windows))]
        return Err(SerializableError::new(
            "raw_input_unavailable",
            "Raw Input is currently available on Windows only",
        ));
    } else {
        state.enabled.store(false, Ordering::Release);
        packwandc::raw_input_stop();
    }
    state.enabled.store(enabled, Ordering::Release);
    Ok(enabled)
}

#[tauri::command]
pub fn raw_input_set_enabled(app: AppHandle, enabled: bool) -> CommandResult<bool> {
    set_enabled(&app, enabled)
}
