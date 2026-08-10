//! Client boundary between Packwand's Rust surfaces and the local ATProto bridge.

#![forbid(unsafe_code)]

mod daemon;
mod types;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;

pub use types::{
	BlobRef, CidLink, Friend, Identity, ManifestSummary, PackShare, PendingInvite, Record,
	RecordPage, StrongRef, TangledRepo,
};

const PACK_COLLECTION: &str = "net.nostalgica.packwand.pack";
const SNIPPET_COLLECTION: &str = "net.nostalgica.packwand.snippet";
const IMAGE_COLLECTION: &str = "net.nostalgica.packwand.image";
const INVITE_COLLECTION: &str = "net.nostalgica.packwand.session.invite";
const CONTACT_COLLECTION: &str = "net.nostalgica.packwand.contact";

use daemon::{DaemonEndpoint, ensure_running};

/// Failures produced while locating, starting, or calling the social bridge.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("packwand-social was not found; set PACKWAND_SOCIAL_BIN")]
	BinaryNotFound,
	#[error("could not run packwand-social: {0}")]
	Io(#[from] std::io::Error),
	#[error("packwand-social {action} failed with {status}: {message}")]
	Command {
		action: &'static str,
		status: ExitStatus,
		message: String,
	},
	#[error("packwand-social returned invalid JSON: {0}")]
	Decode(#[from] serde_json::Error),
	#[error("ATProto bridge returned HTTP {status}: {message}")]
	Http { status: u16, message: String },
	#[error("could not reach the ATProto bridge: {0}")]
	Transport(String),
	#[error("could not start the ATProto bridge: {0}")]
	Daemon(String),
	#[error("invalid social request: {0}")]
	InvalidInput(String),
}

/// High-level ATProto identity operations used by the CLI and Tauri shell.
#[derive(Debug, Clone)]
pub struct IdentityClient {
	social_binary: PathBuf,
	endpoint: Option<DaemonEndpoint>,
}

impl IdentityClient {
	/// Locates `packwand-social` using the configured and development search order.
	pub fn new() -> Result<Self, Error> {
		Self::with_optional_binary(None)
	}

	/// Uses an explicit social binary, primarily for embedding and tests.
	pub fn with_binary(path: impl Into<PathBuf>) -> Result<Self, Error> {
		Self::with_optional_binary(Some(path.into()))
	}

	fn with_optional_binary(explicit: Option<PathBuf>) -> Result<Self, Error> {
		Ok(Self {
			social_binary: find_social_binary(explicit.as_deref())?,
			endpoint: None,
		})
	}

	#[cfg(test)]
	fn with_endpoint(base_url: String, token: String) -> Self {
		Self {
			social_binary: PathBuf::from("packwand-social"),
			endpoint: Some(DaemonEndpoint { base_url, token }),
		}
	}

	/// Runs the interactive OAuth flow. Supplying an identifier avoids stdin prompting.
	pub fn login(&self, identifier: Option<&str>) -> Result<Identity, Error> {
		let mut command = Command::new(&self.social_binary);
		command.arg("login");
		let interactive = identifier.is_none();
		if let Some(identifier) = identifier {
			command.args(["--identifier", identifier]);
		}
		let output = command
			.stdin(Stdio::inherit())
			.stdout(Stdio::piped())
			.stderr(if interactive {
				Stdio::inherit()
			} else {
				Stdio::piped()
			})
			.output()?;
		let message = String::from_utf8_lossy(&output.stderr);
		ensure_success("login", output.status, message.trim())?;
		Ok(serde_json::from_slice(&output.stdout)?)
	}

	/// Returns the persisted identity without a network round trip.
	pub fn whoami(&self) -> Result<Option<Identity>, Error> {
		let output = Command::new(&self.social_binary).arg("whoami").output()?;
		if output.status.success() {
			return Ok(Some(serde_json::from_slice(&output.stdout)?));
		}
		let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
		if message.contains("not signed in") {
			return Ok(None);
		}
		Err(Error::Command {
			action: "whoami",
			status: output.status,
			message,
		})
	}

	/// Revokes the current session where supported and clears local state.
	pub fn logout(&self) -> Result<(), Error> {
		let output = Command::new(&self.social_binary).arg("logout").output()?;
		let message = String::from_utf8_lossy(&output.stderr);
		ensure_success("logout", output.status, message.trim())
	}

	/// Resolves a handle or DID through the daemon's cached identity directory.
	pub fn resolve_identity(&self, identifier: &str) -> Result<Identity, Error> {
		let endpoint = self.endpoint()?;
		let mut url = url::Url::parse(&format!("{}/v1/identity/resolve", endpoint.base_url))
			.map_err(|error| Error::Transport(error.to_string()))?;
		url.query_pairs_mut().append_pair("identifier", identifier);
		self.get(&endpoint, url.as_str())
	}

	/// Creates a generic record in the signed-in account's repository.
	pub fn create_record(
		&self,
		collection: &str,
		record_key: Option<&str>,
		record: serde_json::Value,
	) -> Result<StrongRef, Error> {
		#[derive(Serialize)]
		struct Request<'a> {
			collection: &'a str,
			#[serde(skip_serializing_if = "Option::is_none")]
			rkey: Option<&'a str>,
			record: serde_json::Value,
		}
		let endpoint = self.endpoint()?;
		self.post(
			&endpoint,
			&format!("{}/v1/record", endpoint.base_url),
			&Request {
				collection,
				rkey: record_key,
				record,
			},
		)
	}

	/// Lists one page of records from a repository on the current PDS.
	pub fn list_records(
		&self,
		collection: &str,
		repo: Option<&str>,
		cursor: Option<&str>,
		limit: u8,
	) -> Result<RecordPage, Error> {
		let endpoint = self.endpoint()?;
		let mut url = url::Url::parse(&format!("{}/v1/record", endpoint.base_url))
			.map_err(|error| Error::Transport(error.to_string()))?;
		{
			let mut query = url.query_pairs_mut();
			query
				.append_pair("collection", collection)
				.append_pair("limit", &limit.clamp(1, 100).to_string());
			if let Some(repo) = repo {
				query.append_pair("repo", repo);
			}
			if let Some(cursor) = cursor {
				query.append_pair("cursor", cursor);
			}
		}
		self.get(&endpoint, url.as_str())
	}

	/// Uploads an image to the signed-in PDS and returns its reusable blob reference.
	pub fn upload_blob(&self, mime_type: &str, data: &[u8]) -> Result<BlobRef, Error> {
		if !mime_type.starts_with("image/") {
			return Err(Error::InvalidInput(
				"only image MIME types can be uploaded".into(),
			));
		}
		let endpoint = self.endpoint()?;
		self.post_bytes(
			&endpoint,
			&format!("{}/v1/blob", endpoint.base_url),
			mime_type,
			data,
		)
	}

	/// Publishes a local pack summary to the signed-in ATProto repository.
	pub fn share_pack(&self, share: &PackShare) -> Result<StrongRef, Error> {
		let mut record = serde_json::to_value(share)?;
		record["createdAt"] = serde_json::Value::String(rfc3339_after(Duration::ZERO)?);
		self.create_record(PACK_COLLECTION, None, record)
	}

	/// Publishes a text snippet to the signed-in ATProto repository.
	pub fn share_snippet(&self, text: &str, language: Option<&str>) -> Result<StrongRef, Error> {
		let mut record = serde_json::json!({
			"text": text,
			"createdAt": rfc3339_after(Duration::ZERO)?,
		});
		if let Some(language) = language.filter(|value| !value.is_empty()) {
			record["language"] = serde_json::Value::String(language.to_owned());
		}
		self.create_record(SNIPPET_COLLECTION, None, record)
	}

	/// Publishes a previously uploaded image blob and optional caption.
	pub fn share_image(&self, image: BlobRef, caption: Option<&str>) -> Result<StrongRef, Error> {
		let mut record = serde_json::json!({
			"image": image,
			"createdAt": rfc3339_after(Duration::ZERO)?,
		});
		if let Some(caption) = caption.filter(|value| !value.is_empty()) {
			record["caption"] = serde_json::Value::String(caption.to_owned());
		}
		self.create_record(IMAGE_COLLECTION, None, record)
	}

	/// Lists mutual follows and Packwand-specific contacts.
	pub fn list_friends(&self) -> Result<Vec<Friend>, Error> {
		let endpoint = self.endpoint()?;
		self.get(&endpoint, &format!("{}/v1/friends", endpoint.base_url))
	}

	/// Adds a DID to the Packwand contact fallback list.
	pub fn add_contact(&self, did: &str) -> Result<StrongRef, Error> {
		self.create_record(
			CONTACT_COLLECTION,
			None,
			serde_json::json!({
				"did": did,
				"createdAt": rfc3339_after(Duration::ZERO)?,
			}),
		)
	}

	/// Publishes a collaboration invitation addressed to another DID.
	pub fn send_invite(
		&self,
		to: &str,
		invite: &str,
		valid_for: Duration,
	) -> Result<StrongRef, Error> {
		if !to.starts_with("did:") {
			return Err(Error::InvalidInput("invite recipient must be a DID".into()));
		}
		if !invite.starts_with("pw://") {
			return Err(Error::InvalidInput(
				"collaboration invite must start with pw://".into(),
			));
		}
		if valid_for.is_zero() {
			return Err(Error::InvalidInput(
				"invite lifetime must be greater than zero".into(),
			));
		}
		let created_at = rfc3339_after(Duration::ZERO)?;
		let expires_at = rfc3339_after(valid_for)?;
		self.create_record(
			INVITE_COLLECTION,
			None,
			serde_json::json!({
				"to": to,
				"invite": invite,
				"createdAt": created_at,
				"expiresAt": expires_at,
			}),
		)
	}

	/// Lists unexpired collaboration invites addressed to the signed-in DID.
	pub fn list_pending_invites(&self) -> Result<Vec<PendingInvite>, Error> {
		let endpoint = self.endpoint()?;
		self.get(&endpoint, &format!("{}/v1/invites", endpoint.base_url))
	}

	/// Lists Tangled repositories linked to the signed-in DID by Bobbin.
	pub fn linked_tangled_repos(&self) -> Result<Vec<TangledRepo>, Error> {
		let endpoint = self.endpoint()?;
		self.get(
			&endpoint,
			&format!("{}/v1/tangled/repos", endpoint.base_url),
		)
	}

	fn endpoint(&self) -> Result<DaemonEndpoint, Error> {
		self.endpoint
			.clone()
			.map_or_else(|| ensure_running(&self.social_binary), Ok)
	}

	fn get<T: DeserializeOwned>(&self, endpoint: &DaemonEndpoint, url: &str) -> Result<T, Error> {
		let response = ureq::get(url)
			.timeout(Duration::from_secs(30))
			.set("Authorization", &format!("Bearer {}", endpoint.token))
			.call();
		decode_response(response)
	}

	fn post<T: DeserializeOwned>(
		&self,
		endpoint: &DaemonEndpoint,
		url: &str,
		body: &impl Serialize,
	) -> Result<T, Error> {
		let body = serde_json::to_vec(body)?;
		let response = ureq::post(url)
			.timeout(Duration::from_secs(30))
			.set("Authorization", &format!("Bearer {}", endpoint.token))
			.set("Content-Type", "application/json")
			.send_bytes(&body);
		decode_response(response)
	}

	fn post_bytes<T: DeserializeOwned>(
		&self,
		endpoint: &DaemonEndpoint,
		url: &str,
		content_type: &str,
		body: &[u8],
	) -> Result<T, Error> {
		let response = ureq::post(url)
			.timeout(Duration::from_secs(30))
			.set("Authorization", &format!("Bearer {}", endpoint.token))
			.set("Content-Type", content_type)
			.send_bytes(body);
		decode_response(response)
	}
}

