//! Cross-platform stand-in for `java`, used by the packwand-rs spike tests.
//!
//! Behavior is controlled entirely through environment variables (which the
//! launch plan's `env` map supplies), so the fixture never needs a real Java
//! or Minecraft installation:
//!
//! - `FAKE_JAVA_RECORD`: write received args/env/cwd as JSON to this path.
//!   `raw_args` is argv verbatim; `args` has `@argfile` entries expanded the
//!   way the real launcher expands them, so tests can assert both that the
//!   child saw a value and that the value was not on the command line.
//! - `FAKE_JAVA_STDOUT` / `FAKE_JAVA_STDERR`: newline-separated lines to print.
//! - `FAKE_JAVA_SPAWN_HEARTBEAT`: spawn a grandchild that appends to this
//!   file every 30 ms forever (used to verify process-tree termination).
//! - `FAKE_JAVA_WAIT_FOR_FILE`: poll until this file exists ("wait for a
//!   signal").
//! - `FAKE_JAVA_SLEEP_MS`: sleep this long before exiting.
//! - `FAKE_JAVA_EXIT_CODE`: exit with this code (default 0).
//! - `FAKE_JAVA_MODE=heartbeat`: internal grandchild mode.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
	if env::var("FAKE_JAVA_MODE").as_deref() == Ok("heartbeat") {
		heartbeat();
	}

	// The launcher identifies a JVM before deciding how to pass arguments to
	// it, so the stand-in has to answer that question too. Reporting a
	// modern release is what makes the fixture exercise the argument-file
	// path rather than the pre-Java-9 fallback.
	if env::args().any(|a| a == "-XshowSettings:properties") {
		let version = env::var("FAKE_JAVA_VERSION").unwrap_or_else(|_| "25.0.1".to_string());
		eprintln!("Property settings:");
		eprintln!("    java.version = {version}");
		eprintln!("    java.vendor = Packwand Test Fixture");
		eprintln!("    os.arch = {}", env::consts::ARCH);
		std::process::exit(0);
	}

	if let Ok(path) = env::var("FAKE_JAVA_RECORD") {
		let raw: Vec<String> = env::args().skip(1).collect();
		let record = serde_json::json!({
			"raw_args": raw,
			"args": expand_argfiles(&raw),
			"env": env::vars().collect::<BTreeMap<String, String>>(),
			"cwd": env::current_dir().ok(),
		});
		let bytes = serde_json::to_vec_pretty(&record).expect("serialize record");
		std::fs::write(&path, bytes).expect("write record file");
	}

	if let Ok(path) = env::var("FAKE_JAVA_SPAWN_HEARTBEAT") {
		let exe = env::current_exe().expect("current exe");
		// Deliberately never waited on: the grandchild loops forever so the
		// supervisor's process-tree termination is what must reap it.
		#[allow(clippy::zombie_processes)]
		Command::new(exe)
			.env("FAKE_JAVA_MODE", "heartbeat")
			.env("FAKE_JAVA_HEARTBEAT_FILE", &path)
			.spawn()
			.expect("spawn heartbeat grandchild");
	}

	if let Ok(lines) = env::var("FAKE_JAVA_STDOUT") {
		for line in lines.split('\n') {
			println!("{line}");
		}
	}
	if let Ok(lines) = env::var("FAKE_JAVA_STDERR") {
		for line in lines.split('\n') {
			eprintln!("{line}");
		}
	}

	if let Ok(path) = env::var("FAKE_JAVA_WAIT_FOR_FILE") {
		while !Path::new(&path).exists() {
			thread::sleep(Duration::from_millis(20));
		}
	}

	if let Ok(ms) = env::var("FAKE_JAVA_SLEEP_MS") {
		let ms: u64 = ms.parse().expect("FAKE_JAVA_SLEEP_MS must be an integer");
		thread::sleep(Duration::from_millis(ms));
	}

	let code = env::var("FAKE_JAVA_EXIT_CODE")
		.ok()
		.map(|c| {
			c.parse::<i32>()
				.expect("FAKE_JAVA_EXIT_CODE must be an integer")
		})
		.unwrap_or(0);
	std::process::exit(code);
}

/// Replaces each `@file` argument with the arguments that file contains.
///
/// Written against the launcher's documented argument-file rules rather than
/// against the writer in `packwand-launch`, so that a writer that quotes
/// wrongly fails here instead of round-tripping through a matching bug:
/// whitespace separates arguments outside quotes, and `\\`, `\"`, `\n`, `\r`,
/// `\t` and `\f` are the escapes recognised inside them.
fn expand_argfiles(args: &[String]) -> Vec<String> {
	let mut out = Vec::new();
	for arg in args {
		match arg.strip_prefix('@') {
			Some(path) => match std::fs::read_to_string(path) {
				Ok(body) => out.extend(parse_argfile(&body)),
				Err(e) => panic!("cannot read argument file {path}: {e}"),
			},
			None => out.push(arg.clone()),
		}
	}
	out
}

fn parse_argfile(body: &str) -> Vec<String> {
	let mut args = Vec::new();
	let mut current = String::new();
	let mut started = false;
	let mut quote: Option<char> = None;
	let mut chars = body.chars();
	while let Some(ch) = chars.next() {
		match ch {
			'\\' if quote.is_some() => {
				let escaped = chars.next().expect("dangling escape in argument file");
				current.push(match escaped {
					'n' => '\n',
					'r' => '\r',
					't' => '\t',
					'f' => '\u{c}',
					other => other,
				});
			}
			'"' | '\'' if quote == Some(ch) => quote = None,
			'"' | '\'' if quote.is_none() => {
				quote = Some(ch);
				started = true;
			}
			c if c.is_whitespace() && quote.is_none() => {
				if started {
					args.push(std::mem::take(&mut current));
					started = false;
				}
			}
			c => {
				current.push(c);
				started = true;
			}
		}
	}
	if started {
		args.push(current);
	}
	args
}

fn heartbeat() -> ! {
	let path = env::var("FAKE_JAVA_HEARTBEAT_FILE").expect("heartbeat file path");
	loop {
		if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
			let _ = file.write_all(b".");
		}
		thread::sleep(Duration::from_millis(30));
	}
}
