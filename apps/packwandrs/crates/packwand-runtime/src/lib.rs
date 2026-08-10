//! Java runtime discovery, version parsing, and compatibility selection.
//!
//! Part of the shared Packwand core. This crate
//! must stay free of Tauri, clap, and axum dependencies. Discovery is split
//! from the host environment so tests can run against synthetic JDK layouts:
//! [`DiscoveryConfig::from_host`] captures the real environment, while the
//! pure functions accept explicit paths.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// Where a discovered installation came from, in preference order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySource {
	/// Explicitly configured (e.g. a `--java` flag).
	Explicit,
	/// The `JAVA_HOME` environment variable.
	JavaHome,
	/// A `java` executable found on `PATH`.
	PathEnv,
	/// A well-known vendor installation directory.
	WellKnownDir,
}

/// One discovered Java installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JavaInstallation {
	/// The installation root (a `JAVA_HOME`-style directory).
	pub home: PathBuf,
	/// The `java` executable inside `home`.
	pub executable: PathBuf,
	/// Feature-release version: 8 for `1.8.0_392`, 17 for `17.0.2`.
	pub major_version: u32,
	/// The full `JAVA_VERSION` string from the `release` file.
	pub version: String,
	/// `OS_ARCH` from the `release` file, when present.
	pub architecture: Option<String>,
	/// `IMPLEMENTOR` from the `release` file, when present.
	pub vendor: Option<String>,
	pub source: DiscoverySource,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
	#[error("{home} is not a Java installation: {reason}")]
	NotAJavaHome { home: PathBuf, reason: String },
	#[error("could not parse Java version {0:?}")]
	UnparseableVersion(String),
	#[error("no discovered Java installation satisfies major version {required}; found: [{found}]")]
	NoCompatibleJava { required: u32, found: String },
}

/// The name of the Java executable on this platform.
pub fn java_executable_name() -> &'static str {
	if cfg!(windows) { "java.exe" } else { "java" }
}

/// Parses a `JAVA_VERSION` string into its feature-release number:
/// `1.8.0_392` is 8, `17.0.2` is 17, `9` is 9.
pub fn parse_major_version(version: &str) -> Result<u32, RuntimeError> {
	let unparseable = || RuntimeError::UnparseableVersion(version.to_string());
	let mut parts = version.split('.');
	let first: u32 = parts
		.next()
		.and_then(|p| p.parse().ok())
		.ok_or_else(unparseable)?;
	if first != 1 {
		return Ok(first);
	}
	// Legacy "1.x" scheme: the second component is the feature release.
	parts
		.next()
		.and_then(|p| p.split('_').next())
		.and_then(|p| p.parse().ok())
		.ok_or_else(unparseable)
}

/// Parses the key/value pairs of a JDK `release` file. Values may be
/// double-quoted; unquoted values and lines without `=` are tolerated.
fn parse_release_file(contents: &str) -> BTreeMap<String, String> {
	contents
		.lines()
		.filter_map(|line| {
			let (key, value) = line.split_once('=')?;
			let value = value.trim().trim_matches('"');
			Some((key.trim().to_string(), value.to_string()))
		})
		.collect()
}

/// Inspects one candidate installation root. Requires a `bin/java`
/// executable and a parseable `release` file (every modern JDK and JRE
/// ships one; installations without it are skipped by discovery).
pub fn inspect_java_home(
	home: &Path,
	source: DiscoverySource,
) -> Result<JavaInstallation, RuntimeError> {
	let executable = home.join("bin").join(java_executable_name());
	if !executable.is_file() {
		return Err(RuntimeError::NotAJavaHome {
			home: home.to_path_buf(),
			reason: format!("missing {}", executable.display()),
		});
	}
	let release_path = home.join("release");
	let contents = fs::read_to_string(&release_path).map_err(|e| RuntimeError::NotAJavaHome {
		home: home.to_path_buf(),
		reason: format!("cannot read {}: {e}", release_path.display()),
	})?;
	let keys = parse_release_file(&contents);
	let version = keys
		.get("JAVA_VERSION")
		.cloned()
		.ok_or_else(|| RuntimeError::NotAJavaHome {
			home: home.to_path_buf(),
			reason: "release file has no JAVA_VERSION".to_string(),
		})?;
	Ok(JavaInstallation {
		major_version: parse_major_version(&version)?,
		version,
		architecture: keys.get("OS_ARCH").cloned(),
		vendor: keys.get("IMPLEMENTOR").cloned(),
		home: home.to_path_buf(),
		executable,
		source,
	})
}

/// Candidate locations to search, decoupled from the host environment so
/// discovery is testable against synthetic layouts.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryConfig {
	/// Value of `JAVA_HOME`, if set.
	pub java_home: Option<PathBuf>,
	/// Entries of `PATH`. A `<dir>/java` executable marks `<dir>/..` as a
	/// candidate home.
	pub path_entries: Vec<PathBuf>,
	/// Directories whose immediate children are candidate homes
	/// (for example `C:\Program Files\Eclipse Adoptium`).
	pub vendor_dirs: Vec<PathBuf>,
}

impl DiscoveryConfig {
	/// Captures the real host environment and the platform's well-known
	/// vendor directories.
	pub fn from_host() -> Self {
		let java_home = std::env::var_os("JAVA_HOME").map(PathBuf::from);
		let path_entries = std::env::var_os("PATH")
			.map(|path| std::env::split_paths(&path).collect())
			.unwrap_or_default();
		Self {
			java_home,
			path_entries,
			vendor_dirs: host_vendor_dirs(),
		}
	}
}

