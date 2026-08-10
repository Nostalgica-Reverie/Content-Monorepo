use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::NetError;
use crate::sink::staging_path;

/// How long an entry is trusted when the server gives no lifetime at all.
const DEFAULT_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;

fn now_seconds() -> u64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|elapsed| elapsed.as_secs())
		.unwrap_or(0)
}

/// What one cached response remembers, so the next request for it can be
/// skipped or revalidated instead of re-downloaded.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
	/// Path of the stored body, relative to the namespace directory.
	pub path: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	/// The `ETag` the server last sent.
	pub etag: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	/// The `Last-Modified` the server last sent, verbatim.
	pub last_modified: Option<String>,
	/// When this entry was stored, in epoch seconds.
	pub fetched_at: u64,
	/// How long after `fetched_at` the entry may be used without asking.
	pub max_age: u64,
	/// sha256 of the stored body.
	///
	/// What makes content-addressed invalidation possible: an index document
	/// usually publishes the digest of each document it points at, so
	/// comparing that against this says exactly which children changed —
	/// without a request per child, and without expiring the ones that did
	/// not. Optional because entries written before this existed have none,
	/// and a missing digest simply falls back to time-based freshness.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub sha256: Option<String>,
}

impl Entry {
	/// Whether this entry still needs to be checked with the server.
	pub fn is_fresh(&self, now: u64) -> bool {
		now.saturating_sub(self.fetched_at) < self.max_age
	}
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Index {
	#[serde(default)]
	entries: BTreeMap<String, Entry>,
}

/// A revalidating on-disk cache for metadata documents.
///
/// Modelled on Prism's `HttpMetaCache`, and there for the same reason: without
/// it every cold boot re-downloads Mojang's version manifest and each loader's
/// index in full, even though they change rarely. A fresh entry skips the
/// request entirely; a stale one is revalidated with `If-None-Match` /
/// `If-Modified-Since`, and a 304 costs one round trip instead of a payload.
///
/// Namespaced by "base" — `meta`, `libraries`, `assets`, `providers` — each
/// its own directory with its own index, so clearing one leaves the others.
pub struct MetaCache {
	root: PathBuf,
	index: Mutex<Index>,
	namespace: String,
}

impl MetaCache {
	/// Opens (or creates) the `namespace` cache under `root`.
	pub fn open(root: &Path, namespace: &str) -> Result<Self, NetError> {
		let directory = root.join(namespace);
		fs::create_dir_all(&directory).map_err(NetError::io(&directory))?;
		let index = match fs::read(directory.join("index.json")) {
			Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
			// A corrupt or missing index is not an error: the bodies are
			// content-addressed by key, so the worst case is refetching.
			Err(_) => Index::default(),
		};
		Ok(Self {
			root: directory,
			index: Mutex::new(index),
			namespace: namespace.to_owned(),
		})
	}

	/// The namespace this cache stores.
	pub fn namespace(&self) -> &str {
		&self.namespace
	}

	/// The entry for `key`, if one has been stored.
	pub fn entry(&self, key: &str) -> Option<Entry> {
		self.index
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner)
			.entries
			.get(key)
			.cloned()
	}

	/// The stored body for `key`, when the entry exists and its file is still
	/// readable.
	pub fn read(&self, key: &str) -> Option<Vec<u8>> {
		let entry = self.entry(key)?;
		fs::read(self.root.join(&entry.path)).ok()
	}

	/// The body for `key` only if it is still fresh — meaning no request has
	/// to be made at all.
	pub fn read_fresh(&self, key: &str) -> Option<Vec<u8>> {
		let entry = self.entry(key)?;
		entry
			.is_fresh(now_seconds())
			.then(|| fs::read(self.root.join(&entry.path)).ok())
			.flatten()
	}

