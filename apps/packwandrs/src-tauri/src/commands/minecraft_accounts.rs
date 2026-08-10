//! Signing in to Minecraft, and choosing who plays.
//!
//! Distinct from [`crate::commands::accounts`], which links Modrinth and
//! CurseForge for *publishing*. This is the account the game itself runs as,
//! and the two have no credentials, no storage, and no failure modes in
//! common — sharing a module would only make it easy to confuse them.
//!
//! Nothing secret crosses the IPC boundary. The frontend receives names,
//! uuids and states; refresh tokens stay in the OS credential store and
//! access tokens never leave the launch path.

use std::time::Duration;

use packwand_msa::{Account, AccountList, MsaConfig};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::commands::off_thread;
use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

/// How long the loopback listener waits for the browser to come back.
///
/// Generous: the user may have to enter a password, satisfy two-factor, and
/// pick from a list of profiles. Too short reads to them as the app failing.
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

/// The account list plus whether sign-in is configured at all.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftAccounts {
	pub accounts: Vec<Account>,
	pub active_uuid: Option<String>,
	/// False when this build has no Azure client id, in which case every
	/// launch is offline and the UI should say so rather than offering a
	/// sign-in button that cannot work.
	pub sign_in_available: bool,
}

/// Where the account list lives: beside the managed install it belongs to,
/// which is the same root every launch already resolves.
fn accounts_root(app: &AppHandle) -> CommandResult<std::path::PathBuf> {
	let data_dir = app
		.path()
		.app_data_dir()
		.map_err(|error| SerializableError::new("path", error.to_string()))?;
	Ok(packwand_orchestrator::launch::managed_root(&data_dir))
}

fn store(root: &std::path::Path) -> packwand_msa::Accounts {
	packwand_msa::Accounts::new(root)
}

fn msa_error(error: impl std::fmt::Display) -> SerializableError {
	SerializableError::new("minecraft_account", error.to_string())
}

fn snapshot(
	root: &std::path::Path,
	client_id: Option<&String>,
) -> CommandResult<MinecraftAccounts> {
	let list: AccountList = store(root).load().map_err(msa_error)?;
	Ok(MinecraftAccounts {
		accounts: list.accounts,
		active_uuid: list.active_uuid,
		sign_in_available: client_id.is_some_and(|id| !id.trim().is_empty()),
	})
}

#[tauri::command]
pub async fn minecraft_accounts_list(
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<MinecraftAccounts> {
	let root = accounts_root(&app)?;
	let client_id = state.settings()?.msa_client_id;
	off_thread(move || snapshot(&root, client_id.as_ref())).await
}

/// Runs the full interactive sign-in: opens the system browser, waits for the
/// redirect, and stores the account.
///
/// One command rather than a begin/complete pair because the loopback
/// listener has to stay bound for the whole flow, and handing its lifetime
/// across two IPC calls would mean keeping it in app state where a cancelled
/// sign-in leaks it.
#[tauri::command]
pub async fn minecraft_account_sign_in(
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<MinecraftAccounts> {
	let root = accounts_root(&app)?;
	let client_id = state.settings()?.msa_client_id.ok_or_else(|| {
		SerializableError::new(
			"minecraft_account",
			"no Microsoft client id is configured for this build",
		)
	})?;
	off_thread(move || {
		let config = MsaConfig {
			client_id: client_id.clone(),
		};
		let login = packwand_msa::begin_login(&config).map_err(msa_error)?;
		open_in_browser(&login.authorize_url)?;
		let accounts = store(&root);
		// The token store is chosen before the uuid is known, so the flow
		// writes to a temporary key and the account is re-saved under its own
		// key once the profile comes back.
		let scratch = packwand_msa::InMemoryTokenStore::new();
		let session = packwand_msa::await_login(login, LOGIN_TIMEOUT, &config, &scratch)
			.map_err(msa_error)?;
		let refresh = packwand_msa::TokenStore::load(&scratch).map_err(msa_error)?;
		accounts
			.remember(
				Account {
					uuid: session.uuid.clone(),
					name: session.username.clone(),
					disabled: false,
					last_used_ms: None,
				},
				refresh.as_ref(),
			)
			.map_err(msa_error)?;
		snapshot(&root, Some(&client_id))
	})
	.await
}

#[tauri::command]
pub async fn minecraft_account_sign_out(
	app: AppHandle,
	state: State<'_, AppState>,
	uuid: String,
) -> CommandResult<MinecraftAccounts> {
	let root = accounts_root(&app)?;
	let client_id = state.settings()?.msa_client_id;
	off_thread(move || {
		store(&root).forget(&uuid).map_err(msa_error)?;
		snapshot(&root, client_id.as_ref())
	})
	.await
}

#[tauri::command]
pub async fn minecraft_account_select(
	app: AppHandle,
	state: State<'_, AppState>,
	uuid: String,
) -> CommandResult<MinecraftAccounts> {
	let root = accounts_root(&app)?;
	let client_id = state.settings()?.msa_client_id;
	off_thread(move || {
		let accounts = store(&root);
		let mut list = accounts.load().map_err(msa_error)?;
		if !list.accounts.iter().any(|a| a.uuid == uuid) {
			return Err(SerializableError::new(
				"minecraft_account",
				"that account is not signed in",
			));
		}
		list.active_uuid = Some(uuid);
		accounts.save(&list).map_err(msa_error)?;
		snapshot(&root, client_id.as_ref())
	})
	.await
}

/// Switches an account off without signing it out.
///
/// Distinct from signing out on purpose: this keeps the stored credential, so
/// turning the account back on needs no browser.
#[tauri::command]
pub async fn minecraft_account_set_disabled(
	app: AppHandle,
	state: State<'_, AppState>,
	uuid: String,
	disabled: bool,
) -> CommandResult<MinecraftAccounts> {
	let root = accounts_root(&app)?;
	let client_id = state.settings()?.msa_client_id;
	off_thread(move || {
		let accounts = store(&root);
		let mut list = accounts.load().map_err(msa_error)?;
		let Some(account) = list.accounts.iter_mut().find(|a| a.uuid == uuid) else {
			return Err(SerializableError::new(
				"minecraft_account",
				"that account is not signed in",
			));
		};
		account.disabled = disabled;
		accounts.save(&list).map_err(msa_error)?;
		snapshot(&root, client_id.as_ref())
	})
	.await
}

#[cfg(windows)]
fn open_in_browser(url: &str) -> CommandResult<()> {
	std::process::Command::new("cmd")
		.args(["/C", "start", "", url])
		.spawn()
		.map(|_| ())
		.map_err(|error| SerializableError::new("open_failed", error.to_string()))
}

#[cfg(not(windows))]
fn open_in_browser(url: &str) -> CommandResult<()> {
	let opener = if cfg!(target_os = "macos") {
		"open"
	} else {
		"xdg-open"
	};
	std::process::Command::new(opener)
		.arg(url)
		.spawn()
		.map(|_| ())
		.map_err(|error| SerializableError::new("open_failed", error.to_string()))
}
