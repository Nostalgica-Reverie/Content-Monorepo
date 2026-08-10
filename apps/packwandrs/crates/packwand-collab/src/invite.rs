//! The invite string: where to connect, and the key to connect with.
//!
//! Format: `pw://<host>:<port>#<base64url psk>`.
//!
//! The pre-shared key lives in the fragment rather than the path or a query
//! parameter on purpose. A fragment is the part of a URL that conventionally
//! never leaves the client — it is not sent to servers and not written to
//! proxy logs — and while nothing here speaks HTTP, an invite is a string
//! users paste into chat windows and issue trackers, so the shape that is
//! least likely to be logged somewhere is the right one to pick.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// Bytes in a pre-shared key. Fixed by `Noise_NNpsk0`, which takes exactly 32.
pub const PSK_BYTES: usize = 32;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InviteError {
	#[error("an invite must start with pw://")]
	Scheme,
	#[error("an invite must contain a #key fragment")]
	MissingKey,
	#[error("the invite key is not valid base64url")]
	KeyEncoding,
	#[error("the invite key must be {PSK_BYTES} bytes, got {0}")]
	KeyLength(usize),
	#[error("the invite address is malformed")]
	Address,
	#[error("the invite port is not a number between 1 and 65535")]
	Port,
}

/// A parsed invite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invite {
	pub host: String,
	pub port: u16,
	pub psk: [u8; PSK_BYTES],
}

impl Invite {
	/// Generates an invite for a bound address, with a fresh random key.
	pub fn generate(host: impl Into<String>, port: u16) -> Self {
		use rand::RngCore;
		let mut psk = [0u8; PSK_BYTES];
		rand::rng().fill_bytes(&mut psk);
		Self {
			host: host.into(),
			port,
			psk,
		}
	}

	/// Renders the invite string a user copies.
	pub fn render(&self) -> String {
		format!(
			"pw://{}:{}#{}",
			self.host,
			self.port,
			URL_SAFE_NO_PAD.encode(self.psk)
		)
	}

	/// Parses an invite string.
	///
	/// Deliberately strict. Every rejection here is a case where continuing
	/// would mean dialling somewhere unintended or handshaking with a key of
	/// the wrong size, and a clear refusal beats a confusing connection
	/// failure several seconds later.
	pub fn parse(value: &str) -> Result<Self, InviteError> {
		let value = value.trim();
		let rest = value.strip_prefix("pw://").ok_or(InviteError::Scheme)?;
		let (address, key) = rest.split_once('#').ok_or(InviteError::MissingKey)?;
		if key.is_empty() {
			return Err(InviteError::MissingKey);
		}

		let decoded = URL_SAFE_NO_PAD
			.decode(key)
			.map_err(|_| InviteError::KeyEncoding)?;
		if decoded.len() != PSK_BYTES {
			return Err(InviteError::KeyLength(decoded.len()));
		}
		let mut psk = [0u8; PSK_BYTES];
		psk.copy_from_slice(&decoded);

		// `rsplit_once` rather than `split_once`: an IPv6 literal is full of
		// colons and only the last one separates the port.
		let (host, port) = address.rsplit_once(':').ok_or(InviteError::Address)?;
		let host = host.trim_matches(['[', ']']);
		if host.is_empty() {
			return Err(InviteError::Address);
		}
		let port: u16 = port.parse().map_err(|_| InviteError::Port)?;
		if port == 0 {
			return Err(InviteError::Port);
		}

		Ok(Self {
			host: host.to_owned(),
			port,
			psk,
		})
	}
}

/// Never print a key, even by accident.
impl std::fmt::Display for Invite {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(formatter, "pw://{}:{}#«redacted»", self.host, self.port)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn an_invite_round_trips() {
		let invite = Invite::generate("127.0.0.1", 41234);
		let parsed = Invite::parse(&invite.render()).unwrap();
		assert_eq!(parsed, invite);
	}

	#[test]
	fn generated_keys_differ() {
		let first = Invite::generate("127.0.0.1", 1);
		let second = Invite::generate("127.0.0.1", 1);
		assert_ne!(first.psk, second.psk, "keys must not repeat");
	}

	#[test]
	fn an_ipv6_literal_keeps_its_colons() {
		let invite = Invite {
			host: "::1".into(),
			port: 41234,
			psk: [3u8; PSK_BYTES],
		};
		let parsed =
			Invite::parse("pw://[::1]:41234#AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM").unwrap();
		assert_eq!(parsed, invite);
	}

	#[test]
	fn malformed_invites_are_refused() {
		let key = URL_SAFE_NO_PAD.encode([0u8; PSK_BYTES]);
		for (input, expected) in [
			("http://host:1#abc".to_owned(), InviteError::Scheme),
			("".to_owned(), InviteError::Scheme),
			(format!("pw://host:1{key}"), InviteError::MissingKey),
			("pw://host:1#".to_owned(), InviteError::MissingKey),
			(
				"pw://host:1#not base64!".to_owned(),
				InviteError::KeyEncoding,
			),
			(
				format!("pw://host:1#{}", URL_SAFE_NO_PAD.encode([0u8; 8])),
				InviteError::KeyLength(8),
			),
			(format!("pw://host#{key}"), InviteError::Address),
			(format!("pw://:1234#{key}"), InviteError::Address),
			(format!("pw://host:0#{key}"), InviteError::Port),
			(format!("pw://host:70000#{key}"), InviteError::Port),
			(format!("pw://host:abc#{key}"), InviteError::Port),
		] {
			assert_eq!(
				Invite::parse(&input).unwrap_err(),
				expected,
				"input {input:?}"
			);
		}
	}

	/// An invite is pasted into chat windows; the Display impl must not be the
	/// thing that leaks the key into a log line.
	#[test]
	fn display_redacts_the_key() {
		let shown = Invite::generate("127.0.0.1", 41234).to_string();
		assert!(shown.contains("«redacted»"), "{shown}");
		assert!(!shown.contains('='), "{shown}");
	}
}