#[cfg(windows)]
fn host_vendor_dirs() -> Vec<PathBuf> {
	let mut roots = Vec::new();
	for program_files in ["ProgramFiles", "ProgramFiles(x86)"] {
		let Some(base) = std::env::var_os(program_files).map(PathBuf::from) else {
			continue;
		};
		for vendor in [
			"Java",
			"Eclipse Adoptium",
			"Eclipse Foundation",
			"Microsoft",
			"Zulu",
			"Amazon Corretto",
			"BellSoft",
			"Semeru",
		] {
			roots.push(base.join(vendor));
		}
	}
	if let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
		roots.push(home.join(".jdks"));
	}
	roots
}

#[cfg(not(windows))]
fn host_vendor_dirs() -> Vec<PathBuf> {
	let mut roots = vec![
		PathBuf::from("/usr/lib/jvm"),
		PathBuf::from("/usr/java"),
		PathBuf::from("/opt/java"),
	];
	if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
		roots.push(home.join(".jdks"));
		roots.push(home.join(".sdkman/candidates/java"));
	}
	roots
}

/// On macOS, JDK bundles keep the home under `Contents/Home`.
fn candidate_homes_in(dir: &Path) -> Vec<PathBuf> {
	let Ok(entries) = fs::read_dir(dir) else {
		return Vec::new();
	};
	let mut homes = Vec::new();
	for entry in entries.flatten() {
		let path = entry.path();
		if !path.is_dir() {
			continue;
		}
		let bundle_home = path.join("Contents").join("Home");
		homes.push(if bundle_home.is_dir() {
			bundle_home
		} else {
			path
		});
	}
	homes
}

/// Discovers Java installations from the configured locations.
///
/// Results are deduplicated by home path and sorted by source preference,
/// then by descending major version, so the first compatible entry is the
/// preferred pick. Unreadable or non-Java directories are skipped silently:
/// discovery reports what exists, it does not validate the machine.
pub fn discover(config: &DiscoveryConfig) -> Vec<JavaInstallation> {
	let mut candidates: Vec<(PathBuf, DiscoverySource)> = Vec::new();
	if let Some(home) = &config.java_home {
		candidates.push((home.clone(), DiscoverySource::JavaHome));
	}
	for dir in &config.path_entries {
		if dir.join(java_executable_name()).is_file()
			&& let Some(home) = dir.parent()
		{
			candidates.push((home.to_path_buf(), DiscoverySource::PathEnv));
		}
	}
	for vendor_dir in &config.vendor_dirs {
		for home in candidate_homes_in(vendor_dir) {
			candidates.push((home, DiscoverySource::WellKnownDir));
		}
	}

	let mut installations: Vec<JavaInstallation> = Vec::new();
	for (home, source) in candidates {
		let canonical = fs::canonicalize(&home).unwrap_or(home);
		if installations.iter().any(|i| i.home == canonical) {
			continue;
		}
		if let Ok(installation) = inspect_java_home(&canonical, source) {
			installations.push(installation);
		}
	}
	installations.sort_by(|a, b| {
		a.source
			.cmp(&b.source)
			.then(b.major_version.cmp(&a.major_version))
			.then(a.home.cmp(&b.home))
	});
	installations
}

/// Picks the installation to use for a required feature release
/// (Minecraft's `javaVersion.majorVersion`).
///
/// An exact major match wins; otherwise the smallest major above the
/// requirement is used (newer JVMs run older bytecode; the reverse fails).
/// Ties keep discovery's source preference order.
pub fn select_compatible(
	installations: &[JavaInstallation],
	required_major: u32,
) -> Result<&JavaInstallation, RuntimeError> {
	if let Some(exact) = installations
		.iter()
		.find(|i| i.major_version == required_major)
	{
		return Ok(exact);
	}
	installations
		.iter()
		.filter(|i| i.major_version > required_major)
		.min_by_key(|i| i.major_version)
		.ok_or_else(|| RuntimeError::NoCompatibleJava {
			required: required_major,
			found: installations
				.iter()
				.map(|i| format!("{} ({})", i.version, i.home.display()))
				.collect::<Vec<_>>()
				.join(", "),
		})
}

#[cfg(test)]
mod tests {
	use super::{parse_major_version, parse_release_file};

	#[test]
	fn major_version_parsing() {
		assert_eq!(parse_major_version("1.8.0_392").unwrap(), 8);
		assert_eq!(parse_major_version("17.0.2").unwrap(), 17);
		assert_eq!(parse_major_version("25.0.1").unwrap(), 25);
		assert_eq!(parse_major_version("9").unwrap(), 9);
		assert!(parse_major_version("").is_err());
		assert!(parse_major_version("banana").is_err());
		assert!(parse_major_version("1.x").is_err());
	}

	#[test]
	fn release_file_parsing() {
		let keys = parse_release_file(
			"JAVA_VERSION=\"21.0.5\"\nOS_ARCH=\"x86_64\"\nIMPLEMENTOR=\"Eclipse Adoptium\"\nMODULES=\"java.base java.compiler\"\nGARBAGE\n",
		);
		assert_eq!(keys["JAVA_VERSION"], "21.0.5");
		assert_eq!(keys["OS_ARCH"], "x86_64");
		assert_eq!(keys["IMPLEMENTOR"], "Eclipse Adoptium");
		assert!(!keys.contains_key("GARBAGE"));
	}
}
