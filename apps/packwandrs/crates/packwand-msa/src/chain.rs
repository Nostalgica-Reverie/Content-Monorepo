//! The Microsoft -> Xbox Live -> XSTS -> Minecraft authentication chain.
//!
//! Verified against current documentation this was implemented against
//! (wiki.vg/minecraft.wiki's "Microsoft authentication" pages and
//! real-launcher docs), not just recalled from memory. Endpoints:
//!
//! 1. `https://login.live.com/oauth20_token.srf` — authorization_code or
//!    refresh_token grant, form-encoded, PKCE (no client_secret: this is a
//!    public/native client).
//! 2. `https://user.auth.xboxlive.com/user/authenticate` — MS access token
//!    -> Xbox Live user token + user hash (`uhs`).
//! 3. `https://xsts.auth.xboxlive.com/xsts/authorize` — Xbox Live token ->
//!    XSTS token, relying party `rp://api.minecraftservices.com/`. Failure
//!    responses carry an `XErr` code identifying *why* (no Xbox account,
//!    region-banned, needs adult verification, needs a family group).
//! 4. `https://api.minecraftservices.com/authentication/login_with_xbox` —
//!    `identityToken: "XBL3.0 x=<uhs>;<xsts_token>"` -> Minecraft access
//!    token.
//! 5. `https://api.minecraftservices.com/entitlements/mcstore` — ownership
//!    check. New/un-whitelisted Azure apps get HTTP 403 here specifically
//!    (see `MsaError::NotWhitelisted`), not a generic auth failure.
//! 6. `https://api.minecraftservices.com/minecraft/profile` — real
//!    username/UUID.

use std::io::Read;

use packwand_auth::{SecretString, Session};
use serde::{Deserialize, Serialize};

use crate::MsaConfig;

const MS_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTHORIZE_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_ENTITLEMENTS_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Debug, thiserror::Error)]
pub enum MsaError {
	#[error("network error contacting {0}: {1}")]
	Network(String, String),
	#[error("unexpected response from {0}: {1}")]
	UnexpectedResponse(String, String),
	#[error("Microsoft sign-in was cancelled or denied: {0}")]
	Denied(String),
	#[error(
		"this Microsoft account has no Xbox Live profile — create one at xbox.com, then try signing in again"
	)]
	NoXboxAccount,
	#[error("Xbox Live is not available for this account's country/region")]
	XboxLiveUnavailableInRegion,
	#[error(
		"this Xbox Live account needs adult verification (South Korea) — check account.xbox.com"
	)]
	NeedsAdultVerification,
	#[error(
		"this account is a minor and must be added to a Family by an adult at account.microsoft.com/family before it can sign in here"
	)]
	NeedsFamilyGroup,
	#[error(
		"Minecraft's API returned 403 — this app isn't whitelisted yet. After a first sign-in attempt (this one), submit https://aka.ms/mce-reviewappid with your Azure app's Client ID and Tenant ID, then allow time for Microsoft's review"
	)]
	NotWhitelisted,
	#[error("this Microsoft account does not own Minecraft")]
	NoEntitlement,
	#[error("credential store error: {0}")]
	Store(#[from] crate::store::TokenStoreError),
	#[error("{0}")]
	Other(String),
}

pub(crate) struct TokenPair {
	pub access_token: SecretString,
	pub refresh_token: Option<SecretString>,
}

#[derive(Deserialize)]
struct MsTokenResponse {
	access_token: String,
	#[serde(default)]
	refresh_token: Option<String>,
	#[serde(default)]
	error: Option<String>,
	#[serde(default)]
	error_description: Option<String>,
}

fn post_form(url: &str, body: &str) -> Result<String, MsaError> {
	let response = ureq::post(url)
		.set("Content-Type", "application/x-www-form-urlencoded")
		.send_string(body);
	read_body(url, response)
}

fn post_json(url: &str, body: &str, bearer: Option<&str>) -> Result<(u16, String), MsaError> {
	let mut request = ureq::post(url)
		.set("Content-Type", "application/json")
		.set("Accept", "application/json");
	if let Some(token) = bearer {
		request = request.set("Authorization", &format!("Bearer {token}"));
	}
	match request.send_string(body) {
		Ok(response) => {
			let status = response.status();
			Ok((status, read_ok_body(response)?))
		}
		Err(ureq::Error::Status(status, response)) => Ok((status, read_ok_body(response)?)),
		Err(e) => Err(MsaError::Network(url.to_string(), e.to_string())),
	}
}

