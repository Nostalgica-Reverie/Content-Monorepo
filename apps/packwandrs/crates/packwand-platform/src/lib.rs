//! Focused operating-system services used by Packwand.
//!
//! This crate intentionally exposes product capabilities, not a general
//! kernel ABI. Platform-specific unsafe code is isolated in target modules.

#![deny(unsafe_code)]

mod shell;
mod trace;

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use uuid::Uuid;

pub use shell::{ShellError, ShellOutcome, shell_exec, shell_parse};
pub use trace::{TraceLevel, TraceRecord, trace, trace_drain, trace_dropped};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("unsafe relative path: {0}")]
	UnsafePath(String),
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error("file watcher error: {0}")]
	Watch(String),
	#[error("credential store error: {0}")]
	Credential(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn validate_relative_path(relative: &str) -> Result<()> {
	if relative.contains('\0') || relative.contains(':') || relative.starts_with(['/', '\\']) {
		return Err(Error::UnsafePath(relative.to_owned()));
	}
	let normalized = relative.replace('\\', "/");
	if normalized
		.split('/')
		.any(|part| part == "." || part == "..")
	{
		return Err(Error::UnsafePath(relative.to_owned()));
	}
	let path = Path::new(&normalized);
	if path
		.components()
		.any(|component| !matches!(component, Component::Normal(_)))
		&& !relative.is_empty()
	{
		return Err(Error::UnsafePath(relative.to_owned()));
	}
	Ok(())
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
	let parent = path
		.parent()
		.ok_or_else(|| Error::UnsafePath(path.display().to_string()))?;
	fs::create_dir_all(parent)?;
	let filename = path
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("packwand");
	let id = Uuid::new_v4();
	let temporary = parent.join(format!(".{filename}.{id}.tmp"));
	let backup = parent.join(format!(".{filename}.{id}.backup"));
	fs::write(&temporary, bytes)?;
	let had_target = path.exists();
	if had_target && let Err(error) = fs::rename(path, &backup) {
		let _ = fs::remove_file(&temporary);
		return Err(error.into());
	}
	if let Err(error) = fs::rename(&temporary, path) {
		if had_target {
			let _ = fs::rename(&backup, path);
		}
		let _ = fs::remove_file(&temporary);
		return Err(error.into());
	}
	if had_target {
		let _ = fs::remove_file(backup);
	}
	Ok(())
}

pub struct WorkspaceWatcher {
	_watcher: RecommendedWatcher,
	receiver: Receiver<notify::Result<Event>>,
	root: PathBuf,
	cancelled: Arc<AtomicBool>,
}

pub struct WorkspaceWatchCanceller(Arc<AtomicBool>);

impl WorkspaceWatchCanceller {
	pub fn cancel(&self) {
		self.0.store(true, Ordering::Release);
	}
}

impl WorkspaceWatcher {
	pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
		let root = root.into();
		let (sender, receiver) = mpsc::channel();
		let mut watcher = notify::recommended_watcher(move |event| {
			let _ = sender.send(event);
		})
		.map_err(|error| Error::Watch(error.to_string()))?;
		watcher
			.watch(&root, RecursiveMode::Recursive)
			.map_err(|error| Error::Watch(error.to_string()))?;
		Ok(Self {
			_watcher: watcher,
			receiver,
			root,
			cancelled: Arc::new(AtomicBool::new(false)),
		})
	}

	pub fn read_changes(&self) -> Result<Vec<PathBuf>> {
		if self.cancelled.load(Ordering::Acquire) {
			return Err(Error::Watch("cancelled".into()));
		}
		let event = match self.receiver.recv_timeout(Duration::from_millis(250)) {
			Ok(event) => event,
			Err(mpsc::RecvTimeoutError::Timeout) => return Ok(Vec::new()),
			Err(error) => return Err(Error::Watch(error.to_string())),
		}
		.map_err(|error| Error::Watch(error.to_string()))?;
		Ok(event
			.paths
			.into_iter()
			.filter_map(|path| path.strip_prefix(&self.root).ok().map(Path::to_path_buf))
			.collect())
	}

	pub fn canceller(&self) -> WorkspaceWatchCanceller {
		WorkspaceWatchCanceller(Arc::clone(&self.cancelled))
	}
}

/// The keychain entry name the MSA refresh token has always used.
///
/// Named because it is the one key with a legacy migration path attached; new
/// keys have no history to carry.
pub const MSA_REFRESH_TOKEN_KEY: &str = "msa-refresh-token";

/// One secret in the OS keychain, addressed by name under the `packwand`
/// service.
pub struct CredentialStore {
	entry: keyring::Entry,
	key: String,
}

impl CredentialStore {
	/// The MSA refresh token store, which additionally migrates the secret out
	/// of the pre-keyring Windows credential blob on first read.
	pub fn new() -> Result<Self> {
		Self::for_key(MSA_REFRESH_TOKEN_KEY)
	}

