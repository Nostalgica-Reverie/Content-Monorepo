use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use packwand_pack::Hasher;

use crate::NetError;
use crate::request::Checksum;

/// A staging path for `target` that no concurrent write can collide with.
///
/// The obvious `with_extension("pw-part")` is wrong twice over once these run
/// in parallel: it *replaces* the extension, so `x.json` and `x.toml` stage to
/// the same `x.pw-part`; and one content hash can back two entries of a single
/// plan. Both produce silently wrong bytes, because verification happens
/// before the rename and the file is never read back. Appending, plus a
/// per-process ticket, removes the collision by construction.
pub fn staging_path(target: &Path) -> PathBuf {
	static NEXT: AtomicU64 = AtomicU64::new(0);
	let ticket = NEXT.fetch_add(1, Ordering::Relaxed);
	let name = target
		.file_name()
		.map(|name| name.to_string_lossy().into_owned())
		.unwrap_or_default();
	target.with_file_name(format!("{name}.{ticket}.pw-part"))
}

/// Accumulates a response into a staging file while hashing it, and moves it
/// onto the target only once it verifies.
///
/// This is the property the whole crate is built around, and Prism's sink
/// chain has it too: **nothing unverified is ever visible at the target
/// path.** A truncated or corrupted transfer leaves a `.pw-part` orphan, never
/// a plausible-looking file that a later run would trust.
pub struct FileSink {
	target: PathBuf,
	staging: PathBuf,
	file: fs::File,
	hasher: Option<Hasher>,
	written: u64,
}

impl FileSink {
	/// Opens a sink writing towards `target`, hashing as it goes when a
	/// checksum is required.
	pub fn create(target: &Path, checksum: Option<&Checksum>) -> Result<Self, NetError> {
		if let Some(parent) = target.parent() {
			fs::create_dir_all(parent).map_err(NetError::io(parent))?;
		}
		let staging = staging_path(target);
		let file = fs::File::create(&staging).map_err(NetError::io(&staging))?;
		Ok(Self {
			target: target.to_path_buf(),
			staging,
			file,
			hasher: checksum.map(|checksum| Hasher::new(checksum.format)),
			written: 0,
		})
	}

	/// Feeds one chunk through the hasher and out to the staging file.
	pub fn write(&mut self, chunk: &[u8]) -> Result<(), NetError> {
		if let Some(hasher) = &mut self.hasher {
			hasher.update(chunk);
		}
		self.file
			.write_all(chunk)
			.map_err(NetError::io(&self.staging))?;
		self.written += chunk.len() as u64;
		Ok(())
	}

	/// How many bytes have been written so far.
	pub fn written(&self) -> u64 {
		self.written
	}

	/// Verifies the digest and renames the staging file onto the target.
	///
	/// On mismatch the staging file is removed and the target is left exactly
	/// as it was.
	pub fn commit(mut self, url: &str, checksum: Option<&Checksum>) -> Result<u64, NetError> {
		self.file.flush().map_err(NetError::io(&self.staging))?;
		drop(self.file);

		if let (Some(hasher), Some(checksum)) = (self.hasher.take(), checksum) {
			let actual = hasher.finish();
			if !actual.eq_ignore_ascii_case(&checksum.expected) {
				let _ = fs::remove_file(&self.staging);
				return Err(NetError::Checksum {
					url: url.to_owned(),
					expected: checksum.expected.clone(),
					actual,
				});
			}
		}
		// Windows rename fails onto an existing file, and the target exists
		// here whenever a previous version was wrong or stale.
		if self.target.exists() {
			fs::remove_file(&self.target).map_err(NetError::io(&self.target))?;
		}
		fs::rename(&self.staging, &self.target).map_err(NetError::io(&self.target))?;
		Ok(self.written)
	}

	/// Discards the transfer, leaving nothing behind.
	pub fn abort(self) {
		drop(self.file);
		let _ = fs::remove_file(&self.staging);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use packwand_pack::{HashFormat, hash_bytes};

	#[test]
	fn staging_paths_append_and_never_repeat() {
		let a = staging_path(Path::new("config/shared.json"));
		let b = staging_path(Path::new("config/shared.toml"));
		let c = staging_path(Path::new("config/shared.json"));
		assert_ne!(a, b);
		assert_ne!(a, c, "two writers to one target still get their own file");
		for path in [&a, &b, &c] {
			let name = path.file_name().unwrap().to_string_lossy();
			assert!(name.ends_with(".pw-part"), "{name}");
			assert!(name.starts_with("shared."), "{name}");
		}
	}

	#[test]
	fn a_bad_digest_leaves_the_target_untouched_and_no_orphan() {
		let root = tempfile::tempdir().unwrap();
		let target = root.path().join("mods/a.jar");
		fs::create_dir_all(target.parent().unwrap()).unwrap();
		fs::write(&target, b"original").unwrap();

		let checksum =
			Checksum::parse("sha256", hash_bytes(HashFormat::Sha256, b"expected")).unwrap();
		let mut sink = FileSink::create(&target, Some(&checksum)).unwrap();
		sink.write(b"something else").unwrap();
		let error = sink
			.commit("https://example.invalid/a.jar", Some(&checksum))
			.unwrap_err();

		assert!(matches!(error, NetError::Checksum { .. }));
		assert_eq!(fs::read(&target).unwrap(), b"original");
		let strays: Vec<_> = fs::read_dir(target.parent().unwrap())
			.unwrap()
			.filter_map(Result::ok)
			.map(|entry| entry.file_name().to_string_lossy().into_owned())
			.filter(|name| name.contains("pw-part"))
			.collect();
		assert!(strays.is_empty(), "left behind {strays:?}");
	}

	#[test]
	fn a_good_digest_commits_the_streamed_bytes() {
		let root = tempfile::tempdir().unwrap();
		let target = root.path().join("deep/nested/a.bin");
		let body = b"streamed in pieces";
		let checksum = Checksum::parse("sha512", hash_bytes(HashFormat::Sha512, body)).unwrap();

		let mut sink = FileSink::create(&target, Some(&checksum)).unwrap();
		for chunk in body.chunks(4) {
			sink.write(chunk).unwrap();
		}
		let written = sink
			.commit("https://example.invalid/a.bin", Some(&checksum))
			.unwrap();

		assert_eq!(written, body.len() as u64);
		assert_eq!(fs::read(&target).unwrap(), body);
	}
}
