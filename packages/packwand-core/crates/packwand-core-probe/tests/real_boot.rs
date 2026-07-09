//! Phase 3 of `packwandrs.md`: boot one disposable, real vanilla instance
//! end-to-end, observe its lifecycle, and delete its isolated data.
//!
//! Ignored by default: it needs network access to Mojang's metadata and
//! asset CDNs, a local Java installation compatible with the latest
//! release, and a machine that can open a game window. Run explicitly:
//!
//! ```text
//! cargo test -p packwand-core-probe --test real_boot -- --ignored --nocapture
//! ```

mod common;

use std::process::Command;

/// A log line the vanilla client prints once the game reached its main
/// menu (the sound engine only starts after window, GL context, registry
/// and resource loading all succeeded). Stable across modern versions.
const BOOT_MARKER: &str = "Sound engine started";

#[test]
#[ignore = "network + real Java + game window; run with -- --ignored"]
fn boots_disposable_vanilla_instance_and_cleans_up() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");

    let output = Command::new(common::probe_bin())
        .args(["instance", "bootstrap", "--id", "disposable-boot"])
        .arg("--root")
        .arg(&root)
        .args(["--minecraft", "latest-release"])
        .args(["--username", "PackwandProbe"])
        .args(["--memory-max", "2048", "--workers", "16", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "bootstrap failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(common::probe_bin())
        .args(["launch", "run", "--instance", "disposable-boot"])
        .arg("--root")
        .arg(&root)
        .args(["--username", "PackwandProbe", "--json-events"])
        .args(["--stop-on-line", BOOT_MARKER])
        .args(["--max-runtime-secs", "300"])
        .output()
        .unwrap();
    let events: Vec<serde_json::Value> = String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).expect("event lines are JSON"))
        .collect();

    let booted = events.iter().any(|e| {
        (e["event"] == "stdout" || e["event"] == "stderr")
            && e["line"].as_str().unwrap_or("").contains(BOOT_MARKER)
    });
    assert!(
        booted,
        "the game never reached the boot marker; last events: {:?}",
        events.iter().rev().take(10).collect::<Vec<_>>()
    );
    // The marker triggers cancellation, so the run ends as `cancelled`
    // (exit 3) and the whole process tree is gone.
    assert_eq!(events.last().unwrap()["event"], "cancelled");
    assert_eq!(output.status.code(), Some(3));

    // Disposable: the tempdir (instance data, assets, libraries) is
    // removed when `dir` drops. Verify nothing outside it was written.
    assert!(root
        .join("instances/disposable-boot/instance.json")
        .exists());
}
