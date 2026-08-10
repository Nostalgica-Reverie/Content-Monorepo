use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tauri::State;

use packwand_collab::protocol::{PROXYABLE_GIT_METHODS, Participant};

use crate::commands::off_thread;
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitDiffDocument {
	pub path: String,
	pub original: String,
	pub modified: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitRemote {
	pub name: String,
	pub url: String,
}

/// What the setup flow needs to decide which repository step to offer.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitRepository {
	pub is_repo: bool,
	/// The repository root, which is not necessarily the workspace directory.
	pub root: Option<String>,
	pub branch: Option<String>,
	pub remotes: Vec<GitRemote>,
	pub identity: GitIdentity,
}

/// `user.name` / `user.email` as git itself resolves them, including the
/// global config. Both are optional because a fresh install has neither, and
/// committing without them fails in a way worth catching early.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitIdentity {
	pub name: Option<String>,
	pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCommit {
	pub hash: String,
	pub short_hash: String,
	pub author: String,
	pub email: String,
	pub timestamp: i64,
	pub subject: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranches {
	pub current: String,
	pub local: Vec<String>,
	pub remote: Vec<String>,
}

/// Runs a `git` subprocess to completion.
///
/// Every caller reaches this from [`off_thread`], never from the webview
/// thread: `git status` on a large repository takes long enough that running
/// it inline froze the window on each Source Control refresh.
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
		packwand_platform::validate_relative_path(path)
			.map_err(|error| SerializableError::new("git_path", error.to_string()))?;
	}
	Ok(())
}

#[tauri::command]
pub async fn git_status(state: State<'_, AppState>) -> CommandResult<GitStatus> {
	// `State` is not `'static`, so the workspace path is resolved here and
	// only the owned `PathBuf` crosses into the blocking task.
	let workspace = state.workspace()?;
	off_thread(move || status_inner(&workspace)).await
}