fn get_json(url: &str, bearer: &str) -> Result<(u16, String), MsaError> {
	match ureq::get(url)
		.set("Authorization", &format!("Bearer {bearer}"))
		.set("Accept", "application/json")
		.call()
	{
		Ok(response) => {
			let status = response.status();
			Ok((status, read_ok_body(response)?))
		}
		Err(ureq::Error::Status(status, response)) => Ok((status, read_ok_body(response)?)),
		Err(e) => Err(MsaError::Network(url.to_string(), e.to_string())),
	}
}

fn read_body(url: &str, response: Result<ureq::Response, ureq::Error>) -> Result<String, MsaError> {
	match response {
		Ok(r) => read_ok_body(r),
		Err(ureq::Error::Status(_, r)) => read_ok_body(r),
		Err(e) => Err(MsaError::Network(url.to_string(), e.to_string())),
	}
}

fn read_ok_body(response: ureq::Response) -> Result<String, MsaError> {
	let mut text = String::new();
	response
		.into_reader()
		.take(1024 * 1024)
		.read_to_string(&mut text)
		.map_err(|e| MsaError::Other(format!("failed to read response body: {e}")))?;
	Ok(text)
}

fn urlencode(value: &str) -> String {
	url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn ms_token_request(config: &MsaConfig, form_body: String) -> Result<TokenPair, MsaError> {
	let body = post_form(MS_TOKEN_URL, &form_body)?;
	let parsed: MsTokenResponse = serde_json::from_str(&body)
		.map_err(|e| MsaError::UnexpectedResponse(MS_TOKEN_URL.to_string(), e.to_string()))?;
	if let Some(error) = parsed.error {
		return Err(MsaError::Denied(parsed.error_description.unwrap_or(error)));
	}
	let _ = &config.client_id; // already embedded in form_body by the caller
	Ok(TokenPair {
		access_token: SecretString::new(parsed.access_token),
		refresh_token: parsed.refresh_token.map(SecretString::new),
	})
}

/// Exchanges an authorization code (from the loopback redirect) for tokens.
pub(crate) fn exchange_code(
	config: &MsaConfig,
	code: &str,
	code_verifier: &str,
	redirect_uri: &str,
) -> Result<TokenPair, MsaError> {
	let body = format!(
		"client_id={client_id}&code={code}&grant_type=authorization_code&redirect_uri={redirect_uri}&code_verifier={verifier}",
		client_id = urlencode(&config.client_id),
		code = urlencode(code),
		redirect_uri = urlencode(redirect_uri),
		verifier = urlencode(code_verifier),
	);
	ms_token_request(config, body)
}

/// Silently refreshes without a browser.
pub(crate) fn refresh_token(
	config: &MsaConfig,
	refresh_token: &SecretString,
) -> Result<TokenPair, MsaError> {
	let body = format!(
		"client_id={client_id}&refresh_token={token}&grant_type=refresh_token",
		client_id = urlencode(&config.client_id),
		token = urlencode(refresh_token.expose()),
	);
	ms_token_request(config, body)
}

#[derive(Serialize)]
struct XblAuthRequest {
	#[serde(rename = "Properties")]
	properties: XblAuthProperties,
	#[serde(rename = "RelyingParty")]
	relying_party: String,
	#[serde(rename = "TokenType")]
	token_type: String,
}

#[derive(Serialize)]
struct XblAuthProperties {
	#[serde(rename = "AuthMethod")]
	auth_method: String,
	#[serde(rename = "SiteName")]
	site_name: String,
	#[serde(rename = "RpsTicket")]
	rps_ticket: String,
}

#[derive(Serialize)]
struct XstsRequest {
	#[serde(rename = "Properties")]
	properties: XstsProperties,
	#[serde(rename = "RelyingParty")]
	relying_party: String,
	#[serde(rename = "TokenType")]
	token_type: String,
}

#[derive(Serialize)]
struct XstsProperties {
	#[serde(rename = "SandboxId")]
	sandbox_id: String,
	#[serde(rename = "UserTokens")]
	user_tokens: Vec<String>,
}

#[derive(Deserialize)]
struct XTokenResponse {
	#[serde(rename = "Token")]
	token: String,
	#[serde(rename = "DisplayClaims")]
	display_claims: DisplayClaims,
}

#[derive(Deserialize)]
struct DisplayClaims {
	xui: Vec<XuiEntry>,
}

#[derive(Deserialize)]
struct XuiEntry {
	uhs: String,
}

#[derive(Deserialize)]
struct XstsErrorResponse {
	#[serde(rename = "XErr")]
	x_err: Option<u64>,
}

fn xsts_error_for(x_err: u64) -> MsaError {
	match x_err {
		2148916233 => MsaError::NoXboxAccount,
		2148916235 => MsaError::XboxLiveUnavailableInRegion,
		2148916236 | 2148916237 => MsaError::NeedsAdultVerification,
		2148916238 => MsaError::NeedsFamilyGroup,
		other => {
			MsaError::UnexpectedResponse(XSTS_AUTHORIZE_URL.to_string(), format!("XErr {other}"))
		}
	}
}

struct XboxIdentity {
	uhs: String,
	xsts_token: String,
}

fn xbox_live_chain(ms_access_token: &SecretString) -> Result<XboxIdentity, MsaError> {
	let xbl_body = serde_json::to_string(&XblAuthRequest {
		properties: XblAuthProperties {
			auth_method: "RPS".to_string(),
			site_name: "user.auth.xboxlive.com".to_string(),
			rps_ticket: format!("d={}", ms_access_token.expose()),
		},
		relying_party: "http://auth.xboxlive.com".to_string(),
		token_type: "JWT".to_string(),
	})
	.map_err(|e| MsaError::Other(e.to_string()))?;
	let (status, body) = post_json(XBL_AUTH_URL, &xbl_body, None)?;
	if status != 200 {
		return Err(MsaError::UnexpectedResponse(
			XBL_AUTH_URL.to_string(),
			format!("HTTP {status}: {body}"),
		));
	}
	let xbl: XTokenResponse = serde_json::from_str(&body)
		.map_err(|e| MsaError::UnexpectedResponse(XBL_AUTH_URL.to_string(), e.to_string()))?;
	let uhs = xbl
		.display_claims
		.xui
		.first()
		.map(|x| x.uhs.clone())
		.ok_or_else(|| {
			MsaError::UnexpectedResponse(
				XBL_AUTH_URL.to_string(),
				"no user hash in response".to_string(),
			)
		})?;

	let xsts_body = serde_json::to_string(&XstsRequest {
		properties: XstsProperties {
			sandbox_id: "RETAIL".to_string(),
			user_tokens: vec![xbl.token],
		},
		relying_party: "rp://api.minecraftservices.com/".to_string(),
		token_type: "JWT".to_string(),
	})
	.map_err(|e| MsaError::Other(e.to_string()))?;
	let (status, body) = post_json(XSTS_AUTHORIZE_URL, &xsts_body, None)?;
	if status != 200 {
		let x_err = serde_json::from_str::<XstsErrorResponse>(&body)
			.ok()
			.and_then(|e| e.x_err);
		return Err(match x_err {
			Some(code) => xsts_error_for(code),
			None => MsaError::UnexpectedResponse(
				XSTS_AUTHORIZE_URL.to_string(),
				format!("HTTP {status}: {body}"),
			),
		});
	}
	let xsts: XTokenResponse = serde_json::from_str(&body)
		.map_err(|e| MsaError::UnexpectedResponse(XSTS_AUTHORIZE_URL.to_string(), e.to_string()))?;

	Ok(XboxIdentity {
		uhs,
		xsts_token: xsts.token,
	})
}

#[derive(Serialize)]
struct McLoginRequest {
	#[serde(rename = "identityToken")]
	identity_token: String,
}

#[derive(Deserialize)]
struct McLoginResponse {
	access_token: String,
}

#[derive(Deserialize)]
struct EntitlementsResponse {
	#[serde(default)]
	items: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct ProfileResponse {
	id: String,
	name: String,
}

/// Formats an undashed 32-hex-char Minecraft profile UUID (what
/// `/minecraft/profile` returns) into the hyphenated form
/// `packwand_auth::Session` and Minecraft's own launch arguments expect.
fn format_uuid_with_hyphens(id: &str) -> String {
	if id.len() != 32 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
		return id.to_string(); // defensive: pass through unrecognized shapes
	}
	format!(
		"{}-{}-{}-{}-{}",
		&id[0..8],
		&id[8..12],
		&id[12..16],
		&id[16..20],
		&id[20..32]
	)
}

/// Runs the Xbox Live -> XSTS -> Minecraft login -> entitlement check ->
/// profile fetch chain for an already-obtained Microsoft access token, and
/// produces a real `packwand_auth::Session`.
pub(crate) fn login_with_ms_access_token(
	ms_access_token: &SecretString,
) -> Result<Session, MsaError> {
	let xbox = xbox_live_chain(ms_access_token)?;

	let identity_token = format!("XBL3.0 x={};{}", xbox.uhs, xbox.xsts_token);
	let mc_body = serde_json::to_string(&McLoginRequest { identity_token })
		.map_err(|e| MsaError::Other(e.to_string()))?;
	let (status, body) = post_json(MC_LOGIN_URL, &mc_body, None)?;
	if status != 200 {
		return Err(MsaError::UnexpectedResponse(
			MC_LOGIN_URL.to_string(),
			format!("HTTP {status}: {body}"),
		));
	}
	let mc_login: McLoginResponse = serde_json::from_str(&body)
		.map_err(|e| MsaError::UnexpectedResponse(MC_LOGIN_URL.to_string(), e.to_string()))?;
	let mc_access_token = SecretString::new(mc_login.access_token);

	let (status, body) = get_json(MC_ENTITLEMENTS_URL, mc_access_token.expose())?;
	if status == 403 {
		return Err(MsaError::NotWhitelisted);
	}
	if status != 200 {
		return Err(MsaError::UnexpectedResponse(
			MC_ENTITLEMENTS_URL.to_string(),
			format!("HTTP {status}: {body}"),
		));
	}
	let entitlements: EntitlementsResponse = serde_json::from_str(&body).map_err(|e| {
		MsaError::UnexpectedResponse(MC_ENTITLEMENTS_URL.to_string(), e.to_string())
	})?;
	if entitlements.items.is_empty() {
		return Err(MsaError::NoEntitlement);
	}

	let (status, body) = get_json(MC_PROFILE_URL, mc_access_token.expose())?;
	if status == 403 {
		return Err(MsaError::NotWhitelisted);
	}
	if status != 200 {
		return Err(MsaError::UnexpectedResponse(
			MC_PROFILE_URL.to_string(),
			format!("HTTP {status}: {body}"),
		));
	}
	let profile: ProfileResponse = serde_json::from_str(&body)
		.map_err(|e| MsaError::UnexpectedResponse(MC_PROFILE_URL.to_string(), e.to_string()))?;

	Ok(Session {
		username: profile.name,
		uuid: format_uuid_with_hyphens(&profile.id),
		user_type: "msa".to_string(),
		access_token: mc_access_token,
	})
}

/// The fields this crate cares about from the loopback redirect's query
/// string: either `code`+`state` (success) or `error` (denied/failed).
pub(crate) struct RedirectQuery {
	pub code: Option<String>,
	pub state: Option<String>,
	pub error: Option<String>,
}

pub(crate) fn parse_redirect_query(query: &str) -> Result<RedirectQuery, MsaError> {
	let parsed = url::Url::parse(&format!("http://localhost/?{query}"))
		.map_err(|e| MsaError::Other(format!("malformed redirect query: {e}")))?;
	let mut code = None;
	let mut state = None;
	let mut error = None;
	let mut error_description = None;
	for (key, value) in parsed.query_pairs() {
		match key.as_ref() {
			"code" => code = Some(value.into_owned()),
			"state" => state = Some(value.into_owned()),
			"error" => error = Some(value.into_owned()),
			"error_description" => error_description = Some(value.into_owned()),
			_ => {}
		}
	}
	Ok(RedirectQuery {
		code,
		state,
		error: error_description.or(error),
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn xsts_error_codes_map_to_specific_messages() {
		assert!(matches!(
			xsts_error_for(2148916233),
			MsaError::NoXboxAccount
		));
		assert!(matches!(
			xsts_error_for(2148916235),
			MsaError::XboxLiveUnavailableInRegion
		));
		assert!(matches!(
			xsts_error_for(2148916236),
			MsaError::NeedsAdultVerification
		));
		assert!(matches!(
			xsts_error_for(2148916237),
			MsaError::NeedsAdultVerification
		));
		assert!(matches!(
			xsts_error_for(2148916238),
			MsaError::NeedsFamilyGroup
		));
		assert!(matches!(
			xsts_error_for(999999),
			MsaError::UnexpectedResponse(_, _)
		));
	}

	#[test]
	fn formats_undashed_uuid_with_hyphens() {
		assert_eq!(
			format_uuid_with_hyphens("069a79f444e94726a5befca90e38aaf5"),
			"069a79f4-44e9-4726-a5be-fca90e38aaf5"
		);
	}

	#[test]
	fn passes_through_unrecognized_uuid_shapes() {
		assert_eq!(
			format_uuid_with_hyphens("already-hyphenated"),
			"already-hyphenated"
		);
	}

	#[test]
	fn parses_success_redirect_query() {
		let parsed = parse_redirect_query("code=abc123&state=xyz").unwrap();
		assert_eq!(parsed.code.as_deref(), Some("abc123"));
		assert_eq!(parsed.state.as_deref(), Some("xyz"));
		assert!(parsed.error.is_none());
	}

	#[test]
	fn parses_denied_redirect_query() {
		let parsed =
			parse_redirect_query("error=access_denied&error_description=The+user+cancelled")
				.unwrap();
		assert!(parsed.code.is_none());
		assert_eq!(parsed.error.as_deref(), Some("The user cancelled"));
	}
}
