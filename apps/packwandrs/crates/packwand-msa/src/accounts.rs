//! Several accounts, one at a time per instance.
//!
//! Storage is split by sensitivity: the list of who is signed in is ordinary
//! metadata and lives in a JSON file, while each account's refresh token goes
//! to the OS credential store under its own key. That split is why the list
//! can be read, rendered, and reordered without ever touching a secret.
//!
//! The use-lock exists because Minecraft's session service allows one active
//! session per account. Launching a second instance as the same account
//! silently disconnects the first, which presents to the player as a random
//! kick with no explanation; refusing the second launch is the kinder answer.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use packwand_auth::SecretString;
use serde::{Deserialize, Serialize};

use crate::store::{TokenStore, TokenStoreError};

/// Schema version of the persisted account list.
pub const ACCOUNTS_SCHEMA_VERSION: u32 = 1;

/// One signed-in account, without anything secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
	/// Minecraft profile uuid; the identity everything else keys on.
	pub uuid: String,
	/// Current profile name.
	pub name: String,
	/// Whether the user has switched this account off. A disabled account
	/// keeps its stored token and is simply not used.
	#[serde(default)]
	pub disabled: bool,
	/// Milliseconds since the epoch of the last successful use.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub last_used_ms: Option<u64>,
}

/// The persisted list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountList {
	pub schema_version: u32,
	#[serde(default)]
	pub accounts: Vec<Account>,
	/// Which account launches use when the instance does not name one.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub active_uuid: Option<String>,
}

impl Default for AccountList {
	fn default() -> Self {
		Self {
			schema_version: ACCOUNTS_SCHEMA_VERSION,
			accounts: Vec::new(),
			active_uuid: None,
		}
	}
}

impl AccountList {
	/// The account a launch should use, if any is usable.
	pub fn active(&self) -> Option<&Account> {
		self.active_uuid
			.as_ref()
			.and_then(|uuid| self.accounts.iter().find(|a| &a.uuid == uuid))
			.filter(|a| !a.disabled)
			.or_else(|| self.accounts.iter().find(|a| !a.disabled))
	}

	/// Adds or updates an account, keeping the list keyed by uuid.
	pub fn upsert(&mut self, account: Account) {
		match self.accounts.iter_mut().find(|a| a.uuid == account.uuid) {
			// A re-login refreshes the name but must not silently re-enable
			// an account the user switched off.
			Some(existing) => {
				existing.name = account.name;
				existing.last_used_ms = account.last_used_ms.or(existing.last_used_ms);
			}
			None => {
				if self.active_uuid.is_none() {
					self.active_uuid = Some(account.uuid.clone());
				}
				self.accounts.push(account);
			}
		}
	}

	/// Removes an account and clears it as active if it was.
	pub fn remove(&mut self, uuid: &str) -> bool {
		let before = self.accounts.len();
		self.accounts.retain(|a| a.uuid != uuid);
		if self.active_uuid.as_deref() == Some(uuid) {
			self.active_uuid = self.accounts.first().map(|a| a.uuid.clone());
		}
		self.accounts.len() != before
	}
}

/// The credential-store key holding one account's refresh token.
///
/// Namespaced by uuid so accounts cannot overwrite each other — the single
/// fixed key this replaced allowed exactly one signed-in account.
fn credential_key(uuid: &str) -> String {
	format!("packwand-msa-{uuid}")
}

/// Accounts currently claimed by a running launch, keyed by list path and
/// uuid.
///
/// Process-global, and it has to be: an [`Accounts`] is constructed fresh on
/// every call site — once per launch, once per command — so a map owned by
/// the instance would give every caller its own empty one and the claim would
/// always succeed. That is a lock that reads as working and never locks
/// anything.
///
/// Scoped to this process. Two launcher processes can still claim the same
/// account, which is the rarer case and the one a lock file would cover — at
/// the cost of a crash leaving a file behind that permanently blocks an
/// account until the user finds and deletes it. Missing the rare case is the
/// better failure.
static CLAIMED: Mutex<BTreeMap<(PathBuf, String), ()>> = Mutex::new(BTreeMap::new());

/// Reads and writes the account list and its per-account tokens.
pub struct Accounts {
	path: PathBuf,
}

/// Errors from account storage.
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
	#[error("failed to read or write {path}: {source}")]
	Io {
		path: PathBuf,
		source: std::io::Error,
	},
	#[error("the account list at {path} is not readable: {reason}")]
	Corrupt { path: PathBuf, reason: String },
	#[error(transparent)]
	Store(#[from] TokenStoreError),
	#[error("{name} is already playing on another instance")]
	InUse { name: String },
}

impl Accounts {
	/// Opens the account list stored under `root`.
	pub fn new(root: &Path) -> Self {
		Self {
			path: root.join("accounts.json"),
		}
	}

