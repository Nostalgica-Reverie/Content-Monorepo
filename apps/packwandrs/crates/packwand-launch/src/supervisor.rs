//! Process supervision for approved launch plans.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use command_group::CommandGroup;
use packwand_auth::SecretString;

use crate::censor::Censor;
use serde::Serialize;

use crate::plan::LaunchPlan;

/// Typed lifecycle events emitted by the supervisor.
///
/// Child output is censored before it becomes an event. This used to say
/// that no redaction was needed because the supervisor held no secrets, and
/// that was never quite true and is now plainly false: `resolve_spawn_inputs`
/// puts the access token into this very thread, and the game echoes its own
/// arguments — on a crash, and whenever a mod logs them. Since every consumer
/// of a launch reads it through these events, censoring here is what makes
/// "the token cannot reach a paste site" a property rather than a hope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum LaunchEvent {
	Starting {
		instance_id: String,
	},
	Started {
		instance_id: String,
		pid: u32,
	},
	Stdout {
		instance_id: String,
		line: String,
	},
	Stderr {
		instance_id: String,
		line: String,
	},
	Exited {
		instance_id: String,
		code: Option<i32>,
	},
	Failed {
		instance_id: String,
		error: String,
	},
	Cancelled {
		instance_id: String,
	},
}

/// Environment variables removed from the child before it starts.
///
/// Every one of these injects JVM options or classpath entries from outside
/// the launch plan, so a machine with `_JAVA_OPTIONS` set produces a run that
/// no plan describes and no bug report can reproduce. Dropping them makes the
/// plan the only thing that decides how the JVM starts.
///
/// `JDK_JAVA_OPTIONS` is not on the list this was ported from, which predates
/// it. It matters more here than the rest: it is the one variable that can
/// carry `--disable-@files`, which would break the argument file below.
///
/// Removing `JAVA_HOME` and `JRE_HOME` is safe because `plan.java_executable`
/// is an absolute path — nothing downstream resolves the JVM by variable.
const STRIPPED_ENV: [&str; 8] = [
	"_JAVA_OPTIONS",
	"JAVA_TOOL_OPTIONS",
	"JDK_JAVA_OPTIONS",
	"JAVA_OPTIONS",
	"JAVA_ARGS",
	"CLASSPATH",
	"JAVA_HOME",
	"JRE_HOME",
];

/// Cooperative cancellation flag shared with a running launch.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
	/// Creates a new, non-cancelled cancellation token.
	pub fn new() -> Self {
		Self::default()
	}

	/// Requests cancellation of an associated launch.
	pub fn cancel(&self) {
		self.0.store(true, Ordering::SeqCst);
	}

	/// Checks whether cancellation has been requested.
	pub fn is_cancelled(&self) -> bool {
		self.0.load(Ordering::SeqCst)
	}
}

#[derive(Debug, Default)]
pub struct LaunchOptions {
	/// Permit this launch even while another run of the same instance holds
	/// the run lock. Off by default.
	pub allow_concurrent: bool,
	/// Externally supplied cancellation token (may already be cancelled).
	/// When absent a fresh token is created.
	pub cancel: Option<CancellationToken>,
	/// Values for the plan's `${secret:<name>}` placeholders, resolved
	/// into the command line and environment only at spawn time. The plan
	/// itself, events, and logs never carry the raw values.
	pub secrets: BTreeMap<String, SecretString>,
	/// Values for the plan's `${identity:<name>}` placeholders — player name,
	/// uuid, user type. Also resolved at spawn, but not sensitive.
	pub identity: BTreeMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
	#[error("instance {0:?} is already running")]
	AlreadyRunning(String),
	#[error("failed to create run lock {path}: {source}")]
	Lock {
		path: PathBuf,
		source: std::io::Error,
	},
}

/// Exclusive run lock; the file is removed on every exit path (success,
/// failure, and cancellation) via `Drop`.
struct RunLock {
	path: Option<PathBuf>,
}

