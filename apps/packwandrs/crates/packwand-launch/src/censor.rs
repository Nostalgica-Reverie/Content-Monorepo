//! Removing secrets from anything on its way to a human.
//!
//! The launcher knows a small number of values that must never be displayed:
//! the access token, the session id, and the account uuid. It cannot control
//! whether they *appear* — the game echoes its own arguments on crash, mods
//! print what they like, and a stack trace can carry anything — so the only
//! reliable place to remove them is the boundary every line crosses on its
//! way out.
//!
//! Applied centrally, before a line reaches an event, a file, or a paste
//! site. A filter applied at the UI instead would be one that the next
//! consumer forgets to apply.

use std::collections::BTreeMap;

use packwand_auth::SecretString;

/// Replacement written in place of a censored value.
const REDACTED: &str = "«redacted»";

/// Shortest value worth censoring.
///
/// Substring replacement over a very short value would corrupt unrelated
/// text: a uuid is long and specific, but an offline session id can be a
/// handful of characters, and blanking every occurrence of those would mangle
/// ordinary log lines while protecting nothing worth protecting.
const MIN_CENSORED_LEN: usize = 8;

/// Values to strip from log output.
///
/// Built once per launch from the same session that supplies the spawn
/// values, so nothing can be censored here that was not also sent to the
/// game — and nothing sent to the game is missed.
#[derive(Debug, Clone, Default)]
pub struct Censor {
	/// Longest first, so a value that contains another is replaced whole
	/// rather than leaving a fragment behind.
	values: Vec<String>,
}

impl Censor {
	/// A censor that redacts nothing.
	pub fn new() -> Self {
		Self::default()
	}

	/// Builds a censor from the values a launch resolves at spawn.
	///
	/// Identity values are included as well as secrets: a uuid is not a
	/// credential, but it identifies the account to anyone reading a pasted
	/// log, and Prism censors it for that reason.
	pub fn for_launch(
		secrets: &BTreeMap<String, SecretString>,
		identity: &BTreeMap<String, String>,
	) -> Self {
		let mut censor = Self::new();
		for value in secrets.values() {
			censor.add(value.expose());
		}
		for (name, value) in identity {
			// The player name is the one identity value that is meant to be
			// readable: it appears in chat, in the window title, and in the
			// log line a user is asked to paste.
			if name != "auth_player_name" {
				censor.add(value);
			}
		}
		censor
	}

	/// Adds one value, ignoring anything too short to censor safely.
	pub fn add(&mut self, value: &str) {
		let value = value.trim();
		if value.len() < MIN_CENSORED_LEN || self.values.iter().any(|v| v == value) {
			return;
		}
		self.values.push(value.to_string());
		self.values.sort_by_key(|v| std::cmp::Reverse(v.len()));
	}

	/// Whether this censor would change anything.
	pub fn is_empty(&self) -> bool {
		self.values.is_empty()
	}

	/// Replaces every known secret in `line`.
	///
	/// One pass over the input, for the reason `substitute_secrets` is one
	/// pass: scanning text that has already been written back is how a
	/// replacement becomes a new match. Here the risk is the mirror image —
	/// re-scanning could see a secret spanning the boundary between the
	/// replacement marker and surrounding text — and the same rule removes
	/// it. Everything already emitted is final.
	pub fn censor(&self, line: &str) -> String {
		if self.values.is_empty() {
			return line.to_string();
		}
		let mut out = String::with_capacity(line.len());
		let mut rest = line;
		'outer: while !rest.is_empty() {
			// Longest values first: the ordering is what stops a shorter
			// secret matching inside a longer one and leaving a tail.
			for value in &self.values {
				if rest.starts_with(value.as_str()) {
					out.push_str(REDACTED);
					rest = &rest[value.len()..];
					continue 'outer;
				}
			}
			// No secret starts here; copy one character and move on.
			let mut chars = rest.chars();
			let Some(ch) = chars.next() else { break };
			out.push(ch);
			rest = chars.as_str();
		}
		out
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn censor_of(values: &[&str]) -> Censor {
		let mut censor = Censor::new();
		for value in values {
			censor.add(value);
		}
		censor
	}

	#[test]
	fn a_token_echoed_by_the_game_is_removed() {
		let censor = censor_of(&["eyJhbGciOiJIUzI1NiJ9.super.secret"]);
		let line = "[main/INFO]: Setting user: args --accessToken eyJhbGciOiJIUzI1NiJ9.super.secret --uuid";
		let out = censor.censor(line);
		assert!(!out.contains("eyJhbGciOiJIUzI1NiJ9"), "{out}");
		assert!(out.contains(REDACTED));
		assert!(out.starts_with("[main/INFO]: Setting user:"));
		assert!(out.ends_with("--uuid"));
	}

	#[test]
	fn a_value_appearing_twice_is_removed_both_times() {
		let censor = censor_of(&["0123456789abcdef"]);
		let out = censor.censor("a=0123456789abcdef b=0123456789abcdef");
		assert_eq!(out, format!("a={REDACTED} b={REDACTED}"));
	}

	#[test]
	fn a_longer_secret_wins_over_one_contained_in_it() {
		// Ordering matters: censoring the short value first would leave the
		// rest of the long one exposed in the output.
		let censor = censor_of(&["token-abcdefgh", "token-abcdefgh-extended-suffix"]);
		let out = censor.censor("value=token-abcdefgh-extended-suffix.");
		assert_eq!(out, format!("value={REDACTED}."));
	}

	#[test]
	fn very_short_values_are_left_alone() {
		// Blanking every occurrence of a 3-character session id would mangle
		// ordinary text and protect nothing.
		let censor = censor_of(&["abc"]);
		assert!(censor.is_empty());
		assert_eq!(censor.censor("abc def abc"), "abc def abc");
	}

	#[test]
	fn censoring_never_produces_a_new_secret() {
		// The single-pass rule: output already written is never re-examined,
		// so a replacement cannot combine with its surroundings into another
		// match.
		let censor = censor_of(&["secretvalue1", "«redacted»secretvalue2"]);
		let out = censor.censor("x secretvalue1 y");
		assert_eq!(out, format!("x {REDACTED} y"));
		assert!(!out.contains("secretvalue2"));
	}

	#[test]
	fn a_player_name_stays_readable_but_the_uuid_does_not() {
		let secrets = BTreeMap::from([(
			"auth_access_token".to_string(),
			SecretString::new("tok-0123456789abcdef"),
		)]);
		let identity = BTreeMap::from([
			("auth_player_name".to_string(), "Notch".to_string()),
			(
				"auth_uuid".to_string(),
				"069a79f4-44e9-4726-a5be-fca90e38aaf5".to_string(),
			),
		]);
		let censor = Censor::for_launch(&secrets, &identity);
		let out = censor.censor(
			"Setting user: Notch (069a79f4-44e9-4726-a5be-fca90e38aaf5) tok-0123456789abcdef",
		);
		assert!(out.contains("Notch"), "the player name should stay: {out}");
		assert!(!out.contains("069a79f4"), "{out}");
		assert!(!out.contains("tok-0123456789abcdef"), "{out}");
	}

	#[test]
	fn multibyte_text_is_not_split() {
		// The scan advances by characters, not bytes: a panic here would take
		// down the log reader thread for any line containing non-ASCII.
		let censor = censor_of(&["0123456789abcdef"]);
		let out = censor.censor("日本語 0123456789abcdef ünïcödé");
		assert_eq!(out, format!("日本語 {REDACTED} ünïcödé"));
	}
}