fn rfc3339_after(offset: Duration) -> Result<String, Error> {
	let seconds = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_err(|error| Error::Transport(error.to_string()))?
		.checked_add(offset)
		.ok_or_else(|| Error::Transport("timestamp overflow".into()))?
		.as_secs();
	Ok(format_unix_utc(seconds))
}

fn format_unix_utc(seconds: u64) -> String {
	let days = (seconds / 86_400) as i64;
	let day_seconds = seconds % 86_400;
	let (year, month, day) = civil_from_days(days);
	format!(
		"{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
		day_seconds / 3_600,
		(day_seconds % 3_600) / 60,
		day_seconds % 60
	)
}

/// Converts Unix days using Howard Hinnant's public-domain Gregorian algorithm.
fn civil_from_days(days_since_epoch: i64) -> (i64, u64, u64) {
	let z = days_since_epoch + 719_468;
	let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
	let day_of_era = z - era * 146_097;
	let year_of_era =
		(day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
	let mut year = year_of_era + era * 400;
	let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
	let month_prime = (5 * day_of_year + 2) / 153;
	let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
	let month = month_prime + if month_prime < 10 { 3 } else { -9 };
	year += i64::from(month <= 2);
	(year, month as u64, day as u64)
}

fn decode_response<T: DeserializeOwned>(
	response: Result<ureq::Response, ureq::Error>,
) -> Result<T, Error> {
	match response {
		Ok(response) => Ok(serde_json::from_reader(response.into_reader())?),
		Err(ureq::Error::Status(status, response)) => {
			let value: serde_json::Value = serde_json::from_reader(response.into_reader())
				.unwrap_or_else(|_| serde_json::json!({}));
			let message = value
				.get("error")
				.and_then(serde_json::Value::as_str)
				.unwrap_or("ATProto bridge request failed")
				.to_owned();
			Err(Error::Http { status, message })
		}
		Err(ureq::Error::Transport(error)) => Err(Error::Transport(error.to_string())),
	}
}

fn ensure_success(action: &'static str, status: ExitStatus, message: &str) -> Result<(), Error> {
	if status.success() {
		Ok(())
	} else {
		Err(Error::Command {
			action,
			status,
			message: message.to_owned(),
		})
	}
}

fn find_social_binary(explicit: Option<&Path>) -> Result<PathBuf, Error> {
	let mut candidates = explicit
		.map(Path::to_path_buf)
		.into_iter()
		.collect::<Vec<_>>();
	if let Some(configured) = std::env::var_os("PACKWAND_SOCIAL_BIN") {
		candidates.push(PathBuf::from(configured));
	}
	let mut roots = vec![std::env::current_dir()?];
	if let Ok(executable) = std::env::current_exe()
		&& let Some(parent) = executable.parent()
	{
		roots.push(parent.to_path_buf());
	}
	for mut root in roots {
		loop {
			for relative in social_binary_candidates() {
				candidates.push(root.join(relative));
			}
			if !root.pop() {
				break;
			}
		}
	}
	for candidate in candidates {
		if candidate.is_file() {
			return fs::canonicalize(candidate).map_err(Error::from);
		}
	}
	Err(Error::BinaryNotFound)
}

fn social_binary_candidates() -> [&'static str; 6] {
	if cfg!(windows) {
		[
			"packwand-social.exe",
			"resources/packwand-social.exe",
			"packwand-social/packwand-social.exe",
			"apps/packwandrs/packwand-social/packwand-social.exe",
			"target/debug/packwand-social.exe",
			"target/release/packwand-social.exe",
		]
	} else {
		[
			"packwand-social",
			"resources/packwand-social",
			"packwand-social/packwand-social",
			"apps/packwandrs/packwand-social/packwand-social",
			"target/debug/packwand-social",
			"target/release/packwand-social",
		]
	}
}

#[cfg(test)]
mod tests {
	use std::io::{Read, Write};
	use std::net::TcpListener;
	use std::thread;

	use super::*;

	#[test]
	fn resolves_identity_through_authenticated_daemon() {
		let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind mock daemon");
		let address = listener.local_addr().expect("mock daemon address");
		let server = thread::spawn(move || {
			let (mut stream, _) = listener.accept().expect("accept request");
			stream
				.set_read_timeout(Some(Duration::from_millis(250)))
				.expect("set request timeout");
			let mut request = Vec::new();
			loop {
				let mut chunk = [0u8; 4096];
				match stream.read(&mut chunk) {
					Ok(0) => break,
					Ok(count) => request.extend_from_slice(&chunk[..count]),
					Err(error)
						if matches!(
							error.kind(),
							std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
						) =>
					{
						break;
					}
					Err(error) => panic!("read request: {error}"),
				}
			}
			let request = String::from_utf8_lossy(&request);
			assert!(request.contains("Authorization: Bearer secret"));
			assert!(request.contains("identifier=alice.example"));
			let body =
				r#"{"did":"did:plc:alice","handle":"alice.example","pds":"https://pds.example"}"#;
			write!(
				stream,
				"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
				body.len()
			)
			.expect("write response");
		});
		let client = IdentityClient::with_endpoint(format!("http://{address}"), "secret".into());
		let identity = client
			.resolve_identity("alice.example")
			.expect("resolve identity");
		assert_eq!(identity.did, "did:plc:alice");
		server.join().expect("join mock daemon");
	}

	#[test]
	fn formats_unix_timestamps_as_rfc3339_utc() {
		assert_eq!(format_unix_utc(0), "1970-01-01T00:00:00Z");
		assert_eq!(format_unix_utc(951_827_696), "2000-02-29T12:34:56Z");
	}

	#[test]
	fn uploads_blob_with_the_original_content_type() {
		let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind mock daemon");
		let address = listener.local_addr().expect("mock daemon address");
		let server = thread::spawn(move || {
			let (mut stream, _) = listener.accept().expect("accept request");
			stream
				.set_read_timeout(Some(Duration::from_millis(250)))
				.expect("set request timeout");
			let mut request = Vec::new();
			loop {
				let mut chunk = [0u8; 4096];
				match stream.read(&mut chunk) {
					Ok(0) => break,
					Ok(count) => request.extend_from_slice(&chunk[..count]),
					Err(error)
						if matches!(
							error.kind(),
							std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
						) =>
					{
						break;
					}
					Err(error) => panic!("read request: {error}"),
				}
			}
			let request = String::from_utf8_lossy(&request);
			assert!(request.starts_with("POST /v1/blob "));
			assert!(request.contains("Content-Type: image/png"));
			assert!(request.contains("png"));
			let body =
				r#"{"$type":"blob","ref":{"$link":"bafyblob"},"mimeType":"image/png","size":3}"#;
			write!(
				stream,
				"HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
				body.len()
			)
			.expect("write response");
		});
		let client = IdentityClient::with_endpoint(format!("http://{address}"), "secret".into());
		let blob = client
			.upload_blob("image/png", b"png")
			.expect("upload blob");
		assert_eq!(blob.reference.cid, "bafyblob");
		server.join().expect("join mock daemon");
	}

	#[test]
	fn lists_friends_through_the_authenticated_daemon() {
		let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind mock daemon");
		let address = listener.local_addr().expect("mock daemon address");
		let server = thread::spawn(move || {
			let (mut stream, _) = listener.accept().expect("accept request");
			let mut request = [0u8; 4096];
			let count = stream.read(&mut request).expect("read request");
			let request = String::from_utf8_lossy(&request[..count]);
			assert!(request.starts_with("GET /v1/friends "));
			assert!(request.contains("Authorization: Bearer secret"));
			let body =
				r#"[{"did":"did:plc:bob","handle":"bob.example","sources":["mutual_follow"]}]"#;
			write!(
				stream,
				"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
				body.len()
			)
			.expect("write response");
		});
		let client = IdentityClient::with_endpoint(format!("http://{address}"), "secret".into());
		let friends = client.list_friends().expect("list friends");
		assert_eq!(friends[0].did, "did:plc:bob");
		server.join().expect("join mock daemon");
	}

	#[test]
	fn shares_pack_records_through_the_authenticated_daemon() {
		let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind mock daemon");
		let address = listener.local_addr().expect("mock daemon address");
		let server = thread::spawn(move || {
			let (mut stream, _) = listener.accept().expect("accept request");
			stream
				.set_read_timeout(Some(Duration::from_millis(250)))
				.expect("set request timeout");
			let mut request = Vec::new();
			loop {
				let mut chunk = [0u8; 4096];
				match stream.read(&mut chunk) {
					Ok(0) => break,
					Ok(count) => request.extend_from_slice(&chunk[..count]),
					Err(error)
						if matches!(
							error.kind(),
							std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
						) =>
					{
						break;
					}
					Err(error) => panic!("read request: {error}"),
				}
			}
			let request = String::from_utf8_lossy(&request);
			assert!(request.starts_with("POST /v1/record "));
			assert!(request.contains("Authorization: Bearer secret"));
			assert!(request.contains(r#""collection":"net.nostalgica.packwand.pack""#));
			assert!(request.contains(r#""name":"Shared fixture""#));
			assert!(request.contains(r#""createdAt":""#));
			let body = r#"{"uri":"at://did:plc:alice/net.nostalgica.packwand.pack/one","cid":"bafyrecord"}"#;
			write!(
				stream,
				"HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
				body.len()
			)
			.expect("write response");
		});
		let client = IdentityClient::with_endpoint(format!("http://{address}"), "secret".into());
		let reference = client
			.share_pack(&PackShare {
				name: "Shared fixture".into(),
				description: Some("Pack description".into()),
				manifest: ManifestSummary {
					id: "fixture".into(),
					project_type: "modpack".into(),
					version: "1.0.0".into(),
					minecraft_version: Some("1.21.1".into()),
					loader: Some("fabric".into()),
					environment: None,
					variants: Vec::new(),
				},
				tangled_repo: None,
				git_remote: Some("https://example.invalid/fixture.git".into()),
			})
			.expect("share pack");
		assert_eq!(reference.cid, "bafyrecord");
		server.join().expect("join mock daemon");
	}
}
