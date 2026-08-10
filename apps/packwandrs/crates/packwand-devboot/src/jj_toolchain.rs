use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use packwand_minecraft::InstallProgress;
use packwand_providers::{HttpRequest, Transport, UreqTransport};

use crate::DevBootError;

/// Standalone Jujutsu version managed independently from the embedded library.
pub const PINNED_JJ_VERSION: &str = "0.41.0";

/// Location and exact release version for a managed Jujutsu executable.
pub struct JjToolchainRequest {
	pub root: PathBuf,
	pub version: String,
}

impl JjToolchainRequest {
	/// Creates a request for Packwand's reviewed standalone Jujutsu release.
	pub fn pinned(root: PathBuf) -> Self {
		Self {
			root,
			version: PINNED_JJ_VERSION.into(),
		}
	}
}

/// Downloads and atomically installs the pinned Jujutsu CLI for this host.
pub fn ensure_jj(
	request: &JjToolchainRequest,
	on_progress: impl Fn(InstallProgress) + Sync,
) -> Result<PathBuf, DevBootError> {
	validate_version(&request.version)?;
	let executable = if cfg!(windows) { "jj.exe" } else { "jj" };
	let destination = request
		.root
		.join("tools")
		.join("jj")
		.join(&request.version)
		.join(executable);
	if destination.is_file() {
		return Ok(destination);
	}

	let asset = release_asset(&request.version)?;
	let url = format!(
		"https://github.com/jj-vcs/jj/releases/download/v{}/{}",
		request.version, asset
	);
	let bytes = UreqTransport::for_downloads()
		.get_large(HttpRequest::get(url))
		.map_err(|error| DevBootError::Toolchain(error.to_string()))?;
	on_progress(InstallProgress {
		finished_downloads: 1,
		total_downloads: 1,
		downloaded_bytes: bytes.len() as u64,
		total_bytes: Some(bytes.len() as u64),
		current_download_bytes: bytes.len() as u64,
		current_download_total: Some(bytes.len() as u64),
	});

	let binary = extract_binary(&asset, &bytes, executable)?;
	let parent = destination
		.parent()
		.ok_or_else(|| DevBootError::Toolchain("tool destination has no parent".into()))?;
	fs::create_dir_all(parent).map_err(tool_error)?;
	let staging = destination.with_extension("pw-part");
	fs::write(&staging, binary).map_err(tool_error)?;
	set_executable(&staging)?;
	fs::rename(&staging, &destination).map_err(tool_error)?;
	Ok(destination)
}

fn validate_version(version: &str) -> Result<(), DevBootError> {
	if version.is_empty()
		|| !version
			.chars()
			.all(|character| character.is_ascii_digit() || character == '.')
	{
		return Err(DevBootError::Toolchain("invalid Jujutsu version".into()));
	}
	Ok(())
}

fn release_asset(version: &str) -> Result<String, DevBootError> {
	let triple = match (std::env::consts::OS, std::env::consts::ARCH) {
		("windows", "x86_64") => "x86_64-pc-windows-msvc.zip",
		("linux", "x86_64") => "x86_64-unknown-linux-musl.tar.gz",
		("linux", "aarch64") => "aarch64-unknown-linux-musl.tar.gz",
		("macos", "x86_64") => "x86_64-apple-darwin.tar.gz",
		("macos", "aarch64") => "aarch64-apple-darwin.tar.gz",
		(os, arch) => {
			return Err(DevBootError::Toolchain(format!(
				"Jujutsu has no managed asset for {os}/{arch}"
			)));
		}
	};
	Ok(format!("jj-v{version}-{triple}"))
}

fn extract_binary(asset: &str, bytes: &[u8], executable: &str) -> Result<Vec<u8>, DevBootError> {
	if asset.ends_with(".zip") {
		let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
			.map_err(|error| DevBootError::Toolchain(error.to_string()))?;
		for index in 0..archive.len() {
			let mut entry = archive
				.by_index(index)
				.map_err(|error| DevBootError::Toolchain(error.to_string()))?;
			if entry.enclosed_name().as_deref().and_then(Path::file_name)
				== Some(std::ffi::OsStr::new(executable))
			{
				let mut output = Vec::new();
				entry.read_to_end(&mut output).map_err(tool_error)?;
				return Ok(output);
			}
		}
	} else {
		let decoder = GzDecoder::new(Cursor::new(bytes));
		let mut archive = tar::Archive::new(decoder);
		let entries = archive
			.entries()
			.map_err(|error| DevBootError::Toolchain(error.to_string()))?;
		for entry in entries {
			let mut entry = entry.map_err(tool_error)?;
			let path = entry.path().map_err(tool_error)?;
			if path.file_name() == Some(std::ffi::OsStr::new(executable)) {
				let mut output = Vec::new();
				entry.read_to_end(&mut output).map_err(tool_error)?;
				return Ok(output);
			}
		}
	}
	Err(DevBootError::Toolchain(format!(
		"release archive did not contain {executable}"
	)))
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), DevBootError> {
	use std::os::unix::fs::PermissionsExt;
	let mut permissions = fs::metadata(path).map_err(tool_error)?.permissions();
	permissions.set_mode(0o755);
	fs::set_permissions(path, permissions).map_err(tool_error)
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), DevBootError> {
	Ok(())
}

fn tool_error(error: std::io::Error) -> DevBootError {
	DevBootError::Toolchain(error.to_string())
}

#[cfg(test)]
mod tests {
	use super::{JjToolchainRequest, PINNED_JJ_VERSION, release_asset, validate_version};

	#[test]
	fn pinned_request_is_exact() {
		let request = JjToolchainRequest::pinned("tools".into());
		assert_eq!(request.version, PINNED_JJ_VERSION);
		assert!(
			release_asset(&request.version)
				.unwrap()
				.starts_with("jj-v0.41.0-")
		);
	}

	#[test]
	fn version_cannot_inject_a_release_path() {
		assert!(validate_version("0.41.0").is_ok());
		assert!(validate_version("../latest").is_err());
	}
}
