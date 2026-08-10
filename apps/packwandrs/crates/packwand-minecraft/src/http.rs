//! The metadata seam over [`packwand_net`].
//!
//! Only version metadata comes through here — Mojang's manifest, version
//! documents, asset indexes, loader profiles. File transfers go straight to
//! [`packwand_net::Client`], which streams and verifies them; this trait
//! exists so `MetadataClient` can be pointed at fixtures and the test suite
//! never touches the network.

use std::collections::BTreeMap;
use std::sync::Mutex;

use packwand_net::{Client, Freshness, MetaCache, NetError, Request};

#[derive(Debug, thiserror::Error)]
#[error("GET {url} failed: {message}")]
pub struct HttpError {
	pub url: String,
	pub message: String,
}

impl From<NetError> for HttpError {
	fn from(error: NetError) -> Self {
		let url = match &error {
			NetError::Http { url, .. }
			| NetError::TooLarge { url, .. }
			| NetError::Checksum { url, .. } => url.clone(),
			NetError::Io { path, .. } => path.display().to_string(),
			NetError::HashFormat(format) => format.clone(),
			NetError::NoUrl => String::new(),
		};
		Self {
			url,
			message: error.to_string(),
		}
	}
}

/// Fetches metadata documents.
pub trait HttpClient: Send + Sync {
	fn get(&self, url: &str) -> Result<Vec<u8>, HttpError>;

	/// Fetches a document that lives at a stable URL but whose contents
	/// change — Mojang's version manifest is the one that matters.
	///
	/// Revalidated rather than refetched when a cache is configured, so a
	/// repeated boot pays one conditional request instead of a megabyte. The
	/// default has no cache and is a plain fetch.
	fn get_document(&self, url: &str) -> Result<Vec<u8>, HttpError> {
		self.get(url)
	}

	/// Fetches a document whose parent has published its digest.
	///
	/// The content-addressed half of freshness: a version manifest names the
	/// sha1 of every version document it points at, so a cached copy matching
	/// that digest needs no request at all — not a conditional one, none. And
	/// when the digest differs, the document is known to have changed without
	/// having to ask. Time-based expiry can express neither.
	///
	/// The default has no cache and simply fetches.
	fn get_child_document(&self, url: &str, sha1: Option<&str>) -> Result<Vec<u8>, HttpError> {
		let _ = sha1;
		self.get(url)
	}
}

/// Real client: shared connection pool, one retry policy.
pub struct UreqClient {
	inner: Client,
	documents: Option<MetaCache>,
}

impl UreqClient {
	/// Creates a client on the shared transfer-profile agent.
	pub fn new() -> Self {
		Self {
			inner: Client::downloads(),
			documents: None,
		}
	}

	/// Revalidates mutable metadata documents against `root` instead of
	/// refetching them.
	///
	/// Deliberately narrow. Version documents and asset indexes are immutable
	/// and already persisted next to what they describe, so caching those
	/// again would be a third copy of bytes the installer can already skip.
	pub fn with_document_cache(mut self, root: &std::path::Path) -> Self {
		self.documents = MetaCache::open(root, "meta").ok();
		self
	}

	/// The underlying client, which is what the installer downloads through.
	pub fn inner(&self) -> &Client {
		&self.inner
	}
}

impl Default for UreqClient {
	fn default() -> Self {
		Self::new()
	}
}

impl HttpClient for UreqClient {
	fn get(&self, url: &str) -> Result<Vec<u8>, HttpError> {
		Ok(self.inner.get(&Request::get(url))?)
	}

	fn get_document(&self, url: &str) -> Result<Vec<u8>, HttpError> {
		let Some(cache) = &self.documents else {
			return self.get(url);
		};
		Ok(self
			.inner
			.get_cached_with(&Request::get(url), cache, Freshness::AlwaysRevalidate)?
			.bytes)
	}

	fn get_child_document(&self, url: &str, sha1: Option<&str>) -> Result<Vec<u8>, HttpError> {
		let (Some(cache), Some(sha1)) = (&self.documents, sha1) else {
			return self.get(url);
		};
		// The parent vouched for these exact bytes, so there is nothing left
		// to confirm with the server.
		if cache.matches_digest(url, packwand_pack::HashFormat::Sha1, sha1)
			&& let Some(bytes) = cache.read(url)
		{
			return Ok(bytes);
		}
		let bytes = self.get(url)?;
		// Stored only after the caller's own verification would accept it;
		// storing first would cache a body the digest rejects.
		if packwand_pack::hash_bytes(packwand_pack::HashFormat::Sha1, &bytes)
			.eq_ignore_ascii_case(sha1)
		{
			let _ = cache.store(url, &bytes, None, None, None);
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