	/// Stores `bytes` for `key` along with the validators to revalidate it.
	pub fn store(
		&self,
		key: &str,
		bytes: &[u8],
		etag: Option<String>,
		last_modified: Option<String>,
		max_age: Option<u64>,
	) -> Result<(), NetError> {
		let relative = file_name_for(key);
		let path = self.root.join(&relative);
		let staging = staging_path(&path);
		fs::write(&staging, bytes).map_err(NetError::io(&staging))?;
		if path.exists() {
			fs::remove_file(&path).map_err(NetError::io(&path))?;
		}
		fs::rename(&staging, &path).map_err(NetError::io(&path))?;

		let entry = Entry {
			path: relative,
			etag,
			last_modified,
			fetched_at: now_seconds(),
			max_age: max_age.unwrap_or(DEFAULT_MAX_AGE_SECONDS),
			sha256: Some(packwand_pack::hash_bytes(
				packwand_pack::HashFormat::Sha256,
				bytes,
			)),
		};
		let mut index = self
			.index
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner);
		index.entries.insert(key.to_owned(), entry);
		self.write_index(&index)
	}

	/// Marks `key` as checked just now without rewriting its body — what a 304
	/// response means.
	pub fn touch(&self, key: &str, max_age: Option<u64>) -> Result<(), NetError> {
		let mut index = self
			.index
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner);
		if let Some(entry) = index.entries.get_mut(key) {
			entry.fetched_at = now_seconds();
			if let Some(max_age) = max_age {
				entry.max_age = max_age;
			}
		}
		self.write_index(&index)
	}

	/// The digest of the cached body for `key`, in `format`.
	///
	/// The format is the caller's to choose because it is the *parent's*
	/// choice: Mojang's version manifest publishes sha1 for each version
	/// document, Modrinth publishes sha512, and a cache that only understood
	/// one of them could only ever invalidate half the tree. sha256 is
	/// answered from the index; anything else re-reads the body, which is a
	/// few kilobytes against the round trip it saves.
	pub fn digest(&self, key: &str, format: packwand_pack::HashFormat) -> Option<String> {
		let entry = self.entry(key)?;
		if format == packwand_pack::HashFormat::Sha256
			&& let Some(sha256) = entry.sha256
		{
			return Some(sha256);
		}
		let bytes = fs::read(self.root.join(&entry.path)).ok()?;
		Some(packwand_pack::hash_bytes(format, &bytes))
	}

	/// Whether the cached body for `key` already matches a known digest.
	///
	/// Lets a caller skip a request entirely on the strength of the parent's
	/// word, rather than revalidating to be told nothing changed.
	pub fn matches_digest(
		&self,
		key: &str,
		format: packwand_pack::HashFormat,
		expected: &str,
	) -> bool {
		self.digest(key, format)
			.is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
	}

	/// Applies the digests a parent document publishes for its children.
	///
	/// Returns the keys that were invalidated. This is the transitive part:
	/// refreshing one index tells you exactly which of the documents beneath
	/// it changed, so the unchanged ones stay usable no matter how old they
	/// are, and the changed ones are refetched even if they were nowhere near
	/// expiring. Time-based freshness cannot express either half of that.
	///
	/// A child the parent does not describe is left alone — it will be
	/// revalidated the ordinary way rather than discarded on no evidence.
	pub fn apply_child_digests<'a>(
		&self,
		format: packwand_pack::HashFormat,
		declared: impl IntoIterator<Item = (&'a str, &'a str)>,
	) -> Result<Vec<String>, NetError> {
		let stale: Vec<String> = declared
			.into_iter()
			.filter(|(key, expected)| {
				// Absent digest means absent evidence, not staleness.
				self.digest(key, format)
					.is_some_and(|actual| !actual.eq_ignore_ascii_case(expected))
			})
			.map(|(key, _)| key.to_owned())
			.collect();
		if stale.is_empty() {
			// Nothing changed, so nothing needs writing — the common case
			// once a machine has installed a version once.
			return Ok(stale);
		}
		let mut index = self
			.index
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner);
		for key in &stale {
			if let Some(removed) = index.entries.remove(key) {
				let _ = fs::remove_file(self.root.join(&removed.path));
			}
		}
		self.write_index(&index)?;
		Ok(stale)
	}

	fn write_index(&self, index: &Index) -> Result<(), NetError> {
		let path = self.root.join("index.json");
		let bytes = serde_json::to_vec_pretty(index).unwrap_or_else(|_| b"{}".to_vec());
		let staging = staging_path(&path);
		fs::write(&staging, bytes).map_err(NetError::io(&staging))?;
		if path.exists() {
			fs::remove_file(&path).map_err(NetError::io(&path))?;
		}
		fs::rename(&staging, &path).map_err(NetError::io(&path))
	}
}

/// A filesystem-safe file name for a cache key (usually a URL).
fn file_name_for(key: &str) -> String {
	let stem: String = key
		.chars()
		.map(|character| {
			if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
				character
			} else {
				'_'
			}
		})
		.collect();
	// Two long URLs can sanitize to the same stem, so the key's own digest
	// disambiguates them; the readable prefix is only there for humans
	// inspecting the cache directory.
	let digest = packwand_pack::hash_bytes(packwand_pack::HashFormat::Sha256, key.as_bytes());
	let prefix: String = stem
		.chars()
		.rev()
		.take(48)
		.collect::<Vec<_>>()
		.into_iter()
		.rev()
		.collect();
	format!("{prefix}.{}", &digest[..16])
}