	/// Loads the list, treating "not there yet" as an empty one.
	pub fn load(&self) -> Result<AccountList, AccountError> {
		let bytes = match std::fs::read(&self.path) {
			Ok(bytes) => bytes,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(AccountList::default()),
			Err(source) => {
				return Err(AccountError::Io {
					path: self.path.clone(),
					source,
				});
			}
		};
		let list: AccountList =
			serde_json::from_slice(&bytes).map_err(|e| AccountError::Corrupt {
				path: self.path.clone(),
				reason: e.to_string(),
			})?;
		if list.schema_version > ACCOUNTS_SCHEMA_VERSION {
			return Err(AccountError::Corrupt {
				path: self.path.clone(),
				reason: format!(
					"schema version {} is newer than this build understands ({ACCOUNTS_SCHEMA_VERSION})",
					list.schema_version
				),
			});
		}
		Ok(list)
	}

	/// Writes the list.
	pub fn save(&self, list: &AccountList) -> Result<(), AccountError> {
		if let Some(parent) = self.path.parent() {
			std::fs::create_dir_all(parent).map_err(|source| AccountError::Io {
				path: parent.to_path_buf(),
				source,
			})?;
		}
		let bytes = serde_json::to_vec_pretty(list).map_err(|e| AccountError::Corrupt {
			path: self.path.clone(),
			reason: e.to_string(),
		})?;
		packwand_platform::atomic_write(&self.path, &bytes).map_err(|e| AccountError::Io {
			path: self.path.clone(),
			source: std::io::Error::other(e.to_string()),
		})
	}

	/// The token store for one account.
	pub fn token_store(&self, uuid: &str) -> impl TokenStore + use<> {
		KeyedTokenStore {
			key: credential_key(uuid),
		}
	}

	/// Records a successful sign-in.
	pub fn remember(
		&self,
		account: Account,
		refresh_token: Option<&SecretString>,
	) -> Result<(), AccountError> {
		if let Some(token) = refresh_token {
			self.token_store(&account.uuid).save(token)?;
		}
		let mut list = self.load()?;
		list.upsert(account);
		self.save(&list)
	}

	/// Forgets an account and its stored token.
	pub fn forget(&self, uuid: &str) -> Result<(), AccountError> {
		// Clear the secret first: a list entry with no token is recoverable
		// by signing in again, while a token with no list entry is a secret
		// nothing will ever clean up.
		self.token_store(uuid).clear()?;
		let mut list = self.load()?;
		list.remove(uuid);
		self.save(&list)
	}

	/// Claims an account for one running instance.
	///
	/// Held for as long as the returned guard lives. Minecraft's session
	/// service permits one session per account, so a second concurrent launch
	/// would disconnect the first with no explanation the player can act on.
	pub fn claim(&self, account: &Account) -> Result<AccountClaim, AccountError> {
		let key = (self.path.clone(), account.uuid.clone());
		let mut claimed = CLAIMED
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner);
		if claimed.contains_key(&key) {
			return Err(AccountError::InUse {
				name: account.name.clone(),
			});
		}
		claimed.insert(key.clone(), ());
		Ok(AccountClaim { key })
	}
}

/// Proof that one account is claimed; releases on drop.
///
/// Must outlive the game, not the call that made it: dropping this while
/// Minecraft is still running frees the account for a second launch that
/// would disconnect the first.
#[derive(Debug)]
pub struct AccountClaim {
	key: (PathBuf, String),
}

impl AccountClaim {
	/// The claimed account's uuid.
	pub fn uuid(&self) -> &str {
		&self.key.1
	}
}

impl Drop for AccountClaim {
	fn drop(&mut self) {
		if let Ok(mut claimed) = CLAIMED.lock() {
			claimed.remove(&self.key);
		}
	}
}

/// A [`TokenStore`] over one credential-store key.
struct KeyedTokenStore {
	key: String,
}

impl TokenStore for KeyedTokenStore {
	fn save(&self, refresh_token: &SecretString) -> Result<(), TokenStoreError> {
		packwand_platform::CredentialStore::for_key(&self.key)
			.and_then(|store| store.save(refresh_token.expose()))
			.map_err(|error| TokenStoreError::Backend(error.to_string()))
	}

	fn load(&self) -> Result<Option<SecretString>, TokenStoreError> {
		let value = packwand_platform::CredentialStore::for_key(&self.key)
			.and_then(|store| store.load())
			.map_err(|error| TokenStoreError::Backend(error.to_string()))?;
		Ok(value.map(SecretString::new))
	}

