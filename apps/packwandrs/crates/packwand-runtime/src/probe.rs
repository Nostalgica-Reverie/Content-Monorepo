//! Identifying a JVM by asking it, with the answer cached.
//!
//! Discovery reads `release` files, which is cheap and works for every
//! ordinary installation. It does not work for a bare `java` on `PATH`, a
//! symlink into a wrapper, or a layout with no `release` file at all — and
//! those are exactly the cases where a caller most needs to know what it is
//! about to launch.
//!
//! Asking means starting a JVM, which costs on the order of a hundred
//! milliseconds and must not happen once per launch, per candidate, or per
//! settings-page render. Answers are therefore cached against a signature of
//! the executable — path, size, and modification time — so a JDK that has
//! been upgraded in place is re-probed and one that has not is free.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::discovery::java_executable_name;
use crate::error::RuntimeError;
use crate::version::JavaVersion;

/// How long a JVM gets to print its own properties before being killed.
///
/// A JVM that never answers is the failure mode that makes a launcher look
/// frozen; bounding it turns that into an error message.
const PROBE_TIMEOUT: Duration = Duration::from_secs(20);

/// What a JVM reports about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProbedJava {
	/// The executable that was probed.
	pub executable: PathBuf,
	pub version: JavaVersion,
	/// `java.home`, when the JVM reported one.
	pub home: Option<PathBuf>,
	/// `java.vendor`, when the JVM reported one.
	pub vendor: Option<String>,
	/// `os.arch`, when the JVM reported one.
	pub architecture: Option<String>,
	/// Whether this came from reading a `release` file rather than starting
	/// the JVM. The values are the same; the cost was not.
	pub from_release_file: bool,
}

/// Identity of an executable for caching purposes: path, size, and mtime.
///
/// Size and mtime are what make in-place upgrades visible. A JDK replaced
/// under the same path — the normal result of a package-manager update —
/// keeps its path but changes both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Signature(u64);

impl Signature {
	fn of(executable: &Path) -> Self {
		let mut hasher = DefaultHasher::new();
		executable.hash(&mut hasher);
		if let Ok(meta) = std::fs::metadata(executable) {
			meta.len().hash(&mut hasher);
			if let Ok(modified) = meta.modified()
				&& let Ok(since) = modified.duration_since(std::time::UNIX_EPOCH)
			{
				since.as_secs().hash(&mut hasher);
			}
		}
		Self(hasher.finish())
	}
}

/// Memoized JVM identification.
///
/// Failures are cached alongside successes: a path that is not a working JVM
/// is not going to become one within a session, and re-running a broken
/// executable on every settings render is the cost this exists to avoid.
#[derive(Debug, Default)]
pub struct ProbeCache {
	entries: Mutex<HashMap<Signature, Result<ProbedJava, String>>>,
}

impl ProbeCache {
	/// An empty cache.
	pub fn new() -> Self {
		Self::default()
	}

	/// The process-wide cache, which is what ordinary callers want: probing
	/// is idempotent and the result is worth sharing across subsystems.
	pub fn shared() -> &'static Self {
		static SHARED: OnceLock<ProbeCache> = OnceLock::new();
		SHARED.get_or_init(ProbeCache::new)
	}

	/// Identifies `executable`, answering from cache when it can.
	pub fn probe(&self, executable: &Path) -> Result<ProbedJava, RuntimeError> {
		let signature = Signature::of(executable);
		if let Ok(entries) = self.entries.lock()
			&& let Some(hit) = entries.get(&signature)
		{
			return hit.clone().map_err(|reason| RuntimeError::ProbeFailed {
				executable: executable.to_path_buf(),
				reason,
			});
		}
		let result = probe_uncached(executable);
		if let Ok(mut entries) = self.entries.lock() {
			entries.insert(
				signature,
				result.as_ref().map_err(|e| e.to_string()).cloned(),
			);
		}
		result
	}

	/// Number of cached answers, for tests and diagnostics.
	pub fn len(&self) -> usize {
		self.entries.lock().map(|e| e.len()).unwrap_or(0)
	}

	/// Whether nothing has been probed yet.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

/// Reads the `release` file beside an executable, when there is one.
///
/// Tried before starting the JVM because it answers the same question for
/// almost every real installation without paying for a process.
fn from_release_file(executable: &Path) -> Option<ProbedJava> {
	let home = executable.parent()?.parent()?;
	let installation =
		crate::discovery::inspect_java_home(home, crate::DiscoverySource::Explicit).ok()?;
	Some(ProbedJava {
		version: JavaVersion::parse(&installation.version).ok()?,
		home: Some(installation.home),
		vendor: installation.vendor,
		architecture: installation.architecture,
		executable: executable.to_path_buf(),
		from_release_file: true,
	})
}

fn probe_uncached(executable: &Path) -> Result<ProbedJava, RuntimeError> {
	if let Some(probed) = from_release_file(executable) {
		return Ok(probed);
	}
	let properties = run_show_settings(executable)?;
	let version = properties
		.get("java.version")
		.ok_or_else(|| RuntimeError::ProbeFailed {
			executable: executable.to_path_buf(),
			reason: "output contained no java.version property".to_string(),
		})?;
	Ok(ProbedJava {
		version: JavaVersion::parse(version)?,
		home: properties.get("java.home").map(PathBuf::from),
		vendor: properties.get("java.vendor").cloned(),
		architecture: properties.get("os.arch").cloned(),
		executable: executable.to_path_buf(),
		from_release_file: false,
	})
}