impl RunLock {
	fn acquire(path: PathBuf, instance_id: &str) -> Result<Self, LaunchError> {
		match fs::OpenOptions::new()
			.write(true)
			.create_new(true)
			.open(&path)
		{
			Ok(_) => Ok(Self { path: Some(path) }),
			Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
				Err(LaunchError::AlreadyRunning(instance_id.to_string()))
			}
			Err(source) => Err(LaunchError::Lock { path, source }),
		}
	}

	fn none() -> Self {
		Self { path: None }
	}
}

impl Drop for RunLock {
	fn drop(&mut self) {
		if let Some(path) = self.path.take() {
			let _ = fs::remove_file(path);
		}
	}
}

/// Escapes one argument for a JVM argument file.
///
/// Argument files are not argv: the launcher re-splits them on whitespace and
/// processes escape sequences inside quoted tokens. A Windows classpath is the
/// case that punishes getting this wrong — `C:\libs\a.jar` would have `\l` and
/// `\a` eaten as escapes, silently producing a classpath that points nowhere.
/// Quoting unconditionally and escaping the backslash is what keeps a path
/// round-tripping.
///
/// Literal newlines are escaped rather than emitted: a quoted token cannot
/// span lines in an argument file, so a raw newline would split one argument
/// into two.
fn quote_argfile_token(value: &str) -> String {
	let mut out = String::with_capacity(value.len() + 2);
	out.push('"');
	for ch in value.chars() {
		match ch {
			'\\' => out.push_str("\\\\"),
			'"' => out.push_str("\\\""),
			'\n' => out.push_str("\\n"),
			'\r' => out.push_str("\\r"),
			'\t' => out.push_str("\\t"),
			'\u{c}' => out.push_str("\\f"),
			_ => out.push(ch),
		}
	}
	out.push('"');
	out
}

/// The full argument vector rendered as JVM argument-file content.
fn render_argfile(args: &[String]) -> String {
	let mut out = String::new();
	for arg in args {
		out.push_str(&quote_argfile_token(arg));
		out.push('\n');
	}
	out
}

/// Whether the child JVM can be handed an argument file at all.
///
/// Two thresholds, both about the *child* rather than the JDK this launcher
/// was built against — `plan.java_executable` is whatever the game requires,
/// and Minecraft 1.16 and earlier still run on Java 8:
///
/// * **9** is when `@argfile` was added. An 8 would read `@/path/to/file` as
///   a class name and fail to start, which is a worse outcome than the
///   exposure this is avoiding.
/// * **18** is JEP 400, when the launcher began decoding argument files as
///   UTF-8. Before it, the file is read in the platform encoding — CP-1252 on
///   a Western Windows install — so a non-ASCII player name or install path
///   arrives corrupted. Restricting the file to ASCII content on those
///   versions keeps the common case protected without risking the rest.
///
/// `None` means the JVM could not be identified, which falls back to the
/// command line: an unlaunchable game is worse than a visible token.
fn argfile_is_usable(child_major: Option<u32>, rendered: &str) -> bool {
	match child_major {
		Some(major) if major >= 18 => true,
		Some(major) if major >= 9 => rendered.is_ascii(),
		_ => false,
	}
}

/// The feature release of the JVM about to be launched, if it can be
/// identified. Cached process-wide, so this costs nothing after the first
/// launch of a given JVM.
fn child_major_version(executable: &Path) -> Option<u32> {
	packwand_runtime::ProbeCache::shared()
		.probe(executable)
		.ok()
		.map(|probed| probed.version.major)
}

/// A JVM argument file holding the resolved argument vector, removed on every
/// exit path via `Drop`.
///
/// This exists so the access token stays off the command line, where `ps` and
/// Task Manager expose it to every other user on the machine. The file is the
/// narrower exposure — same-user only — but it is not free: the token sits in
/// it for as long as the game runs, because there is no moment at which the
/// JVM can be observed to have finished reading it. Deleting on child exit
/// trades a race for a longer window deliberately; a delete-after-timer would
/// keep the window short but can truncate a slow JVM's startup.
struct ArgFile {
	path: PathBuf,
}

