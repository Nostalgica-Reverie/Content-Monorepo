//! Plan execution: verified, staged downloads plus natives extraction and
//! legacy asset materialization.
//!
//! Every download is written to a `.pw-part` staging file and renamed into
//! place only after its checksum verified, so an interrupted install never
//! leaves a corrupt file that a later run would trust. Files already
//! present with the right checksum are skipped, which makes installation
//! resumable and re-runnable.

use std::fs;
use std::io;
use std::path::Path;

use packwand_net::{Checksum, Client, Download, NetError, Request};
use packwand_parallel::Jobs;

use crate::MinecraftError;
use crate::plan::{CopyAction, ExtractAction, InstallPlan};

/// What one [`Installer::execute`] run did.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InstallReport {
	pub downloaded: usize,
	pub skipped: usize,
	pub extracted: usize,
	pub copied: usize,
}

/// One byte-level progress update for an [`Installer::execute`] run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallProgress {
	pub finished_downloads: usize,
	pub total_downloads: usize,
	pub downloaded_bytes: u64,
	pub total_bytes: Option<u64>,
}

/// Progress callback fired during download streaming and after skip/finish
/// transitions.
pub type ProgressFn<'a> = &'a (dyn Fn(InstallProgress) + Sync);

/// Executes transactional file downloads and extract operations.
pub struct Installer<'a> {
	http: &'a Client,
	/// Concurrent download workers. Asset indexes reference thousands of
	/// small files; a small pool matters, an unbounded one is abusive.
	workers: usize,
}

fn io_error(path: &Path) -> impl FnOnce(io::Error) -> MinecraftError + '_ {
	move |source| MinecraftError::Io {
		path: path.to_path_buf(),
		message: source.to_string(),
	}
}

/// Unpacks one native-library archive into its destination directory.
///
/// Public because extraction is a launch-time concern rather than an
/// install-time one: the archives are downloaded once and shared, while the
/// unpacked libraries belong to a single run and are removed when it ends.
pub fn extract_natives(action: &ExtractAction) -> Result<(), MinecraftError> {
	let file = fs::File::open(&action.archive).map_err(io_error(&action.archive))?;
	let mut archive = zip::ZipArchive::new(file).map_err(|e| MinecraftError::Archive {
		path: action.archive.clone(),
		message: e.to_string(),
	})?;
	fs::create_dir_all(&action.dest).map_err(io_error(&action.dest))?;
	for i in 0..archive.len() {
		let mut entry = archive.by_index(i).map_err(|e| MinecraftError::Archive {
			path: action.archive.clone(),
			message: e.to_string(),
		})?;
		let name = entry.name().to_string();
		if action
			.excludes
			.iter()
			.any(|prefix| name.starts_with(prefix))
		{
			continue;
		}
		// enclosed_name refuses traversal and absolute entry paths.
		let Some(relative) = entry.enclosed_name() else {
			return Err(MinecraftError::Archive {
				path: action.archive.clone(),
				message: format!("archive entry {name:?} escapes the extraction directory"),
			});
		};
		let target = action.dest.join(relative);
		if entry.is_dir() {
			fs::create_dir_all(&target).map_err(io_error(&target))?;
			continue;
		}
		if let Some(parent) = target.parent() {
			fs::create_dir_all(parent).map_err(io_error(parent))?;
		}
		let mut out = fs::File::create(&target).map_err(io_error(&target))?;
		io::copy(&mut entry, &mut out).map_err(io_error(&target))?;
	}
	Ok(())
}

fn copy_object(action: &CopyAction) -> Result<(), MinecraftError> {
	if let Some(parent) = action.to.parent() {
		fs::create_dir_all(parent).map_err(io_error(parent))?;
	}
	fs::copy(&action.from, &action.to).map_err(io_error(&action.to))?;
	Ok(())
}

/// The default download width, shared with every other batch operation in
/// Packwand so `--jobs` and the app setting reach this path too.
fn default_workers() -> usize {
	packwand_parallel::configured().get()
}

impl<'a> Installer<'a> {
	/// Creates a new installer using the given HTTP client.
	///
	/// The default worker count follows the machine, capped the same way
	/// `--jobs` is elsewhere. Downloads are I/O-bound, so the ceiling is a
	/// provider's request budget rather than cores.
	pub fn new(http: &'a Client) -> Self {
		Self {
			http,
			workers: default_workers(),
		}
	}

	/// Sets the number of parallel download worker threads. `0` restores the
	/// machine-derived default.
	pub fn with_workers(mut self, workers: usize) -> Self {
		self.workers = if workers == 0 {
			default_workers()
		} else {
			workers
		};
		self
	}

