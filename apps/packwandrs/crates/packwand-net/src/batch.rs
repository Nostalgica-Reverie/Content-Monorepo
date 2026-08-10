use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use packwand_parallel::Jobs;

use crate::client::Client;
use crate::error::NetError;
use crate::request::{Checksum, Request};

/// One file a batch should end up with.
#[derive(Debug, Clone)]
pub struct Download {
	/// Where to fetch it from, mirrors included.
	pub request: Request,
	/// Where it goes.
	pub target: PathBuf,
	/// What it must hash to, when the source publishes a digest.
	pub checksum: Option<Checksum>,
	/// The declared size, used for a total before any transfer starts.
	pub size: Option<u64>,
}

/// A snapshot of a batch's progress.
///
/// Shaped after Prism's `TaskStepProgress`: a running total that a UI can
/// render directly, rather than something the caller has to aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchProgress {
	/// Items finished, whether transferred or already satisfied.
	pub finished: usize,
	/// Items in the batch.
	pub total: usize,
	/// Bytes accounted for so far.
	pub bytes: u64,
	/// Total bytes, when every item declared a size.
	pub total_bytes: Option<u64>,
}

/// What a batch did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BatchReport {
	/// Items actually transferred.
	pub downloaded: usize,
	/// Items already present and verified.
	pub skipped: usize,
	/// Bytes transferred.
	pub bytes: u64,
}

/// Fetches every item, up to `jobs` at a time, skipping those already on disk
/// and verified.
///
/// Failures do not cancel peers already in flight; the first error is returned
/// once the batch drains. Anything that did complete stays on disk, so a retry
/// resumes rather than starting over.
pub fn download_all(
	client: &Client,
	items: &[Download],
	jobs: Jobs,
	progress: &(dyn Fn(BatchProgress) + Sync),
) -> Result<BatchReport, NetError> {
	let total = items.len();
	let total_bytes = items.iter().map(|item| item.size).sum::<Option<u64>>();
	let finished = AtomicUsize::new(0);
	let downloaded = AtomicUsize::new(0);
	let skipped = AtomicUsize::new(0);
	let bytes = AtomicU64::new(0);
	let errors: Mutex<Vec<NetError>> = Mutex::new(Vec::new());

	let report_progress = |added: u64| {
		let accounted = bytes.fetch_add(added, Ordering::Relaxed) + added;
		progress(BatchProgress {
			finished: finished.load(Ordering::Relaxed),
			total,
			bytes: accounted,
			total_bytes,
		});
	};

	packwand_parallel::for_each(items, jobs, |item| {
		if satisfied(item) {
			skipped.fetch_add(1, Ordering::Relaxed);
			finished.fetch_add(1, Ordering::Relaxed);
			report_progress(item.size.unwrap_or_else(|| on_disk_len(item)));
			return;
		}
		let mut last = 0u64;
		let outcome = client.download_to(
			&item.request,
			&item.target,
			item.checksum.as_ref(),
			&mut |current, _| {
				let delta = current.saturating_sub(last);
				last = current;
				report_progress(delta);
			},
		);
		match outcome.and_then(|written| check_size(item, written)) {
			Ok(()) => {
				downloaded.fetch_add(1, Ordering::Relaxed);
				finished.fetch_add(1, Ordering::Relaxed);
				// A zero-byte tick after the counter moves, so the caller's
				// last update carries the final count. Without it the run
				// ends reporting fewer completions than it made, because the
				// transfer's own last callback fires before this increment.
				report_progress(0);
			}
			Err(error) => errors
				.lock()
				.unwrap_or_else(std::sync::PoisonError::into_inner)
				.push(error),
		}
	});

	if let Some(error) = errors
		.into_inner()
		.unwrap_or_else(std::sync::PoisonError::into_inner)
		.pop()
	{
		return Err(error);
	}
	Ok(BatchReport {
		downloaded: downloaded.into_inner(),
		skipped: skipped.into_inner(),
		bytes: bytes.into_inner(),
	})
}

/// A declared size that disagrees with what arrived means the metadata is
/// inconsistent, not that the transfer broke — the digest already matched if
/// one was published. Either way the file must not be trusted.
fn check_size(item: &Download, written: u64) -> Result<(), NetError> {
	let Some(expected) = item.size else {
		return Ok(());
	};
	if written == expected {
		return Ok(());
	}
	let _ = std::fs::remove_file(&item.target);
	Err(NetError::Http {
		url: item.request.primary_or_empty().to_owned(),
		message: format!("expected {expected} bytes, received {written}"),
		status: None,
		body_snippet: None,
	})
}

fn on_disk_len(item: &Download) -> u64 {
	std::fs::metadata(&item.target)
		.map(|metadata| metadata.len())
		.unwrap_or(0)
}

/// Whether the target already holds what this item would fetch.
///
/// With a checksum this is authoritative. Without one, a declared size is the
/// next best signal, and bare presence the last — the same ladder the
/// Minecraft installer has always used, kept because Mojang publishes digests
/// for most but not all artifacts.
fn satisfied(item: &Download) -> bool {
	let Ok(metadata) = std::fs::metadata(&item.target) else {
		return false;
	};
	if !metadata.is_file() {
		return false;
	}
	match (&item.checksum, item.size) {
		(Some(checksum), _) => packwand_pack::hash_file(checksum.format, &item.target)
			.map(|actual| actual.eq_ignore_ascii_case(&checksum.expected))
			.unwrap_or(false),
		(None, Some(size)) => metadata.len() == size,
		(None, None) => true,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use packwand_pack::{HashFormat, hash_bytes};

	#[test]
	fn an_already_correct_file_is_skipped_without_a_request() {
		let root = tempfile::tempdir().unwrap();
		let target = root.path().join("mods/a.jar");
		std::fs::create_dir_all(target.parent().unwrap()).unwrap();
		std::fs::write(&target, b"present").unwrap();

		let item = Download {
			// Unreachable on purpose: reaching the network would fail the test.
			request: Request::get("https://packwand.invalid/never"),
			target,
			checksum: Some(
				Checksum::parse("sha256", hash_bytes(HashFormat::Sha256, b"present")).unwrap(),
			),
			size: Some(7),
		};
		let report = download_all(&Client::downloads(), &[item], Jobs::new(4), &|_| {}).unwrap();
		assert_eq!(report.skipped, 1);
		assert_eq!(report.downloaded, 0);
	}

	#[test]
	fn a_wrong_checksum_on_disk_is_not_treated_as_satisfied() {
		let root = tempfile::tempdir().unwrap();
		let target = root.path().join("mods/a.jar");
		std::fs::create_dir_all(target.parent().unwrap()).unwrap();
		std::fs::write(&target, b"stale").unwrap();

		let item = Download {
			request: Request::get("https://packwand.invalid/never"),
			target,
			checksum: Some(
				Checksum::parse("sha256", hash_bytes(HashFormat::Sha256, b"fresh")).unwrap(),
			),
			size: None,
		};
		assert!(!satisfied(&item));
	}
}
