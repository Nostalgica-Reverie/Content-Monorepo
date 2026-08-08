//! A per-pack cache of facts that depend only on one file's bytes.
//!
//! `refresh`, `content-lint`, `registry` and `preflight` all walk the same
//! pack and each re-read and re-hash every file on every invocation. Once
//! those loops were parallelized the remaining cost was simply that the work
//! was being redone: hashing 558 MB takes what it takes, whether or not the
//! bytes changed since the last run.
//!
//! So this stores what was computed, keyed on whether the file still looks
//! identical. A hit costs no `open()` and no `read()` — only the size and
//! mtime that a directory enumeration already returns.
//!
//! # What is cached, and what deliberately is not
//!
//! Only **per-file** facts: the hash, and whether the bytes are UTF-8 and
//! parse as JSON. Everything expensive in a content lint is *cross-file* —
//! case-collision detection needs the whole path set, duplicate detection
//! needs files grouped by size, reference validation needs the set of known
//! paths — so none of it can be keyed on a single file. Those analyses
//! recompute every run, which is cheap once no file has to be re-read.
//!
//! Caching a per-file *issue list* would look tempting and be wrong: a stale
//! entry would silently change a report, and a cache that can change output is
//! a correctness bug rather than an optimization.
//!
//! # Staleness
//!
//! An entry is trusted when size and modification time both match. That is the
//! same bet `make` has made for decades. It can be defeated — write the same
//! number of bytes within one filesystem timestamp tick and the change is
//! missed — so every command that consults the cache also offers a way to
//! bypass it, and the cache is safe to delete at any time.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

/// Where a pack's cache lives, relative to its root.
const CACHE_PATH: &str = ".packwand/cache.json";

/// Bumped when [`FileFacts`] changes shape, so an old cache is discarded
/// rather than deserialized into something that means something else now.
const SCHEMA_VERSION: u32 = 1;

/// What a file looked like, and what was computed from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileFacts {
    /// Size in bytes, part of the staleness key.
    pub size: u64,
    /// Modification time in nanoseconds since the epoch, part of the key.
    pub modified_ns: u128,
    /// Lowercase hex SHA-512 of the file's bytes.
    pub sha512: String,
    /// Whether the bytes decode as UTF-8.
    pub is_utf8: bool,
    /// Whether the bytes parse as JSON. `None` for files never parsed as JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json_parses: Option<bool>,
}

/// The on-disk form.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CacheDocument {
    #[serde(default)]
    schema_version: u32,
    /// Keyed by pack-relative, forward-slashed path so a cache written on one
    /// platform is still valid on another.
    #[serde(default)]
    files: BTreeMap<String, FileFacts>,
}

/// A pack's cache, loaded into memory.
#[derive(Debug)]
pub struct ContentCache {
    path: PathBuf,
    entries: BTreeMap<String, FileFacts>,
    /// Facts recorded during this run, written back on [`ContentCache::store`].
    fresh: BTreeMap<String, FileFacts>,
    enabled: bool,
    hits: usize,
    misses: usize,
}

