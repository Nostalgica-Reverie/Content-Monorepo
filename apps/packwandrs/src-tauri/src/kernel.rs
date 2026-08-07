//! Drains the bounded Rust platform trace ring into Packwand's output dock.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

const DRAIN_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TracePayload {
    pub sequence: u64,
    pub tone: String,
    pub module: String,
    pub message: String,
    pub origin: String,
    pub platform_code: Option<i32>,
}

pub struct KernelState {
    shutdown: Arc<AtomicBool>,
}

impl Drop for KernelState {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
    }
}

pub fn start(app: &AppHandle) {
    packwand_platform::trace(
        packwand_platform::TraceLevel::Info,
        "core",
        "Packwand Rust platform services initialised",
        "src-tauri/src/kernel.rs",
        None,
    );
    let shutdown = Arc::new(AtomicBool::new(false));
    app.manage(KernelState {
        shutdown: Arc::clone(&shutdown),
    });
    let pump = app.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("packwand-trace-drain".into())
        .spawn(move || {
            let mut last_dropped = 0;
            while !shutdown.load(Ordering::Acquire) {
                std::thread::sleep(DRAIN_INTERVAL);
                for record in packwand_platform::trace_drain() {
                    emit_trace(&pump, &payload_for(record));
                }
                let dropped = packwand_platform::trace_dropped();
                if dropped > last_dropped {
                    emit_trace(
                        &pump,
                        &TracePayload {
                            sequence: 0,
                            tone: "error".into(),
                            module: "core".into(),
                            message: format!("{} trace record(s) dropped", dropped - last_dropped),
                            origin: "src-tauri/src/kernel.rs".into(),
                            platform_code: None,
                        },
                    );
                    last_dropped = dropped;
                }
            }
        })
    {
        emit_trace(
            app,
            &TracePayload {
                sequence: 0,
                tone: "error".into(),
                module: "core".into(),
                message: format!("could not start the trace drain: {error}"),
                origin: "src-tauri/src/kernel.rs".into(),
                platform_code: None,
            },
        );
    }
}

fn payload_for(record: packwand_platform::TraceRecord) -> TracePayload {
    TracePayload {
        sequence: record.sequence,
        tone: if record.level == packwand_platform::TraceLevel::Error {
            "error".into()
        } else {
            "info".into()
        },
        module: record.module,
        message: record.message,
        origin: record.origin,
        platform_code: record.platform_code,
    }
}

fn emit_trace(app: &AppHandle, payload: &TracePayload) {
    let _ = app.emit("kernel:trace", payload);
}