	fn clear(&self) -> Result<(), TokenStoreError> {
		packwand_platform::CredentialStore::for_key(&self.key)
			.and_then(|store| store.clear())
			.map_err(|error| TokenStoreError::Backend(error.to_string()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn account(uuid: &str, name: &str) -> Account {
		Account {
			uuid: uuid.to_string(),
			name: name.to_string(),
			disabled: false,
			last_used_ms: None,
		}
	}

	#[test]
	fn the_list_round_trips_and_starts_empty() {
		let dir = tempfile::tempdir().unwrap();
		let accounts = Accounts::new(dir.path());
		assert!(accounts.load().unwrap().accounts.is_empty());

		let mut list = AccountList::default();
		list.upsert(account("uuid-a", "Alice"));
		list.upsert(account("uuid-b", "Bob"));
		accounts.save(&list).unwrap();

		let loaded = accounts.load().unwrap();
		assert_eq!(loaded.accounts.len(), 2);
		// The first account added becomes active without being asked.
		assert_eq!(loaded.active_uuid.as_deref(), Some("uuid-a"));
		assert_eq!(loaded.active().unwrap().name, "Alice");
	}

	#[test]
	fn re_signing_in_updates_the_name_without_re_enabling_the_account() {
		// A user who switched an account off did so deliberately; a refresh
		// that silently turned it back on would be a surprise.
		let mut list = AccountList::default();
		list.upsert(account("uuid-a", "Alice"));
		list.accounts[0].disabled = true;
		list.upsert(account("uuid-a", "AliceRenamed"));
		assert_eq!(list.accounts.len(), 1);
		assert_eq!(list.accounts[0].name, "AliceRenamed");
		assert!(list.accounts[0].disabled, "the account was re-enabled");
	}

	#[test]
	fn a_disabled_account_is_skipped_when_choosing_who_plays() {
		let mut list = AccountList::default();
		list.upsert(account("uuid-a", "Alice"));
		list.upsert(account("uuid-b", "Bob"));
		list.accounts[0].disabled = true;
		assert_eq!(list.active_uuid.as_deref(), Some("uuid-a"));
		assert_eq!(
			list.active().map(|a| a.name.as_str()),
			Some("Bob"),
			"a disabled active account should fall through to a usable one"
		);
	}

	#[test]
	fn removing_the_active_account_promotes_another() {
		let mut list = AccountList::default();
		list.upsert(account("uuid-a", "Alice"));
		list.upsert(account("uuid-b", "Bob"));
		assert!(list.remove("uuid-a"));
		assert_eq!(list.active_uuid.as_deref(), Some("uuid-b"));
		assert!(!list.remove("uuid-a"), "removing twice should report false");
	}

	#[test]
	fn an_account_cannot_be_claimed_twice_at_once() {
		// Minecraft allows one session per account; the second launch has to
		// be refused rather than kicking the first.
		//
		// Deliberately through two separate `Accounts` values, because that is
		// what production does — one is constructed per launch. An earlier
		// version of this test reused a single instance and passed against a
		// lock that never engaged anywhere real.
		let dir = tempfile::tempdir().unwrap();
		let first_launch = Accounts::new(dir.path());
		let second_launch = Accounts::new(dir.path());
		let alice = account("uuid-a", "Alice");
		let bob = account("uuid-b", "Bob");

		let claim = first_launch.claim(&alice).unwrap();
		assert_eq!(claim.uuid(), "uuid-a");
		let error = second_launch.claim(&alice).unwrap_err();
		assert!(matches!(error, AccountError::InUse { .. }), "{error}");
		// A different account is unaffected.
		let _bob = second_launch.claim(&bob).unwrap();

		drop(claim);
		second_launch
			.claim(&alice)
			.expect("the claim should be released");
	}

	#[test]
	fn two_launcher_roots_do_not_share_claims() {
		// The claim is per account *per account list*: a separate root is a
		// separate installation and has nothing to do with this one.
		let first = tempfile::tempdir().unwrap();
		let second = tempfile::tempdir().unwrap();
		let alice = account("uuid-a", "Alice");
		let _held = Accounts::new(first.path()).claim(&alice).unwrap();
		Accounts::new(second.path())
			.claim(&alice)
			.expect("a different root should be unaffected");
	}

	#[test]
	fn each_account_gets_its_own_credential_key() {
		// The single fixed key this replaced meant a second sign-in silently
		// overwrote the first account's token.
		assert_ne!(credential_key("uuid-a"), credential_key("uuid-b"));
		assert!(credential_key("uuid-a").contains("uuid-a"));
	}

	#[test]
	fn a_newer_schema_is_refused_rather_than_misread() {
		let dir = tempfile::tempdir().unwrap();
		let accounts = Accounts::new(dir.path());
		std::fs::write(
			dir.path().join("accounts.json"),
			br#"{"schemaVersion": 99, "accounts": []}"#,
		)
		.unwrap();
		assert!(matches!(
			accounts.load().unwrap_err(),
			AccountError::Corrupt { .. }
		));
	}
}
