use std::io::Read;
use std::path::Path;

use crate::agent::{Profile, agent, wait_host_rate_limit};
use crate::cache::{MetaCache, max_age_of};
use crate::error::NetError;
use crate::request::{Checksum, Request};
use crate::retry::{MAX_ATTEMPTS, is_transient, wait_for};
use crate::sink::FileSink;

const CHUNK_BYTES: usize = 64 * 1024;

/// What to send. Carrying the body here keeps one retry loop for both verbs
/// rather than a near-duplicate per method.
#[derive(Clone, Copy)]
enum Method<'a> {
	Get,
	PostJson(&'a [u8]),
}

/// Called as a transfer proceeds, with bytes so far and the total when the
/// server declared one.
pub type ProgressFn<'a> = &'a mut dyn FnMut(u64, Option<u64>);

/// Where a body came from, so callers can report "downloaded" separately from
/// "already had it".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
	/// Fetched over the network.
	Network,
	/// Served from the metadata cache without a request.
	CacheFresh,
	/// Revalidated with the server, which answered 304.
	CacheRevalidated,
	/// The request failed and a cached copy was served instead.
	///
	/// Distinct from [`Self::CacheRevalidated`] on purpose: nobody confirmed
	/// this is current. A caller resolving something mutable — "the latest
	/// release" — needs to know it is answering from a possibly week-old
	/// document rather than a checked one.
	Stale,
}

impl Source {
	/// Whether this answer was confirmed against the server.
	pub fn is_current(self) -> bool {
		!matches!(self, Self::Stale)
	}
}

/// Whether a cache entry that has not expired may be used as-is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
	/// Serve an unexpired entry without contacting the server.
	UseCached,
	/// Always ask, but with validators attached — a 304 still avoids the
	/// payload. For documents whose contents change under a stable URL, where
	/// a stale answer would be wrong rather than merely old.
	AlwaysRevalidate,
}

/// A fetched document plus where it came from.
#[derive(Debug, Clone)]
pub struct Fetched {
	/// The body.
	pub bytes: Vec<u8>,
	/// How it was obtained.
	pub source: Source,
}

/// The HTTP client every Packwand fetch goes through.
///
/// Cheap to construct — the underlying agent, and so the connection pool, is
/// process-wide and shared.
pub struct Client {
	agent: ureq::Agent,
	profile: Profile,
	max_body_bytes: u64,
}

impl Client {
	/// A client for metadata and API calls.
	pub fn api() -> Self {
		Self::with_profile(Profile::Api)
	}

	/// A client for file transfers.
	///
	/// Callers downloading a jar must use this: the API ceiling silently fails
	/// any file over 16 MiB, an ordinary size for a modpack jar.
	pub fn downloads() -> Self {
		Self::with_profile(Profile::Download)
	}

	fn with_profile(profile: Profile) -> Self {
		Self {
			agent: agent(profile),
			profile,
			max_body_bytes: profile.max_body_bytes(),
		}
	}

	/// Overrides the response ceiling for this client.
	pub fn with_max_body_bytes(mut self, limit: u64) -> Self {
		self.max_body_bytes = limit;
		self
	}

	/// Which profile this client was built for.
	pub fn profile(&self) -> Profile {
		self.profile
	}

	/// Fetches a body into memory.
	pub fn get(&self, request: &Request) -> Result<Vec<u8>, NetError> {
		self.get_with_progress(request, &mut |_, _| {})
	}

	/// Posts a JSON body and reads the response.
	///
	/// Retried on the same terms as a GET, which is only sound because the
	/// callers are idempotent queries — CurseForge's fingerprint match. A
	/// publish upload must not come through here: retrying one after an
	/// ambiguous failure can publish twice.
	pub fn post_json(&self, request: &Request, body: &[u8]) -> Result<Vec<u8>, NetError> {
		let (response, url) = self.send(request, Method::PostJson(body))?;
		self.read_body(&url, response)
	}