impl ArgFile {
	/// Writes rendered argument-file `content` to a fresh file beside the run
	/// lock.
	///
	/// The name carries a process-unique counter so two concurrent launches of
	/// one instance (`allow_concurrent`) cannot overwrite each other's file.
	fn write(dir: &Path, content: &str) -> std::io::Result<Self> {
		static NEXT: AtomicU64 = AtomicU64::new(0);
		let n = NEXT.fetch_add(1, Ordering::Relaxed);
		let path = dir.join(format!(".packwand-args-{}-{n}", std::process::id()));
		let mut options = fs::OpenOptions::new();
		options.write(true).create_new(true);
		// Owner-only: the point of the file is that it is less exposed than
		// argv, which a mode-0644 file next to the game would not be.
		#[cfg(unix)]
		{
			use std::os::unix::fs::OpenOptionsExt;
			options.mode(0o600);
		}
		let mut file = options.open(&path)?;
		file.write_all(content.as_bytes())?;
		file.flush()?;
		Ok(Self { path })
	}

	/// The `@file` argument handed to the JVM.
	fn argument(&self) -> String {
		format!("@{}", self.path.display())
	}
}

impl Drop for ArgFile {
	fn drop(&mut self) {
		let _ = fs::remove_file(&self.path);
	}
}

/// A running (or finished) launch: an event stream plus cancellation.
pub struct LaunchHandle {
	events: Receiver<LaunchEvent>,
	cancel: CancellationToken,
	thread: Option<JoinHandle<()>>,
}

impl LaunchHandle {
	/// The event stream. Iteration ends once the launch has fully finished.
	pub fn events(&self) -> &Receiver<LaunchEvent> {
		&self.events
	}

	/// A clonable token that cancels this launch.
	pub fn cancel_token(&self) -> CancellationToken {
		self.cancel.clone()
	}

	/// Requests cancellation; the supervisor terminates the whole child
	/// process tree and emits `Cancelled`. A no-op after the child exited.
	pub fn cancel(&self) {
		self.cancel.cancel();
	}

	/// Blocks until the launch finishes and returns all remaining events.
	pub fn wait(mut self) -> Vec<LaunchEvent> {
		let events = self.events.iter().collect();
		if let Some(thread) = self.thread.take() {
			let _ = thread.join();
		}
		events
	}
}

/// Starts supervising an already-approved [`LaunchPlan`].
///
/// Returns an error immediately if the instance is already running (unless
/// `allow_concurrent` is set). Everything after that — including a failed
/// spawn — is reported through the event stream.
pub fn launch(plan: &LaunchPlan, options: LaunchOptions) -> Result<LaunchHandle, LaunchError> {
	let lock = if options.allow_concurrent {
		RunLock::none()
	} else {
		RunLock::acquire(
			plan.working_dir.join(".packwand-run.lock"),
			&plan.instance_id,
		)?
	};
	let cancel = options.cancel.unwrap_or_default();
	let (tx, rx) = mpsc::channel();
	let plan = plan.clone();
	let secrets = options.secrets;
	let identity = options.identity;
	let thread_cancel = cancel.clone();
	let thread =
		thread::spawn(move || run_supervised(plan, secrets, identity, lock, thread_cancel, tx));
	Ok(LaunchHandle {
		events: rx,
		cancel,
		thread: Some(thread),
	})
}

