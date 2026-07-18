//! Process supervision for approved launch plans.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use packwand_auth::SecretString;
use serde::Serialize;

use crate::plan::LaunchPlan;

/// Typed lifecycle events emitted by the supervisor.
///
/// Child stdout/stderr is forwarded verbatim; the supervisor itself never
/// holds secret values, so nothing here needs redaction.
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
    let thread_cancel = cancel.clone();
    let thread = thread::spawn(move || run_supervised(plan, secrets, lock, thread_cancel, tx));
    Ok(LaunchHandle {
        events: rx,
        cancel,
        thread: Some(thread),
    })
}

/// Replaces every `${secret:<name>}` occurrence with its value from
/// `secrets`. An unknown or malformed placeholder is an error: spawning a
/// child with a literal placeholder would leak the launch contract's shape
/// to the game and silently break authentication.
fn substitute_secrets(
    input: &str,
    secrets: &BTreeMap<String, SecretString>,
) -> Result<String, String> {
    const MARKER: &str = "${secret:";
    let mut out = String::new();
    let mut rest = input;
    while let Some(start) = rest.find(MARKER) {
        out.push_str(&rest[..start]);
        let after = &rest[start + MARKER.len()..];
        let Some(end) = after.find('}') else {
            return Err("malformed ${secret:...} placeholder".to_string());
        };
        let name = &after[..end];
        let value = secrets
            .get(name)
            .ok_or_else(|| format!("no value provided for secret placeholder {name:?}"))?;
        out.push_str(value.expose());
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
) -> Result<(Vec<String>, BTreeMap<String, String>), String> {
    let args = plan
        .command_arguments()
        .iter()
        .map(|arg| substitute_secrets(arg, secrets))
        .collect::<Result<Vec<_>, _>>()?;
    let env = plan
        .env
        .iter()
        .map(|(k, v)| Ok((k.clone(), substitute_secrets(v, secrets)?)))
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    Ok((args, env))
}

fn run_supervised(
    plan: LaunchPlan,
    secrets: BTreeMap<String, SecretString>,
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
    let (args, env) = match resolve_spawn_inputs(&plan, &secrets) {
        Ok(resolved) => resolved,
        Err(message) => {
            let _ = tx.send(LaunchEvent::Failed {
                instance_id: id,
                error: message,
            });
            return;
        }
    };
    let mut command = Command::new(&plan.java_executable);
    command
        .args(args)
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
    let mut child = match command.spawn() {
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
    let stdout_reader = spawn_line_reader(child.stdout.take(), tx.clone(), id.clone(), true);
    let stderr_reader = spawn_line_reader(child.stderr.take(), tx.clone(), id.clone(), false);
    let mut cancelled = false;
    let status = loop {
        if cancel.is_cancelled() && !cancelled {
            cancelled = true;
            kill_process_tree(pid);
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
) -> Option<JoinHandle<()>> {
    let stream = stream?;
    Some(thread::spawn(move || {
        for line in BufReader::new(stream).lines() {
            let Ok(line) = line else { break };
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

/// Terminates the child and its whole process tree.
#[cfg(windows)]
fn kill_process_tree(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Terminates the child and its whole process tree. The child was spawned
/// as its own process group leader, so its pid doubles as the pgid.
#[cfg(unix)]
fn kill_process_tree(pid: u32) {
    use nix::sys::signal::{killpg, Signal};
    use nix::unistd::Pid;
    let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_substitution() {
        let secrets = BTreeMap::from([(
            "auth_access_token".to_string(),
            SecretString::new("tok-123"),
        )]);
        assert_eq!(
            substitute_secrets("--accessToken ${secret:auth_access_token}", &secrets).unwrap(),
            "--accessToken tok-123"
        );
        assert_eq!(substitute_secrets("plain", &secrets).unwrap(), "plain");
        // Unknown names and malformed placeholders refuse to spawn.
        let err = substitute_secrets("${secret:missing}", &secrets).unwrap_err();
        assert!(err.contains("missing"), "{err}");
        assert!(substitute_secrets("${secret:unterminated", &secrets).is_err());
        // Non-secret placeholders pass through for the child to see.
        assert_eq!(
            substitute_secrets("${not_a_secret}", &secrets).unwrap(),
            "${not_a_secret}"
        );
    }
}
