use serde::Serialize;
use serde_json::{Value, json};
use tauri::State;

use crate::commands::off_thread;
use crate::error::CommandResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiRoute {
	pub method: &'static str,
	pub path: &'static str,
	pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiContract {
	pub transport: &'static str,
	pub version: &'static str,
	pub routes: Vec<ApiRoute>,
}

fn routes() -> Vec<ApiRoute> {
	vec![
		ApiRoute {
			method: "GET",
			path: "/health",
			description: "Process and workspace health",
		},
		ApiRoute {
			method: "GET",
			path: "/api/v1/projects",
			description: "Typed project inventory",
		},
		ApiRoute {
			method: "GET",
			path: "/api/v1/commands",
			description: "Native API contract",
		},
		ApiRoute {
			method: "GET",
			path: "/api/v1/diagnostics",
			description: "Validation, lint, and parity snapshot",
		},
	]
}

#[tauri::command]
pub fn api_contract() -> ApiContract {
	ApiContract {
		transport: "tauri-ipc",
		version: "v1",
		routes: routes(),
	}
}

#[tauri::command]
pub async fn api_inspect(path: String, state: State<'_, AppState>) -> CommandResult<Value> {
	let root = state.workspace()?;
	// `/api/v1/diagnostics` runs the full validate + lint + parity sweep over
	// the workspace, which is far too slow to answer on the window's thread.
	off_thread(move || inspect_inner(&root, &path)).await
}

fn inspect_inner(root: &std::path::Path, path: &str) -> CommandResult<Value> {
	Ok(match path {
		"/health" => json!({"ok": true, "workspace": root, "runtime": "rust"}),
		"/api/v1/projects" => serde_json::to_value(packwand_workspace::discover(root)?)?,
		"/api/v1/commands" => serde_json::to_value(api_contract())?,
		"/api/v1/diagnostics" => json!({
			"validate": packwand_diagnostics::validate_projects(root)?,
			"lint": packwand_diagnostics::lint_workspace(root),
			"parity": packwand_diagnostics::parity_workspace(root)?,
		}),
		_ => {
			return Err(crate::error::SerializableError::new(
				"not_found",
				format!("unknown native API route {path}"),
			));
		}
	})
}
