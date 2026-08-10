//! Installing a Java runtime when the machine has none that will do.
//!
//! Mojang publishes JREs as ordinary metadata packages: one index keyed by
//! platform, each entry naming a component and pointing at a file manifest.
//! Treating them that way — rather than as a vendor integration — is what
//! keeps this to one code path. A new component appearing in the index needs
//! no code change here, because the feature release is read from the entry's
//! own version string rather than from a table mapping component names to
//! numbers.
//!
//! Adoptium is the fallback, and only that. Minecraft asks for Java 8, 16, 17
//! and 21, all of which Mojang ships for every platform it supports; what
//! Mojang does not ship is any runtime for Linux on ARM, which is the case
//! that actually reaches [`adoptium_url`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use packwand_net::{Checksum, Client, Download, Request, download_all};
use packwand_parallel::Jobs;
use serde::{Deserialize, Serialize};

use crate::error::RuntimeError;
use crate::probe::executable_in;
use crate::version::JavaVersion;

/// Mojang's index of downloadable runtimes.
pub const MOJANG_RUNTIME_INDEX: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

/// Mojang's platform key for this host.
///
/// Returns an error rather than guessing: installing a runtime built for the
/// wrong architecture produces an executable that fails at spawn with nothing
/// explaining why.
pub fn runtime_os() -> Result<&'static str, RuntimeError> {
	let key = match (std::env::consts::OS, std::env::consts::ARCH) {
		("windows", "x86_64") => "windows-x64",
		("windows", "x86") => "windows-x86",
		("windows", "aarch64") => "windows-arm64",
		("macos", "aarch64") => "mac-os-arm64",
		("macos", "x86_64") => "mac-os",
		("linux", "x86_64") => "linux",
		("linux", "x86") => "linux-i386",
		_ => return Err(RuntimeError::UnsupportedPlatform),
	};
	Ok(key)
}

#[derive(Debug, Clone, Deserialize)]
struct RawRelease {
	manifest: RawManifestRef,
	version: RawVersion,
}

#[derive(Debug, Clone, Deserialize)]
struct RawManifestRef {
	sha1: String,
	url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawVersion {
	name: String,
}

/// The runtime chosen for a requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeSelection {
	/// Mojang's component name, e.g. `java-runtime-delta`.
	pub component: String,
	pub version: JavaVersion,
	/// URL of the component's file manifest.
	pub manifest_url: String,
	/// Expected sha1 of that manifest.
	pub manifest_sha1: String,
}

/// Mojang's runtime index: platform, then component, then releases.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
	platforms: BTreeMap<String, BTreeMap<String, Vec<RawRelease>>>,
}

impl Catalog {
	/// Parses the index document.
	pub fn parse(bytes: &[u8]) -> Result<Self, RuntimeError> {
		let platforms = serde_json::from_slice(bytes).map_err(|e| RuntimeError::Metadata {
			url: MOJANG_RUNTIME_INDEX.to_string(),
			reason: e.to_string(),
		})?;
		Ok(Self { platforms })
	}

	/// Fetches and parses the index.
	pub fn fetch(client: &Client) -> Result<Self, RuntimeError> {
		let bytes = client
			.get(&Request::get(MOJANG_RUNTIME_INDEX))
			.map_err(|e| RuntimeError::Metadata {
				url: MOJANG_RUNTIME_INDEX.to_string(),
				reason: e.to_string(),
			})?;
		Self::parse(&bytes)
	}

	/// Every component available for a platform, newest release of each.
	///
	/// A component with no parseable release is dropped rather than failing
	/// the whole index: one malformed entry should not make every other
	/// runtime unavailable.
	pub fn components_for(&self, runtime_os: &str) -> Vec<RuntimeSelection> {
		let Some(components) = self.platforms.get(runtime_os) else {
			return Vec::new();
		};
		let mut out: Vec<RuntimeSelection> = components
			.iter()
			.filter_map(|(component, releases)| {
				let newest = releases
					.iter()
					.filter_map(|r| JavaVersion::parse(&r.version.name).ok().map(|v| (v, r)))
					.max_by(|a, b| a.0.cmp(&b.0))?;
				Some(RuntimeSelection {
					component: component.clone(),
					version: newest.0,
					manifest_url: newest.1.manifest.url.clone(),
					manifest_sha1: newest.1.manifest.sha1.clone(),
				})
			})
			.collect();
		out.sort_by(|a, b| {
			a.version
				.cmp(&b.version)
				.then(a.component.cmp(&b.component))
		});
		out
	}

