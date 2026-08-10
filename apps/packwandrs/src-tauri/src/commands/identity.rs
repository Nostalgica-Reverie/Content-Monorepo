use packwand_identity_client::{Identity, IdentityClient};

use crate::commands::off_thread;
use crate::error::{CommandResult, SerializableError};

/// Starts ATProto OAuth in the social helper and returns the public identity.
#[tauri::command]
pub async fn account_login(identifier: String) -> CommandResult<Identity> {
	let identifier = identifier.trim().to_owned();
	if identifier.is_empty() {
		return Err(SerializableError::new(
			"atproto_identity",
			"enter an ATProto handle or DID",
		));
	}
	off_thread(move || {
		IdentityClient::new()
			.and_then(|client| client.login(Some(&identifier)))
			.map_err(identity_error)
	})
	.await
}

/// Returns the persisted ATProto identity without resolving it over the network.
#[tauri::command]
pub async fn account_whoami() -> CommandResult<Option<Identity>> {
	off_thread(|| {
		IdentityClient::new()
			.and_then(|client| client.whoami())
			.map_err(identity_error)
	})
	.await
}

/// Revokes and clears the current ATProto OAuth session.
#[tauri::command]
pub async fn account_logout() -> CommandResult<()> {
	off_thread(|| {
		IdentityClient::new()
			.and_then(|client| client.logout())
			.map_err(identity_error)
	})
	.await
}

fn identity_error(error: packwand_identity_client::Error) -> SerializableError {
	SerializableError::new("atproto_identity", error.to_string())
}