fn status_inner(workspace: &Path) -> CommandResult<GitStatus> {
	let output = checked(
		workspace,
		&["status", "--porcelain=v1", "-z", "--untracked-files=all"],
	)?;
	let mut changes = parse_changes(&output.stdout);
	changes.sort_by(|left, right| left.path.cmp(&right.path));

	let branch_output = checked(workspace, &["branch", "--show-current"])?;
	let mut branch = String::from_utf8_lossy(&branch_output.stdout)
		.trim()
		.to_owned();
	if branch.is_empty() {
		let head = git(workspace, &["rev-parse", "--short", "HEAD"])?;
		branch = String::from_utf8_lossy(&head.stdout).trim().to_owned();
		if branch.is_empty() {
			branch = "No commits yet".into();
		}
	}

	let (ahead, behind) = git(
		workspace,
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
pub async fn git_stage(paths: Vec<String>, state: State<'_, AppState>) -> CommandResult<()> {
	validate_paths(&paths)?;
	let workspace = state.workspace()?;
	off_thread(move || {
		let mut args = vec!["add", "--"];
		args.extend(paths.iter().map(String::as_str));
		checked(&workspace, &args)?;
		Ok(())
	})
	.await
}

#[tauri::command]
pub async fn git_unstage(paths: Vec<String>, state: State<'_, AppState>) -> CommandResult<()> {
	validate_paths(&paths)?;
	let workspace = state.workspace()?;
	off_thread(move || {
		let mut args = vec!["restore", "--staged", "--"];
		args.extend(paths.iter().map(String::as_str));
		checked(&workspace, &args)?;
		Ok(())
	})
	.await
}

#[tauri::command]
pub async fn git_diff(
	path: String,
	staged: bool,
	state: State<'_, AppState>,
) -> CommandResult<String> {
	validate_paths(std::slice::from_ref(&path))?;
	let workspace = state.workspace()?;
	off_thread(move || {
		let mut args = vec!["diff", "--no-ext-diff"];
		if staged {
			args.push("--cached");
		}
		args.extend(["--", path.as_str()]);
		let output = checked(&workspace, &args)?;
		Ok(String::from_utf8_lossy(&output.stdout).into_owned())
	})
	.await
}

fn git_text(workspace: &Path, args: &[&str]) -> String {
	git(workspace, args)
		.ok()
		.filter(|output| output.status.success())
		.map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
		.unwrap_or_default()
}

#[tauri::command]
pub async fn git_diff_document(
	path: String,
	staged: bool,
	state: State<'_, AppState>,
) -> CommandResult<GitDiffDocument> {
	validate_paths(std::slice::from_ref(&path))?;
	let workspace = state.workspace()?;
	off_thread(move || diff_document_inner(&workspace, path, staged)).await
}

fn diff_document_inner(
	workspace: &Path,
	path: String,
	staged: bool,
) -> CommandResult<GitDiffDocument> {
	let head_spec = format!("HEAD:{path}");
	let index_spec = format!(":{path}");
	let original = if staged {
		git_text(workspace, &["show", head_spec.as_str()])
	} else {
		git_text(workspace, &["show", index_spec.as_str()])
	};
	let modified = if staged {
		git_text(workspace, &["show", index_spec.as_str()])
	} else {
		match fs::read(workspace.join(&path)) {
			Ok(bytes) => String::from_utf8(bytes).map_err(|_| {
				SerializableError::new("binary_file", "Monaco cannot display this binary diff")
			})?,
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
			Err(error) => return Err(error.into()),
		}
	};
	Ok(GitDiffDocument {
		path,
		original,
		modified,
	})
}

#[tauri::command]
pub async fn git_commit(
	message: String,
	co_authors: Option<Vec<Participant>>,
	state: State<'_, AppState>,
) -> CommandResult<String> {
	let co_authors =
		co_authors.unwrap_or_else(|| crate::commands::collab::commit_co_authors(&state));
	let message = commit_message(&message, &co_authors)?;
	let workspace = state.workspace()?;
	let result = off_thread(move || commit_inner(&workspace, &message)).await;
	if result.is_ok() {
		crate::commands::collab::clear_commit_co_authors(&state);
	}
	result
}

fn commit_inner(workspace: &Path, message: &str) -> CommandResult<String> {
	let output = checked(workspace, &["commit", "-m", message])?;
	Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn commit_message(message: &str, participants: &[Participant]) -> CommandResult<String> {
	let message = message.trim_end();
	if message.is_empty() {
		return Err(SerializableError::new(
			"git_commit",
			"enter a commit message",
		));
	}
	let mut identities = std::collections::BTreeSet::new();
	for participant in participants {
		let name = if participant.git_name.trim().is_empty() {
			participant.display_name.trim()
		} else {
			participant.git_name.trim()
		};
		let email = participant.git_email.trim();
		if !name.is_empty() && !email.is_empty() {
			identities.insert((name.to_owned(), email.to_owned()));
		}
	}
	if identities.is_empty() {
		return Ok(message.to_owned());
	}
	let trailers = identities
		.into_iter()
		.map(|(name, email)| format!("Co-authored-by: {name} <{email}>"))
		.collect::<Vec<_>>()
		.join("\n");
	Ok(format!("{message}\n\n{trailers}"))
}

/// Accepts a remote URL that is safe to hand to `git clone`.
///
/// Two distinct hazards, both of which have to be rejected before the URL
/// reaches the subprocess:
///
///  - A URL beginning with `-` is parsed by git as an *option*, not an
///    operand. `--upload-pack=<cmd>` is the classic case and runs `<cmd>`.
///    Callers also pass `--`, but belt and braces: a leading dash is never a
///    legitimate repository URL.
///  - The `ext::` transport is documented to execute an arbitrary shell
///    command, so it is a remote-code-execution primitive wearing a URL
///    costume. `git clone 'ext::sh -c whoami'` is all it takes.
///
/// Everything else is allowlisted rather than denylisted, because the set of
/// transports git supports grows and a denylist silently stops being complete.
fn validate_remote_url(url: &str) -> CommandResult<()> {
	let trimmed = url.trim();
	if trimmed.is_empty() {
		return Err(SerializableError::new(
			"git_remote_url",
			"enter a repository URL",
		));
	}
	if trimmed.starts_with('-') {
		return Err(SerializableError::new(
			"git_remote_url",
			"a repository URL cannot begin with '-'",
		));
	}
	const ALLOWED: [&str; 5] = ["https://", "http://", "ssh://", "git://", "git+ssh://"];
	if ALLOWED.iter().any(|scheme| trimmed.starts_with(scheme)) {
		return Ok(());
	}
	// scp-like shorthand: user@host:path/to/repo.git — no scheme, exactly the
	// form every forge prints next to the HTTPS one. Require a user@host part
	// so a bare Windows path (`C:\repos\x`) cannot slip through this branch.
	if let Some((authority, path)) = trimmed.split_once(':')
		&& authority.contains('@')
		&& !authority.contains('/')
		&& !path.is_empty()
	{
		return Ok(());
	}
	Err(SerializableError::new(
		"git_remote_url",
		"use an https://, ssh:// or user@host:path repository URL",
	))
}

/// Rejects a git ref or remote name that could be read as an option.
fn validate_name(kind: &'static str, value: &str) -> CommandResult<()> {
	let trimmed = value.trim();
	if trimmed.is_empty() || trimmed.starts_with('-') {
		return Err(SerializableError::new(
			kind,
			format!("{trimmed:?} is not a valid name"),
		));
	}
	if trimmed
		.chars()
		.any(|character| character.is_whitespace() || matches!(character, '~' | '^' | ':' | '\\'))
	{
		return Err(SerializableError::new(
			kind,
			format!("{trimmed:?} contains characters git does not allow"),
		));
	}
	Ok(())
}

fn config_value(workspace: &Path, key: &str) -> Option<String> {
	let value = git_text(workspace, &["config", "--get", key])
		.trim()
		.to_owned();
	(!value.is_empty()).then_some(value)
}

fn remotes_inner(workspace: &Path) -> Vec<GitRemote> {
	git_text(workspace, &["remote", "-v"])
		.lines()
		.filter_map(|line| {
			let (name, rest) = line.split_once('\t')?;
			let (url, _) = rest.rsplit_once(' ')?;
			Some(GitRemote {
				name: name.to_owned(),
				url: url.to_owned(),
			})
		})
		// `remote -v` prints a fetch and a push line per remote; they are
		// almost always the same URL and two identical rows help nobody.
		.fold(Vec::new(), |mut unique, remote| {
			if !unique.iter().any(|existing: &GitRemote| {
				existing.name == remote.name && existing.url == remote.url
			}) {
				unique.push(remote);
			}
			unique
		})
}

#[tauri::command]
pub async fn git_repository(state: State<'_, AppState>) -> CommandResult<GitRepository> {
	let workspace = state.workspace()?;
	off_thread(move || Ok(repository_inner(&workspace))).await
}

fn repository_inner(workspace: &Path) -> GitRepository {
	let root = git_text(workspace, &["rev-parse", "--show-toplevel"])
		.trim()
		.to_owned();
	let identity = GitIdentity {
		name: config_value(workspace, "user.name"),
		email: config_value(workspace, "user.email"),
	};
	if root.is_empty() {
		return GitRepository {
			is_repo: false,
			root: None,
			branch: None,
			remotes: Vec::new(),
			identity,
		};
	}
	let branch = git_text(workspace, &["branch", "--show-current"])
		.trim()
		.to_owned();
	GitRepository {
		is_repo: true,
		root: Some(root.replace('\\', "/")),
		branch: (!branch.is_empty()).then_some(branch),
		remotes: remotes_inner(workspace),
		identity,
	}
}

pub(crate) fn resolved_identity(workspace: &Path) -> GitIdentity {
	repository_inner(workspace).identity
}

#[tauri::command]
pub async fn git_init(state: State<'_, AppState>) -> CommandResult<GitRepository> {
	let workspace = state.workspace()?;
	off_thread(move || {
		if repository_inner(&workspace).is_repo {
			return Err(SerializableError::new(
				"git_init",
				"this directory is already a git repository",
			));
		}
		checked(&workspace, &["init"])?;
		Ok(repository_inner(&workspace))
	})
	.await
}

/// Clones `url` into `directory`, which must be empty or absent.
///
/// The emptiness check is ours rather than git's: `git clone` into a
/// non-empty directory fails with a message about the *destination path*
/// existing, which reads like a bug in the app rather than a choice the user
/// needs to make.
#[tauri::command]
pub async fn git_clone(url: String, directory: String) -> CommandResult<String> {
	validate_remote_url(&url)?;
	off_thread(move || {
		let target = std::path::PathBuf::from(&directory);
		if target.is_dir()
			&& target
				.read_dir()
				.map(|mut entries| entries.next().is_some())
				.unwrap_or(false)
		{
			return Err(SerializableError::new(
				"git_clone",
				"choose an empty directory to clone into",
			));
		}
		let parent = target.parent().ok_or_else(|| {
			SerializableError::new("git_clone", "choose a directory inside an existing folder")
		})?;
		fs::create_dir_all(parent)?;
		let name = target
			.file_name()
			.and_then(|value| value.to_str())
			.ok_or_else(|| SerializableError::new("git_clone", "invalid destination directory"))?;
		checked(parent, &["clone", "--", url.trim(), name])?;
		Ok(target.to_string_lossy().replace('\\', "/"))
	})
	.await
}

#[tauri::command]
pub async fn git_remote_add(
	name: String,
	url: String,
	state: State<'_, AppState>,
) -> CommandResult<Vec<GitRemote>> {
	validate_name("git_remote_name", &name)?;
	validate_remote_url(&url)?;
	let workspace = state.workspace()?;
	off_thread(move || {
		let existing = remotes_inner(&workspace);
		let args = if existing.iter().any(|remote| remote.name == name.trim()) {
			["remote", "set-url", name.trim(), url.trim()]
		} else {
			["remote", "add", name.trim(), url.trim()]
		};
		checked(&workspace, &args)?;
		Ok(remotes_inner(&workspace))
	})
	.await
}

#[tauri::command]
pub async fn git_set_identity(
	name: String,
	email: String,
	state: State<'_, AppState>,
) -> CommandResult<GitIdentity> {
	let workspace = state.workspace()?;
	off_thread(move || {
		// `--local`: this writes into the repository the user just connected,
		// never their global git config. Silently changing a global identity
		// from inside a modpack tool would be indefensible.
		checked(&workspace, &["config", "--local", "user.name", name.trim()])?;
		checked(
			&workspace,
			&["config", "--local", "user.email", email.trim()],
		)?;
		Ok(GitIdentity {
			name: config_value(&workspace, "user.name"),
			email: config_value(&workspace, "user.email"),
		})
	})
	.await
}

#[tauri::command]
pub async fn git_fetch(state: State<'_, AppState>) -> CommandResult<()> {
	let workspace = state.workspace()?;
	off_thread(move || {
		checked(&workspace, &["fetch", "--prune"])?;
		Ok(())
	})
	.await
}

#[tauri::command]
pub async fn git_pull(state: State<'_, AppState>) -> CommandResult<String> {
	let workspace = state.workspace()?;
	off_thread(move || {
		// `--ff-only`: a merge commit created by a background pull is a merge
		// the user did not ask for and cannot see coming. Diverged branches
		// have to be resolved deliberately.
		let output = checked(&workspace, &["pull", "--ff-only"])?;
		Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
	})
	.await
}

#[tauri::command]
pub async fn git_push(state: State<'_, AppState>) -> CommandResult<String> {
	let workspace = state.workspace()?;
	off_thread(move || {
		let branch = git_text(&workspace, &["branch", "--show-current"])
			.trim()
			.to_owned();
		if branch.is_empty() {
			return Err(SerializableError::new(
				"git_push",
				"cannot push from a detached HEAD",
			));
		}
		// Set upstream on first push so the branch tracks, without ever
		// forcing: `--force-with-lease` is still a rewrite the user has to
		// choose, and this path is not where they would choose it.
		let output = checked(
			&workspace,
			&["push", "--set-upstream", "origin", branch.as_str()],
		)?;
		let text = String::from_utf8_lossy(&output.stderr);
		Ok(text.trim().to_owned())
	})
	.await
}

#[tauri::command]
pub async fn git_branches(state: State<'_, AppState>) -> CommandResult<GitBranches> {
	let workspace = state.workspace()?;
	off_thread(move || {
		let current = git_text(&workspace, &["branch", "--show-current"])
			.trim()
			.to_owned();
		let local = git_text(
			&workspace,
			&["for-each-ref", "--format=%(refname:short)", "refs/heads"],
		)
		.lines()
		.map(str::to_owned)
		.collect();
		let remote = git_text(
			&workspace,
			&["for-each-ref", "--format=%(refname:short)", "refs/remotes"],
		)
		.lines()
		.filter(|name| !name.ends_with("/HEAD"))
		.map(str::to_owned)
		.collect();
		Ok(GitBranches {
			current,
			local,
			remote,
		})
	})
	.await
}

#[tauri::command]
pub async fn git_checkout(branch: String, state: State<'_, AppState>) -> CommandResult<()> {
	validate_name("git_branch", &branch)?;
	let workspace = state.workspace()?;
	off_thread(move || {
		// `switch`, not `checkout`: `checkout <name>` is ambiguous when a file
		// and a branch share a name, and `checkout -- <name>` means the file,
		// which would silently discard the user's edits instead of moving
		// branch. `switch` only ever means the branch. The leading-dash
		// rejection in `validate_name` is what makes passing it bare safe.
		checked(&workspace, &["switch", branch.trim()])?;
		Ok(())
	})
	.await
}

/// Recent commits, newest first.
///
/// Fields are separated by unit separator (0x1f) and records by record
/// separator (0x1e) rather than by anything printable: a commit subject can
/// contain any character a human can type, including whatever delimiter looked
/// safe at the time.
#[tauri::command]
pub async fn git_log(
	limit: Option<u32>,
	state: State<'_, AppState>,
) -> CommandResult<Vec<GitCommit>> {
	let workspace = state.workspace()?;
	let limit = limit.unwrap_or(50).clamp(1, 500);
	off_thread(move || {
		let count = format!("--max-count={limit}");
		let output = git(
			&workspace,
			&[
				"log",
				count.as_str(),
				"--format=%H\x1f%h\x1f%an\x1f%ae\x1f%at\x1f%s\x1e",
			],
		)?;
		// A repository with no commits exits non-zero here; that is an empty
		// log, not an error worth surfacing.
		if !output.status.success() {
			return Ok(Vec::new());
		}
		Ok(parse_log(&String::from_utf8_lossy(&output.stdout)))
	})
	.await
}

fn parse_log(text: &str) -> Vec<GitCommit> {
	text.split('\x1e')
		.map(str::trim)
		.filter(|record| !record.is_empty())
		.filter_map(|record| {
			let mut fields = record.split('\x1f');
			Some(GitCommit {
				hash: fields.next()?.to_owned(),
				short_hash: fields.next()?.to_owned(),
				author: fields.next()?.to_owned(),
				email: fields.next()?.to_owned(),
				timestamp: fields.next()?.parse().unwrap_or_default(),
				subject: fields.next()?.to_owned(),
			})
		})
		.collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathsParameters {
	paths: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiffParameters {
	path: String,
	staged: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommitParameters {
	message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogParameters {
	limit: Option<u32>,
}

fn remote_parameters<T: DeserializeOwned>(value: &serde_json::Value) -> CommandResult<T> {
	serde_json::from_value(value.clone())
		.map_err(|error| SerializableError::new("collab_protocol", error.to_string()))
}

pub(crate) fn remote_git_dispatch(
	workspace: &Path,
	method: &str,
	parameters: &serde_json::Value,
	allow_git_write: bool,
	co_authors: &[Participant],
) -> CommandResult<serde_json::Value> {
	let Some((_, requires_write)) = PROXYABLE_GIT_METHODS
		.iter()
		.find(|(candidate, _)| *candidate == method)
	else {
		return Err(SerializableError::new(
			"collab_protocol",
			format!("git method {method:?} is not proxyable"),
		));
	};
	if *requires_write && !allow_git_write {
		return Err(SerializableError::new(
			"collab_permission",
			"the host has disabled guest git writes",
		));
	}

	match method {
		"git_status" => Ok(serde_json::to_value(status_inner(workspace)?)?),
		"git_stage" => {
			let input: PathsParameters = remote_parameters(parameters)?;
			validate_paths(&input.paths)?;
			let mut args = vec!["add", "--"];
			args.extend(input.paths.iter().map(String::as_str));
			checked(workspace, &args)?;
			Ok(serde_json::Value::Null)
		}
		"git_unstage" => {
			let input: PathsParameters = remote_parameters(parameters)?;
			validate_paths(&input.paths)?;
			let mut args = vec!["restore", "--staged", "--"];
			args.extend(input.paths.iter().map(String::as_str));
			checked(workspace, &args)?;
			Ok(serde_json::Value::Null)
		}
		"git_diff" => {
			let input: DiffParameters = remote_parameters(parameters)?;
			validate_paths(std::slice::from_ref(&input.path))?;
			let mut args = vec!["diff", "--no-ext-diff"];
			if input.staged {
				args.push("--cached");
			}
			args.extend(["--", input.path.as_str()]);
			let output = checked(workspace, &args)?;
			Ok(serde_json::to_value(
				String::from_utf8_lossy(&output.stdout).into_owned(),
			)?)
		}
		"git_diff_document" => {
			let input: DiffParameters = remote_parameters(parameters)?;
			validate_paths(std::slice::from_ref(&input.path))?;
			Ok(serde_json::to_value(diff_document_inner(
				workspace,
				input.path,
				input.staged,
			)?)?)
		}
		"git_commit" => {
			let input: CommitParameters = remote_parameters(parameters)?;
			let message = commit_message(&input.message, co_authors)?;
			Ok(serde_json::to_value(commit_inner(workspace, &message)?)?)
		}
		"git_repository" => Ok(serde_json::to_value(repository_inner(workspace))?),
		"git_branches" => {
			let current = git_text(workspace, &["branch", "--show-current"])
				.trim()
				.to_owned();
			let local = git_text(
				workspace,
				&["for-each-ref", "--format=%(refname:short)", "refs/heads"],
			)
			.lines()
			.map(str::to_owned)
			.collect();
			let remote = git_text(
				workspace,
				&["for-each-ref", "--format=%(refname:short)", "refs/remotes"],
			)
			.lines()
			.filter(|name| !name.ends_with("/HEAD"))
			.map(str::to_owned)
			.collect();
			Ok(serde_json::to_value(GitBranches {
				current,
				local,
				remote,
			})?)
		}
		"git_log" => {
			let input: LogParameters = remote_parameters(parameters)?;
			let count = format!("--max-count={}", input.limit.unwrap_or(50).clamp(1, 500));
			let output = git(
				workspace,
				&[
					"log",
					count.as_str(),
					"--format=%H\x1f%h\x1f%an\x1f%ae\x1f%at\x1f%s\x1e",
				],
			)?;
			let commits = if output.status.success() {
				parse_log(&String::from_utf8_lossy(&output.stdout))
			} else {
				Vec::new()
			};
			Ok(serde_json::to_value(commits)?)
		}
		_ => unreachable!("the allowlist and dispatcher must stay exhaustive"),
	}
}

#[cfg(test)]
mod tests {
	use super::{commit_message, parse_changes, parse_log, validate_name, validate_remote_url};
	use packwand_collab::protocol::{Participant, ParticipantId};

	fn participant(id: u64, name: &str, email: &str) -> Participant {
		Participant {
			id: ParticipantId(id),
			display_name: name.to_owned(),
			git_name: name.to_owned(),
			git_email: email.to_owned(),
		}
	}

	#[test]
	fn formats_co_author_trailers_with_one_blank_line() {
		let message = commit_message(
			"ship multiplayer",
			&[participant(2, "Guest", "guest@example.com")],
		)
		.unwrap();
		assert_eq!(
			message,
			"ship multiplayer\n\nCo-authored-by: Guest <guest@example.com>"
		);
	}

	#[test]
	fn co_author_trailers_are_deduplicated() {
		let guest = participant(2, "Guest", "guest@example.com");
		let message = commit_message("change", &[guest.clone(), guest]).unwrap();
		assert_eq!(message.matches("Co-authored-by:").count(), 1);
	}

	#[test]
	fn trailing_whitespace_does_not_create_extra_blank_lines() {
		let guest = participant(2, "Guest", "guest@example.com");
		assert_eq!(
			commit_message("change\n\n  ", &[guest]).unwrap(),
			"change\n\nCo-authored-by: Guest <guest@example.com>"
		);
	}

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

	#[test]
	fn accepts_the_url_forms_forges_actually_print() {
		for url in [
			"https://git.nostalgica.net/Reverie-Projects/monorepo.git",
			"http://localhost:3000/x/y.git",
			"ssh://git@github.com/omo50/thing.git",
			"git://example.com/thing.git",
			"git@github.com:omo50/thing.git",
		] {
			assert!(validate_remote_url(url).is_ok(), "should accept {url}");
		}
	}

	/// `ext::` runs an arbitrary command, and a leading dash is read as an
	/// option rather than an operand. Both are remote code execution.
	#[test]
	fn rejects_urls_that_would_execute_a_command() {
		for url in [
			"ext::sh -c whoami",
			"--upload-pack=calc.exe",
			"-u",
			"",
			"   ",
			"C:\\Users\\me\\repo",
			"/tmp/repo",
			"file:///tmp/repo",
		] {
			assert!(validate_remote_url(url).is_err(), "should reject {url:?}");
		}
	}

	#[test]
	fn rejects_ref_names_that_look_like_options_or_revisions() {
		for name in [
			"",
			"-f",
			"--force",
			"with space",
			"a~1",
			"a^",
			"a:b",
			"a\\b",
		] {
			assert!(
				validate_name("git_branch", name).is_err(),
				"should reject {name:?}"
			);
		}
		assert!(validate_name("git_branch", "feature/thing-1").is_ok());
	}

	/// Subjects can contain anything a human types, so the delimiters are
	/// non-printable and the parser must not be confused by punctuation.
	#[test]
	fn parses_log_records_with_awkward_subjects() {
		let text = "aaa\x1fa\x1fA U Thor\x1fa@b.c\x1f1700000000\x1ffix: a|b\ttab and \"quotes\"\x1e\
                    bbb\x1fb\x1fB\x1fb@c.d\x1f1700000001\x1fdocs\x1e";
		let parsed = parse_log(text);
		assert_eq!(parsed.len(), 2);
		assert_eq!(parsed[0].subject, "fix: a|b\ttab and \"quotes\"");
		assert_eq!(parsed[0].timestamp, 1_700_000_000);
		assert_eq!(parsed[1].short_hash, "b");
	}

	#[test]
	fn parses_an_empty_log_as_no_commits() {
		assert!(parse_log("").is_empty());
		assert!(parse_log("\n").is_empty());
	}
}