	/// Picks the runtime to install for a required feature release.
	///
	/// Same rule as [`crate::select_compatible`] applies to installed JVMs:
	/// an exact major wins, otherwise the smallest major above it, because a
	/// newer JVM runs older bytecode and the reverse does not.
	pub fn select(
		&self,
		runtime_os: &str,
		required_major: u32,
	) -> Result<RuntimeSelection, RuntimeError> {
		let available = self.components_for(runtime_os);
		let unavailable = || RuntimeError::NoRuntimeAvailable {
			required: required_major,
			runtime_os: runtime_os.to_string(),
		};
		if let Some(exact) = available
			.iter()
			.filter(|s| s.version.major == required_major)
			.max_by(|a, b| a.version.cmp(&b.version))
		{
			return Ok(exact.clone());
		}
		available
			.into_iter()
			.filter(|s| s.version.major > required_major)
			.min_by(|a, b| a.version.cmp(&b.version))
			.ok_or_else(unavailable)
	}
}

/// What a manifest entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
	File,
	Directory,
	Link,
}

/// One entry of a component's file manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeFile {
	/// Path relative to the installation root.
	pub path: String,
	pub kind: FileKind,
	/// Download URL, for [`FileKind::File`].
	pub url: Option<String>,
	pub sha1: Option<String>,
	pub size: Option<u64>,
	/// Whether the file needs the executable bit on Unix.
	pub executable: bool,
	/// Link target, for [`FileKind::Link`].
	pub target: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawFileManifest {
	files: BTreeMap<String, RawEntry>,
}

