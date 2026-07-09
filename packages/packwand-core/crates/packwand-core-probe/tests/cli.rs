//! End-to-end tests of the four probe commands, driving the built binary
//! exactly as the experiment specifies.

mod common;

use std::path::Path;
use std::process::Command;

fn write_spec(dir: &Path, id: &str, extra_env: &[(&str, &str)]) -> std::path::PathBuf {
    let env: serde_json::Map<String, serde_json::Value> = extra_env
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
        .collect();
    let spec = serde_json::json!({
        "id": id,
        "name": format!("CLI fixture {id}"),
        "java_executable": common::fake_java(),
        "main_class": "fixture.Main",
        "env": env,
    });
    let path = dir.join(format!("{id}.spec.json"));
    std::fs::write(&path, serde_json::to_vec_pretty(&spec).unwrap()).unwrap();
    path
}

fn probe(args: &[&str], root: &Path) -> std::process::Output {
    Command::new(common::probe_bin())
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap()
}

#[test]
fn create_list_plan_run_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    let spec = write_spec(
        dir.path(),
        "cli-e2e",
        &[("FAKE_JAVA_STDOUT", "cli says hi")],
    );

    let output = Command::new(common::probe_bin())
        .args(["instance", "create", "--spec"])
        .arg(&spec)
        .arg("--root")
        .arg(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let record: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(record["id"], "cli-e2e");
    assert_eq!(record["schema_version"], 1);

    let output = probe(&["instance", "list", "--json"], &root);
    assert!(output.status.success());
    let entries: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(entries[0]["status"], "ok");
    assert_eq!(entries[0]["id"], "cli-e2e");

    let output = probe(
        &["launch", "plan", "--instance", "cli-e2e", "--json"],
        &root,
    );
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["instance_id"], "cli-e2e");
    assert_eq!(plan["main_class"], "fixture.Main");

    let output = probe(
        &["launch", "run", "--instance", "cli-e2e", "--json-events"],
        &root,
    );
    assert!(
        output.status.success(),
        "run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let events: Vec<serde_json::Value> = String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).expect("each line is one JSON event"))
        .collect();
    assert_eq!(events.first().unwrap()["event"], "starting");
    assert_eq!(events[1]["event"], "started");
    assert!(events
        .iter()
        .any(|e| e["event"] == "stdout" && e["line"] == "cli says hi"));
    let last = events.last().unwrap();
    assert_eq!(last["event"], "exited");
    assert_eq!(last["code"], 0);
}

#[test]
fn run_propagates_child_exit_code() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    let spec = write_spec(dir.path(), "cli-exit", &[("FAKE_JAVA_EXIT_CODE", "9")]);
    let output = Command::new(common::probe_bin())
        .args(["instance", "create", "--spec"])
        .arg(&spec)
        .arg("--root")
        .arg(&root)
        .output()
        .unwrap();
    assert!(output.status.success());

    let output = probe(
        &["launch", "run", "--instance", "cli-exit", "--json-events"],
        &root,
    );
    assert_eq!(output.status.code(), Some(9));
}

#[test]
fn list_reports_corrupt_instance_without_failing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    let corrupt = root.join("instances").join("broken");
    std::fs::create_dir_all(&corrupt).unwrap();
    std::fs::write(corrupt.join("instance.json"), b"not json at all").unwrap();

    let output = probe(&["instance", "list", "--json"], &root);
    assert!(output.status.success());
    let entries: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(entries[0]["status"], "error");
    assert_eq!(entries[0]["id"], "broken");
}

#[test]
fn plan_for_missing_instance_fails_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    let output = probe(&["launch", "plan", "--instance", "ghost", "--json"], &root);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not found"));
}