/// Parses the lifetime a response claims: `Cache-Control: max-age` first, then
/// `Expires`, and `None` when it says nothing.
pub fn max_age_of(cache_control: Option<&str>, expires: Option<&str>) -> Option<u64> {
	if let Some(value) = cache_control {
		if value.split(',').any(|part| part.trim() == "no-store") {
			return Some(0);
		}
		for part in value.split(',') {
			if let Some(seconds) = part.trim().strip_prefix("max-age=")
				&& let Ok(seconds) = seconds.trim().parse::<u64>()
			{
				return Some(seconds);
			}
		}
	}
	let expires = crate::retry::parse_http_date(expires?)?;
	Some((expires - now_seconds() as i64).max(0) as u64)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_fresh_entry_reads_back_and_a_stale_one_does_not() {
		let root = tempfile::tempdir().unwrap();
		let cache = MetaCache::open(root.path(), "meta").unwrap();
		let key = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

		assert!(cache.read_fresh(key).is_none(), "nothing cached yet");
		cache
			.store(key, b"{}", Some("\"abc\"".into()), None, Some(600))
			.unwrap();
		assert_eq!(cache.read_fresh(key).as_deref(), Some(&b"{}"[..]));

		// Expired: the body is still there to revalidate against, but it is no
		// longer usable without asking.
		cache.store(key, b"{}", None, None, Some(0)).unwrap();
		assert!(cache.read_fresh(key).is_none());
		assert_eq!(cache.read(key).as_deref(), Some(&b"{}"[..]));
	}

	#[test]
	fn entries_survive_reopening() {
		let root = tempfile::tempdir().unwrap();
		let key = "https://example.invalid/a.json";
		MetaCache::open(root.path(), "meta")
			.unwrap()
			.store(key, b"payload", Some("\"e\"".into()), None, Some(600))
			.unwrap();

		let reopened = MetaCache::open(root.path(), "meta").unwrap();
		assert_eq!(reopened.read_fresh(key).as_deref(), Some(&b"payload"[..]));
		assert_eq!(reopened.entry(key).unwrap().etag.as_deref(), Some("\"e\""));
	}

	#[test]
	fn keys_that_sanitize_alike_still_get_their_own_file() {
		assert_ne!(
			file_name_for("https://example.invalid/a/b"),
			file_name_for("https://example.invalid/a_b")
		);
	}

	#[test]
	fn a_parent_digest_invalidates_only_the_children_that_changed() {
		// The property time-based expiry cannot express: refreshing an index
		// says exactly which documents beneath it moved. The others stay
		// usable however old they are, and the changed one goes even though
		// it was nowhere near expiring.
		let root = tempfile::tempdir().unwrap();
		let cache = MetaCache::open(root.path(), "meta").unwrap();
		let unchanged = "https://example.invalid/1.21.1.json";
		let changed = "https://example.invalid/1.21.4.json";
		cache
			.store(unchanged, b"one", None, None, Some(99999))
			.unwrap();
		cache
			.store(changed, b"two", None, None, Some(99999))
			.unwrap();

		// sha1 deliberately: it is what Mojang's version manifest publishes,
		// and a cache that only spoke sha256 could not use it.
		let sha1 = packwand_pack::HashFormat::Sha1;
		let digest = |bytes: &[u8]| packwand_pack::hash_bytes(sha1, bytes);
		let invalidated = cache
			.apply_child_digests(
				sha1,
				[
					(unchanged, digest(b"one").as_str()),
					(changed, digest(b"different now").as_str()),
				],
			)
			.unwrap();

		assert_eq!(invalidated, [changed]);
		assert_eq!(cache.read_fresh(unchanged).as_deref(), Some(&b"one"[..]));
		assert!(cache.read(changed).is_none(), "the changed child survived");
	}

	#[test]
	fn a_matching_digest_needs_no_request_at_all() {
		let root = tempfile::tempdir().unwrap();
		let cache = MetaCache::open(root.path(), "meta").unwrap();
		let key = "https://example.invalid/a.json";
		cache.store(key, b"body", None, None, Some(600)).unwrap();
		for format in [
			packwand_pack::HashFormat::Sha1,
			packwand_pack::HashFormat::Sha256,
			packwand_pack::HashFormat::Sha512,
		] {
			let sha = packwand_pack::hash_bytes(format, b"body");
			assert!(cache.matches_digest(key, format, &sha), "{format:?}");
			// Case is not meaningful in a hex digest, and sources disagree.
			assert!(cache.matches_digest(key, format, &sha.to_uppercase()));
			assert!(!cache.matches_digest(key, format, "0000"));
			assert!(!cache.matches_digest("https://example.invalid/absent", format, &sha));
		}
	}

	#[test]
	fn an_unknown_child_is_left_alone_rather_than_discarded() {
		// No evidence is not evidence of staleness: a child the parent does
		// not describe must keep revalidating the ordinary way.
		let root = tempfile::tempdir().unwrap();
		let cache = MetaCache::open(root.path(), "meta").unwrap();
		let key = "https://example.invalid/known.json";
		cache.store(key, b"body", None, None, Some(600)).unwrap();
		let invalidated = cache
			.apply_child_digests(
				packwand_pack::HashFormat::Sha1,
				[("https://example.invalid/never-seen.json", "abc")],
			)
			.unwrap();
		assert!(invalidated.is_empty());
		assert_eq!(cache.read_fresh(key).as_deref(), Some(&b"body"[..]));
	}

	#[test]
	fn lifetimes_prefer_cache_control_then_expires() {
		assert_eq!(max_age_of(Some("public, max-age=300"), None), Some(300));
		assert_eq!(max_age_of(Some("no-store"), None), Some(0));
		assert_eq!(max_age_of(None, None), None);
		assert_eq!(max_age_of(Some("public"), None), None);
		// A past Expires clamps to zero rather than going negative.
		assert_eq!(
			max_age_of(None, Some("Sun, 06 Nov 1994 08:49:37 GMT")),
			Some(0)
		);
	}
}