#[derive(Debug, Deserialize)]
struct RawEntry {
	#[serde(rename = "type")]
	kind: String,
	#[serde(default)]
	executable: bool,
	#[serde(default)]
	downloads: Option<RawDownloads>,
	#[serde(default)]
	target: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawDownloads {
	raw: RawArtifact,
}

#[derive(Debug, Deserialize)]
struct RawArtifact {
	sha1: String,
	size: u64,
	url: String,
}

/// Parses a component's file manifest into installable entries.
pub fn parse_file_manifest(bytes: &[u8], url: &str) -> Result<Vec<RuntimeFile>, RuntimeError> {
	let raw: RawFileManifest =
		serde_json::from_slice(bytes).map_err(|e| RuntimeError::Metadata {
			url: url.to_string(),
			reason: e.to_string(),
		})?;
	let mut files: Vec<RuntimeFile> = raw
		.files
		.into_iter()
		.map(|(path, entry)| {
			let kind = match entry.kind.as_str() {
				"directory" => FileKind::Directory,
				"link" => FileKind::Link,
				_ => FileKind::File,
			};
			let artifact = entry.downloads.map(|d| d.raw);
			RuntimeFile {
				path,
				kind,
				url: artifact.as_ref().map(|a| a.url.clone()),
				sha1: artifact.as_ref().map(|a| a.sha1.clone()),
				size: artifact.as_ref().map(|a| a.size),
				executable: entry.executable,
				target: entry.target,
			}
		})
		.collect();
	// Deterministic order so an install is reproducible and directories are
	// created before the files inside them.
	files.sort_by(|a, b| a.path.cmp(&b.path));
	Ok(files)
}

/// Rejects a manifest path that would write outside the installation root.
///
/// The manifest is a remote document; a `../` component in it would otherwise
/// let a compromised or mistaken index write anywhere the launcher can.
fn safe_join(root: &Path, relative: &str) -> Result<PathBuf, RuntimeError> {
	let mut path = root.to_path_buf();
	for component in relative.split(['/', '\\']) {
		if component.is_empty() || component == "." {
			continue;
		}
		if component == ".." {
			return Err(RuntimeError::Install {
				path: root.to_path_buf(),
				reason: format!("manifest entry {relative:?} escapes the installation root"),
			});
		}
		path.push(component);
	}
	Ok(path)
}

/// Installs a selected runtime into `dest`, returning its `java` executable.
///
/// Existing verified files are left alone, so an interrupted install resumes
/// rather than starting over.
pub fn install_runtime(
	client: &Client,
	selection: &RuntimeSelection,
	dest: &Path,
	jobs: Jobs,
	progress: &(dyn Fn(packwand_net::BatchProgress) + Sync),
) -> Result<PathBuf, RuntimeError> {
	let manifest_request = Request::get(&selection.manifest_url);
	let bytes = client
		.get(&manifest_request)
		.map_err(|e| RuntimeError::Metadata {
			url: selection.manifest_url.clone(),
			reason: e.to_string(),
		})?;
	let files = parse_file_manifest(&bytes, &selection.manifest_url)?;

	let install_error = |reason: String| RuntimeError::Install {
		path: dest.to_path_buf(),
		reason,
	};

	for entry in files.iter().filter(|f| f.kind == FileKind::Directory) {
		let path = safe_join(dest, &entry.path)?;
		std::fs::create_dir_all(&path).map_err(|e| install_error(e.to_string()))?;
	}

	let mut downloads = Vec::new();
	for entry in files.iter().filter(|f| f.kind == FileKind::File) {
		let Some(url) = &entry.url else { continue };
		let target = safe_join(dest, &entry.path)?;
		if let Some(parent) = target.parent() {
			std::fs::create_dir_all(parent).map_err(|e| install_error(e.to_string()))?;
		}
		let checksum = entry
			.sha1
			.as_ref()
			.and_then(|hex| Checksum::parse("sha1", hex).ok());
		downloads.push(Download {
			request: Request::get(url),
			target,
			checksum,
			size: entry.size,
		});
	}
	download_all(client, &downloads, jobs, progress).map_err(|e| install_error(e.to_string()))?;

	for entry in &files {
		let path = safe_join(dest, &entry.path)?;
		match entry.kind {
			FileKind::Link => {
				let Some(target) = &entry.target else {
					continue;
				};
				create_link(&path, target).map_err(|e| install_error(e.to_string()))?;
			}
			FileKind::File if entry.executable => {
				set_executable(&path).map_err(|e| install_error(e.to_string()))?;
			}
			_ => {}
		}
	}

	let executable = executable_in(dest);
	if !executable.is_file() {
		return Err(install_error(format!(
			"the installed runtime has no {}",
			executable.display()
		)));
	}
	Ok(executable)
}

#[cfg(unix)]
fn set_executable(path: &Path) -> std::io::Result<()> {
	use std::os::unix::fs::PermissionsExt;
	let mut perms = std::fs::metadata(path)?.permissions();
	perms.set_mode(perms.mode() | 0o755);
	std::fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> std::io::Result<()> {
	Ok(())
}

#[cfg(unix)]
fn create_link(path: &Path, target: &str) -> std::io::Result<()> {
	if path.symlink_metadata().is_ok() {
		std::fs::remove_file(path)?;
	}
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}
	std::os::unix::fs::symlink(target, path)
}

/// Windows runtimes in Mojang's index contain no link entries; if one ever
/// appears, copying the target keeps the install usable without requiring the
/// symlink privilege that Windows withholds from unprivileged processes.
#[cfg(not(unix))]
fn create_link(path: &Path, target: &str) -> std::io::Result<()> {
	let Some(parent) = path.parent() else {
		return Ok(());
	};
	std::fs::create_dir_all(parent)?;
	let resolved = parent.join(target);
	if resolved.is_file() {
		std::fs::copy(&resolved, path)?;
	}
	Ok(())
}

/// Adoptium's download URL for a feature release on this host.
///
/// Only reached when Mojang's index has nothing for the platform — in
/// practice Linux on ARM, which Mojang does not publish runtimes for.
pub fn adoptium_url(major: u32) -> Result<String, RuntimeError> {
	let os = match std::env::consts::OS {
		"windows" => "windows",
		"macos" => "mac",
		"linux" => "linux",
		_ => return Err(RuntimeError::UnsupportedPlatform),
	};
	let arch = match std::env::consts::ARCH {
		"x86_64" => "x64",
		"aarch64" => "aarch64",
		"x86" => "x86",
		_ => return Err(RuntimeError::UnsupportedPlatform),
	};
	Ok(format!(
		"https://api.adoptium.net/v3/binary/latest/{major}/ga/{os}/{arch}/jre/hotspot/normal/eclipse"
	))
}

#[cfg(test)]
mod tests {
	use super::*;

