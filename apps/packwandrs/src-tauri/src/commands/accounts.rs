//! Linking the app to Modrinth and CurseForge, and holding the credentials.
//!
//! Two providers, two genuinely different mechanisms, and the difference is
//! not a design choice on our side:
//!
//!  - **Modrinth** issues personal access tokens from the user's settings page
//!    and validates them against `GET /v2/user`. Modrinth also runs an OAuth
//!    flow, but its launcher endpoint is scoped to the official client, so a
//!    third-party app cannot use it without its own registered client id.
//!    Token entry is the path that works today; see `link_modrinth`.
//!  - **CurseForge** has no third-party user OAuth at all. "Connecting"
//!    CurseForge means storing an API key from their developer console, so the
//!    UI must say *Connect*, never *Sign in*.
//!
//! Secrets never cross the IPC boundary outward. Every type here that reaches
//! the frontend reports whether a credential exists and who it belongs to —
//! never the credential. [`packwand_auth::SecretString`] implements neither
//! `Serialize` nor `Deserialize`, which makes that a compile-time property
//! rather than a review checklist item.

use packwand_auth::{CredentialStore, SecretString};
use packwand_platform::KeyringCredentialStore;
use packwand_providers::{HttpRequest, Transport, UreqTransport};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::off_thread;
use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

/// Keychain entry names. Stable strings: changing one silently orphans a
/// stored secret rather than failing, and the user sees a working link turn
/// into an empty one for no visible reason.
pub const MODRINTH_TOKEN_KEY: &str = "modrinth-token";
pub const CURSEFORGE_API_KEY: &str = "curseforge-api-key";
pub const CURSEFORGE_UPLOAD_TOKEN_KEY: &str = "curseforge-upload-token";

