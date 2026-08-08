use crate::error::{CommandResult, SerializableError};

pub mod api;
pub mod automation;
pub mod diagnostics;
pub mod editor;
pub mod exports;
pub mod extensions;
pub mod git;
pub mod instances;
pub mod jobs;
pub mod mods;
pub mod packeater;
pub mod packs;
pub mod projects;
pub mod providers;
pub mod richtext;
pub mod settings;
pub mod shell;
pub mod themes;
pub mod workspace;

/// Runs blocking work off the webview's main thread.
///
/// A synchronous `#[tauri::command] fn` is answered inline on the thread that
/// drives the window, so any filesystem walk, subprocess, or hashing pass in
/// its body freezes the UI for its whole duration. Commands that do real work
/// take the `async fn` + `spawn_blocking` shape instead, and this is the
/// shared shell for the ones whose body is a single owned call.
///
/// Deliberately not `#[tauri::command(async)]` on the original sync function:
/// that attribute runs the blocking body on a Tokio *worker* thread rather
/// than a blocking one, which would starve the runtime that jobs, progress
/// events, and instance supervision share.
///
/// `State<'_, AppState>` is not `'static`, so callers resolve anything they
/// need from state (the workspace path, a pack root) *before* calling this
/// and move only owned values into `work`.
pub(crate) async fn off_thread<T, F>(work: F) -> CommandResult<T>
where
    T: Send + 'static,
    F: FnOnce() -> CommandResult<T> + Send + 'static,
{
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| SerializableError::new("task", error.to_string()))?
}