	/// A store for any other named secret — provider tokens, API keys.
	pub fn for_key(key: &str) -> Result<Self> {
		let entry = keyring::Entry::new("packwand", key)
			.map_err(|error| Error::Credential(error.to_string()))?;
		Ok(Self {
			entry,
			key: key.to_owned(),
		})
	}

	pub fn save(&self, secret: &str) -> Result<()> {
		self.entry
			.set_password(secret)
			.map_err(|error| Error::Credential(error.to_string()))
	}

	pub fn load(&self) -> Result<Option<String>> {
		match self.entry.get_password() {
			Ok(secret) => Ok(Some(secret)),
			Err(keyring::Error::NoEntry) => {
				#[cfg(windows)]
				if self.key == MSA_REFRESH_TOKEN_KEY
					&& let Some(secret) = legacy_windows_credential::load()?
				{
					self.save(&secret)?;
					legacy_windows_credential::delete()?;
					return Ok(Some(secret));
				}
				Ok(None)
			}
			Err(error) => Err(Error::Credential(error.to_string())),
		}
	}

	pub fn clear(&self) -> Result<()> {
		match self.entry.delete_credential() {
			Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
			Err(error) => Err(Error::Credential(error.to_string())),
		}
	}
}

/// Adapts the OS keychain to [`packwand_auth::CredentialStore`].
///
/// A fresh [`keyring::Entry`] per operation rather than a cached one: entries
/// are per-key, the trait is keyed per call, and keyring lookups are cheap
/// next to the IPC round trip that reaches this at all.
#[derive(Debug, Default, Clone, Copy)]
pub struct KeyringCredentialStore;

impl packwand_auth::CredentialStore for KeyringCredentialStore {
	fn get(
		&self,
		key: &str,
	) -> std::result::Result<Option<packwand_auth::SecretString>, packwand_auth::AuthError> {
		CredentialStore::for_key(key)
			.and_then(|store| store.load())
			.map(|value| value.map(packwand_auth::SecretString::new))
			.map_err(|error| store_error(key, error))
	}

	fn set(
		&self,
		key: &str,
		value: packwand_auth::SecretString,
	) -> std::result::Result<(), packwand_auth::AuthError> {
		CredentialStore::for_key(key)
			.and_then(|store| store.save(value.expose()))
			.map_err(|error| store_error(key, error))
	}

	fn delete(&self, key: &str) -> std::result::Result<(), packwand_auth::AuthError> {
		CredentialStore::for_key(key)
			.and_then(|store| store.clear())
			.map_err(|error| store_error(key, error))
	}
}

fn store_error(key: &str, error: Error) -> packwand_auth::AuthError {
	packwand_auth::AuthError::Store {
		key: key.to_owned(),
		message: error.to_string(),
	}
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod legacy_windows_credential {
	use std::slice;

	use windows::Win32::Foundation::{ERROR_NOT_FOUND, GetLastError};
	use windows::Win32::Security::Credentials::{
		CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree, CredReadW,
	};
	use windows::core::w;

	use super::{Error, Result};

	pub fn load() -> Result<Option<String>> {
		let mut credential: *mut CREDENTIALW = std::ptr::null_mut();
		if let Err(error) = unsafe {
			CredReadW(
				w!("packwand/msa-refresh-token"),
				CRED_TYPE_GENERIC,
				None,
				&mut credential,
			)
		} {
			if unsafe { GetLastError() } == ERROR_NOT_FOUND {
				return Ok(None);
			}
			return Err(Error::Credential(format!(
				"could not read legacy credential: {error}"
			)));
		}
		if credential.is_null() {
			return Ok(None);
		}
		let value = unsafe {
			let credential_ref = &*credential;
			let bytes = slice::from_raw_parts(
				credential_ref.CredentialBlob,
				credential_ref.CredentialBlobSize as usize,
			);
			String::from_utf8(bytes.to_vec())
		};
		unsafe {
			CredFree(credential.cast());
		}
		value
			.map(Some)
			.map_err(|error| Error::Credential(format!("legacy credential is not UTF-8: {error}")))
	}

	pub fn delete() -> Result<()> {
		match unsafe { CredDeleteW(w!("packwand/msa-refresh-token"), CRED_TYPE_GENERIC, None) } {
			Ok(()) => Ok(()),
			Err(_) if unsafe { GetLastError() } == ERROR_NOT_FOUND => Ok(()),
			Err(error) => Err(Error::Credential(format!(
				"could not delete migrated legacy credential: {error}"
			))),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::validate_relative_path;

	#[test]
	fn relative_paths_reject_escape_forms() {
		assert!(validate_relative_path("data/example/file.json").is_ok());
		assert!(validate_relative_path("").is_ok());
		for invalid in [
			"../secret",
			"data/../secret",
			"/absolute",
			"C:\\absolute",
			"data\\..\\secret",
		] {
			assert!(validate_relative_path(invalid).is_err(), "{invalid}");
		}
	}
}