	/// Fetches a body into memory, reporting progress as it arrives.
	pub fn get_with_progress(
		&self,
		request: &Request,
		progress: ProgressFn<'_>,
	) -> Result<Vec<u8>, NetError> {
		let (response, url) = self.send(request, Method::Get)?;
		let total = content_length(&response);
		let mut reader = response.into_reader();
		let mut bytes = Vec::with_capacity(total.unwrap_or(0).min(1 << 20) as usize);
		let mut chunk = vec![0u8; CHUNK_BYTES];
		loop {
			let read = reader
				.read(&mut chunk)
				.map_err(|error| self.transport_error(&url, error.to_string()))?;
			if read == 0 {
				break;
			}
			bytes.extend_from_slice(&chunk[..read]);
			if bytes.len() as u64 > self.max_body_bytes {
				return Err(NetError::TooLarge {
					url,
					limit: self.max_body_bytes,
				});
			}
			progress(bytes.len() as u64, total);
		}
		Ok(bytes)
	}

	/// Streams a body to `target`, hashing as it goes, and moves it into place
	/// only if it verifies.
	///
	/// Peak memory is one chunk regardless of file size — the reason this
	/// exists rather than fetching to a `Vec` and writing it out.
	pub fn download_to(
		&self,
		request: &Request,
		target: &Path,
		checksum: Option<&Checksum>,
		progress: ProgressFn<'_>,
	) -> Result<u64, NetError> {
		let (response, url) = self.send(request, Method::Get)?;
		let total = content_length(&response);
		let mut reader = response.into_reader();
		let mut sink = FileSink::create(target, checksum)?;
		let mut chunk = vec![0u8; CHUNK_BYTES];
		loop {
			let read = match reader.read(&mut chunk) {
				Ok(read) => read,
				Err(error) => {
					sink.abort();
					return Err(self.transport_error(&url, error.to_string()));
				}
			};
			if read == 0 {
				break;
			}
			// No `abort()` here: a write failure means the staging file itself
			// is unusable, and `FileSink` leaves nothing at the target either way.
			sink.write(&chunk[..read])?;
			if sink.written() > self.max_body_bytes {
				sink.abort();
				return Err(NetError::TooLarge {
					url,
					limit: self.max_body_bytes,
				});
			}
			progress(sink.written(), total);
		}
		sink.commit(&url, checksum)
	}

	/// Fetches a document through `cache`, skipping the request when the
	/// stored copy is still fresh and revalidating it when it is not.
	pub fn get_cached(&self, request: &Request, cache: &MetaCache) -> Result<Fetched, NetError> {
		self.get_cached_with(request, cache, Freshness::UseCached)
	}

	/// [`Self::get_cached`] with control over whether a fresh entry may be
	/// served without asking the server.
	pub fn get_cached_with(
		&self,
		request: &Request,
		cache: &MetaCache,
		freshness: Freshness,
	) -> Result<Fetched, NetError> {
		let key = request.primary()?.to_owned();
		if freshness == Freshness::UseCached
			&& let Some(bytes) = cache.read_fresh(&key)
		{
			return Ok(Fetched {
				bytes,
				source: Source::CacheFresh,
			});
		}

		let stored = cache.entry(&key);
		let mut conditional = request.clone();
		if let Some(entry) = &stored {
			if let Some(etag) = &entry.etag {
				conditional = conditional.header("If-None-Match", etag.clone());
			}
			if let Some(modified) = &entry.last_modified {
				conditional = conditional.header("If-Modified-Since", modified.clone());
			}
		}

		let (response, url) = match self.send(&conditional, Method::Get) {
			Ok(ok) => ok,
			// A network failure with something on disk is better answered with
			// the stale copy than with an error; offline still works. The
			// source records that nothing confirmed it.
			Err(error) => {
				return match cache.read(&key) {
					Some(bytes) => Ok(Fetched {
						bytes,
						source: Source::Stale,
					}),
					None => Err(error),
				};
			}
		};

		let max_age = max_age_of(response.header("Cache-Control"), response.header("Expires"));
		if response.status() == 304
			&& let Some(bytes) = cache.read(&key)
		{
			cache.touch(&key, max_age)?;
			return Ok(Fetched {
				bytes,
				source: Source::CacheRevalidated,
			});
		}

		let etag = response.header("ETag").map(str::to_owned);
		let last_modified = response.header("Last-Modified").map(str::to_owned);
		let bytes = self.read_body(&url, response)?;
		cache.store(&key, &bytes, etag, last_modified, max_age)?;
		Ok(Fetched {
			bytes,
			source: Source::Network,
		})
	}