	/// Executes a plan: all downloads (parallel, verified, staged), then
	/// extractions, then copies. Fails on the first error; completed files
	/// are left in place, so a retry resumes where it stopped.
	pub fn execute(
		&self,
		plan: &InstallPlan,
		progress: ProgressFn<'_>,
	) -> Result<InstallReport, MinecraftError> {
		let items: Vec<Download> = plan
			.downloads
			.iter()
			.map(|action| {
				Ok(Download {
					request: Request::get(action.url.clone()),
					target: action.target.clone(),
					checksum: action
						.sha1
						.as_deref()
						.map(|expected| Checksum::parse("sha1", expected))
						.transpose()?,
					size: action.size,
				})
			})
			.collect::<Result<_, NetError>>()?;

		let total_downloads = items.len();
		let report =
			packwand_net::download_all(self.http, &items, Jobs::new(self.workers), &|update| {
				progress(InstallProgress {
					finished_downloads: update.finished,
					total_downloads: update.total,
					downloaded_bytes: update.bytes,
					total_bytes: update.total_bytes,
				});
			})?;
		debug_assert!(report.downloaded + report.skipped == total_downloads);

		let mut report = InstallReport {
			downloaded: report.downloaded,
			skipped: report.skipped,
			..InstallReport::default()
		};
		// After the downloads, because an extraction reads an archive one of
		// them just produced.
		for extraction in &plan.extractions {
			extract_natives(extraction)?;
			report.extracted += 1;
		}
		for copy in &plan.copies {
			copy_object(copy)?;
			report.copied += 1;
		}
		Ok(report)
	}
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use packwand_net::testing::{Reply, StubServer};
	use packwand_pack::HashFormat;

	use super::*;
	use crate::plan::DownloadAction;

	fn sha1_of(bytes: &[u8]) -> String {
		packwand_pack::hash_bytes(HashFormat::Sha1, bytes)
	}

	fn action(url: String, target: PathBuf, bytes: &[u8]) -> DownloadAction {
		DownloadAction {
			url,
			target,
			sha1: Some(sha1_of(bytes)),
			size: Some(bytes.len() as u64),
		}
	}

	#[test]
	fn downloads_verify_and_are_resumable() {
		let dir = tempfile::tempdir().unwrap();
		let body = b"library bytes".to_vec();
		let server = StubServer::start([("/lib.jar".to_owned(), Reply::body(body.clone()))]);
		let plan = InstallPlan {
			downloads: vec![action(
				server.url("/lib.jar"),
				dir.path().join("lib/lib.jar"),
				&body,
			)],
			..InstallPlan::default()
		};
		let client = Client::downloads();
		let installer = Installer::new(&client).with_workers(2);

		let report = installer.execute(&plan, &|_| {}).unwrap();
		assert_eq!(report.downloaded, 1);
		assert_eq!(report.skipped, 0);
		assert_eq!(fs::read(dir.path().join("lib/lib.jar")).unwrap(), body);
		assert!(
			std::fs::read_dir(dir.path().join("lib"))
				.unwrap()
				.filter_map(Result::ok)
				.all(|entry| !entry.file_name().to_string_lossy().contains("pw-part")),
			"staging file cleaned"
		);

		// Second run: the file verifies, so nothing is requested again.
		let report = installer.execute(&plan, &|_| {}).unwrap();
		assert_eq!(report.downloaded, 0);
		assert_eq!(report.skipped, 1);
		assert_eq!(server.hits("/lib.jar"), 1);
	}

	#[test]
	fn corrupt_existing_file_is_replaced() {
		let dir = tempfile::tempdir().unwrap();
		let body = b"good".to_vec();
		let target = dir.path().join("f.bin");
		fs::write(&target, b"corrupt").unwrap();
		let server = StubServer::start([("/f".to_owned(), Reply::body(body.clone()))]);
		let plan = InstallPlan {
			downloads: vec![action(server.url("/f"), target.clone(), &body)],
			..InstallPlan::default()
		};
		let client = Client::downloads();
		let report = Installer::new(&client).execute(&plan, &|_| {}).unwrap();
		assert_eq!(report.downloaded, 1);
		assert_eq!(fs::read(&target).unwrap(), body);
	}

	#[test]
	fn checksum_mismatch_fails_and_leaves_no_file() {
		let dir = tempfile::tempdir().unwrap();
		let target = dir.path().join("f.bin");
		let server = StubServer::start([("/f".to_owned(), Reply::body(b"tampered".to_vec()))]);
		let plan = InstallPlan {
			downloads: vec![DownloadAction {
				url: server.url("/f"),
				target: target.clone(),
				sha1: Some(sha1_of(b"expected")),
				size: None,
			}],
			..InstallPlan::default()
		};
		let client = Client::downloads();
		let error = Installer::new(&client).execute(&plan, &|_| {}).unwrap_err();
		assert!(error.to_string().contains("checksum"), "{error}");
		assert!(!target.exists());
	}

