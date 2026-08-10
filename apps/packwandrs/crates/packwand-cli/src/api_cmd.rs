use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::ArgMatches;
use serde::Serialize;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiAction {
	name: &'static str,
	method: &'static str,
	path: &'static str,
	destructive: bool,
}

const ACTIONS: &[ApiAction] = &[
	ApiAction {
		name: "health",
		method: "GET",
		path: "/health",
		destructive: false,
	},
	ApiAction {
		name: "projects.list",
		method: "GET",
		path: "/api/v1/projects",
		destructive: false,
	},
	ApiAction {
		name: "commands.list",
		method: "GET",
		path: "/api/v1/commands",
		destructive: false,
	},
	ApiAction {
		name: "diagnostics.summary",
		method: "GET",
		path: "/api/v1/diagnostics",
		destructive: false,
	},
];

pub fn run(args: &ArgMatches) -> Result {
	let Some(("serve", sub)) = args.subcommand() else {
		return Err("api requires serve".into());
	};
	let bind = sub
		.get_one::<String>("bind")
		.map(String::as_str)
		.unwrap_or("127.0.0.1:0");
	let token_path = sub.get_one::<String>("token-file").map(PathBuf::from);
	let token = token_path
		.as_ref()
		.map(|path| load_token(path, sub.get_flag("generate-token")))
		.transpose()?;
	let listener = TcpListener::bind(bind)?;
	let address = listener.local_addr()?;
	let url = format!("http://{address}");
	if let Some(path) = sub.get_one::<String>("print-port-file") {
		fs::write(path, format!("{url}\n"))?;
	}
	println!("Packwand API listening at {url}");
	let root = std::env::current_dir()?;
	for connection in listener.incoming() {
		match connection {
			Ok(mut stream) => {
				if let Err(error) = handle(&mut stream, &root, token.as_deref()) {
					eprintln!("API request failed: {error}");
					let _ = respond(
						&mut stream,
						500,
						&serde_json::json!({"error":error.to_string()}),
					);
				}
			}
			Err(error) => eprintln!("API connection failed: {error}"),
		}
	}
	Ok(())
}

fn handle(stream: &mut TcpStream, root: &Path, token: Option<&str>) -> Result {
	let mut bytes = [0u8; 32 * 1024];
	let count = stream.read(&mut bytes)?;
	let request = std::str::from_utf8(&bytes[..count])?;
	let line = request.lines().next().ok_or("empty request")?;
	let mut parts = line.split_whitespace();
	let method = parts.next().unwrap_or("");
	let path = parts.next().unwrap_or("").split('?').next().unwrap_or("");
	if method != "GET" {
		return respond(
			stream,
			405,
			&serde_json::json!({"error":"method not allowed"}),
		);
	}
	if let Some(token) = token {
		let authorized = request.lines().any(|line| {
			line.strip_prefix("Authorization: Bearer ")
				.or_else(|| line.strip_prefix("authorization: Bearer "))
				.is_some_and(|provided| provided.trim() == token)
		});
		if !authorized {
			return respond(stream, 401, &serde_json::json!({"error":"unauthorized"}));
		}
	}
	match path {
		"/health" => respond(
			stream,
			200,
			&serde_json::json!({"ok":true,"version":"26.2.0"}),
		),
		"/api/v1/commands" => respond(stream, 200, &ACTIONS),
		"/api/v1/projects" => {
			let projects = packwand_workspace::discover(root)?;
			respond(stream, 200, &serde_json::json!({"projects":projects}))
		}
		"/api/v1/diagnostics" => {
			let manifests = packwand_diagnostics::validate_projects(root)?;
			let parity = packwand_diagnostics::parity_workspace(root)?;
			respond(
				stream,
				200,
				&serde_json::json!({
					"manifests": manifests,
					"parity": parity,
				}),
			)
		}
		_ => respond(stream, 404, &serde_json::json!({"error":"not found"})),
	}
}

fn respond(stream: &mut TcpStream, status: u16, value: &impl Serialize) -> Result {
	let mut body = serde_json::to_vec_pretty(value)?;
	body.push(b'\n');
	let reason = match status {
		200 => "OK",
		401 => "Unauthorized",
		404 => "Not Found",
		405 => "Method Not Allowed",
		_ => "Internal Server Error",
	};
	write!(
		stream,
		"HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n\r\n",
		body.len()
	)?;
	stream.write_all(&body)?;
	stream.flush()?;
	Ok(())
}

fn load_token(path: &Path, generate: bool) -> Result<String> {
	match fs::read_to_string(path) {
		Ok(token) if !token.trim().is_empty() => Ok(token.trim().into()),
		Ok(_) | Err(_) if generate => {
			let seed = format!(
				"{}:{}:{}",
				std::process::id(),
				SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
				path.display()
			);
			let token =
				packwand_pack::hash_bytes(packwand_pack::HashFormat::Sha256, seed.as_bytes());
			if let Some(parent) = path.parent() {
				fs::create_dir_all(parent)?;
			}
			fs::write(path, format!("{token}\n"))?;
			Ok(token)
		}
		Err(error) => Err(error.into()),
		Ok(_) => Err("token file is empty; pass --generate-token".into()),
	}
}