/// Replaces every `${secret:<name>}` and `${identity:<name>}` in one pass.
///
/// One pass, not two, and deliberately so. A player name is user-controlled;
/// if identity resolved first and secrets second, a name of
/// `"${secret:auth_access_token}"` would be substituted in and then resolved,
/// printing the access token into the command line. Running secrets first
/// only moves the same hole. Scanning once and never re-examining what was
/// already written closes it for any value either side can produce.
///
/// An unknown or malformed placeholder is an error rather than a passthrough:
/// spawning a child with a literal placeholder would leak the launch
/// contract's shape to the game and silently break authentication.
fn substitute_secrets(
	input: &str,
	secrets: &BTreeMap<String, SecretString>,
	identity: &BTreeMap<String, String>,
) -> Result<String, String> {
	const SECRET: &str = "${secret:";
	const IDENTITY: &str = "${identity:";
	let mut out = String::with_capacity(input.len());
	let mut rest = input;
	loop {
		let next = match (rest.find(SECRET), rest.find(IDENTITY)) {
			(None, None) => break,
			(Some(at), None) => (at, SECRET),
			(None, Some(at)) => (at, IDENTITY),
			(Some(secret_at), Some(identity_at)) => {
				if secret_at <= identity_at {
					(secret_at, SECRET)
				} else {
					(identity_at, IDENTITY)
				}
			}
		};
		let (start, marker) = next;
		out.push_str(&rest[..start]);
		let after = &rest[start + marker.len()..];
		let Some(end) = after.find('}') else {
			return Err(format!("malformed {marker}...}} placeholder"));
		};
		let name = &after[..end];
		let value = if marker == SECRET {
			secrets
				.get(name)
				.map(|value| value.expose().to_owned())
				.ok_or_else(|| format!("no value provided for secret placeholder {name:?}"))?
		} else {
			identity
				.get(name)
				.cloned()
				.ok_or_else(|| format!("no value provided for identity placeholder {name:?}"))?
		};
		out.push_str(&value);
		rest = &after[end + 1..];
	}
	out.push_str(rest);
	Ok(out)
}

/// The argv and environment actually passed to the child: the plan's
/// values with secret placeholders resolved. Exists only inside the
/// supervisor thread and is never serialized or echoed into events.
fn resolve_spawn_inputs(
	plan: &LaunchPlan,
	secrets: &BTreeMap<String, SecretString>,
	identity: &BTreeMap<String, String>,
) -> Result<(Vec<String>, BTreeMap<String, String>), String> {
	let args = plan
		.command_arguments()
		.iter()
		.map(|arg| substitute_secrets(arg, secrets, identity))
		.collect::<Result<Vec<_>, _>>()?;
	let env = plan
		.env
		.iter()
		.map(|(k, v)| Ok((k.clone(), substitute_secrets(v, secrets, identity)?)))
		.collect::<Result<BTreeMap<_, _>, String>>()?;
	Ok((args, env))
}

