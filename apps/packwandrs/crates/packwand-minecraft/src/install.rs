//! Plan execution: verified, staged downloads plus natives extraction and
//! legacy asset materialization.
//!
//! Every download is written to a `.pw-part` staging file and renamed into
//! place only after its checksum verified, so an interrupted install never
//! leaves a corrupt file that a later run would trust. Files already
//! present with the right checksum are skipped, which makes installation
//! resumable and re-runnable.

use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use sha1::{Digest, Sha1};

use crate::MinecraftError;
use crate::http::HttpClient;
use crate::plan::{CopyAction, DownloadAction, ExtractAction, InstallPlan};

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
    pub current_download_bytes: u64,
    pub current_download_total: Option<u64>,
}

/// Progress callback fired during download streaming and after skip/finish
/// transitions.
pub type ProgressFn<'a> = &'a (dyn Fn(InstallProgress) + Sync);

/// Executes transactional file downloads and extract operations.
pub struct Installer<'a> {
    http: &'a dyn HttpClient,
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

fn sha1_hex(bytes: &[u8]) -> String {
    let digest = Sha1::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// SHA-1 of a file's contents, streamed rather than buffered.
///
/// An asset index is thousands of files; reading each one whole just to
/// digest and drop it made peak memory track the largest file for no reason.
fn sha1_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha1::new();
    io::copy(&mut file, &mut hasher)?;
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// Does an existing file already satisfy this action?
fn is_satisfied(action: &DownloadAction) -> bool {
    let Ok(metadata) = fs::metadata(&action.target) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    match (&action.sha1, action.size) {
        (Some(sha1), _) => match sha1_file(&action.target) {
            Ok(actual) => actual.eq_ignore_ascii_case(sha1),
            Err(_) => false,
        },
        (None, Some(size)) => metadata.len() == size,
        // Nothing to verify against: presence is the best available signal.
        (None, None) => true,
    }
}

fn satisfied_bytes(action: &DownloadAction) -> u64 {
    action.size.unwrap_or_else(|| {
        fs::metadata(&action.target)
            .map(|metadata| metadata.len())
            .unwrap_or(0)
    })
}

fn write_staged(target: &Path, bytes: &[u8]) -> Result<(), MinecraftError> {
    let parent = target
        .parent()
        .ok_or_else(|| MinecraftError::UnsafePath(target.display().to_string()))?;
    fs::create_dir_all(parent).map_err(io_error(parent))?;
    let staged = target.with_extension("pw-part");
    fs::write(&staged, bytes).map_err(io_error(&staged))?;
    // Windows rename fails onto an existing file; the target only exists
    // here when a previous version failed verification, so replace it.
    if target.exists() {
        fs::remove_file(target).map_err(io_error(target))?;
    }
    fs::rename(&staged, target).map_err(io_error(target))
}

fn perform_download(
    http: &dyn HttpClient,
    action: &DownloadAction,
    on_chunk: &mut dyn FnMut(u64, Option<u64>),
) -> Result<bool, MinecraftError> {
    if is_satisfied(action) {
        return Ok(false);
    }
    let mut downloaded = 0u64;
    let bytes = http.get_with_progress(&action.url, &mut |read, total| {
        downloaded += read as u64;
        on_chunk(downloaded, total.or(action.size));
    })?;
    if let Some(expected) = &action.sha1 {
        let actual = sha1_hex(&bytes);
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(MinecraftError::ChecksumMismatch {
                url: action.url.clone(),
                expected: expected.clone(),
                actual,
            });
        }
    }
    if let Some(expected) = action.size
        && bytes.len() as u64 != expected
    {
        return Err(MinecraftError::SizeMismatch {
            url: action.url.clone(),
            expected,
            actual: bytes.len() as u64,
        });
    }
    write_staged(&action.target, &bytes)?;
    Ok(true)
}

