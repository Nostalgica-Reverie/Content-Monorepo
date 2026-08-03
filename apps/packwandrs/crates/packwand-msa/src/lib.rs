//! Real Microsoft/Xbox Live/Minecraft account authentication.
//!
//! Kept separate from `packwand-auth` per that crate's own doc comment:
//! Microsoft/Minecraft OAuth "must not be bolted onto this crate ad hoc."
//! This crate's job is only to *produce* a real `packwand_auth::Session` —
//! everything downstream (`packwand-launch`, `packwand-devboot`) already
//! consumes `Session` generically and needs no changes.
//!
//! Flow: Authorization Code + PKCE via the system's default browser and a
//! local loopback HTTP listener (RFC 8252 — the standard pattern for
//! native apps; not an embedded webview, which Microsoft's own guidance
//! discourages for credential entry). No client secret: this is registered
//! as a public/native Azure AD app.

mod chain;
mod loopback;
mod pkce;
mod store;

use std::time::Duration;

use packwand_auth::Session;

pub use chain::MsaError;
pub use store::{InMemoryTokenStore, PwcKeyStore, TokenStore, TokenStoreError};

const AUTHORIZE_URL: &str = "https://login.live.com/oauth20_authorize.srf";
const SCOPE: &str = "XboxLive.signin offline_access";

/// The Azure AD app registration this build signs in as. `client_id` is not
/// secret (public/native OAuth clients don't have one) — see
/// the account registration and whitelist requirements for
/// Minecraft's API.
#[derive(Clone)]
pub struct MsaConfig {
    pub client_id: String,
}

/// An in-progress interactive sign-in: the URL to open in the system
/// browser, plus everything needed to complete the flow once it redirects
/// back.
pub struct LoginSession {
    pub authorize_url: String,
    loopback: loopback::Loopback,
    pkce: pkce::Pkce,
    state: String,
    redirect_uri: String,
}

fn urlencode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

/// Begins an interactive sign-in: binds a local loopback listener and
/// builds the URL to open in the system's default browser. Does not open
/// the browser itself — the caller (the Tauri adapter) decides how.
pub fn begin_login(config: &MsaConfig) -> Result<LoginSession, MsaError> {
    let (loopback, port) = loopback::Loopback::bind()?;
    let redirect_uri = format!("http://localhost:{port}");
    let pkce = pkce::Pkce::generate();
    let state = pkce::random_state();
    let authorize_url = format!(
        "{AUTHORIZE_URL}?client_id={client_id}&response_type=code&redirect_uri={redirect}&scope={scope}&state={state}&code_challenge={challenge}&code_challenge_method=S256",
        client_id = urlencode(&config.client_id),
        redirect = urlencode(&redirect_uri),
        scope = urlencode(SCOPE),
        challenge = pkce.challenge,
    );
    Ok(LoginSession {
        authorize_url,
        loopback,
        pkce,
        state,
        redirect_uri,
    })
}

/// Blocks until the browser redirects back (or `timeout` elapses), then
/// exchanges the code for tokens, runs the Xbox Live/XSTS/Minecraft chain,
/// persists the refresh token via `store`, and returns a real `Session`.
pub fn await_login(
    session: LoginSession,
    timeout: Duration,
    config: &MsaConfig,
    store: &dyn TokenStore,
) -> Result<Session, MsaError> {
    let query = session.loopback.await_query(timeout)?;
    let redirect = chain::parse_redirect_query(&query)?;
    if let Some(err) = redirect.error {
        return Err(MsaError::Denied(err));
    }
    if redirect.state.as_deref() != Some(session.state.as_str()) {
        return Err(MsaError::Other(
            "sign-in state mismatch — possible interference; please try again".to_string(),
        ));
    }
    let code = redirect
        .code
        .ok_or_else(|| MsaError::Other("no authorization code in the redirect".to_string()))?;

    let tokens =
        chain::exchange_code(config, &code, &session.pkce.verifier, &session.redirect_uri)?;
    if let Some(refresh) = &tokens.refresh_token {
        store.save(refresh)?;
    }
    chain::login_with_ms_access_token(&tokens.access_token)
}

/// Silently re-authenticates using a stored refresh token — no browser.
/// Returns `Ok(None)` (not an error) when no refresh token is stored yet,
/// so callers can fall back to offline/dev-testing without treating "never
/// signed in" as a failure.
pub fn refresh(config: &MsaConfig, store: &dyn TokenStore) -> Result<Option<Session>, MsaError> {
    let Some(stored_refresh_token) = store.load()? else {
        return Ok(None);
    };
    let tokens = chain::refresh_token(config, &stored_refresh_token)?;
    if let Some(new_refresh) = &tokens.refresh_token {
        store.save(new_refresh)?;
    }
    Ok(Some(chain::login_with_ms_access_token(
        &tokens.access_token,
    )?))
}

/// Signs out: clears the stored refresh token. Does not revoke it with
/// Microsoft (there's no simple API for that here) — it just stops this
/// app from silently reusing it.
pub fn logout(store: &dyn TokenStore) -> Result<(), MsaError> {
    Ok(store.clear()?)
}
