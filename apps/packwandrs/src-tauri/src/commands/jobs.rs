use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::error::{CommandResult, SerializableError};
use crate::events::{emit_job_finished, emit_job_log, emit_job_progress, emit_job_started};
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Running,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRecord {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub status: JobStatus,
    pub fraction: f64,
    pub message: Option<String>,
    pub logs: Vec<String>,
    pub error: Option<SerializableError>,
}

struct JobEntry {
    record: JobRecord,
    cancellation: CancellationToken,
}

#[derive(Clone, Default)]
pub struct JobManager {
    entries: Arc<RwLock<HashMap<String, JobEntry>>>,
}

#[derive(Clone)]
pub struct JobContext {
    id: String,
    app: AppHandle,
    manager: JobManager,
    cancellation: CancellationToken,
}

impl JobContext {
    /// The id of the job this context reports progress for — lets a Tauri
    /// command correlate its own domain events (e.g. instance status) with
    /// the generic job that is tracking the underlying work.
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    pub fn cancelled_error(&self) -> SerializableError {
        SerializableError::new("cancelled", "job was cancelled")
    }

    pub async fn log(&self, line: impl Into<String>) {
        let line = line.into();
        if let Some(entry) = self.manager.entries.write().await.get_mut(&self.id) {
            entry.record.logs.push(line.clone());
            if entry.record.logs.len() > 2_000 {
                entry.record.logs.remove(0);
            }
        }
        let _ = emit_job_log(&self.app, self.id.clone(), line);
    }

    pub async fn progress(&self, fraction: f64, message: impl Into<Option<String>>) {
        let fraction = fraction.clamp(0.0, 1.0);
        let message = message.into();
        if let Some(entry) = self.manager.entries.write().await.get_mut(&self.id) {
            entry.record.fraction = fraction;
            entry.record.message.clone_from(&message);
        }
        let _ = emit_job_progress(&self.app, self.id.clone(), fraction, message);
    }
}

impl JobManager {
    pub async fn spawn<F, Fut>(
        &self,
        app: AppHandle,
        kind: impl Into<String>,
        label: impl Into<String>,
        work: F,
    ) -> JobRecord
    where
        F: FnOnce(JobContext) -> Fut + Send + 'static,
        Fut: Future<Output = CommandResult<()>> + Send + 'static,
    {
        let id = Uuid::new_v4().to_string();
        let cancellation = CancellationToken::new();
        let record = JobRecord {
            id: id.clone(),
            kind: kind.into(),
            label: label.into(),
            status: JobStatus::Running,
            fraction: 0.0,
            message: None,
            logs: Vec::new(),
            error: None,
        };
        self.entries.write().await.insert(
            id.clone(),
            JobEntry {
                record: record.clone(),
                cancellation: cancellation.clone(),
            },
        );
        let _ = emit_job_started(&app, record.clone());

        let manager = self.clone();
        let context = JobContext {
            id: id.clone(),
            app: app.clone(),
            manager: manager.clone(),
            cancellation,
        };
        tauri::async_runtime::spawn(async move {
            let result = work(context.clone()).await;
            let cancelled = context.is_cancelled();
            let (status, error, event) = match result {
                Ok(()) if cancelled => (JobStatus::Cancelled, None, "job:done"),
                Ok(()) => (JobStatus::Done, None, "job:done"),
                Err(error) if cancelled || error.kind == "cancelled" => {
                    (JobStatus::Cancelled, None, "job:done")
                }
                Err(error) => (JobStatus::Failed, Some(error), "job:failed"),
            };
            if let Some(entry) = manager.entries.write().await.get_mut(&id) {
                entry.record.status = status;
                entry.record.error.clone_from(&error);
                if status == JobStatus::Done {
                    entry.record.fraction = 1.0;
                }
            }
            let _ = emit_job_finished(&app, event, id, status, error);
        });
        record
    }

    async fn list(&self) -> Vec<JobRecord> {
        let mut jobs: Vec<_> = self
            .entries
            .read()
            .await
            .values()
            .map(|entry| entry.record.clone())
            .collect();
        jobs.sort_by(|left, right| right.id.cmp(&left.id));
        jobs
    }

    async fn get(&self, id: &str) -> Option<JobRecord> {
        self.entries
            .read()
            .await
            .get(id)
            .map(|entry| entry.record.clone())
    }

    pub(crate) async fn cancel(&self, id: &str) -> bool {
        let entries = self.entries.read().await;
        let Some(entry) = entries.get(id) else {
            return false;
        };
        if entry.record.status != JobStatus::Running {
            return false;
        }
        entry.cancellation.cancel();
        true
    }
}

#[tauri::command]
pub async fn jobs_list(state: State<'_, AppState>) -> CommandResult<Vec<JobRecord>> {
    Ok(state.jobs.list().await)
}

#[tauri::command]
pub async fn jobs_get(id: String, state: State<'_, AppState>) -> CommandResult<JobRecord> {
    state
        .jobs
        .get(&id)
        .await
        .ok_or_else(|| SerializableError::new("not_found", format!("job {id} was not found")))
}

#[tauri::command]
pub async fn jobs_cancel(id: String, state: State<'_, AppState>) -> CommandResult<bool> {
    Ok(state.jobs.cancel(&id).await)
}

#[tauri::command]
pub async fn jobs_start_demo(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
    Ok(state
        .jobs
        .spawn(
            app,
            "bridge",
            "Verify in-process job bridge",
            |context| async move {
                context.log("Job started in the Rust process").await;
                for step in 1..=5 {
                    if context.is_cancelled() {
                        return Err(context.cancelled_error());
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(120)).await;
                    context
                        .progress(
                            f64::from(step) / 5.0,
                            Some(format!("Bridge check {step}/5")),
                        )
                        .await;
                    context.log(format!("Completed bridge step {step}")).await;
                }
                Ok(())
            },
        )
        .await)
}
