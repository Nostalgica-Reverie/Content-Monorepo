use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use packwand_providers::{HttpRequest, Transport};

use crate::InstallerError;
use crate::index::verify;

/// Downloads a verified release to the versioned cache and delegates to it.
/// A failed update attempt runs the last-known-good binary when one exists.
pub fn download_and_delegate(
	transport: &dyn Transport,
	url: &str,
	cache_root: &Path,
	version: &str,
	hash_format: &str,
	hash: &str,
	last_known_good: Option<&Path>,
	args: impl IntoIterator<Item = String>,
) -> Result<ExitStatus, InstallerError> {
	if version.is_empty()
		|| !version
			.chars()
			.all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
	{
		return Err(InstallerError::InvalidPath(
			"invalid installer version".into(),
		));
	}
	let candidate = cached_binary(cache_root, version);
	let update = if candidate.is_file() {
		Ok(())
	} else {
		stage_update(transport, url, &candidate, hash_format, hash)
	};
	match update {
		Ok(()) => delegate(&candidate, last_known_good, args),
		Err(error) => match last_known_good.filter(|path| path.is_file()) {
			Some(fallback) => delegate(fallback, None, args),
			None => Err(error),
		},
	}
}

fn stage_update(
	transport: &dyn Transport,
	url: &str,
	candidate: &Path,
	hash_format: &str,
	hash: &str,
) -> Result<(), InstallerError> {
	let bytes = transport
		.get_large(HttpRequest::get(url))
		.map_err(|error| InstallerError::Transport(error.to_string()))?;
	verify(url, hash_format, hash, &bytes)?;
	let parent = candidate
		.parent()
		.ok_or_else(|| InstallerError::InvalidPath(candidate.display().to_string()))?;
	fs::create_dir_all(parent)?;
	let staging = candidate.with_extension("pw-part");
	fs::write(&staging, bytes)?;
	set_executable(&staging)?;
	fs::rename(staging, candidate)?;
	Ok(())
}

/// Delegates installer arguments to an updated binary, falling back to the
/// last-known-good executable when the candidate is unavailable.
pub fn delegate(
	candidate: &Path,
	last_known_good: Option<&Path>,
	args: impl IntoIterator<Item = String>,
) -> Result<ExitStatus, InstallerError> {
	let arguments = args.into_iter().collect::<Vec<_>>();
	match Command::new(candidate).args(&arguments).status() {
		Ok(status) => Ok(status),
		Err(candidate_error) => {
			let fallback = last_known_good
				.filter(|path| *path != candidate && path.is_file())
				.ok_or(candidate_error)?;
			Command::new(fallback)
				.args(arguments)
				.status()
				.map_err(Into::into)
		}
	}
}

/// Stable cache destination for one downloaded installer release.
pub fn cached_binary(root: &Path, version: &str) -> PathBuf {
	let executable = if cfg!(windows) {
		"packwand-installer.exe"
	} else {
		"packwand-installer"
	};
	root.join("installer").join(version).join(executable)
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), InstallerError> {
	use std::os::unix::fs::PermissionsExt;
	let mut permissions = fs::metadata(path)?.permissions();
	permissions.set_mode(0o755);
	fs::set_permissions(path, permissions)?;
	Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), InstallerError> {
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::{cached_binary, download_and_delegate, stage_update};
	use packwand_pack::{HashFormat, hash_bytes};
	use packwand_providers::{HttpRequest, Transport, TransportError};

	struct MemoryTransport(Result<Vec<u8>, TransportError>);

	impl Transport for MemoryTransport {
		fn get(&self, _request: HttpRequest) -> Result<Vec<u8>, TransportError> {
			self.0.clone()
		}
	}

	#[test]
	fn stages_a_verified_versioned_binary() {
		let root = tempfile::tempdir().unwrap();
		let bytes = b"installer release".to_vec();
		let candidate = cached_binary(root.path(), "27.0.0");
		stage_update(
			&MemoryTransport(Ok(bytes.clone())),
			"https://example.invalid/installer",
			&candidate,
			"sha256",
			&hash_bytes(HashFormat::Sha256, &bytes),
		)
		.unwrap();
		assert_eq!(std::fs::read(candidate).unwrap(), bytes);
	}

	#[test]
	fn failed_download_delegates_to_last_known_good() {
		let root = tempfile::tempdir().unwrap();
		let fallback = platform_shell();
		let status = download_and_delegate(
			&MemoryTransport(Err(TransportError {
				url: "https://example.invalid/installer".into(),
				message: "offline".into(),
				status: None,
				body_snippet: None,
			})),
			"https://example.invalid/installer",
			root.path(),
			"27.0.0",
			"sha256",
			"unused",
			Some(&fallback),
			platform_success_args(),
		)
		.unwrap();
		assert!(status.success());
	}

	#[cfg(windows)]
	fn platform_shell() -> std::path::PathBuf {
		std::env::var_os("ComSpec")
			.map(Into::into)
			.expect("ComSpec should point to cmd.exe")
	}

	#[cfg(windows)]
	fn platform_success_args() -> Vec<String> {
		vec!["/D".into(), "/C".into(), "exit 0".into()]
	}

	#[cfg(not(windows))]
	fn platform_shell() -> std::path::PathBuf {
		"/bin/sh".into()
	}

	#[cfg(not(windows))]
	fn platform_success_args() -> Vec<String> {
		vec!["-c".into(), "exit 0".into()]
	}
}