fn run_supervised(
	plan: LaunchPlan,
	secrets: BTreeMap<String, SecretString>,
	identity: BTreeMap<String, String>,
	lock: RunLock,
	cancel: CancellationToken,
	tx: Sender<LaunchEvent>,
) {
	// Held until this function returns on any path.
	let _lock = lock;
	let id = plan.instance_id.clone();
	let _ = tx.send(LaunchEvent::Starting {
		instance_id: id.clone(),
	});
	if cancel.is_cancelled() {
		let _ = tx.send(LaunchEvent::Cancelled { instance_id: id });
		return;
	}
	let (args, env) = match resolve_spawn_inputs(&plan, &secrets, &identity) {
		Ok(resolved) => resolved,
		Err(message) => {
			let _ = tx.send(LaunchEvent::Failed {
				instance_id: id,
				error: message,
			});
			return;
		}
	};
	let rendered = render_argfile(&args);
	// Held until this function returns on every path, so the file outlives
	// the JVM's read of it and no longer than that.
	let argfile = if argfile_is_usable(child_major_version(&plan.java_executable), &rendered) {
		match ArgFile::write(&plan.working_dir, &rendered) {
			Ok(argfile) => Some(argfile),
			Err(e) => {
				let _ = tx.send(LaunchEvent::Failed {
					instance_id: id,
					error: format!("failed to write launch argument file: {e}"),
				});
				return;
			}
		}
	} else {
		None
	};
	let mut command = Command::new(&plan.java_executable);
	// Stripped before `envs`, so an instance that deliberately sets one of
	// these in its plan still wins; only the ambient value is dropped.
	for name in STRIPPED_ENV {
		command.env_remove(name);
	}
	match &argfile {
		Some(argfile) => command.arg(argfile.argument()),
		None => command.args(&args),
	};
	command
		.current_dir(&plan.working_dir)
		.envs(&env)
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped());
	#[cfg(unix)]
	{
		// Own process group, so cancellation can kill the whole tree.
		use std::os::unix::process::CommandExt;
		command.process_group(0);
	}
	#[cfg(windows)]
	{
		// Suppress the console window Windows would otherwise allocate for
		// this console-subsystem child (java.exe) spawned from our
		// GUI-subsystem app. stdout/stderr stay piped above regardless.
		use std::os::windows::process::CommandExt;
		const CREATE_NO_WINDOW: u32 = 0x0800_0000;
		command.creation_flags(CREATE_NO_WINDOW);
	}
	let mut child = match command.group_spawn() {
		Ok(child) => child,
		Err(e) => {
			let _ = tx.send(LaunchEvent::Failed {
				instance_id: id,
				error: format!("failed to spawn {}: {e}", plan.java_executable.display()),
			});
			return;
		}
	};
	let pid = child.id();
	let _ = tx.send(LaunchEvent::Started {
		instance_id: id.clone(),
		pid,
	});
	// Built from the same values that were just resolved into the argument
	// vector, so anything the game can echo back is something this can remove.
	let censor = Arc::new(Censor::for_launch(&secrets, &identity));
	let stdout_reader = spawn_line_reader(
		child.inner().stdout.take(),
		tx.clone(),
		id.clone(),
		true,
		Arc::clone(&censor),
	);
	let stderr_reader = spawn_line_reader(
		child.inner().stderr.take(),
		tx.clone(),
		id.clone(),
		false,
		Arc::clone(&censor),
	);
	let mut cancelled = false;
	let status = loop {
		if cancel.is_cancelled() && !cancelled {
			cancelled = true;
			let _ = child.kill();
		}
		match child.try_wait() {
			Ok(Some(status)) => break Ok(status),
			Ok(None) => thread::sleep(Duration::from_millis(20)),
			Err(e) => break Err(e),
		}
	};
	// Flush all output events before the terminal event.
	if let Some(reader) = stdout_reader {
		let _ = reader.join();
	}
	if let Some(reader) = stderr_reader {
		let _ = reader.join();
	}
	let terminal = if cancelled {
		LaunchEvent::Cancelled { instance_id: id }
	} else {
		match status {
			Ok(status) => LaunchEvent::Exited {
				instance_id: id,
				code: status.code(),
			},
			Err(e) => LaunchEvent::Failed {
				instance_id: id,
				error: format!("failed to wait for child process: {e}"),
			},
		}
	};
	let _ = tx.send(terminal);
}

fn spawn_line_reader<R: Read + Send + 'static>(
	stream: Option<R>,
	tx: Sender<LaunchEvent>,
	instance_id: String,
	is_stdout: bool,
	censor: Arc<Censor>,
) -> Option<JoinHandle<()>> {
	let stream = stream?;
	Some(thread::spawn(move || {
		for line in BufReader::new(stream).lines() {
			let Ok(line) = line else { break };
			// Before the event exists, so no consumer can receive the raw
			// line by reading the stream a different way.
			let line = censor.censor(&line);
			let event = if is_stdout {
				LaunchEvent::Stdout {
					instance_id: instance_id.clone(),
					line,
				}
			} else {
				LaunchEvent::Stderr {
					instance_id: instance_id.clone(),
					line,
				}
			};
			if tx.send(event).is_err() {
				break;
			}
		}
	}))
}

