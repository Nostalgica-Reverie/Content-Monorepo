use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use packwand_ops::Workspace;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub use packwand_installer::ManualDownload;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerTestReport {
	pub pack: PathBuf,
	pub installer: PathBuf,
	pub instance: PathBuf,
	pub url: String,
	pub success: bool,
}

fn manual_pending_path(game_dir: &Path) -> PathBuf {
	game_dir.join(".packwand-installer").join("manual-pending.json")
}

/// Reads the manual-download backlog the last native-installer run left
/// behind (CurseForge files an author has disabled third-party distribution
/// for), if any. The install itself still succeeds without these — they're
/// reported so a GUI can prompt for them instead of the instance silently
/// missing content.
pub fn manual_pending(game_dir: impl AsRef<Path>) -> Result<Vec<ManualDownload>> {
	let bytes = match fs::read(manual_pending_path(game_dir.as_ref())) {
		Ok(bytes) => bytes,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
		Err(error) => return Err(error.into()),
	};
	serde_json::from_slice(&bytes).map_err(|error| error.into())
}

/// Places a user-selected file for one pending manual download — the same
/// "point us at the jar you already downloaded" flow Prism uses for
/// CurseForge files that forbid third-party downloads. Verifies the file
/// matches what the pack expects before accepting it.
pub fn provide_manual_download(source: impl AsRef<Path>, pending: &ManualDownload) -> Result<()> {
	let bytes = fs::read(source.as_ref())?;
	packwand_installer::index::verify(
		&pending.target.display().to_string(),
		&pending.hash_format,
		&pending.hash,
		&bytes,
	)
	.map_err(|error| Box::new(error) as Box<dyn Error + Send + Sync>)?;
	if let Some(parent) = pending.target.parent() {
		fs::create_dir_all(parent)?;
	}
	fs::write(&pending.target, bytes)?;
	Ok(())
}

/// Serves a local pack on an ephemeral loopback port and drives the native
/// Packwand installer against it.
///
/// The loopback server exists only for the installer protocol; it is not part
/// of the desktop application's runtime data bridge. A non-zero installer exit
/// status is returned to the caller so the built-in launcher cannot continue
/// into Minecraft bootstrap with partially installed content.
pub fn install_with_native_installer(
	pack: impl AsRef<Path>,
	installer: Option<&Path>,
	instance: impl AsRef<Path>,
) -> Result<InstallerTestReport> {
	install_with_runner(pack, installer, instance, |installer, instance, url| {
		let status = Command::new(installer)
			.args(["--side", "client", "--game-dir"])
			.arg(instance)
			.arg(url)
			.current_dir(instance)
			.status()?;
		if status.success() {
			Ok(())
		} else {
			Err(format!("packwand-installer failed with {status}").into())
		}
	})
}

fn install_with_runner(
	pack: impl AsRef<Path>,
	installer: Option<&Path>,
	instance: impl AsRef<Path>,
	run: impl FnOnce(&Path, &Path, &str) -> Result<()>,
) -> Result<InstallerTestReport> {
	let pack = pack.as_ref().canonicalize()?;
	Workspace::open(pack.clone())?;
	let installer = find_installer(installer, &pack)?;
	let instance = instance.as_ref().to_path_buf();
	fs::create_dir_all(&instance)?;
	let listener = TcpListener::bind(("127.0.0.1", 0))?;
	let port = listener.local_addr()?.port();
	let stop = Arc::new(AtomicBool::new(false));
	let server_stop = stop.clone();
	let server_pack = pack.clone();
	let server = thread::spawn(move || {
		serve(listener, server_pack, server_stop).map_err(|error| error.to_string())
	});
	if !wait_for_port(port, Duration::from_secs(10)) {
		stop.store(true, Ordering::Relaxed);
		return Err("installer test server did not become ready".into());
	}
	let url = format!("http://127.0.0.1:{port}/pack.toml");
	let run_result = run(&installer, &instance, &url);
	stop.store(true, Ordering::Relaxed);
	let _ = TcpStream::connect(("127.0.0.1", port));
	match server.join() {
		Ok(Ok(())) => {}
		Ok(Err(error)) => return Err(error.into()),
		Err(_) => return Err("installer server thread panicked".into()),
	}
	run_result?;
	Ok(InstallerTestReport {
		pack,
		installer,
		instance,
		url,
		success: true,
	})
}

/// Compatibility name used by `packwand test`.
pub fn test_with_installer(
	pack: impl AsRef<Path>,
	installer: Option<&Path>,
	instance: impl AsRef<Path>,
) -> Result<InstallerTestReport> {
	install_with_native_installer(pack, installer, instance)
}

fn find_installer(explicit: Option<&Path>, pack: &Path) -> Result<PathBuf> {
	for candidate in explicit.map(Path::to_path_buf).into_iter().chain(
		["PACKWAND_INSTALLER_BIN"]
			.into_iter()
			.filter_map(|name| std::env::var_os(name).map(PathBuf::from)),
	) {
		if candidate.is_file() {
			return Ok(candidate.canonicalize()?);
		}
	}
	let mut roots = vec![pack.to_path_buf(), std::env::current_dir()?];
	if let Ok(exe) = std::env::current_exe()
		&& let Some(parent) = exe.parent()
	{
		roots.push(parent.to_path_buf());
	}
	for mut root in roots {
		loop {
			for relative in [
				"resources/packwand-installer.exe",
				"resources/packwand-installer",
				"packwand-installer.exe",
				"packwand-installer",
				"apps/packwandrs/target/release/packwand-installer.exe",
				"apps/packwandrs/target/release/packwand-installer",
			] {
				let candidate = root.join(relative);
				if candidate.is_file() {
					return Ok(candidate.canonicalize()?);
				}
			}
			if !root.pop() {
				break;
			}
		}
	}
	Err("packwand-installer was not found; set PACKWAND_INSTALLER_BIN".into())
}

