//! Bringing up the packwandc native core and draining its trace ring.
//!
//! The native core records failures in a fixed ring; this module drains it.
//!
//! The ring is passive, so this module polls it. Boot failures are reported to
//! the output dock while the rest of the workbench remains usable.

use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use packwandc_host::Host;

/// How often the trace ring is drained.
///
/// The ring holds 256 records. This interval only has to keep up with the
/// *average* rate; bursts are absorbed by the ring and, past that, counted as
/// drops rather than blocking the kernel thread that produced them.
const DRAIN_INTERVAL: Duration = Duration::from_millis(250);

/// A trace record on its way to the output dock.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TracePayload {
    pub sequence: u64,
    /// `info`, `error` or `success` — the dock's existing tones, so the UI
    /// contract does not change.
    pub tone: String,
    pub module: String,
    pub message: String,
    /// `file:line` in the C tree, already repo-relative via -ffile-prefix-map.
    pub origin: String,
    /// OS error code, when the failing path had one.
    pub platform_code: Option<i32>,
}

/// Owns the running core for the lifetime of the process.
///
/// Held in Tauri's managed state so that `Drop` runs at shutdown rather than at
/// the end of `setup`, which would tear the kernel down immediately after
/// bringing it up.
pub struct KernelState {
    _host: Arc<Host>,
}

/// Boot the core and start draining its trace ring into the output dock.
///
/// Never returns an error: see the module comment on why a failed native boot
/// degrades rather than aborts.
pub fn start(app: &AppHandle) {
    let host = match Host::start() {
        Ok(host) => Arc::new(host),
        Err(error) => {
            // Reported through the same path traces use, so it lands in the
            // dock the user is already looking at rather than only on stderr.
            emit_trace(
                app,
                &TracePayload {
                    sequence: 0,
                    tone: "error".into(),
                    module: "core".into(),
                    message: format!("native core failed to start: {error}"),
                    origin: "src-tauri/src/kernel.rs".into(),
                    platform_code: None,
                },
            );
            return;
        }
    };

    app.manage(KernelState {
        _host: Arc::clone(&host),
    });

    let pump = app.clone();
    let drain_host = Arc::clone(&host);
    if let Err(error) = std::thread::Builder::new()
        .name("packwandc-trace-drain".into())
        .spawn(move || drain_loop(&pump, &drain_host))
    {
        // The core is up but nothing will read its ring. Say so: the symptom
        // otherwise is an empty log that looks like nothing going wrong.
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

fn drain_loop(app: &AppHandle, host: &Host) {
    let mut last_dropped = 0u64;
    loop {
        std::thread::sleep(DRAIN_INTERVAL);

        let drained = host.drain_trace(|record| {
            emit_trace(app, &payload_for(record));
        });
        if drained.is_err() {
            // The core is gone or was never booted. Nothing here can fix that,
            // and retrying forever would spin; stop pumping.
            return;
        }

        // A drop leaves no record behind, so silence would look like calm.
        // Only the *change* is reported, or a saturated ring would say the same
        // thing every quarter second.
        match host.trace_dropped() {
            Ok(dropped) if dropped > last_dropped => {
                let lost = dropped - last_dropped;
                last_dropped = dropped;
                emit_trace(
                    app,
                    &TracePayload {
                        sequence: 0,
                        tone: "error".into(),
                        module: "core".into(),
                        message: format!(
                            "{lost} trace record(s) dropped: the ring filled faster than it drained"
                        ),
                        origin: "src-tauri/src/kernel.rs".into(),
                        platform_code: None,
                    },
                );
            }
            Ok(_) => {}
            Err(_) => return,
        }
    }
}

fn payload_for(record: &packwandc::TraceRecord) -> TracePayload {
    TracePayload {
        sequence: record.sequence,
        tone: if record.level >= packwandc::trace_level::ERROR {
            "error".into()
        } else {
            "info".into()
        },
        module: record.module.to_owned(),
        message: record.message.to_owned(),
        origin: format!("{}:{}", record.file, record.line),
        platform_code: (record.platform_code != 0).then_some(record.platform_code),
    }
}

fn emit_trace(app: &AppHandle, payload: &TracePayload) {
    // Best-effort: a failed emit means the window is going away, and there is
    // nowhere better to report that than the log we just failed to write to.
    let _ = app.emit("kernel:trace", payload);
}