fn extract_natives(action: &ExtractAction) -> Result<(), MinecraftError> {
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

impl<'a> Installer<'a> {
    /// Creates a new installer using the given HTTP client.
    pub fn new(http: &'a dyn HttpClient) -> Self {
        Self { http, workers: 8 }
    }

    /// Sets the number of parallel download worker threads.
    pub fn with_workers(mut self, workers: usize) -> Self {
        self.workers = workers.max(1);
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
        let total_downloads = plan.downloads.len();
        let total_bytes = if plan
            .downloads
            .iter()
            .all(|download| download.size.is_some())
        {
            Some(
                plan.downloads
                    .iter()
                    .map(|download| download.size.unwrap_or(0))
                    .sum(),
            )
        } else {
            None
        };
        let queue: Mutex<VecDeque<&DownloadAction>> = Mutex::new(plan.downloads.iter().collect());
        let failed = AtomicBool::new(false);
        let errors: Mutex<Vec<MinecraftError>> = Mutex::new(Vec::new());
        let downloaded = AtomicUsize::new(0);
        let skipped = AtomicUsize::new(0);
        let completed = AtomicUsize::new(0);
        let downloaded_bytes = AtomicU64::new(0);

        std::thread::scope(|scope| {
            for _ in 0..self.workers.min(total_downloads.max(1)) {
                scope.spawn(|| {
                    loop {
                        if failed.load(Ordering::SeqCst) {
                            return;
                        }
                        let action = {
                            let mut queue = queue.lock().expect("download queue poisoned");
                            queue.pop_front()
                        };
                        let Some(action) = action else { return };

                        if is_satisfied(action) {
                            skipped.fetch_add(1, Ordering::SeqCst);
                            let satisfied = satisfied_bytes(action);
                            let aggregate =
                                downloaded_bytes.fetch_add(satisfied, Ordering::SeqCst) + satisfied;
                            let finished = completed.fetch_add(1, Ordering::SeqCst) + 1;
                            progress(InstallProgress {
                                finished_downloads: finished,
                                total_downloads,
                                downloaded_bytes: aggregate,
                                total_bytes,
                                current_download_bytes: satisfied,
                                current_download_total: action.size.or(Some(satisfied)),
                            });
                            continue;
                        }

                        let mut last_chunk_bytes = 0u64;
                        match perform_download(self.http, action, &mut |current, current_total| {
                            let delta = current.saturating_sub(last_chunk_bytes);
                            last_chunk_bytes = current;
                            let aggregate =
                                downloaded_bytes.fetch_add(delta, Ordering::SeqCst) + delta;
                            progress(InstallProgress {
                                finished_downloads: completed.load(Ordering::SeqCst),
                                total_downloads,
                                downloaded_bytes: aggregate,
                                total_bytes,
                                current_download_bytes: current,
                                current_download_total: current_total,
                            });
                        }) {
                            Ok(true) => {
                                downloaded.fetch_add(1, Ordering::SeqCst);
                                let finished = completed.fetch_add(1, Ordering::SeqCst) + 1;
                                progress(InstallProgress {
                                    finished_downloads: finished,
                                    total_downloads,
                                    downloaded_bytes: downloaded_bytes.load(Ordering::SeqCst),
                                    total_bytes,
                                    current_download_bytes: last_chunk_bytes,
                                    current_download_total: action.size,
                                });
                            }
                            Ok(false) => {
                                unreachable!("satisfied downloads are handled before streaming")
                            }
                            Err(e) => {
                                failed.store(true, Ordering::SeqCst);
                                errors.lock().expect("error list poisoned").push(e);
                                return;
                            }
                        }
                    }
                });
            }
        });

        if let Some(error) = errors.into_inner().expect("error list poisoned").pop() {
            return Err(error);
        }

        let mut report = InstallReport {
            downloaded: downloaded.load(Ordering::SeqCst),
            skipped: skipped.load(Ordering::SeqCst),
            ..InstallReport::default()
        };
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
    use std::sync::Mutex;

    use super::*;
    use crate::http::{FixtureHttpClient, HttpError};

    fn action(url: &str, target: PathBuf, bytes: &[u8]) -> DownloadAction {
        DownloadAction {
            url: url.to_string(),
            target,
            sha1: Some(sha1_hex(bytes)),
            size: Some(bytes.len() as u64),
        }
    }

    #[test]
    fn downloads_verify_and_are_resumable() {
        let dir = tempfile::tempdir().unwrap();
        let body = b"library bytes".to_vec();
        let http = FixtureHttpClient::new([("http://x/lib.jar".to_string(), body.clone())]);
        let plan = InstallPlan {
            downloads: vec![action(
                "http://x/lib.jar",
                dir.path().join("lib/lib.jar"),
                &body,
            )],
            ..InstallPlan::default()
        };
        let installer = Installer::new(&http).with_workers(2);
        let report = installer.execute(&plan, &|_| {}).unwrap();
        assert_eq!(report.downloaded, 1);
        assert_eq!(report.skipped, 0);
        assert_eq!(fs::read(dir.path().join("lib/lib.jar")).unwrap(), body);
        assert!(
            !dir.path().join("lib/lib.pw-part").exists(),
            "staging file cleaned"
        );

        // Second run: file verifies, no network request happens.
        let report = installer.execute(&plan, &|_| {}).unwrap();
        assert_eq!(report.downloaded, 0);
        assert_eq!(report.skipped, 1);
        assert_eq!(http.requests.lock().unwrap().len(), 1);
    }

    #[test]
    fn corrupt_existing_file_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let body = b"good".to_vec();
        let target = dir.path().join("f.bin");
        fs::write(&target, b"corrupt").unwrap();
        let http = FixtureHttpClient::new([("http://x/f".to_string(), body.clone())]);
        let plan = InstallPlan {
            downloads: vec![action("http://x/f", target.clone(), &body)],
            ..InstallPlan::default()
        };
        let report = Installer::new(&http).execute(&plan, &|_| {}).unwrap();
        assert_eq!(report.downloaded, 1);
        assert_eq!(fs::read(&target).unwrap(), body);
    }

    #[test]
    fn checksum_mismatch_fails_and_leaves_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("f.bin");
        let http = FixtureHttpClient::new([("http://x/f".to_string(), b"tampered".to_vec())]);
        let plan = InstallPlan {
            downloads: vec![DownloadAction {
                url: "http://x/f".to_string(),
                target: target.clone(),
                sha1: Some(sha1_hex(b"expected")),
                size: None,
            }],
            ..InstallPlan::default()
        };
        let err = Installer::new(&http).execute(&plan, &|_| {}).unwrap_err();
        assert!(err.to_string().contains("checksum"), "{err}");
        assert!(!target.exists());
    }

    #[test]
    fn copies_materialize_assets() {
        let dir = tempfile::tempdir().unwrap();
        let object = dir.path().join("objects/aa/aabb");
        fs::create_dir_all(object.parent().unwrap()).unwrap();
        fs::write(&object, b"asset").unwrap();
        let http = FixtureHttpClient::default();
        let plan = InstallPlan {
            copies: vec![CopyAction {
                from: object,
                to: dir.path().join("virtual/legacy/icons/icon.png"),
            }],
            ..InstallPlan::default()
        };
        let report = Installer::new(&http).execute(&plan, &|_| {}).unwrap();
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
        let http = FixtureHttpClient::default();
        let dest = dir.path().join("natives");
        let plan = InstallPlan {
            extractions: vec![ExtractAction {
                archive: archive_path,
                dest: dest.clone(),
                excludes: vec!["META-INF/".to_string()],
            }],
            ..InstallPlan::default()
        };
        let report = Installer::new(&http).execute(&plan, &|_| {}).unwrap();
        assert_eq!(report.extracted, 1);
        assert_eq!(fs::read(dest.join("lwjgl.dll")).unwrap(), b"dll bytes");
        assert!(!dest.join("META-INF").exists());
    }

    struct ChunkedHttpClient {
        body: Vec<u8>,
        total: Option<u64>,
    }

    impl HttpClient for ChunkedHttpClient {
        fn get(&self, _url: &str) -> Result<Vec<u8>, HttpError> {
            Ok(self.body.clone())
        }

        fn get_with_progress(
            &self,
            _url: &str,
            on_chunk: &mut dyn FnMut(usize, Option<u64>),
        ) -> Result<Vec<u8>, HttpError> {
            for chunk in self.body.chunks(2) {
                on_chunk(chunk.len(), self.total);
            }
            Ok(self.body.clone())
        }
    }

    #[test]
    fn progress_streams_bytes_with_known_total() {
        let dir = tempfile::tempdir().unwrap();
        let body = b"asset-bytes".to_vec();
        let target = dir.path().join("assets/object.bin");
        let plan = InstallPlan {
            downloads: vec![DownloadAction {
                url: "http://x/object".to_string(),
                target,
                sha1: Some(sha1_hex(&body)),
                size: Some(body.len() as u64),
            }],
            ..InstallPlan::default()
        };
        let events = Mutex::new(Vec::new());
        let report = Installer::new(&ChunkedHttpClient {
            body: body.clone(),
            total: Some(body.len() as u64),
        })
        .with_workers(1)
        .execute(&plan, &|event| {
            events.lock().unwrap().push(event);
        })
        .unwrap();
        assert_eq!(report.downloaded, 1);
        let events = events.into_inner().unwrap();
        assert!(
            events.len() > 2,
            "expected streaming updates, got {events:?}"
        );
        assert_eq!(events.last().unwrap().finished_downloads, 1);
        assert_eq!(events.last().unwrap().downloaded_bytes, body.len() as u64);
        assert_eq!(events.last().unwrap().total_bytes, Some(body.len() as u64));
    }

    #[test]
    fn progress_handles_unknown_content_length() {
        let dir = tempfile::tempdir().unwrap();
        let body = b"unknown-total".to_vec();
        let target = dir.path().join("assets/object.bin");
        let plan = InstallPlan {
            downloads: vec![DownloadAction {
                url: "http://x/object".to_string(),
                target,
                sha1: Some(sha1_hex(&body)),
                size: None,
            }],
            ..InstallPlan::default()
        };
        let events = Mutex::new(Vec::new());
        Installer::new(&ChunkedHttpClient {
            body: body.clone(),
            total: None,
        })
        .with_workers(1)
        .execute(&plan, &|event| {
            events.lock().unwrap().push(event);
        })
        .unwrap();
        let events = events.into_inner().unwrap();
        assert!(
            events
                .iter()
                .any(|event| event.current_download_total.is_none())
        );
        assert!(events.iter().all(|event| event.total_bytes.is_none()));
        assert_eq!(events.last().unwrap().downloaded_bytes, body.len() as u64);
    }
}
