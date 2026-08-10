//! Minimal HTTP seam: production uses [`UreqClient`]; tests implement
//! [`HttpClient`] over in-memory fixtures so nothing touches the network.

use std::collections::BTreeMap;
use std::io::Read;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
#[error("GET {url} failed: {message}")]
pub struct HttpError {
	pub url: String,
	pub message: String,
}

/// HTTP client abstraction for downloading files.
pub trait HttpClient: Send + Sync {
	fn get(&self, url: &str) -> Result<Vec<u8>, HttpError>;

	fn get_with_progress(
		&self,
		url: &str,
		on_chunk: &mut dyn FnMut(usize, Option<u64>),
	) -> Result<Vec<u8>, HttpError> {
		let bytes = self.get(url)?;
		on_chunk(bytes.len(), Some(bytes.len() as u64));
		Ok(bytes)
	}
}

/// Real client with connection reuse and timeouts.
pub struct UreqClient {
	agent: ureq::Agent,
	/// Refuse responses larger than this (default 512 MiB) so a
	/// misbehaving server cannot exhaust memory.
	max_body_bytes: u64,
}

impl UreqClient {
	/// Creates a new HTTP client with connection reuse and default timeouts.
	pub fn new() -> Self {
		Self {
			agent: ureq::AgentBuilder::new()
				.timeout_connect(Duration::from_secs(15))
				.timeout_read(Duration::from_secs(120))
				.user_agent(concat!("packwand-rs/", env!("CARGO_PKG_VERSION")))
				.build(),
			max_body_bytes: 512 * 1024 * 1024,
		}
	}
}

impl Default for UreqClient {
	fn default() -> Self {
		Self::new()
	}
}

impl HttpClient for UreqClient {
	fn get(&self, url: &str) -> Result<Vec<u8>, HttpError> {
		self.get_with_progress(url, &mut |_, _| {})
	}

	fn get_with_progress(
		&self,
		url: &str,
		on_chunk: &mut dyn FnMut(usize, Option<u64>),
	) -> Result<Vec<u8>, HttpError> {
		let error = |message: String| HttpError {
			url: url.to_string(),
			message,
		};
		let response = self
			.agent
			.get(url)
			.call()
			.map_err(|e| error(e.to_string()))?;
		let content_total = response
			.header("Content-Length")
			.and_then(|value| value.parse::<u64>().ok());
		let mut reader = response.into_reader();
		let mut bytes = Vec::new();
		let mut chunk = [0u8; 64 * 1024];
		loop {
			let read = reader.read(&mut chunk).map_err(|e| error(e.to_string()))?;
			if read == 0 {
				break;
			}
			bytes.extend_from_slice(&chunk[..read]);
			if bytes.len() as u64 > self.max_body_bytes {
				return Err(error(format!(
					"response exceeded the {} byte limit",
					self.max_body_bytes
				)));
			}
			on_chunk(read, content_total);
		}
		Ok(bytes)
	}
}

/// In-memory fixture client for tests. Unknown URLs return an error, and
/// every request is recorded.
#[derive(Default)]
pub struct FixtureHttpClient {
	responses: BTreeMap<String, Vec<u8>>,
	pub requests: Mutex<Vec<String>>,
}

impl FixtureHttpClient {
	/// Creates a new test fixture client with preconfigured responses.
	pub fn new(responses: impl IntoIterator<Item = (String, Vec<u8>)>) -> Self {
		Self {
			responses: responses.into_iter().collect(),
			requests: Mutex::new(Vec::new()),
		}
	}
}

impl HttpClient for FixtureHttpClient {
	fn get(&self, url: &str) -> Result<Vec<u8>, HttpError> {
		self.requests
			.lock()
			.expect("request log poisoned")
			.push(url.to_string());
		self.responses.get(url).cloned().ok_or_else(|| HttpError {
			url: url.to_string(),
			message: "no fixture response registered".to_string(),
		})
	}
}
