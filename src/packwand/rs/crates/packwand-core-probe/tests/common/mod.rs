//! Shared fixture helpers for the spike test suites.
//!
//! Each test binary uses a different subset of these helpers, so the
//! per-binary dead-code lint is silenced.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use packwand_instance::{InstanceSpec, MemoryLimits};
use packwand_launch::LaunchEvent;

/// Path to the cross-platform Java stand-in binary.
pub fn fake_java() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fake-java"))
}

/// Path to the probe CLI binary.
pub fn probe_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_packwand-core-probe"))
}

/// An instance spec whose "Java" is the fake-java helper, controlled via
/// the plan's env map.
pub fn fake_java_spec(id: &str, env: BTreeMap<String, String>) -> InstanceSpec {
    InstanceSpec {
        id: id.to_string(),
        name: format!("Fixture {id}"),
        java_executable: fake_java(),
        jvm_args: vec![],
        main_class: "fixture.Main".to_string(),
        classpath: vec![],
        game_args: vec![],
        env,
        memory: MemoryLimits::default(),
        session_placeholders: vec![],
    }
}

/// Receives the next event or panics after `secs` seconds.
pub fn next_event(rx: &Receiver<LaunchEvent>, secs: u64) -> LaunchEvent {
    rx.recv_timeout(Duration::from_secs(secs))
        .expect("timed out waiting for launch event")
}

/// Polls until `predicate` holds or the deadline passes.
pub fn wait_until(what: &str, secs: u64, mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < deadline {
        if predicate() {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("timed out waiting for {what}");
}

/// Size of a file, or 0 if it does not exist yet.
pub fn file_len(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}
