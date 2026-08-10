//! A loopback HTTP server for exercising the client against real sockets.
//!
//! Behind the `testing` feature, because the branching that matters here —
//! conditional requests, 304, `Retry-After`, mirror fallthrough — cannot be
//! reached from a unit test of a parser. Every packwand-net bug worth catching
//! lives between "build a request" and "hand back bytes", and that needs a
//! server that answers.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// How the stub should answer one path.
#[derive(Debug, Clone)]
pub enum Reply {
	/// 200 with these bytes, and the given validators.
	Ok {
		body: Vec<u8>,
		etag: Option<String>,
		last_modified: Option<String>,
		cache_control: Option<String>,
	},
	/// 304 when the request carries a matching `If-None-Match`, 200 otherwise.
	NotModifiedIfMatch { etag: String, body: Vec<u8> },
	/// A bare status with no body.
	Status(u16),
	/// 429 carrying `Retry-After: <seconds>` for the first `times` requests,
	/// then 200 with `body`.
	RetryAfter {
		times: usize,
		seconds: u64,
		body: Vec<u8>,
	},
}

impl Reply {
	/// 200 with a body and no caching headers.
	pub fn body(bytes: impl Into<Vec<u8>>) -> Self {
		Self::Ok {
			body: bytes.into(),
			etag: None,
			last_modified: None,
			cache_control: None,
		}
	}

	/// 200 with an `ETag` and a `max-age`.
	pub fn cacheable(bytes: impl Into<Vec<u8>>, etag: &str, max_age: u64) -> Self {
		Self::Ok {
			body: bytes.into(),
			etag: Some(etag.to_owned()),
			last_modified: None,
			cache_control: Some(format!("max-age={max_age}")),
		}
	}
}

/// A running loopback server. Shuts down when dropped.
pub struct StubServer {
	port: u16,
	hits: Arc<Mutex<HashMap<String, usize>>>,
	stop: Arc<AtomicUsize>,
	thread: Option<thread::JoinHandle<()>>,
}

impl StubServer {
	/// Starts a server answering `routes`, keyed by request path
	/// (`"/version_manifest.json"`). Unlisted paths get a 404.
	pub fn start(routes: impl IntoIterator<Item = (String, Reply)>) -> Self {
		let routes: HashMap<String, Reply> = routes.into_iter().collect();
		let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind a loopback port");
		let port = listener.local_addr().expect("local addr").port();
		listener
			.set_nonblocking(true)
			.expect("non-blocking listener");
		let hits: Arc<Mutex<HashMap<String, usize>>> = Arc::default();
		let stop = Arc::new(AtomicUsize::new(0));

		let served = hits.clone();
		let halt = stop.clone();
		let thread = thread::spawn(move || {
			let mut attempts: HashMap<String, usize> = HashMap::new();
			while halt.load(Ordering::Relaxed) == 0 {
				match listener.accept() {
					Ok((stream, _)) => {
						let _ = stream.set_nonblocking(false);
						handle(stream, &routes, &served, &mut attempts);
					}
					Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
						thread::sleep(std::time::Duration::from_millis(5));
					}
					Err(_) => break,
				}
			}
		});

		Self {
			port,
			hits,
			stop,
			thread: Some(thread),
		}
	}

	/// The base URL, without a trailing slash.
	pub fn base(&self) -> String {
		format!("http://127.0.0.1:{}", self.port)
	}

	/// A full URL for one path.
	pub fn url(&self, path: &str) -> String {
		format!("{}{path}", self.base())
	}

	/// How many requests reached `path`.
	pub fn hits(&self, path: &str) -> usize {
		self.hits
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner)
			.get(path)
			.copied()
			.unwrap_or(0)
	}
}

impl Drop for StubServer {
	fn drop(&mut self) {
		self.stop.store(1, Ordering::Relaxed);
		// Unblock the accept loop's next poll.
		let _ = TcpStream::connect(("127.0.0.1", self.port));
		if let Some(thread) = self.thread.take() {
			let _ = thread.join();
		}
	}
}

fn handle(
	mut stream: TcpStream,
	routes: &HashMap<String, Reply>,
	hits: &Arc<Mutex<HashMap<String, usize>>>,
	attempts: &mut HashMap<String, usize>,
) {
	let mut reader = BufReader::new(match stream.try_clone() {
		Ok(clone) => clone,
		Err(_) => return,
	});
	let mut request_line = String::new();
	if reader.read_line(&mut request_line).is_err() {
		return;
	}
	let path = request_line
		.split_whitespace()
		.nth(1)
		.unwrap_or("/")
		.to_owned();

	let mut if_none_match = None;
	loop {
		let mut line = String::new();
		match reader.read_line(&mut line) {
			Ok(0) => break,
			Ok(_) => {
				if line.trim().is_empty() {
					break;
				}
				if let Some(value) = line
					.strip_prefix("If-None-Match:")
					.or_else(|| line.strip_prefix("if-none-match:"))
				{
					if_none_match = Some(value.trim().to_owned());
				}
			}
			Err(_) => break,
		}
	}

	*hits
		.lock()
		.unwrap_or_else(std::sync::PoisonError::into_inner)
		.entry(path.clone())
		.or_insert(0) += 1;

	let response = match routes.get(&path) {
		None => reply(404, &[], &[]),
		Some(Reply::Status(code)) => reply(*code, &[], &[]),
		Some(Reply::Ok {
			body,
			etag,
			last_modified,
			cache_control,
		}) => {
			let mut headers = Vec::new();
			if let Some(etag) = etag {
				headers.push(format!("ETag: {etag}"));
			}
			if let Some(modified) = last_modified {
				headers.push(format!("Last-Modified: {modified}"));
			}
			if let Some(control) = cache_control {
				headers.push(format!("Cache-Control: {control}"));
			}
			reply(200, body, &headers)
		}
		Some(Reply::NotModifiedIfMatch { etag, body }) => {
			if if_none_match.as_deref() == Some(etag.as_str()) {
				reply(304, &[], &[format!("ETag: {etag}")])
			} else {
				reply(200, body, &[format!("ETag: {etag}")])
			}
		}
		Some(Reply::RetryAfter {
			times,
			seconds,
			body,
		}) => {
			let seen = attempts.entry(path.clone()).or_insert(0);
			*seen += 1;
			if *seen <= *times {
				reply(429, &[], &[format!("Retry-After: {seconds}")])
			} else {
				reply(200, body, &[])
			}
		}
	};
	let _ = stream.write_all(&response);
	let _ = stream.flush();
}

fn reply(status: u16, body: &[u8], headers: &[String]) -> Vec<u8> {
	let reason = match status {
		200 => "OK",
		304 => "Not Modified",
		404 => "Not Found",
		429 => "Too Many Requests",
		_ => "Error",
	};
	let mut out = format!("HTTP/1.1 {status} {reason}\r\n");
	for header in headers {
		out.push_str(header);
		out.push_str("\r\n");
	}
	// 304 must not carry a body, and a Content-Length on one confuses clients.
	if status != 304 {
		out.push_str(&format!("Content-Length: {}\r\n", body.len()));
	}
	out.push_str("Connection: close\r\n\r\n");
	let mut bytes = out.into_bytes();
	if status != 304 {
		bytes.extend_from_slice(body);
	}
	bytes
}