fn serve(listener: TcpListener, root: PathBuf, stop: Arc<AtomicBool>) -> Result<()> {
	listener.set_nonblocking(true)?;
	while !stop.load(Ordering::Relaxed) {
		match listener.accept() {
			Ok((mut stream, _)) => {
				stream.set_nonblocking(false)?;
				stream.set_read_timeout(Some(Duration::from_secs(10)))?;
				if let Err(error) = serve_one(&mut stream, &root) {
					let _ = reply(&mut stream, 500, b"internal error\n");
					eprintln!("installer test request failed: {error}");
				}
			}
			Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
				thread::sleep(Duration::from_millis(20))
			}
			Err(error) => return Err(error.into()),
		}
	}
	Ok(())
}

fn serve_one(stream: &mut TcpStream, root: &Path) -> Result<()> {
	let mut request = [0u8; 16 * 1024];
	let count = stream.read(&mut request)?;
	if count == 0 {
		return Ok(());
	}
	let line = std::str::from_utf8(&request[..count])?
		.lines()
		.next()
		.ok_or("empty HTTP request")?;
	let mut parts = line.split_whitespace();
	if parts.next() != Some("GET") {
		return reply(stream, 405, b"method not allowed\n");
	}
	let relative = parts
		.next()
		.ok_or("missing target")?
		.trim_start_matches('/')
		.replace("%20", " ")
		.replace('\\', "/");
	if relative.is_empty()
		|| Path::new(&relative)
			.components()
			.any(|part| !matches!(part, Component::Normal(_)))
	{
		return reply(stream, 404, b"not found\n");
	}
	let mut workspace = Workspace::open(root.to_path_buf())?;
	if relative == "pack.toml" {
		workspace.refresh_metadata_index()?;
	}
	let allowed = relative == "pack.toml"
		|| relative == workspace.pack().index.file.replace('\\', "/")
		|| workspace
			.index()
			.files
			.iter()
			.any(|item| item.file.replace('\\', "/") == relative);
	if !allowed {
		return reply(stream, 404, b"not found\n");
	}
	match fs::read(root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR))) {
		Ok(body) => reply(stream, 200, &body),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
			reply(stream, 404, b"not found\n")
		}
		Err(error) => Err(error.into()),
	}
}

fn reply(stream: &mut TcpStream, status: u16, body: &[u8]) -> Result<()> {
	let reason = if status == 200 { "OK" } else { "Error" };
	write!(
		stream,
		"HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n\r\n",
		body.len()
	)?;
	stream.write_all(body)?;
	stream.flush()?;
	Ok(())
}

fn wait_for_port(port: u16, timeout: Duration) -> bool {
	let deadline = Instant::now() + timeout;
	while Instant::now() < deadline {
		if TcpStream::connect_timeout(
			&format!("127.0.0.1:{port}").parse().expect("valid address"),
			Duration::from_millis(250),
		)
		.is_ok()
		{
			return true;
		}
		thread::sleep(Duration::from_millis(100));
	}
	false
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;

	use super::install_with_runner;
	use packwand_installer::InstallSide;
	use packwand_ops::Workspace;

	#[test]
	fn built_in_launcher_server_installs_with_native_library() {
		let pack = tempfile::tempdir().unwrap();
		let instance = tempfile::tempdir().unwrap();
		let config = pack.path().join("config").join("fixture.txt");
		std::fs::create_dir_all(config.parent().unwrap()).unwrap();
		std::fs::write(&config, b"native launcher contract\n").unwrap();
		let metadata = packwand_pack::Pack {
			name: "Built-in launcher fixture".into(),
			version: "1.0.0".into(),
			pack_format: packwand_pack::CURRENT_PACK_FORMAT.into(),
			versions: BTreeMap::from([("minecraft".into(), "1.21.1".into())]),
			..Default::default()
		};
		std::fs::write(
			pack.path().join("pack.toml"),
			toml::to_string_pretty(&metadata).unwrap(),
		)
		.unwrap();
		std::fs::write(
			pack.path().join(packwand_pack::metafile::INDEX_FILE),
			serde_json::to_vec_pretty(&packwand_pack::Index::default()).unwrap(),
		)
		.unwrap();
		Workspace::open(pack.path().to_path_buf())
			.unwrap()
			.refresh_metadata_index()
			.unwrap();

		let current_executable = std::env::current_exe().unwrap();
		let report = install_with_runner(
			pack.path(),
			Some(&current_executable),
			instance.path(),
			|_, game_dir, url| {
				packwand_installer::install(url, game_dir, InstallSide::Client)
					.map(|_| ())
					.map_err(|error| Box::new(error) as _)
			},
		)
		.unwrap();
		assert!(report.success);
		assert_eq!(
			std::fs::read(instance.path().join("config").join("fixture.txt")).unwrap(),
			b"native launcher contract\n"
		);
	}
}
