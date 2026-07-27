use std::path::Path;
use std::process::{Command, Output};

use serde::Serialize;
use tauri::State;

use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitChange {
    pub path: String,
    pub index_status: String,
    pub worktree_status: String,
    pub staged: bool,
    pub untracked: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub changes: Vec<GitChange>,
}

fn git(workspace: &Path, args: &[&str]) -> CommandResult<Output> {
    Command::new("git")
        .args(args)
        .current_dir(workspace)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_LITERAL_PATHSPECS", "1")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.fsmonitor")
        .env("GIT_CONFIG_VALUE_0", "false")
        .output()
        .map_err(|error| SerializableError::new("git_unavailable", error.to_string()))
}

fn checked(workspace: &Path, args: &[&str]) -> CommandResult<Output> {
    let output = git(workspace, args)?;
    if output.status.success() {
        return Ok(output);
    }
    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(SerializableError::new(
        "git",
        if message.is_empty() {
            format!("git exited with {}", output.status)
        } else {
            message
        },
    ))
}

fn validate_paths(paths: &[String]) -> CommandResult<()> {
    if paths.is_empty() {
        return Err(SerializableError::new(
            "git_path",
            "select at least one file",
        ));
    }
    for path in paths {
        packwandc::validate_relative_path(path)
            .map_err(|error| SerializableError::new("git_path", error.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn git_status(state: State<'_, AppState>) -> CommandResult<GitStatus> {
    let workspace = state.workspace()?;
    let output = checked(
        &workspace,
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    )?;
    let mut changes = parse_changes(&output.stdout);
    changes.sort_by(|left, right| left.path.cmp(&right.path));

    let branch_output = checked(&workspace, &["branch", "--show-current"])?;
    let mut branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_owned();
    if branch.is_empty() {
        let head = git(&workspace, &["rev-parse", "--short", "HEAD"])?;
        branch = String::from_utf8_lossy(&head.stdout).trim().to_owned();
        if branch.is_empty() {
            branch = "No commits yet".into();
        }
    }

    let (ahead, behind) = git(
        &workspace,
        &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
    )
    .ok()
    .filter(|output| output.status.success())
    .and_then(|output| {
        let text = String::from_utf8_lossy(&output.stdout);
        let mut counts = text
            .split_whitespace()
            .filter_map(|value| value.parse::<usize>().ok());
        Some((counts.next()?, counts.next()?))
    })
    .unwrap_or((0, 0));

    Ok(GitStatus {
        branch,
        ahead,
        behind,
        changes,
    })
}

fn parse_changes(bytes: &[u8]) -> Vec<GitChange> {
    let records = bytes.split(|byte| *byte == 0).collect::<Vec<_>>();
    let mut changes = Vec::new();
    let mut index = 0usize;
    while index < records.len() {
        let record = records[index];
        index += 1;
        if record.len() < 4 {
            continue;
        }
        let index_status = record[0] as char;
        let worktree_status = record[1] as char;
        let path = String::from_utf8_lossy(&record[3..]).replace('\\', "/");
        changes.push(GitChange {
            path,
            index_status: index_status.to_string(),
            worktree_status: worktree_status.to_string(),
            staged: index_status != ' ' && index_status != '?',
            untracked: index_status == '?' && worktree_status == '?',
        });
        if matches!(index_status, 'R' | 'C') || matches!(worktree_status, 'R' | 'C') {
            index += 1;
        }
    }
    changes
}

#[tauri::command]
pub fn git_stage(paths: Vec<String>, state: State<'_, AppState>) -> CommandResult<()> {
    validate_paths(&paths)?;
    let workspace = state.workspace()?;
    let mut args = vec!["add", "--"];
    args.extend(paths.iter().map(String::as_str));
    checked(&workspace, &args)?;
    Ok(())
}

#[tauri::command]
pub fn git_unstage(paths: Vec<String>, state: State<'_, AppState>) -> CommandResult<()> {
    validate_paths(&paths)?;
    let workspace = state.workspace()?;
    let mut args = vec!["restore", "--staged", "--"];
    args.extend(paths.iter().map(String::as_str));
    checked(&workspace, &args)?;
    Ok(())
}

#[tauri::command]
pub fn git_diff(path: String, staged: bool, state: State<'_, AppState>) -> CommandResult<String> {
    validate_paths(std::slice::from_ref(&path))?;
    let workspace = state.workspace()?;
    let mut args = vec!["diff", "--no-ext-diff"];
    if staged {
        args.push("--cached");
    }
    args.extend(["--", path.as_str()]);
    let output = checked(&workspace, &args)?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[tauri::command]
pub fn git_commit(message: String, state: State<'_, AppState>) -> CommandResult<String> {
    let message = message.trim();
    if message.is_empty() {
        return Err(SerializableError::new(
            "git_commit",
            "enter a commit message",
        ));
    }
    let workspace = state.workspace()?;
    let output = checked(&workspace, &["commit", "-m", message])?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::parse_changes;

    #[test]
    fn parses_staged_modified_and_untracked_rows() {
        let parsed = parse_changes(b"M  src/main.rs\0 M README.md\0?? new file.txt\0");
        let staged = &parsed[0];
        assert!(staged.staged);
        assert!(!staged.untracked);
        let modified = &parsed[1];
        assert!(!modified.staged);
        let untracked = &parsed[2];
        assert!(untracked.untracked);
        assert_eq!(untracked.path, "new file.txt");
    }
}
