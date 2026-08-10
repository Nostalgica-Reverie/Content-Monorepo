use packwand_vcs::{StackEntry, VcsError};
use tauri::State;

use crate::commands::off_thread;
use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

#[tauri::command]
pub async fn changes_enable(state: State<'_, AppState>) -> CommandResult<()> {
	let root = state.workspace()?;
	let tool_root = state.tool_root();
	off_thread(move || {
		ensure_tool(tool_root)?;
		packwand_vcs::enable_colocated(&root).map_err(vcs_error)
	})
	.await
}

#[tauri::command]
pub async fn changes_log(state: State<'_, AppState>) -> CommandResult<Vec<StackEntry>> {
	let root = state.workspace()?;
	let tool_root = state.tool_root();
	off_thread(move || {
		ensure_tool(tool_root)?;
		packwand_vcs::stack_log(&root).map_err(vcs_error)
	})
	.await
}

#[tauri::command]
pub async fn changes_new(
	parent: Option<String>,
	state: State<'_, AppState>,
) -> CommandResult<StackEntry> {
	let root = state.workspace()?;
	let tool_root = state.tool_root();
	off_thread(move || {
		ensure_tool(tool_root)?;
		packwand_vcs::new_change(&root, parent.as_deref()).map_err(vcs_error)
	})
	.await
}

#[tauri::command]
pub async fn changes_describe(
	change_id: String,
	message: String,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	let root = state.workspace()?;
	let tool_root = state.tool_root();
	off_thread(move || {
		ensure_tool(tool_root)?;
		packwand_vcs::describe(&root, &change_id, &message).map_err(vcs_error)
	})
	.await
}

#[tauri::command]
pub async fn changes_squash(
	change_id: String,
	into_parent: bool,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	let root = state.workspace()?;
	let tool_root = state.tool_root();
	off_thread(move || {
		ensure_tool(tool_root)?;
		packwand_vcs::squash(&root, &change_id, into_parent).map_err(vcs_error)
	})
	.await
}

fn ensure_tool(root: std::path::PathBuf) -> CommandResult<()> {
	let request = packwand_devboot::jj_toolchain::JjToolchainRequest::pinned(root);
	let binary = packwand_devboot::jj_toolchain::ensure_jj(&request, |_| {})
		.map_err(|error| SerializableError::new("vcs_tool", error.to_string()))?;
	packwand_vcs::configure_jj_binary(binary).map_err(vcs_error)
}

fn vcs_error(error: VcsError) -> SerializableError {
	let kind = match error {
		VcsError::NotInitialized(_) => "vcs_not_initialized",
		VcsError::JjNotFound => "vcs_tool_missing",
		VcsError::Concurrent(_) => "vcs_concurrent",
		VcsError::Divergent { .. } => "vcs_divergent",
		VcsError::InvalidInput(_) => "vcs_invalid_input",
		VcsError::Command(_)
		| VcsError::InvalidOutput(_)
		| VcsError::Library(_)
		| VcsError::Io(_) => "vcs",
	};
	SerializableError::new(kind, error.to_string())
}
