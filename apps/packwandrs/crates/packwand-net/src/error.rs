use std::path::PathBuf;

/// Anything that can go wrong fetching or committing a remote resource.
#[derive(Debug, thiserror::Error)]
pub enum NetError {
	/// The request failed, or the server answered with a non-2xx status.
	#[error("GET {url} failed: {message}")]
	Http {
		/// The URL that failed. For a request with mirrors this is the last
		/// one tried.
		url: String,
		/// A human-readable cause.
		message: String,
		/// Present when the failure was a status code rather than a
		/// connection-level error.
		status: Option<u16>,
		/// The first ~500 characters of the body, which is how a real API
		/// error is told apart from a CDN block page carrying the same status.
		body_snippet: Option<String>,
	},
	/// The body was larger than the profile's ceiling.
	#[error("GET {url} exceeded the {limit} byte limit")]
	TooLarge {
		/// The URL whose response was oversized.
		url: String,
		/// The ceiling that was exceeded, in bytes.
		limit: u64,
	},
	/// The bytes arrived intact but were not what was asked for. The target
	/// is never written in this case.
	#[error("checksum mismatch for {url}: expected {expected}, got {actual}")]
	Checksum {
		/// The URL whose body failed verification.
		url: String,
		/// The checksum the caller required.
		expected: String,
		/// What the body actually hashed to.
		actual: String,
	},
	/// A hash format string no implementation matches.
	#[error("unknown hash format {0}")]
	HashFormat(String),
	#[error("failed to write {path}: {message}")]
	/// A filesystem failure while staging or committing.
	Io {
		/// The path being written.
		path: PathBuf,
		/// The underlying cause.
		message: String,
	},
	/// A request was built with no URL to try.
	#[error("request has no URL")]
	NoUrl,
}

impl NetError {
	/// The HTTP status, when this failure carried one.
	pub fn status(&self) -> Option<u16> {
		match self {
			Self::Http { status, .. } => *status,
			_ => None,
		}
	}

	pub(crate) fn io(path: &std::path::Path) -> impl FnOnce(std::io::Error) -> Self + '_ {
		move |source| Self::Io {
			path: path.to_path_buf(),
			message: source.to_string(),
		}
	}
}
