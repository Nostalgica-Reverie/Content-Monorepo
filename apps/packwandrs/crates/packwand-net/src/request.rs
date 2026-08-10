use packwand_pack::HashFormat;

use crate::NetError;

/// What a response must hash to before it is allowed to reach disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checksum {
	/// The digest algorithm.
	pub format: HashFormat,
	/// The expected digest, hex, compared case-insensitively.
	pub expected: String,
}

impl Checksum {
	/// Builds a checksum from the wire spelling of a format, as pack metadata
	/// stores it.
	pub fn parse(format: &str, expected: impl Into<String>) -> Result<Self, NetError> {
		Ok(Self {
			format: format
				.parse()
				.map_err(|_| NetError::HashFormat(format.to_owned()))?,
			expected: expected.into(),
		})
	}
}

/// One resource to fetch, with the mirrors that also serve it.
///
/// Mirrors are the piece Prism has a primitive for and never wired up: a
/// resource available from several hosts should not fail because one of them
/// is down.
#[derive(Debug, Clone, Default)]
pub struct Request {
	urls: Vec<String>,
	headers: Vec<(String, String)>,
}

impl Request {
	/// A request for one URL.
	pub fn get(url: impl Into<String>) -> Self {
		Self {
			urls: vec![url.into()],
			headers: Vec::new(),
		}
	}

	/// Adds an alternate URL serving the same bytes, tried in order after the
	/// ones already present.
	pub fn mirror(mut self, url: impl Into<String>) -> Self {
		self.urls.push(url.into());
		self
	}

	/// Adds a request header.
	pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
		self.headers.push((name.into(), value.into()));
		self
	}

	/// Every URL to try, in order.
	pub fn urls(&self) -> &[String] {
		&self.urls
	}

	/// The primary URL, used for error messages and cache keys.
	pub fn primary(&self) -> Result<&str, NetError> {
		self.urls.first().map(String::as_str).ok_or(NetError::NoUrl)
	}

	/// The primary URL, or an empty string — for building an error message,
	/// where failing to name the URL must not itself fail.
	pub fn primary_or_empty(&self) -> &str {
		self.urls.first().map_or("", String::as_str)
	}

	/// The headers to send.
	pub fn headers(&self) -> &[(String, String)] {
		&self.headers
	}
}
