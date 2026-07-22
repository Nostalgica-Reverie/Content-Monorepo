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

pub fn emit_packs_changed(app: &AppHandle) -> CommandResult<()> {
    emit(app, "packs:changed", ())
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