/// Cached display identity, so the UI can name the account without a network
/// round trip on every render. Not a secret: a username is already public.
const MODRINTH_IDENTITY_KEY: &str = "modrinth-identity";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
// snake_case to match `ProviderKind`, which the frontend already switches on.
#[serde(rename_all = "snake_case")]
pub enum AccountProvider {
	Modrinth,
	CurseForge,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountState {
	pub provider: AccountProvider,
	pub linked: bool,
	/// Username for Modrinth; `None` for CurseForge, which has no user
	/// resource behind an API key.
	pub identity: Option<String>,
	/// Whether the credential needed to *publish* is present, which is a
	/// different credential from the one needed to browse on CurseForge.
	pub can_publish: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountsSnapshot {
	pub accounts: Vec<AccountState>,
	/// True when every credential publishing needs is present.
	pub can_publish: bool,
}

fn store() -> KeyringCredentialStore {
	KeyringCredentialStore
}

fn secret(key: &str) -> CommandResult<Option<String>> {
	store()
		.get(key)
		.map(|value| value.map(|secret| secret.expose().to_owned()))
		.map_err(|error| SerializableError::new("credential_store", error.to_string()))
}

fn put(key: &str, value: &str) -> CommandResult<()> {
	store()
		.set(key, SecretString::new(value))
		.map_err(|error| SerializableError::new("credential_store", error.to_string()))
}

fn forget(key: &str) -> CommandResult<()> {
	store()
		.delete(key)
		.map_err(|error| SerializableError::new("credential_store", error.to_string()))
}

/// The stored CurseForge API key, if the user has connected one.
///
/// Public because `commands::providers` prefers it over the environment
/// variables before falling back to `configured_api_key`.
pub fn stored_curseforge_key() -> Option<String> {
	secret(CURSEFORGE_API_KEY)
		.ok()
		.flatten()
		.filter(|value| !value.trim().is_empty())
}

/// The stored Modrinth token, if any.
pub fn stored_modrinth_token() -> Option<String> {
	secret(MODRINTH_TOKEN_KEY)
		.ok()
		.flatten()
		.filter(|value| !value.trim().is_empty())
}

/// The stored CurseForge upload token, which is distinct from the read key.
pub fn stored_curseforge_upload_token() -> Option<String> {
	secret(CURSEFORGE_UPLOAD_TOKEN_KEY)
		.ok()
		.flatten()
		.filter(|value| !value.trim().is_empty())
}

#[tauri::command]
pub async fn accounts_state() -> CommandResult<AccountsSnapshot> {
	off_thread(snapshot).await
}

fn snapshot() -> CommandResult<AccountsSnapshot> {
	let modrinth_token = stored_modrinth_token();
	let curseforge_key = stored_curseforge_key();
	let curseforge_upload = stored_curseforge_upload_token();
	let accounts = vec![
		AccountState {
			provider: AccountProvider::Modrinth,
			linked: modrinth_token.is_some(),
			identity: secret(MODRINTH_IDENTITY_KEY).ok().flatten(),
			// Modrinth publishes with the same token it reads with.
			can_publish: modrinth_token.is_some(),
		},
		AccountState {
			provider: AccountProvider::CurseForge,
			linked: curseforge_key.is_some(),
			identity: None,
			can_publish: curseforge_upload.is_some(),
		},
	];
	let can_publish = accounts.iter().all(|account| account.can_publish);
	Ok(AccountsSnapshot {
		accounts,
		can_publish,
	})
}

/// The cached public Modrinth username used as the collaboration display
/// identity. Credentials remain inside the keychain boundary.
pub(crate) fn stored_modrinth_identity() -> Option<String> {
	secret(MODRINTH_IDENTITY_KEY)
		.ok()
		.flatten()
		.filter(|value| !value.trim().is_empty())
}

#[derive(Debug, Deserialize)]
struct ModrinthUser {
	username: String,
}

/// Confirms a Modrinth token works, returning the username it belongs to.
///
/// Validating before storing matters more than it looks: a mistyped token
/// stored silently turns every later publish into an authentication failure
/// far away from the screen where the mistake was made.
fn verify_modrinth_token(token: &str) -> CommandResult<String> {
	let request =
		HttpRequest::get("https://api.modrinth.com/v2/user").header("Authorization", token);
	let bytes = UreqTransport::new().get(request).map_err(|error| {
		if error.status == Some(401) {
			SerializableError::new(
				"modrinth_token",
				"Modrinth rejected that token. Check it was copied whole and has not expired.",
			)
		} else {
			SerializableError::new("modrinth_token", error.message)
		}
	})?;
	let user: ModrinthUser = serde_json::from_slice(&bytes).map_err(|error| {
		SerializableError::new(
			"modrinth_token",
			format!("Modrinth returned an unexpected response: {error}"),
		)
	})?;
	Ok(user.username)
}

/// Confirms a CurseForge API key works.
///
/// `/v1/games` is the cheapest authenticated endpoint: it needs the key, is
/// tiny, and does not depend on any project existing.
fn verify_curseforge_key(key: &str) -> CommandResult<()> {
	let request = HttpRequest::get("https://api.curseforge.com/v1/games").header("X-API-Key", key);
	UreqTransport::new().get(request).map_err(|error| {
		if matches!(error.status, Some(401) | Some(403)) {
			SerializableError::new(
				"curseforge_key",
				"CurseForge rejected that key. Keys come from the CurseForge developer console.",
			)
		} else {
			SerializableError::new("curseforge_key", error.message)
		}
	})?;
	Ok(())
}

/// Links Modrinth from a personal access token.
///
/// Named `link_modrinth` rather than `sign_in`: no browser flow happens here.
/// If Modrinth ever registers a client id for this app, an OAuth command joins
/// this one and both write the same keychain entry, so nothing downstream
/// changes.
#[tauri::command]
pub async fn accounts_link_modrinth(token: String) -> CommandResult<AccountsSnapshot> {
	let token = token.trim().to_owned();
	if token.is_empty() {
		return Err(SerializableError::new(
			"modrinth_token",
			"paste a Modrinth personal access token",
		));
	}
	off_thread(move || {
		let username = verify_modrinth_token(&token)?;
		put(MODRINTH_TOKEN_KEY, &token)?;
		put(MODRINTH_IDENTITY_KEY, &username)?;
		snapshot()
	})
	.await
}

#[tauri::command]
pub async fn accounts_link_curseforge(api_key: String) -> CommandResult<AccountsSnapshot> {
	let api_key = api_key.trim().to_owned();
	if api_key.is_empty() {
		return Err(SerializableError::new(
			"curseforge_key",
			"paste a CurseForge API key",
		));
	}
	off_thread(move || {
		verify_curseforge_key(&api_key)?;
		put(CURSEFORGE_API_KEY, &api_key)?;
		snapshot()
	})
	.await
}

/// Stores the CurseForge *upload* token.
///
/// Deliberately separate from [`accounts_link_curseforge`]: the read API key
/// and the upload token are different credentials from different pages, and
/// conflating them produces a link that browses fine and fails at publish.
/// Not verified on entry — CurseForge exposes no cheap endpoint that accepts
/// this token without attempting a real upload.
#[tauri::command]
pub async fn accounts_set_publish_token(token: String) -> CommandResult<AccountsSnapshot> {
	let token = token.trim().to_owned();
	off_thread(move || {
		if token.is_empty() {
			forget(CURSEFORGE_UPLOAD_TOKEN_KEY)?;
		} else {
			put(CURSEFORGE_UPLOAD_TOKEN_KEY, &token)?;
		}
		snapshot()
	})
	.await
}

#[tauri::command]
pub async fn accounts_unlink(provider: AccountProvider) -> CommandResult<AccountsSnapshot> {
	off_thread(move || {
		match provider {
			AccountProvider::Modrinth => {
				forget(MODRINTH_TOKEN_KEY)?;
				forget(MODRINTH_IDENTITY_KEY)?;
			}
			AccountProvider::CurseForge => {
				forget(CURSEFORGE_API_KEY)?;
				forget(CURSEFORGE_UPLOAD_TOKEN_KEY)?;
			}
		}
		snapshot()
	})
	.await
}

/// The stored credentials, in the shape `packwand-build` wants.
///
/// `.or_env()` keeps the environment as a fallback, so a developer who
/// exported `MODRINTH_TOKEN` in their shell still publishes without connecting
/// anything, and Forgejo Actions is untouched. Credentials the user connected
/// take precedence, because that is the more deliberate of the two.
pub fn publish_credentials() -> packwand_build::PublishCredentials {
	packwand_build::PublishCredentials {
		modrinth_token: stored_modrinth_token(),
		curseforge_token: stored_curseforge_upload_token(),
		curseforge_api_key: stored_curseforge_key(),
	}
	.or_env()
}

/// Whether a publish would have every credential it needs.
///
/// Lets the UI disable the button and say which link is missing, rather than
/// letting the user discover it from a failed upload halfway through a job.
#[tauri::command]
pub async fn accounts_prepare_publish(_state: State<'_, AppState>) -> CommandResult<bool> {
	off_thread(|| {
		let credentials = publish_credentials();
		Ok(credentials.modrinth_token.is_some() || credentials.curseforge_token.is_some())
	})
	.await
}