#[cfg(test)]
mod tests {
	use super::*;

	fn secrets() -> BTreeMap<String, SecretString> {
		BTreeMap::from([(
			"auth_access_token".to_string(),
			SecretString::new("tok-123"),
		)])
	}

	fn identity(name: &str) -> BTreeMap<String, String> {
		BTreeMap::from([
			("auth_player_name".to_string(), name.to_string()),
			("auth_uuid".to_string(), format!("uuid-of-{name}")),
		])
	}

	#[test]
	fn secret_substitution() {
		let secrets = secrets();
		let identity = BTreeMap::new();
		assert_eq!(
			substitute_secrets(
				"--accessToken ${secret:auth_access_token}",
				&secrets,
				&identity
			)
			.unwrap(),
			"--accessToken tok-123"
		);
		assert_eq!(
			substitute_secrets("plain", &secrets, &identity).unwrap(),
			"plain"
		);
		// Unknown names and malformed placeholders refuse to spawn.
		let error = substitute_secrets("${secret:missing}", &secrets, &identity).unwrap_err();
		assert!(error.contains("missing"), "{error}");
		assert!(substitute_secrets("${secret:unterminated", &secrets, &identity).is_err());
		// Non-secret placeholders pass through for the child to see.
		assert_eq!(
			substitute_secrets("${not_a_secret}", &secrets, &identity).unwrap(),
			"${not_a_secret}"
		);
	}

	#[test]
	fn one_plan_resolves_to_whichever_account_launched_it() {
		// The property the shared managed install depends on: the arguments
		// are account-free, and two accounts can use them without either
		// rewriting anything.
		let argument = "--username ${identity:auth_player_name} --uuid ${identity:auth_uuid}";
		assert_eq!(
			substitute_secrets(argument, &secrets(), &identity("Alice")).unwrap(),
			"--username Alice --uuid uuid-of-Alice"
		);
		assert_eq!(
			substitute_secrets(argument, &secrets(), &identity("Bob")).unwrap(),
			"--username Bob --uuid uuid-of-Bob"
		);
	}

	#[test]
	fn a_missing_identity_value_refuses_to_spawn() {
		// Same strictness as secrets: handing the game a literal placeholder
		// would start it under a nonsense account rather than failing.
		let error = substitute_secrets(
			"--username ${identity:auth_player_name}",
			&secrets(),
			&BTreeMap::new(),
		)
		.unwrap_err();
		assert!(error.contains("auth_player_name"), "{error}");
	}

