use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use clap::ArgMatches;
use packwand_ops::Workspace;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub fn run(args: &ArgMatches) -> Result {
    let port = args
        .get_one::<String>("port")
        .ok_or("missing port")?
        .parse::<u16>()?;
    let refresh = args
        .get_one::<String>("refresh")
        .is_none_or(|value| value.parse::<bool>().unwrap_or(true));
    let basic = args.get_flag("basic");
    let root = std::env::current_dir()?.canonicalize()?;
    if !basic {
        Workspace::open(root.clone())?;
    }
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!(
        "serving {} at http://127.0.0.1:{}/",
        root.display(),
        listener.local_addr()?.port()
    );
    serve_listener(
        listener,
        root,
        basic,
        refresh,
        Arc::new(AtomicBool::new(false)),
    )
}

pub fn serve_listener(
    listener: TcpListener,
    root: PathBuf,
    basic: bool,
    refresh: bool,
    stop: Arc<AtomicBool>,
) -> Result {
    listener.set_nonblocking(true)?;
    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_read_timeout(Some(Duration::from_secs(10)))?;
                if let Err(error) = handle(&mut stream, &root, basic, refresh) {
                    eprintln!("serve request failed: {error}");
                    let _ = response(
                        &mut stream,
                        500,
                        "text/plain; charset=utf-8",
                        b"internal error\n",
                        false,
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => eprintln!("serve connection failed: {error}"),
        }
    }
    Ok(())
}

fn handle(stream: &mut TcpStream, root: &Path, basic: bool, refresh: bool) -> Result {
    let mut request = [0u8; 16 * 1024];
    let count = stream.read(&mut request)?;
    if count == 0 {
        return Ok(());
    }
    let request = std::str::from_utf8(&request[..count])?;
    let line = request.lines().next().ok_or("empty HTTP request")?;
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or("missing HTTP method")?;
    let raw_target = parts.next().ok_or("missing HTTP target")?;
    if !matches!(method, "GET" | "HEAD") {
        return response(
            stream,
            405,
            "text/plain; charset=utf-8",
            b"method not allowed\n",
            method == "HEAD",
        );
    }
    let target = raw_target.split('?').next().unwrap_or("/");
    if target == "/" {
        let body = br#"<!doctype html><meta charset="utf-8"><title>Packwand</title><h1>Packwand development server</h1><p><a href="/pack.toml">Install pack.toml</a></p>"#;
        return response(
            stream,
            200,
            "text/html; charset=utf-8",
            body,
            method == "HEAD",
        );
    }
    let relative = decode_path(target.trim_start_matches('/'))?;
    let path = safe_join(root, &relative)?;
    if !basic {
        let mut workspace = Workspace::open(root.to_path_buf())?;
        let pack_file = "pack.toml";
        if relative == pack_file && refresh {
            workspace.refresh_metadata_index()?;
        }
        let pack = workspace.pack();
        let index_name = pack.index.file.replace('\\', "/");
        let permitted = relative == pack_file
            || relative == index_name
            || workspace
                .index()
                .files
                .iter()
                .any(|entry| entry.file.replace('\\', "/") == relative);
        if !permitted {
            return response(
                stream,
                404,
                "text/plain; charset=utf-8",
                b"file not found\n",
                method == "HEAD",
            );
        }
    }
    let body = match fs::read(&path) {
        Ok(body) => body,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return response(
                stream,
                404,
                "text/plain; charset=utf-8",
                b"file not found\n",
                method == "HEAD",
            );
        }
        Err(error) => return Err(error.into()),
    };
    response(stream, 200, content_type(&path), &body, method == "HEAD")
}

fn response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    head: bool,
) -> Result {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n\r\n",
        body.len()
    )?;
    if !head {
        stream.write_all(body)?;
    }
    stream.flush()?;
    Ok(())
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if relative.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("unsafe request path {relative:?}").into());
    }
    Ok(root.join(path))
}

fn decode_path(value: &str) -> Result<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err("invalid percent-encoded path".into());
            }
            let high = hex(bytes[index + 1]).ok_or("invalid percent-encoded path")?;
            let low = hex(bytes[index + 2]).ok_or("invalid percent-encoded path")?;
            output.push(high * 16 + low);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    let decoded = String::from_utf8(output)?.replace('\\', "/");
    if decoded.contains('\0') {
        return Err("request path contains a null byte".into());
    }
    Ok(decoded)
}

const fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("toml") => "application/toml; charset=utf-8",
        Some("json" | "mcmeta") => "application/json; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("txt" | "md" | "cfg" | "properties") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_path, safe_join};
    use std::path::Path;

    #[test]
    fn decodes_safe_paths_and_rejects_traversal() {
        assert_eq!(
            decode_path("mods/My%20Mod.pw.json").unwrap(),
            "mods/My Mod.pw.json"
        );
        assert!(safe_join(Path::new("root"), "../secret").is_err());
        assert!(safe_join(Path::new("root"), "mods/example.pw.json").is_ok());
    }
}
