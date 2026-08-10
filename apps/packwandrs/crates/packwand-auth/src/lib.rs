//! Account sessions and the credential-store abstraction.
//!
//! Part of the shared Packwand core. This crate
//! must stay free of Tauri, clap, and axum dependencies.
//!
//! Secrets are held in [`SecretString`], which cannot be serialized and
//! redacts itself in `Debug`/`Display` output, so tokens cannot leak into
//! plans, logs, or events by accident. Only the process-boundary code that
//! actually spawns the game may call [`SecretString::expose`].
//!
//! Only offline sessions are implemented. Microsoft/Minecraft OAuth is a
//! separate, threat-modeled subsystem and must
//! not be bolted onto this crate ad hoc; the [`CredentialStore`] trait is
//! the seam where an OS-keychain-backed implementation will plug in.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Mutex;

use md5::{Digest, Md5};

/// A secret value that never appears in serialized or formatted output.
///
/// Deliberately implements neither `serde::Serialize` nor `Deserialize`.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
	/// Wraps a secret value that will be redacted in Debug and Display output.
	pub fn new(value: impl Into<String>) -> Self {
		Self(value.into())
	}

	/// The raw secret. Call sites should be limited to the code that hands
	/// the value to the child process or the credential store.
	pub fn expose(&self) -> &str {
		&self.0
	}
}

impl fmt::Debug for SecretString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("SecretString(«redacted»)")
	}
}

impl fmt::Display for SecretString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("«redacted»")
	}
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
	#[error("credential store failure for key {key:?}: {message}")]
	Store { key: String, message: String },
	#[error("username {0:?} is invalid: 1-16 characters, no whitespace")]
	InvalidUsername(String),
}

/// Storage for named secrets.
///
/// Implementations must persist only what they are given and must never log
/// secret values. The production implementation will use OS credential
/// storage; [`InMemoryCredentialStore`] exists for tests and for sessions
/// that must not outlive the process.
pub trait CredentialStore: Send + Sync {
	fn get(&self, key: &str) -> Result<Option<SecretString>, AuthError>;
	fn set(&self, key: &str, value: SecretString) -> Result<(), AuthError>;
	fn delete(&self, key: &str) -> Result<(), AuthError>;
}

/// Process-lifetime credential store for tests and offline sessions.
#[derive(Default)]
pub struct InMemoryCredentialStore {
	entries: Mutex<BTreeMap<String, SecretString>>,
}

impl InMemoryCredentialStore {
	/// Creates a new in-memory credential store.
	pub fn new() -> Self {
		Self::default()
	}

	fn lock(
		&self,
		key: &str,
	) -> Result<std::sync::MutexGuard<'_, BTreeMap<String, SecretString>>, AuthError> {
		self.entries.lock().map_err(|_| AuthError::Store {
			key: key.to_string(),
			message: "store mutex poisoned".to_string(),
		})
	}
}

impl CredentialStore for InMemoryCredentialStore {
	fn get(&self, key: &str) -> Result<Option<SecretString>, AuthError> {
		Ok(self.lock(key)?.get(key).cloned())
	}

	fn set(&self, key: &str, value: SecretString) -> Result<(), AuthError> {
		self.lock(key)?.insert(key.to_string(), value);
		Ok(())
	}

	fn delete(&self, key: &str) -> Result<(), AuthError> {
		self.lock(key)?.remove(key);
		Ok(())
	}
}

/// A resolved account session for one launch.
///
/// `username`, `uuid`, and `user_type` are identity, not secrets; the
/// access token is the only secret-shaped value.
#[derive(Debug, Clone)]
pub struct Session {
	pub username: String,
	/// Hyphenated lowercase UUID.
	pub uuid: String,
	/// Minecraft's `${user_type}`: `msa` for Microsoft accounts,
	/// `legacy`/`mojang` historically; offline sessions use `legacy`.
	pub user_type: String,
	pub access_token: SecretString,
}

impl Session {
	/// Secret values keyed by the placeholder names a launch plan may
	/// reference as `${secret:<name>}`.
	pub fn secrets(&self) -> BTreeMap<String, SecretString> {
		BTreeMap::from([("auth_access_token".to_string(), self.access_token.clone())])
	}

