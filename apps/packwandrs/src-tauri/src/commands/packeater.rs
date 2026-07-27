use std::path::PathBuf;

use tauri::{AppHandle, State};

use crate::commands::jobs::JobRecord;
use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

/// One discovered `packeater.json` marker, relative to the workspace.
#[derive(Debug, serde::Serialize)]
pub struct PackeaterMarker {
    pub path: String,
    pub directory: String,
}

/// Lists the Packeater markers under a pack, so the UI can show what would be
/// compressed before anyone commits to a run.
#[tauri::command]
pub fn packeater_markers(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<PackeaterMarker>> {
    let workspace = state.workspace()?;
    let root = pack_root(&workspace, &id)?;
    let markers = packwand_build::discover_packeater_markers(&root)
        .map_err(|error| SerializableError::new("packeater", error.to_string()))?;
    Ok(markers
        .into_iter()
        .map(|marker| {
            let directory = marker
                .parent()
                .map(|parent| display_relative(&workspace, parent))
                .unwrap_or_default();
            PackeaterMarker {
                path: display_relative(&workspace, &marker),
                directory,
            }
        })
        .collect())
}

/// Compresses every Packeater marker under a pack. Runs as a job because a large
/// resource pack takes long enough that a blocking command would freeze the UI.
#[tauri::command]
// `output` is deliberately one word: every other command in this crate takes
// single-word parameters, so there is no in-repo precedent for how a multi-word
// name is cased across the IPC boundary.
pub async fn packeater_run(
    id: String,
    output: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
    let workspace = state.workspace()?;
    let root = pack_root(&workspace, &id)?;
    let destination = output
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("build"));
    let label = format!("Packeater {id}");
    Ok(state
        .jobs
        .spawn(app, "packeater.run", label, move |context| async move {
            let markers = tokio::task::spawn_blocking({
                let root = root.clone();
                move || packwand_build::discover_packeater_markers(&root)
            })
            .await
            .map_err(|error| SerializableError::new("task", error.to_string()))?
            .map_err(|error| SerializableError::new("packeater", error.to_string()))?;

            if markers.is_empty() {
                context
                    .log("No packeater.json markers found; nothing to compress")
                    .await;
                context.progress(1.0, Some("Nothing to do".into())).await;
                return Ok(());
            }

            let total = markers.len();
            context.log(format!("Compressing {total} marker(s)")).await;
            for (index, marker) in markers.into_iter().enumerate() {
                let name = marker
                    .parent()
                    .and_then(|parent| parent.file_name())
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "pack".into());
                let output = destination.join(format!("{name}.zip"));
                context.log(format!("packeater {name}")).await;
                let bytes = tokio::task::spawn_blocking({
                    let marker = marker.clone();
                    let output = output.clone();
                    move || packwand_build::run_packeater(marker, output)
                })
                .await
                .map_err(|error| SerializableError::new("task", error.to_string()))?
                .map_err(|error| SerializableError::new("packeater", error.to_string()))?;
                context
                    .log(format!("{name}: {bytes} byte(s) -> {}", output.display()))
                    .await;
                context
                    .progress(
                        (index + 1) as f64 / total as f64,
                        Some(format!("Compressed {}/{total}", index + 1)),
                    )
                    .await;
            }
            Ok(())
        })
        .await)
}

/// Workspace-relative, forward-slashed path for display.
fn display_relative(workspace: &std::path::Path, path: &std::path::Path) -> String {
    path.strip_prefix(workspace)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
