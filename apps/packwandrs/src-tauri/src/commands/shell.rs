//! pw4shell: the packwand command console (packwandc.md 5.8).
//!
//! The language lives in the C kernel, not here. This module is the seam
//! between the webview, the kernel parser, and the trusted CLI dispatcher.
//!
//! The kernel handles `help`, `version`, `echo`, `status` and `trace` itself.
//! Anything else comes back as [`ShellOutcome::ForHost`] with the parsed words,
//! and are passed as argv to Packwand's bundled CLI in a validated workspace
//! folder. No line is handed to an operating-system shell and no executable is
//! selected from user input or PATH.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

/// One line of console output, tagged for the dock's tone styling.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellLine {
    pub text: String,
    /// `info`, `error` or `success`, matching the output dock's tones.
    pub tone: String,
}

impl ShellLine {
    fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: "info".into(),
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: "error".into(),
        }
    }
}

/// The result of running one console line.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellResult {
    pub lines: Vec<ShellLine>,
    /// True when a kernel built-in or a host verb handled the line.
    pub handled: bool,
}

/// Run one pw4shell line.
///
/// # Errors
///
/// Returns an error only when the native core cannot be reached. A malformed
/// line or an unknown verb is reported as output, not as a command failure —
/// a console that raises an exception on a typo is unusable.
#[tauri::command]
pub async fn shell_exec(
    app: AppHandle,
    state: State<'_, AppState>,
    line: String,
    cwd: Option<String>,
) -> CommandResult<ShellResult> {
    // A fresh port per line. Ports are a fixed pool of eight, and `Port`
    // releases its ring slot on drop, so this cannot leak the pool away.
    let port = packwandc::Port::open()
        .map_err(|error| SerializableError::new("packwandc", error.to_string()))?;

    match packwandc::shell_exec(&line, Some(&port)) {
        Ok(packwandc::ShellOutcome::Handled) => {
            let lines = port
                .drain_lines()
                .map_err(|error| SerializableError::new("packwandc", error.to_string()))?
                .into_iter()
                .map(ShellLine::info)
                .collect();
            Ok(ShellResult {
                lines,
                handled: true,
            })
        }
        Ok(packwandc::ShellOutcome::Empty) => Ok(ShellResult {
            lines: Vec::new(),
            handled: true,
        }),
        Ok(packwandc::ShellOutcome::ForHost(words)) => {
            let workspace = state.workspace()?;
            let cwd = validated_cwd(&workspace, cwd.as_deref())?;
            let executable = bundled_cli(&app)?;
            tauri::async_runtime::spawn_blocking(move || dispatch_cli(&executable, &cwd, &words))
                .await
                .map_err(|error| SerializableError::new("shell_join", error.to_string()))?
        }
        Err(error) => Ok(ShellResult {
            // The kernel's Error carries the detail record — which module
            // rejected the line, and why — so this is a real diagnostic rather
            // than "syntax error".
            lines: vec![ShellLine::error(error.to_string())],
            handled: false,
        }),
    }
}

/// Tokenise a line without running it, using the kernel's own quoting rules.
///
/// Exposed so the console can preview or complete a line without the frontend
/// reimplementing quoting in TypeScript, where it would drift from C.
///
/// # Errors
///
/// Returns an error if the line is malformed.
#[tauri::command]
pub fn shell_parse(line: String) -> CommandResult<Vec<String>> {
    packwandc::shell_parse(&line)
        .map_err(|error| SerializableError::new("packwandc", error.to_string()))
}

fn bundled_cli(app: &AppHandle) -> CommandResult<PathBuf> {
    let name = if cfg!(windows) {
        "packwand.exe"
    } else {
        "packwand"
    };
    let mut candidates = Vec::new();
    if let Ok(root) = app.path().resource_dir() {
        candidates.push(root.join(name));
        candidates.push(root.join("resources").join(name));
    }
    if let Ok(current) = std::env::current_exe()
        && let Some(parent) = current.parent()
    {
        candidates.push(parent.join(name));
    }
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            SerializableError::new("cli_unavailable", "the bundled Packwand CLI was not found")
        })
}

/// Restrict the terminal cwd to the configured workspace. Canonicalising both
/// sides also prevents a project symlink from escaping that boundary.
fn validated_cwd(workspace: &Path, requested: Option<&str>) -> CommandResult<PathBuf> {
    let workspace = workspace.canonicalize().map_err(|error| {
        SerializableError::new(
            "workspace_unavailable",
            format!("cannot open workspace {}: {error}", workspace.display()),
        )
    })?;
    let requested = requested
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                workspace.join(path)
            }
        })
        .unwrap_or_else(|| workspace.clone());
    let requested = requested.canonicalize().map_err(|error| {
        SerializableError::new(
            "folder_unavailable",
            format!(
                "cannot open terminal folder {}: {error}",
                requested.display()
            ),
        )
    })?;
    if !requested.starts_with(&workspace) {
        return Err(SerializableError::new(
            "folder_outside_workspace",
            format!(
                "terminal folder {} is outside workspace {}",
                requested.display(),
                workspace.display()
            ),
        ));
    }
    Ok(requested)
}

/// Execute only the bundled CLI. A leading `packwand` is accepted for natural
/// copy/paste from documentation, but is not required inside pw4shell.
fn dispatch_cli(executable: &Path, cwd: &Path, words: &[String]) -> CommandResult<ShellResult> {
    let args = match words.first().map(String::as_str) {
        Some("packwand") => &words[1..],
        _ => words,
    };
    let output = Command::new(executable)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|error| SerializableError::new("cli_start", error.to_string()))?;

    let mut lines = text_lines(&output.stdout, "info");
    lines.extend(text_lines(&output.stderr, "error"));
    if lines.is_empty() {
        lines.push(if output.status.success() {
            ShellLine {
                text: "command completed".into(),
                tone: "success".into(),
            }
        } else {
            ShellLine::error(format!("packwand exited with {}", output.status))
        });
    }
    Ok(ShellResult {
        lines,
        handled: output.status.success(),
    })
}

fn text_lines(bytes: &[u8], tone: &str) -> Vec<ShellLine> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(|text| ShellLine {
            text: text.to_owned(),
            tone: tone.to_owned(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{text_lines, validated_cwd};

    #[test]
    fn terminal_folder_must_stay_inside_workspace() {
        let workspace = tempfile::tempdir().unwrap();
        let project = workspace.path().join("modpacks").join("example");
        std::fs::create_dir_all(&project).unwrap();
        assert_eq!(
            validated_cwd(workspace.path(), Some(project.to_str().unwrap())).unwrap(),
            project.canonicalize().unwrap()
        );
        assert_eq!(
            validated_cwd(workspace.path(), Some("modpacks/example")).unwrap(),
            project.canonicalize().unwrap()
        );

        let outside = tempfile::tempdir().unwrap();
        assert!(validated_cwd(workspace.path(), outside.path().to_str()).is_err());
    }

    #[test]
    fn cli_output_is_split_into_terminal_lines() {
        let lines = text_lines(b"one\ntwo\n", "info");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].text, "two");
    }
}