	/// Non-secret account values, keyed by the `${identity:<name>}`
	/// placeholders a launch plan may reference.
	///
	/// Separate from [`Self::secrets`] because these are not sensitive and
	/// must stay readable — they end up in a log line the user is expected to
	/// paste. `auth_xuid` and `clientid` are empty for every account type this
	/// launcher supports, but the arguments still reference them.
	pub fn identity(&self) -> BTreeMap<String, String> {
		BTreeMap::from([
			("auth_player_name".to_string(), self.username.clone()),
			("profile_name".to_string(), self.username.clone()),
			("auth_uuid".to_string(), self.uuid.clone()),
			("user_type".to_string(), self.user_type.clone()),
			("auth_xuid".to_string(), String::new()),
			("clientid".to_string(), String::new()),
		])
	}
}

/// The UUID Mojang's own code derives for offline players:
/// a version-3 (md5) UUID of `OfflinePlayer:<name>`, matching Java's
/// `UUID.nameUUIDFromBytes`.
pub fn offline_player_uuid(username: &str) -> String {
	let mut digest: [u8; 16] = Md5::digest(format!("OfflinePlayer:{username}").as_bytes()).into();
	digest[6] = (digest[6] & 0x0f) | 0x30; // version 3
	digest[8] = (digest[8] & 0x3f) | 0x80; // IETF variant
	let h: Vec<String> = digest.iter().map(|b| format!("{b:02x}")).collect();
	format!(
		"{}{}{}{}-{}{}-{}{}-{}{}-{}{}{}{}{}{}",
		h[0],
		h[1],
		h[2],
		h[3],
		h[4],
		h[5],
		h[6],
		h[7],
		h[8],
		h[9],
		h[10],
		h[11],
		h[12],
		h[13],
		h[14],
		h[15]
	)
}

/// Builds an offline (unauthenticated) session for the given username.
///
/// The access token is a fixed non-empty placeholder: the vanilla client
/// requires the argument to be present but performs no validation offline.
pub fn offline_session(username: &str) -> Result<Session, AuthError> {
	let valid =
		!username.is_empty() && username.len() <= 16 && !username.chars().any(char::is_whitespace);
	if !valid {
		return Err(AuthError::InvalidUsername(username.to_string()));
	}
	Ok(Session {
		username: username.to_string(),
		uuid: offline_player_uuid(username),
		user_type: "legacy".to_string(),
		access_token: SecretString::new("offline"),
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn secret_string_redacts_debug_and_display() {
		let secret = SecretString::new("hunter2");
		assert_eq!(format!("{secret:?}"), "SecretString(«redacted»)");
		assert_eq!(format!("{secret}"), "«redacted»");
		assert_eq!(secret.expose(), "hunter2");
	}

	#[test]
	fn offline_uuid_matches_java_name_uuid_from_bytes() {
		// Precomputed: UUID.nameUUIDFromBytes("OfflinePlayer:Notch".getBytes(UTF_8)).
		assert_eq!(
			offline_player_uuid("Notch"),
			"b50ad385-829d-3141-a216-7e7d7539ba7f"
		);
		// Version and variant bits are always stamped.
		let uuid = offline_player_uuid("someone_else");
		assert_eq!(&uuid[14..15], "3");
		assert!(matches!(&uuid[19..20], "8" | "9" | "a" | "b"));
	}

	#[test]
	fn offline_session_validates_username() {
		assert!(offline_session("").is_err());
		assert!(offline_session("has space").is_err());
		assert!(offline_session("seventeen_letters").is_err());
		let session = offline_session("Steve").unwrap();
		assert_eq!(session.user_type, "legacy");
		assert_eq!(session.uuid, offline_player_uuid("Steve"));
		assert_eq!(
			session.secrets().keys().collect::<Vec<_>>(),
			vec!["auth_access_token"]
		);
	}

	#[test]
	fn in_memory_store_roundtrip() {
		let store = InMemoryCredentialStore::new();
		assert!(store.get("token").unwrap().is_none());
		store.set("token", SecretString::new("value")).unwrap();
		assert_eq!(store.get("token").unwrap().unwrap().expose(), "value");
		store.delete("token").unwrap();
		assert!(store.get("token").unwrap().is_none());
	}
}
