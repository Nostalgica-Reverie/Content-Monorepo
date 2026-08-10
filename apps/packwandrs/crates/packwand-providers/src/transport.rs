//! The provider-facing HTTP seam.
//!
//! Everything underneath — shared agents, per-host pacing, the retry policy,
//! `Retry-After` handling — lives in [`packwand_net`]. This file is the
//! adapter that keeps providers talking in their own vocabulary; it used to be
//! a second, independent HTTP stack.

use packwand_net::{Client, NetError, Request};

/// One request a provider wants to make.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
	pub url: String,
	pub headers: Vec<(String, String)>,
}

impl HttpRequest {
	pub fn get(url: impl Into<String>) -> Self {
		Self {
			url: url.into(),
			headers: Vec::new(),
		}
	}

	pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
		self.headers.push((name.into(), value.into()));
		self
	}

	fn to_net(&self) -> Request {
		let mut request = Request::get(self.url.clone());
		for (name, value) in &self.headers {
			request = request.header(name.clone(), value.clone());
		}
		request
	}
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("GET {url} failed: {message}")]
pub struct TransportError {
	pub url: String,
	pub message: String,
	/// HTTP status code, when the failure was a non-2xx response rather than
	/// a connection/transport-level failure.
	pub status: Option<u16>,
	/// First ~500 characters of the response body, when available. Lets
	/// callers distinguish a real API error payload from a generic
	/// CDN/WAF block page (e.g. CloudFront's static "Request blocked" HTML),
	/// which returns the same status code but never reaches the actual API.
	pub body_snippet: Option<String>,
}

impl From<NetError> for TransportError {
	fn from(error: NetError) -> Self {
		match error {
			NetError::Http {
				url,
				message,
				status,
				body_snippet,
			} => Self {
				url,
				message,
				status,
				body_snippet,
			},
			other => Self {
				url: String::new(),
				message: other.to_string(),
				status: other.status(),
				body_snippet: None,
			},
		}
	}
}

pub trait Transport: Send + Sync {
	fn get(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError>;

	/// Fetches a file rather than an API response.
	///
	/// Release assets are downloaded in full to hash them, and a mod jar
	/// routinely exceeds the ceiling that is right for JSON. Defaults to
	/// [`Transport::get`] so test doubles need not care.
	fn get_large(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError> {
		self.get(request)
	}

	fn post_json(&self, request: HttpRequest, _body: &[u8]) -> Result<Vec<u8>, TransportError> {
		Err(TransportError {
			url: request.url,
			message: "transport does not support JSON POST requests".into(),
			status: None,
			body_snippet: None,
		})
	}
}

/// The production transport.
///
/// Cheap to construct per operation: the agents behind both clients are
/// process-wide, so the connection pool and the host pacing survive.
pub struct UreqTransport {
	api: Client,
	downloads: Client,
}

impl UreqTransport {
	/// A transport whose API calls use the metadata profile.
	pub fn new() -> Self {
		Self {
			api: Client::api(),
			downloads: Client::downloads(),
		}
	}

	/// A transport whose plain `get` already uses the transfer profile.
	///
	/// Callers that fetch a jar through `get` rather than `get_large` need
	/// this: the API ceiling silently fails any file over 16 MiB, an ordinary
	/// size for a modpack jar.
	pub fn for_downloads() -> Self {
		Self {
			api: Client::downloads(),
			downloads: Client::downloads(),
		}
	}
}

impl Default for UreqTransport {
	fn default() -> Self {
		Self::new()
	}
}

impl Transport for UreqTransport {
	fn get(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError> {
		Ok(self.api.get(&request.to_net())?)
	}

	fn get_large(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError> {
		Ok(self.downloads.get(&request.to_net())?)
	}

	fn post_json(&self, request: HttpRequest, body: &[u8]) -> Result<Vec<u8>, TransportError> {
		Ok(self.api.post_json(&request.to_net(), body)?)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn requests_carry_their_headers_across_the_seam() {
		let request = HttpRequest::get("https://api.modrinth.com/v2/x")
			.header("Authorization", "token")
			.header("Accept", "application/json");
		let net = request.to_net();
		assert_eq!(net.urls(), ["https://api.modrinth.com/v2/x"]);
		assert_eq!(
			net.headers(),
			[
				("Authorization".to_owned(), "token".to_owned()),
				("Accept".to_owned(), "application/json".to_owned()),
			]
		);
	}

	#[test]
	fn status_failures_keep_the_detail_providers_branch_on() {
		let error: TransportError = NetError::Http {
			url: "https://api.curseforge.com/v1/mods".into(),
			message: "http status 403".into(),
			status: Some(403),
			body_snippet: Some("Request blocked".into()),
		}
		.into();
		assert_eq!(error.status, Some(403));
		assert_eq!(error.body_snippet.as_deref(), Some("Request blocked"));
	}
}
