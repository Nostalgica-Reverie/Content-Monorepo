//! Cross-platform stand-in for `java`, used by the packwand-rs spike tests.
//!
//! Behavior is controlled entirely through environment variables (which the
//! launch plan's `env` map supplies), so the fixture never needs a real Java
//! or Minecraft installation:
//!
//! - `FAKE_JAVA_RECORD`: write received args/env/cwd as JSON to this path.
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

    if let Ok(path) = env::var("FAKE_JAVA_RECORD") {
        let record = serde_json::json!({
            "args": env::args().skip(1).collect::<Vec<_>>(),
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

fn heartbeat() -> ! {
    let path = env::var("FAKE_JAVA_HEARTBEAT_FILE").expect("heartbeat file path");
    loop {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = file.write_all(b".");
        }
        thread::sleep(Duration::from_millis(30));
    }
}
