//! Secure storage for the Microsoft refresh token, via the OS credential
//! store (Windows Credential Manager / macOS Keychain / Linux Secret
//! Service, all through packwandc). Only the refresh token is
//! persisted; access tokens are short-lived (~24h, per Microsoft/Minecraft)
//! and kept in memory only. One fixed account key for v1 — a single signed
//! in account at a time, matching `packwand-auth`'s existing
//! `InMemoryCredentialStore`'s minimal scope.

use packwand_auth::SecretString;

#[derive(Debug, thiserror::Error)]
pub enum TokenStoreError {
    #[error("credential store error: {0}")]
    Backend(String),
}

pub trait TokenStore: Send + Sync {
    fn save(&self, refresh_token: &SecretString) -> Result<(), TokenStoreError>;
    fn load(&self) -> Result<Option<SecretString>, TokenStoreError>;
    fn clear(&self) -> Result<(), TokenStoreError>;
}

/// OS-native credential storage backed by packwandc.
#[derive(Default)]
pub struct PwcKeyStore;

impl PwcKeyStore {
    pub fn new() -> Self {
        Self
    }
}

impl TokenStore for PwcKeyStore {
    fn save(&self, refresh_token: &SecretString) -> Result<(), TokenStoreError> {
        packwandc::KeyStore
            .save(refresh_token.expose().as_bytes())
            .map_err(|error| TokenStoreError::Backend(error.to_string()))
    }

    fn load(&self) -> Result<Option<SecretString>, TokenStoreError> {
        let Some(bytes) = packwandc::KeyStore
            .load()
            .map_err(|error| TokenStoreError::Backend(error.to_string()))?
        else {
            return Ok(None);
        };
        let value = String::from_utf8(bytes).map_err(|error| {
            TokenStoreError::Backend(format!("stored credential is not UTF-8: {error}"))
        })?;
        Ok(Some(SecretString::new(value)))
    }

    fn clear(&self) -> Result<(), TokenStoreError> {
        packwandc::KeyStore
            .clear()
            .map_err(|error| TokenStoreError::Backend(error.to_string()))
    }
}
/// Process-lifetime store for tests — never touches the real OS credential
/// manager. Mirrors `packwand_auth::InMemoryCredentialStore`'s role.
#[derive(Default)]
pub struct InMemoryTokenStore(std::sync::Mutex<Option<SecretString>>);

impl InMemoryTokenStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TokenStore for InMemoryTokenStore {
    fn save(&self, refresh_token: &SecretString) -> Result<(), TokenStoreError> {
        *self
            .0
            .lock()
            .map_err(|_| TokenStoreError::Backend("poisoned".to_string()))? =
            Some(refresh_token.clone());
        Ok(())
    }

    fn load(&self) -> Result<Option<SecretString>, TokenStoreError> {
        Ok(self
            .0
            .lock()
            .map_err(|_| TokenStoreError::Backend("poisoned".to_string()))?
            .clone())
    }

    fn clear(&self) -> Result<(), TokenStoreError> {
        *self
            .0
            .lock()
            .map_err(|_| TokenStoreError::Backend("poisoned".to_string()))? = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_store_roundtrip() {
        let store = InMemoryTokenStore::new();
        assert!(store.load().unwrap().is_none());
        store.save(&SecretString::new("refresh-value")).unwrap();
        assert_eq!(store.load().unwrap().unwrap().expose(), "refresh-value");
        store.clear().unwrap();
        assert!(store.load().unwrap().is_none());
    }
}