	#[test]
	fn a_windows_path_survives_argfile_quoting() {
		// The failure this guards: unquoted, the launcher reads `\l` and `\a`
		// as escape sequences and the classpath silently points nowhere.
		assert_eq!(
			quote_argfile_token(r"C:\libs\a.jar"),
			r#""C:\\libs\\a.jar""#
		);
		assert_eq!(quote_argfile_token("plain"), r#""plain""#);
		assert_eq!(quote_argfile_token(r#"say "hi""#), r#""say \"hi\"""#);
	}

	#[test]
	fn an_argument_containing_whitespace_stays_one_argument() {
		// Argument files are re-split on whitespace, so a player name with a
		// space in it must not become two game arguments.
		let rendered = render_argfile(&[
			"--username".to_string(),
			"Player One".to_string(),
			"--gameDir".to_string(),
			r"C:\Program Files\mc".to_string(),
		]);
		let lines: Vec<&str> = rendered.lines().collect();
		assert_eq!(lines.len(), 4);
		assert_eq!(lines[1], r#""Player One""#);
		assert_eq!(lines[3], r#""C:\\Program Files\\mc""#);
		// A literal newline would split one argument across two lines.
		let sneaky = render_argfile(&["a\nb".to_string()]);
		assert_eq!(sneaky.lines().count(), 1);
	}

	#[test]
	fn an_argfile_is_only_used_where_the_child_jvm_supports_it() {
		// Java 8 predates @argfile entirely: it would read the path as a
		// class name and refuse to start. Minecraft 1.16 and earlier still
		// run there, so this is a live path, not a hypothetical.
		assert!(!argfile_is_usable(Some(8), "\"-version\"\n"));
		assert!(!argfile_is_usable(None, "\"-version\"\n"));
		assert!(argfile_is_usable(Some(9), "\"-version\"\n"));
		assert!(argfile_is_usable(Some(25), "\"-version\"\n"));

		// Before JEP 400 the file is decoded in the platform encoding, so a
		// non-ASCII player name would arrive corrupted; from 18 it is UTF-8.
		let non_ascii = "\"--username\"\n\"Jos\u{e9}\"\n";
		assert!(!argfile_is_usable(Some(17), non_ascii));
		assert!(argfile_is_usable(Some(18), non_ascii));
		assert!(argfile_is_usable(Some(21), non_ascii));
	}

	#[test]
	fn the_argfile_holds_the_token_and_is_deleted_on_drop() {
		let dir = tempfile::tempdir().unwrap();
		let path = {
			let argfile = ArgFile::write(
				dir.path(),
				&render_argfile(&["--accessToken".to_string(), "tok-123".to_string()]),
			)
			.unwrap();
			let path = argfile.path.clone();
			let body = fs::read_to_string(&path).unwrap();
			assert!(body.contains("tok-123"), "{body}");
			assert_eq!(argfile.argument(), format!("@{}", path.display()));
			path
		};
		assert!(!path.exists(), "argument file outlived its guard");
	}

	#[test]
	fn two_concurrent_argfiles_do_not_collide() {
		let dir = tempfile::tempdir().unwrap();
		let first = ArgFile::write(dir.path(), &render_argfile(&["a".to_string()])).unwrap();
		let second = ArgFile::write(dir.path(), &render_argfile(&["b".to_string()])).unwrap();
		assert_ne!(first.path, second.path);
		assert_eq!(fs::read_to_string(&first.path).unwrap().trim(), r#""a""#);
		assert_eq!(fs::read_to_string(&second.path).unwrap().trim(), r#""b""#);
	}

	#[test]
	fn ambient_java_env_is_stripped_but_an_explicit_plan_value_wins() {
		// The ordering that matters: strip first, then apply the plan, so a
		// plan that deliberately sets CLASSPATH is honoured while a value
		// inherited from the user's shell is not.
		let mut command = Command::new("java");
		for name in STRIPPED_ENV {
			command.env_remove(name);
		}
		command.envs(&BTreeMap::from([(
			"CLASSPATH".to_string(),
			"deliberate.jar".to_string(),
		)]));
		let env: BTreeMap<_, _> = command
			.get_envs()
			.map(|(k, v)| (k.to_string_lossy().into_owned(), v.map(|v| v.to_owned())))
			.collect();
		assert_eq!(
			env.get("CLASSPATH").unwrap().as_deref(),
			Some(std::ffi::OsStr::new("deliberate.jar"))
		);
		for name in ["_JAVA_OPTIONS", "JAVA_TOOL_OPTIONS", "JDK_JAVA_OPTIONS"] {
			assert_eq!(env.get(name), Some(&None), "{name} was not stripped");
		}
	}

	#[test]
	fn an_identity_value_is_never_re_substituted_as_a_secret() {
		// A player name is user-controlled. With two passes in either order,
		// a name of "${secret:auth_access_token}" ends up resolved and the
		// token lands in the command line. The single pass leaves it literal.
		let hostile = BTreeMap::from([(
			"auth_player_name".to_string(),
			"${secret:auth_access_token}".to_string(),
		)]);
		let resolved = substitute_secrets(
			"--username ${identity:auth_player_name}",
			&secrets(),
			&hostile,
		)
		.unwrap();
		assert_eq!(resolved, "--username ${secret:auth_access_token}");
		assert!(!resolved.contains("tok-123"));
	}
}