/// Runs `java -XshowSettings:properties -version` and collects the
/// `key = value` lines it prints.
///
/// The properties go to stderr, and `-version` means the JVM prints them and
/// exits rather than looking for a class to run.
fn run_show_settings(executable: &Path) -> Result<HashMap<String, String>, RuntimeError> {
	let failed = |reason: String| RuntimeError::ProbeFailed {
		executable: executable.to_path_buf(),
		reason,
	};
	let mut command = Command::new(executable);
	command
		.args(["-XshowSettings:properties", "-version"])
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped());
	#[cfg(windows)]
	{
		use std::os::windows::process::CommandExt;
		const CREATE_NO_WINDOW: u32 = 0x0800_0000;
		command.creation_flags(CREATE_NO_WINDOW);
	}
	let mut child = command.spawn().map_err(|e| failed(e.to_string()))?;
	let deadline = Instant::now() + PROBE_TIMEOUT;
	loop {
		match child.try_wait() {
			Ok(Some(_)) => break,
			Ok(None) if Instant::now() >= deadline => {
				let _ = child.kill();
				let _ = child.wait();
				return Err(RuntimeError::ProbeTimedOut {
					executable: executable.to_path_buf(),
					seconds: PROBE_TIMEOUT.as_secs(),
				});
			}
			Ok(None) => std::thread::sleep(Duration::from_millis(10)),
			Err(e) => return Err(failed(e.to_string())),
		}
	}
	let output = child
		.wait_with_output()
		.map_err(|e| failed(e.to_string()))?;
	let text = format!(
		"{}{}",
		String::from_utf8_lossy(&output.stderr),
		String::from_utf8_lossy(&output.stdout)
	);
	Ok(parse_show_settings(&text))
}

/// Extracts `key = value` property lines from `-XshowSettings` output.
///
/// Multi-valued properties such as `java.class.path` continue on indented
/// lines with no `=`; those continuations are skipped rather than parsed,
/// since nothing here needs them.
fn parse_show_settings(text: &str) -> HashMap<String, String> {
	text.lines()
		.filter_map(|line| {
			let (key, value) = line.split_once('=')?;
			let key = key.trim();
			// Property names have no spaces; anything else is prose such as
			// the `openjdk version "25"` banner.
			if key.is_empty() || key.contains(char::is_whitespace) {
				return None;
			}
			Some((key.to_string(), value.trim().to_string()))
		})
		.collect()
}

/// Locates a `java` executable inside an installation home.
pub fn executable_in(home: &Path) -> PathBuf {
	home.join("bin").join(java_executable_name())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn show_settings_output_is_parsed() {
		let text = concat!(
			"VM settings:\n",
			"    Max. Heap Size (Estimated): 8.00G\n",
			"Property settings:\n",
			"    java.class.path = \n",
			"    java.home = /opt/jdk-21\n",
			"    java.vendor = Eclipse Adoptium\n",
			"    java.version = 21.0.5\n",
			"    os.arch = x86_64\n",
			"        continuation-line-with-no-equals\n",
			"openjdk version \"21.0.5\" 2024-10-15\n",
		);
		let props = parse_show_settings(text);
		assert_eq!(props["java.version"], "21.0.5");
		assert_eq!(props["java.home"], "/opt/jdk-21");
		assert_eq!(props["os.arch"], "x86_64");
		assert_eq!(props["java.vendor"], "Eclipse Adoptium");
		// The banner line contains no `=` and must not become a property.
		assert!(!props.contains_key("openjdk version \"21.0.5\" 2024-10-15"));
	}

	#[test]
	fn a_signature_changes_when_the_file_changes() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("java");
		std::fs::write(&path, b"one").unwrap();
		let first = Signature::of(&path);
		assert_eq!(first, Signature::of(&path), "signature is not stable");
		// An in-place upgrade keeps the path and changes the contents; the
		// signature has to notice, or the old answer is served forever.
		std::fs::write(&path, b"a different length entirely").unwrap();
		assert_ne!(first, Signature::of(&path));
	}

	#[test]
	fn a_failing_probe_is_cached_rather_than_retried() {
		let cache = ProbeCache::new();
		let missing = std::path::Path::new("definitely-not-a-java-executable");
		assert!(cache.probe(missing).is_err());
		assert_eq!(cache.len(), 1);
		assert!(cache.probe(missing).is_err());
		assert_eq!(cache.len(), 1, "a second probe added another entry");
	}

	#[test]
	fn a_release_file_answers_without_starting_a_jvm() {
		let dir = tempfile::tempdir().unwrap();
		let home = dir.path().join("jdk-21");
		std::fs::create_dir_all(home.join("bin")).unwrap();
		let executable = super::executable_in(&home);
		std::fs::write(&executable, b"not really a jvm").unwrap();
		std::fs::write(
			home.join("release"),
			"JAVA_VERSION=\"21.0.5\"\nOS_ARCH=\"x86_64\"\nIMPLEMENTOR=\"Eclipse Adoptium\"\n",
		)
		.unwrap();

		// The file is not an executable at all, so a probe that answers has
		// necessarily answered from the release file.
		let probed = ProbeCache::new().probe(&executable).unwrap();
		assert!(probed.from_release_file);
		assert_eq!(probed.version.major, 21);
		assert_eq!(probed.vendor.as_deref(), Some("Eclipse Adoptium"));
	}
}
