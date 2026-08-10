//! A one-shot local HTTP listener that captures an OAuth redirect's query
//! string, then closes. RFC 8252 ("OAuth 2.0 for Native Apps") recommends
//! exactly this loopback-redirect pattern over an embedded webview for
//! credential entry — the user signs in in their own trusted default
//! browser, not a window this app controls.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

use crate::MsaError;

pub struct Loopback {
	listener: TcpListener,
}

impl Loopback {
	/// Binds an OS-assigned ephemeral local port.
	pub fn bind() -> Result<(Self, u16), MsaError> {
		let listener = TcpListener::bind("127.0.0.1:0")
			.map_err(|e| MsaError::Other(format!("failed to bind a local port: {e}")))?;
		let port = listener
			.local_addr()
			.map_err(|e| MsaError::Other(e.to_string()))?
			.port();
		Ok((Self { listener }, port))
	}

	/// Blocks for the one browser redirect and returns its raw query
	/// string. Times out rather than hanging forever if the user never
	/// completes the browser flow.
	pub fn await_query(&self, timeout: Duration) -> Result<String, MsaError> {
		self.listener
			.set_nonblocking(true)
			.map_err(|e| MsaError::Other(e.to_string()))?;
		let deadline = Instant::now() + timeout;
		loop {
			match self.listener.accept() {
				Ok((stream, _)) => return handle_connection(stream),
				Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
					if Instant::now() >= deadline {
						return Err(MsaError::Other(
							"timed out waiting for the browser sign-in to complete".to_string(),
						));
					}
					std::thread::sleep(Duration::from_millis(100));
				}
				Err(e) => return Err(MsaError::Other(e.to_string())),
			}
		}
	}
}

fn handle_connection(mut stream: TcpStream) -> Result<String, MsaError> {
	let mut reader = BufReader::new(
		stream
			.try_clone()
			.map_err(|e| MsaError::Other(e.to_string()))?,
	);
	let mut request_line = String::new();
	reader
		.read_line(&mut request_line)
		.map_err(|e| MsaError::Other(e.to_string()))?;
	// Expect "GET /?code=...&state=... HTTP/1.1".
	let path = request_line
		.split_whitespace()
		.nth(1)
		.ok_or_else(|| MsaError::Other("malformed redirect request".to_string()))?;
	let query = path
		.split_once('?')
		.map(|(_, q)| q)
		.unwrap_or("")
		.to_string();

	let body = "<html><body><p>Signed in - you can close this tab and return to Packwand.</p></body></html>";
	let response = format!(
		"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
		body.len(),
		body
	);
	let _ = stream.write_all(response.as_bytes());
	Ok(query)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Read;
	use std::net::TcpStream as ClientStream;

	#[test]
	fn captures_query_string_and_responds_ok() {
		let (loopback, port) = Loopback::bind().unwrap();
		let handle = std::thread::spawn(move || loopback.await_query(Duration::from_secs(5)));

		let mut client = ClientStream::connect(("127.0.0.1", port)).unwrap();
		client
			.write_all(b"GET /?code=abc123&state=xyz HTTP/1.1\r\nHost: localhost\r\n\r\n")
			.unwrap();
		let mut response = String::new();
		client.read_to_string(&mut response).ok();

		let query = handle.join().unwrap().unwrap();
		assert_eq!(query, "code=abc123&state=xyz");
		assert!(response.starts_with("HTTP/1.1 200 OK"));
	}

	#[test]
	fn times_out_if_nothing_connects() {
		let (loopback, _port) = Loopback::bind().unwrap();
		let err = loopback
			.await_query(Duration::from_millis(200))
			.unwrap_err();
		assert!(matches!(err, MsaError::Other(msg) if msg.contains("timed out")));
	}
}
