use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::Error;

#[derive(Debug, Clone)]
pub(crate) struct DaemonEndpoint {
	pub(crate) base_url: String,
	pub(crate) token: String,
}

static DAEMON: OnceLock<Mutex<Option<DaemonEndpoint>>> = OnceLock::new();

pub(crate) fn ensure_running(binary: &Path) -> Result<DaemonEndpoint, Error> {
	let daemon = DAEMON.get_or_init(|| Mutex::new(None));
	let mut cached = daemon
		.lock()
		.map_err(|_| Error::Daemon("daemon state lock was poisoned".into()))?;
	if let Some(endpoint) = cached.as_ref()
		&& endpoint.healthy()
	{
		return Ok(endpoint.clone());
	}

	let endpoint = start(binary)?;
	*cached = Some(endpoint.clone());
	Ok(endpoint)
}

impl DaemonEndpoint {
	fn healthy(&self) -> bool {
		matches!(
			ureq::get(&format!("{}/health", self.base_url))
				.timeout(Duration::from_millis(500))
				.call(),
			Ok(response) if response.status() == 200
		)
	}
}

fn start(binary: &Path) -> Result<DaemonEndpoint, Error> {
	let nonce = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_err(|error| Error::Daemon(error.to_string()))?
		.as_nanos();
	let directory = std::env::temp_dir()
		.join("packwand-social")
		.join(format!("{}-{nonce}", std::process::id()));
	fs::create_dir_all(&directory)?;
	let token_file = directory.join("token");
	let port_file = directory.join("port");
	Command::new(binary)
		.args(["serve", "--bind", "127.0.0.1:0", "--token-file"])
		.arg(&token_file)
		.arg("--generate-token")
		.arg("--print-port-file")
		.arg(&port_file)
		.args(["--idle-timeout", "2m"])
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::inherit())
		.spawn()
		.map_err(|error| Error::Daemon(format!("start {}: {error}", binary.display())))?;

	let deadline = Instant::now() + Duration::from_secs(10);
	while Instant::now() < deadline {
		if let (Ok(base_url), Ok(token)) = (read_trimmed(&port_file), read_trimmed(&token_file)) {
			let endpoint = DaemonEndpoint { base_url, token };
			if endpoint.healthy() {
				return Ok(endpoint);
			}
		}
		thread::sleep(Duration::from_millis(100));
	}
	Err(Error::Daemon(
		"packwand-social did not become ready within 10 seconds".into(),
	))
}

fn read_trimmed(path: &PathBuf) -> Result<String, Error> {
	let value = fs::read_to_string(path)?;
	let value = value.trim();
	if value.is_empty() {
		return Err(Error::Daemon(format!("{} is empty", path.display())));
	}
	Ok(value.to_owned())
}