	const INDEX: &str = r#"{
      "windows-x64": {
        "java-runtime-gamma": [
          {"manifest": {"sha1": "aa", "size": 1, "url": "https://example/gamma.json"},
           "version": {"name": "17.0.8", "released": "2023-08-16T00:00:00+00:00"}}
        ],
        "java-runtime-delta": [
          {"manifest": {"sha1": "bb", "size": 1, "url": "https://example/delta-old.json"},
           "version": {"name": "21.0.1", "released": "2023-10-16T00:00:00+00:00"}},
          {"manifest": {"sha1": "cc", "size": 1, "url": "https://example/delta.json"},
           "version": {"name": "21.0.5", "released": "2024-10-16T00:00:00+00:00"}}
        ],
        "jre-legacy": [
          {"manifest": {"sha1": "dd", "size": 1, "url": "https://example/legacy.json"},
           "version": {"name": "1.8.0_392", "released": "2023-10-16T00:00:00+00:00"}}
        ],
        "java-runtime-next": [
          {"manifest": {"sha1": "ee", "size": 1, "url": "https://example/next.json"},
           "version": {"name": "26-ea", "released": "2025-10-16T00:00:00+00:00"}}
        ]
      },
      "linux": {}
    }"#;

	#[test]
	fn the_feature_release_comes_from_the_entry_not_a_table() {
		// A component this code has never heard of still resolves, because
		// the major is read from its own version string.
		let catalog = Catalog::parse(INDEX.as_bytes()).unwrap();
		let components = catalog.components_for("windows-x64");
		let by_name = |name: &str| {
			components
				.iter()
				.find(|c| c.component == name)
				.unwrap()
				.clone()
		};
		assert_eq!(by_name("jre-legacy").version.major, 8);
		assert_eq!(by_name("java-runtime-gamma").version.major, 17);
		assert_eq!(by_name("java-runtime-delta").version.major, 21);
		assert_eq!(by_name("java-runtime-next").version.major, 26);
	}

	#[test]
	fn the_newest_release_of_a_component_wins() {
		let catalog = Catalog::parse(INDEX.as_bytes()).unwrap();
		let delta = catalog.select("windows-x64", 21).unwrap();
		assert_eq!(delta.version.original, "21.0.5");
		assert_eq!(delta.manifest_url, "https://example/delta.json");
	}

	#[test]
	fn selection_takes_an_exact_major_then_the_smallest_above() {
		let catalog = Catalog::parse(INDEX.as_bytes()).unwrap();
		assert_eq!(
			catalog.select("windows-x64", 8).unwrap().component,
			"jre-legacy"
		);
		assert_eq!(
			catalog.select("windows-x64", 17).unwrap().component,
			"java-runtime-gamma"
		);
		// Nothing is exactly 16, so the smallest above it is chosen.
		assert_eq!(
			catalog.select("windows-x64", 16).unwrap().component,
			"java-runtime-gamma"
		);
		// Nothing at all satisfies this, and nothing is on the linux key.
		assert!(catalog.select("windows-x64", 99).is_err());
		assert!(catalog.select("linux", 17).is_err());
		assert!(catalog.select("no-such-platform", 17).is_err());
	}

	#[test]
	fn file_manifests_split_into_files_directories_and_links() {
		let manifest = r#"{"files": {
          "bin": {"type": "directory"},
          "bin/java": {"type": "file", "executable": true,
            "downloads": {"raw": {"sha1": "abc", "size": 42, "url": "https://example/java"}}},
          "legal/link": {"type": "link", "target": "../other"}
        }}"#;
		let files = parse_file_manifest(manifest.as_bytes(), "u").unwrap();
		assert_eq!(files.len(), 3);
		assert_eq!(files[0].path, "bin");
		assert_eq!(files[0].kind, FileKind::Directory);
		assert_eq!(files[1].path, "bin/java");
		assert!(files[1].executable);
		assert_eq!(files[1].size, Some(42));
		assert_eq!(files[1].url.as_deref(), Some("https://example/java"));
		assert_eq!(files[2].kind, FileKind::Link);
		assert_eq!(files[2].target.as_deref(), Some("../other"));
	}

	#[test]
	fn a_manifest_path_cannot_escape_the_installation_root() {
		let root = Path::new("/runtimes/delta");
		assert!(safe_join(root, "../../etc/passwd").is_err());
		assert!(safe_join(root, "bin/../../..").is_err());
		assert_eq!(
			safe_join(root, "bin/java").unwrap(),
			root.join("bin").join("java")
		);
	}
}