	#[test]
	fn a_size_that_disagrees_with_the_metadata_is_rejected() {
		let dir = tempfile::tempdir().unwrap();
		let target = dir.path().join("f.bin");
		let server = StubServer::start([("/f".to_owned(), Reply::body(b"four".to_vec()))]);
		let plan = InstallPlan {
			downloads: vec![DownloadAction {
				url: server.url("/f"),
				target: target.clone(),
				sha1: None,
				size: Some(999),
			}],
			..InstallPlan::default()
		};
		let client = Client::downloads();
		let error = Installer::new(&client).execute(&plan, &|_| {}).unwrap_err();
		assert!(error.to_string().contains("999"), "{error}");
		assert!(!target.exists());
	}

	#[test]
	fn progress_reports_a_running_total_and_ends_complete() {
		let dir = tempfile::tempdir().unwrap();
		let bodies: Vec<Vec<u8>> = (0..8).map(|i| vec![b'a' + i as u8; 4096]).collect();
		let routes: Vec<_> = bodies
			.iter()
			.enumerate()
			.map(|(i, body)| (format!("/{i}"), Reply::body(body.clone())))
			.collect();
		let server = StubServer::start(routes);
		let plan = InstallPlan {
			downloads: bodies
				.iter()
				.enumerate()
				.map(|(i, body)| {
					action(
						server.url(&format!("/{i}")),
						dir.path().join(format!("{i}.bin")),
						body,
					)
				})
				.collect(),
			..InstallPlan::default()
		};

		let seen: std::sync::Mutex<Vec<InstallProgress>> = std::sync::Mutex::new(Vec::new());
		let client = Client::downloads();
		Installer::new(&client)
			.with_workers(4)
			.execute(&plan, &|update| {
				seen.lock().unwrap().push(update);
			})
			.unwrap();

		let seen = seen.into_inner().unwrap();
		let last = seen.last().expect("at least one update");
		assert_eq!(last.finished_downloads, 8);
		assert_eq!(last.total_downloads, 8);
		assert_eq!(last.total_bytes, Some(8 * 4096));
		assert_eq!(last.downloaded_bytes, 8 * 4096);
		// Bytes only ever accumulate, whichever worker reports them.
		assert!(
			seen.windows(2)
				.all(|pair| pair[1].downloaded_bytes >= pair[0].downloaded_bytes)
		);
	}

	#[test]
	fn copies_materialize_assets() {
		let dir = tempfile::tempdir().unwrap();
		let object = dir.path().join("objects/aa/aabb");
		fs::create_dir_all(object.parent().unwrap()).unwrap();
		fs::write(&object, b"asset").unwrap();
		let plan = InstallPlan {
			copies: vec![CopyAction {
				from: object,
				to: dir.path().join("virtual/legacy/icons/icon.png"),
			}],
			..InstallPlan::default()
		};
		let client = Client::downloads();
		let report = Installer::new(&client).execute(&plan, &|_| {}).unwrap();
		assert_eq!(report.copied, 1);
		assert_eq!(
			fs::read(dir.path().join("virtual/legacy/icons/icon.png")).unwrap(),
			b"asset"
		);
	}
	#[test]
	fn natives_extraction_respects_excludes_and_traversal() {
		use std::io::Write;
		let dir = tempfile::tempdir().unwrap();
		let archive_path = dir.path().join("natives.jar");
		{
			let file = fs::File::create(&archive_path).unwrap();
			let mut writer = zip::ZipWriter::new(file);
			let options = zip::write::SimpleFileOptions::default();
			writer.start_file("lwjgl.dll", options).unwrap();
			writer.write_all(b"dll bytes").unwrap();
			writer.start_file("META-INF/MANIFEST.MF", options).unwrap();
			writer.write_all(b"manifest").unwrap();
			writer.finish().unwrap();
		}
		let dest = dir.path().join("natives");
		let plan = InstallPlan {
			extractions: vec![ExtractAction {
				archive: archive_path,
				dest: dest.clone(),
				excludes: vec!["META-INF/".to_string()],
			}],
			..InstallPlan::default()
		};
		let client = Client::downloads();
		let report = Installer::new(&client).execute(&plan, &|_| {}).unwrap();
		assert_eq!(report.extracted, 1);
		assert_eq!(fs::read(dest.join("lwjgl.dll")).unwrap(), b"dll bytes");
		assert!(!dest.join("META-INF").exists());
	}
}
