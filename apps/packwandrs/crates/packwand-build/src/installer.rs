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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerTestReport {
    pub pack: PathBuf,
    pub installer: PathBuf,
    pub instance: PathBuf,
    pub url: String,
    pub success: bool,
}

/// Serve a pack on an ephemeral loopback port and drive packwiz-installer
/// against it. The loopback server exists only for the installer protocol;
/// it is not part of the desktop application's runtime data bridge.
pub fn test_with_installer(
    pack: impl AsRef<Path>,
    installer: Option<&Path>,
    instance: impl AsRef<Path>,
) -> Result<InstallerTestReport> {
    let pack = pack.as_ref().canonicalize()?;
    Workspace::open(pack.clone())?;
    let java = Command::new("java")
        .arg("-version")
        .output()
        .map_err(|_| "java not found in PATH")?;
    if !java.status.success() {
        return Err("java -version failed".into());
    }
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
    let status = Command::new("java")
        .args(["-cp", installer.to_string_lossy().as_ref()])
        .arg("link.infra.packwiz.installer.Main")
        .args(["-g", "--continue-on-error", &url])
        .current_dir(&instance)
        .status()?;
    stop.store(true, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
    match server.join() {
        Ok(Ok(())) => {}
        Ok(Err(error)) => return Err(error.into()),
        Err(_) => return Err("installer server thread panicked".into()),
    }
    if !status.success() {
        return Err(format!("packwiz-installer failed with {status}").into());
    }
    Ok(InstallerTestReport {
        pack,
        installer,
        instance,
        url,
        success: true,
    })
}

fn find_installer(explicit: Option<&Path>, pack: &Path) -> Result<PathBuf> {
    for candidate in explicit.map(Path::to_path_buf).into_iter().chain(
        ["PACKWAND_INSTALLER_JAR", "PACKWIZ_INSTALLER_JAR"]
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
                "resources/packwiz-installer.jar",
                "packwiz-installer.jar",
                "apps/packwand-installer/build/dist/packwiz-installer.jar",
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
    Err("packwiz-installer.jar was not found; set PACKWAND_INSTALLER_JAR".into())
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