impl ContentCache {
    /// Loads a pack's cache, or an empty one when absent, unreadable, or from
    /// an older schema. A cache is never a reason to fail: the worst outcome
    /// of ignoring it is doing the work.
    #[must_use]
    pub fn load(root: &Path) -> Self {
        let path = root.join(CACHE_PATH);
        let entries = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<CacheDocument>(&bytes).ok())
            .filter(|document| document.schema_version == SCHEMA_VERSION)
            .map(|document| document.files)
            .unwrap_or_default();
        Self {
            path,
            entries,
            fresh: BTreeMap::new(),
            enabled: true,
            hits: 0,
            misses: 0,
        }
    }

    /// A cache that never hits and never writes, for `--no-cache`.
    ///
    /// Deliberately a real object rather than an `Option`, so callers have one
    /// code path and cannot accidentally diverge cached and uncached results.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            path: PathBuf::new(),
            entries: BTreeMap::new(),
            fresh: BTreeMap::new(),
            enabled: false,
            hits: 0,
            misses: 0,
        }
    }

    /// The staleness key for a file: its size and modification time.
    ///
    /// Returns `None` when the file cannot be stat'd, which callers treat as
    /// an unconditional miss.
    #[must_use]
    pub fn key(path: &Path) -> Option<(u64, u128)> {
        let metadata = fs::metadata(path).ok()?;
        let modified = metadata
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos();
        Some((metadata.len(), modified))
    }

    /// Cached facts for `relative`, if the file still matches `key`.
    ///
    /// Split from [`ContentCache::facts`] so a caller can do the lookups
    /// sequentially — they are only `stat` calls — and then compute the misses
    /// in parallel, which is the shape every batch caller here wants.
    #[must_use]
    pub fn lookup(&self, relative: &str, key: (u64, u128)) -> Option<FileFacts> {
        if !self.enabled {
            return None;
        }
        self.fresh
            .get(relative)
            .or_else(|| self.entries.get(relative))
            .filter(|cached| cached.size == key.0 && cached.modified_ns == key.1)
            .cloned()
    }

    /// Records freshly computed facts so the next run can skip the file.
    pub fn record(&mut self, relative: &str, key: (u64, u128), mut facts: FileFacts) {
        if !self.enabled {
            return;
        }
        facts.size = key.0;
        facts.modified_ns = key.1;
        self.fresh.insert(relative.to_owned(), facts);
    }

    /// Counts a cache hit. [`ContentCache::lookup`] does not count on its own
    /// so a caller can probe without disturbing the statistics.
    pub fn note_hit_only(&mut self) {
        self.hits += 1;
    }

    /// Notes that a file had to be read.
    pub fn note_miss(&mut self) {
        self.misses += 1;
    }

    /// Facts for `path`, computed by `compute` on a miss.
    ///
    /// `relative` must be the pack-relative, forward-slashed path — it is the
    /// cache key, and a caller that passes an absolute path would produce a
    /// cache that never hits on another machine.
    pub fn facts<E>(
        &mut self,
        relative: &str,
        path: &Path,
        compute: impl FnOnce(&[u8]) -> Result<FileFacts, E>,
    ) -> Result<FileFacts, E>
    where
        E: From<std::io::Error>,
    {
        let stat = fs::metadata(path).ok();
        let key = stat.as_ref().and_then(|metadata| {
            let modified = metadata
                .modified()
                .ok()?
                .duration_since(UNIX_EPOCH)
                .ok()?
                .as_nanos();
            Some((metadata.len(), modified))
        });

        // `fresh` before `entries`: several analyses visit the same file in
        // one run — refresh hashes it, then a content lint reads it — and the
        // second visit should hit what the first just computed, not only what
        // a previous run left on disk.
        if self.enabled
            && let Some((size, modified_ns)) = key
            && let Some(cached) = self
                .fresh
                .get(relative)
                .or_else(|| self.entries.get(relative))
            && cached.size == size
            && cached.modified_ns == modified_ns
        {
            self.hits += 1;
            let cached = cached.clone();
            self.fresh.insert(relative.to_owned(), cached.clone());
            return Ok(cached);
        }

        self.misses += 1;
        let bytes = fs::read(path)?;
        let mut facts = compute(&bytes)?;
        if let Some((size, modified_ns)) = key {
            facts.size = size;
            facts.modified_ns = modified_ns;
            if self.enabled {
                self.fresh.insert(relative.to_owned(), facts.clone());
            }
        }
        Ok(facts)
    }

    /// Writes the facts gathered this run.
    ///
    /// Only paths seen this run are kept, so a cache cannot grow without bound
    /// as files are renamed or deleted. Failure is ignored: a cache that could
    /// not be written costs time on the next run and nothing else.
    pub fn store(&self) {
        if !self.enabled || self.fresh.is_empty() {
            return;
        }
        let Some(parent) = self.path.parent() else {
            return;
        };
        if fs::create_dir_all(parent).is_err() {
            return;
        }
        let document = CacheDocument {
            schema_version: SCHEMA_VERSION,
            files: self.fresh.clone(),
        };
        if let Ok(bytes) = serde_json::to_vec(&document) {
            // Written through a temporary so an interrupted run leaves the
            // previous cache intact rather than a truncated one. A corrupt
            // cache is recoverable (it is ignored on load) but a needless
            // full rebuild is still worth avoiding.
            let temporary = self.path.with_extension("tmp");
            if fs::write(&temporary, &bytes).is_ok() {
                let _ = fs::rename(&temporary, &self.path);
            }
        }
    }

    /// Cache hits and misses this run, for `--verbose` style reporting.
    #[must_use]
    pub fn stats(&self) -> (usize, usize) {
        (self.hits, self.misses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn facts_from(bytes: &[u8]) -> Result<FileFacts, io::Error> {
        Ok(FileFacts {
            size: 0,
            modified_ns: 0,
            sha512: packwand_pack::hash_bytes(packwand_pack::HashFormat::Sha512, bytes),
            is_utf8: std::str::from_utf8(bytes).is_ok(),
            json_parses: None,
        })
    }

    #[test]
    fn a_second_look_at_an_unchanged_file_hits() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, b"hello").unwrap();

        let mut cache = ContentCache::load(dir.path());
        let first = cache.facts("a.txt", &file, facts_from).unwrap();
        let second = cache
            .facts("a.txt", &file, |_| -> Result<FileFacts, io::Error> {
                panic!("a cache hit must not read or recompute")
            })
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(cache.stats(), (1, 1));
    }

    #[test]
    fn the_cache_survives_a_round_trip_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, b"hello").unwrap();

        let mut first = ContentCache::load(dir.path());
        let facts = first.facts("a.txt", &file, facts_from).unwrap();
        first.store();

        let mut second = ContentCache::load(dir.path());
        let reloaded = second
            .facts("a.txt", &file, |_| -> Result<FileFacts, io::Error> {
                panic!("a warm cache must not recompute")
            })
            .unwrap();
        assert_eq!(facts, reloaded);
        assert_eq!(second.stats(), (1, 0));
    }

    /// The property the whole design rests on: a changed file must miss.
    #[test]
    fn editing_a_file_invalidates_its_entry() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, b"hello").unwrap();
        let mut cache = ContentCache::load(dir.path());
        let before = cache.facts("a.txt", &file, facts_from).unwrap();
        cache.store();

        // A different length guarantees a different key even if the clock
        // granularity would have hidden a same-size edit.
        fs::write(&file, b"hello world").unwrap();
        let mut reopened = ContentCache::load(dir.path());
        let after = reopened.facts("a.txt", &file, facts_from).unwrap();
        assert_ne!(before.sha512, after.sha512);
        assert_eq!(reopened.stats(), (0, 1), "an edited file must miss");
    }

    #[test]
    fn a_disabled_cache_never_hits_and_never_writes() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, b"hello").unwrap();
        let mut cache = ContentCache::disabled();
        let first = cache.facts("a.txt", &file, facts_from).unwrap();
        let second = cache.facts("a.txt", &file, facts_from).unwrap();
        assert_eq!(first.sha512, second.sha512, "results must still be correct");
        assert_eq!(cache.stats(), (0, 2));
        cache.store();
        assert!(!dir.path().join(CACHE_PATH).exists());
    }

    /// A cache from an older schema must be ignored rather than misread.
    #[test]
    fn an_old_schema_is_discarded() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join(CACHE_PATH);
        fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        fs::write(
            &cache_path,
            br#"{"schemaVersion": 0, "files": {"a.txt": {}}}"#,
        )
        .unwrap();
        let cache = ContentCache::load(dir.path());
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn a_corrupt_cache_is_ignored_rather_than_fatal() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join(CACHE_PATH);
        fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        fs::write(&cache_path, b"this is not json").unwrap();
        let cache = ContentCache::load(dir.path());
        assert!(cache.entries.is_empty());
    }

    /// Entries for files that no longer exist must not accumulate forever.
    #[test]
    fn only_paths_seen_this_run_are_kept() {
        let dir = tempfile::tempdir().unwrap();
        let first_file = dir.path().join("a.txt");
        let second_file = dir.path().join("b.txt");
        fs::write(&first_file, b"a").unwrap();
        fs::write(&second_file, b"b").unwrap();

        let mut cache = ContentCache::load(dir.path());
        cache.facts("a.txt", &first_file, facts_from).unwrap();
        cache.facts("b.txt", &second_file, facts_from).unwrap();
        cache.store();

        // A later run that only looks at one of them drops the other.
        let mut later = ContentCache::load(dir.path());
        later.facts("a.txt", &first_file, facts_from).unwrap();
        later.store();

        let final_cache = ContentCache::load(dir.path());
        assert!(final_cache.entries.contains_key("a.txt"));
        assert!(!final_cache.entries.contains_key("b.txt"));
    }
}