	/// Issues the request against each URL in turn, retrying transient
	/// failures, and returns the first response that arrives.
	///
	/// Every attempt — retries included — pays the host pacing cost, so a
	/// retry storm cannot blow a provider's request budget either.
	fn send(
		&self,
		request: &Request,
		method: Method<'_>,
	) -> Result<(ureq::Response, String), NetError> {
		let urls = request.urls();
		if urls.is_empty() {
			return Err(NetError::NoUrl);
		}
		let mut last = None;
		for url in urls {
			match self.send_one(url, request, method) {
				Ok(response) => return Ok((response, url.clone())),
				Err(error) => last = Some(error),
			}
		}
		Err(last.expect("at least one URL was tried"))
	}

	fn send_one(
		&self,
		url: &str,
		request: &Request,
		method: Method<'_>,
	) -> Result<ureq::Response, NetError> {
		for attempt in 1..=MAX_ATTEMPTS {
			wait_host_rate_limit(url);
			let mut call = match method {
				Method::Get => self.agent.get(url),
				Method::PostJson(_) => self.agent.post(url).set("Content-Type", "application/json"),
			};
			for (name, value) in request.headers() {
				call = call.set(name, value);
			}
			let sent = match method {
				Method::Get => call.call(),
				Method::PostJson(body) => call.send_bytes(body),
			};
			let error = match sent {
				Ok(response) => return Ok(response),
				// 304 is the expected answer to a conditional request, and
				// ureq reports every non-2xx as an error.
				Err(ureq::Error::Status(304, response)) => return Ok(response),
				Err(error) => error,
			};
			if attempt == MAX_ATTEMPTS || !is_transient(&error) {
				return Err(status_error(url, error));
			}
			std::thread::sleep(wait_for(&error, attempt));
		}
		unreachable!("the loop returns on its final attempt")
	}

	fn read_body(&self, url: &str, response: ureq::Response) -> Result<Vec<u8>, NetError> {
		let limit = self.max_body_bytes;
		let mut reader = response.into_reader().take(limit + 1);
		let mut bytes = Vec::new();
		reader
			.read_to_end(&mut bytes)
			.map_err(|error| self.transport_error(url, error.to_string()))?;
		if bytes.len() as u64 > limit {
			return Err(NetError::TooLarge {
				url: url.to_owned(),
				limit,
			});
		}
		Ok(bytes)
	}

	fn transport_error(&self, url: &str, message: String) -> NetError {
		NetError::Http {
			url: url.to_owned(),
			message,
			status: None,
			body_snippet: None,
		}
	}
}

fn content_length(response: &ureq::Response) -> Option<u64> {
	response
		.header("Content-Length")
		.and_then(|value| value.parse().ok())
}

/// ureq's error embeds the whole response, which trips clippy's large-error
/// lint; the size is ureq's and boxing here would only move the allocation.
#[allow(clippy::result_large_err)]
fn status_error(url: &str, error: ureq::Error) -> NetError {
	match error {
		ureq::Error::Status(code, response) => {
			let body_snippet = response
				.into_string()
				.ok()
				.map(|body| body.chars().take(500).collect());
			NetError::Http {
				url: url.to_owned(),
				message: format!("http status {code}"),
				status: Some(code),
				body_snippet,
			}
		}
		ureq::Error::Transport(inner) => NetError::Http {
			url: url.to_owned(),
			message: inner.to_string(),
			status: None,
			body_snippet: None,
		},
	}
}
