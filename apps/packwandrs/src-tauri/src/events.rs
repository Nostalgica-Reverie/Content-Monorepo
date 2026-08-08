use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::commands::jobs::{JobRecord, JobStatus};
use crate::error::{CommandResult, SerializableError};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobLogPayload {
    pub id: String,
    pub line: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgressPayload {
    pub id: String,
    pub fraction: f64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobFinishedPayload {
    pub id: String,
    pub status: JobStatus,
    pub error: Option<SerializableError>,
}

fn emit<T: Serialize + Clone>(app: &AppHandle, event: &str, payload: T) -> CommandResult<()> {
    app.emit(event, payload)
        .map_err(|error| SerializableError::new("event", error.to_string()))
}

pub fn emit_job_started(app: &AppHandle, job: JobRecord) -> CommandResult<()> {
    emit(app, "job:started", job)
}

pub fn emit_job_log(app: &AppHandle, id: String, line: String) -> CommandResult<()> {
    emit(app, "job:log", JobLogPayload { id, line })
}

pub fn emit_job_progress(
    app: &AppHandle,
    id: String,
    fraction: f64,
    message: Option<String>,
) -> CommandResult<()> {
    emit(
        app,
        "job:progress",
        JobProgressPayload {
            id,
            fraction,
            message,
        },
    )
}

pub fn emit_job_finished(
    app: &AppHandle,
    event: &str,
    id: String,
    status: JobStatus,
    error: Option<SerializableError>,
) -> CommandResult<()> {
    emit(app, event, JobFinishedPayload { id, status, error })
}

/// How long one `packs:changed` emission suppresses the next.
const PACKS_CHANGED_WINDOW: Duration = Duration::from_millis(150);

/// When the next `packs:changed` may be emitted, and whether one was
/// suppressed and still owes the frontend an emission.
static PACKS_CHANGED_GATE: Mutex<Option<(Instant, bool)>> = Mutex::new(None);

/// Signals that pack contents on disk changed.
///
/// Fired by every write command and by the workspace watcher thread, so a
/// single save can trigger several within a few milliseconds. Listeners
/// re-fetch full state on this event, and not all of them debounce — the
/// workbench re-walks and re-parses every language file, which was happening
/// once per keystroke-save.
///
/// Coalescing is **trailing-edge**: a suppressed emission schedules one at the
/// end of the window rather than being dropped. Listeners derive their whole
/// view from the event, so swallowing the last one in a burst would leave the
/// UI showing stale content until some unrelated change happened to fire.
pub fn emit_packs_changed(app: &AppHandle) -> CommandResult<()> {
    let now = Instant::now();
    {
        let mut gate = PACKS_CHANGED_GATE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((next_allowed, pending)) = *gate
            && now < next_allowed
        {
            if !pending {
                // First suppression in this window: arrange for exactly one
                // trailing emission once the window closes.
                *gate = Some((next_allowed, true));
                let app = app.clone();
                let delay = next_allowed.saturating_duration_since(now);
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(delay).await;
                    {
                        let mut gate = PACKS_CHANGED_GATE
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        *gate = Some((Instant::now() + PACKS_CHANGED_WINDOW, false));
                    }
                    let _ = emit(&app, "packs:changed", ());
                });
            }
            return Ok(());
        }
        *gate = Some((now + PACKS_CHANGED_WINDOW, false));
    }
    emit(app, "packs:changed", ())
}

pub fn emit_workspace_files_changed(app: &AppHandle, paths: Vec<String>) -> CommandResult<()> {
    emit(app, "workspace:files-changed", paths)
}

pub fn emit_instance_status(
    app: &AppHandle,
    payload: crate::commands::instances::InstanceStatusPayload,
) -> CommandResult<()> {
    emit(app, "instance:status", payload)
}

pub fn emit_settings_changed<T: Serialize + Clone>(
    app: &AppHandle,
    settings: T,
) -> CommandResult<()> {
    emit(app, "settings:changed", settings)
}
