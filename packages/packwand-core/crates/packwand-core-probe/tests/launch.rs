//! Process-lifecycle fixtures: success, failure, cancellation (before
//! spawn, during execution, after exit), duplicate launches, concurrent
//! instances, process-tree termination, and token-like output passthrough.

mod common;

use std::collections::BTreeMap;
use std::path::Path;

use common::{fake_java_spec, file_len, next_event, wait_until};
use packwand_instance::{FsInstanceRepository, InstanceRecord, InstanceRepository};
use packwand_launch::{
    build_launch_plan, launch, CancellationToken, LaunchError, LaunchEvent, LaunchOptions,
    LaunchPlan,
};

fn plan_for(
    root: &Path,
    id: &str,
    env: BTreeMap<String, String>,
) -> (FsInstanceRepository, InstanceRecord, LaunchPlan) {
    let repo = FsInstanceRepository::new(root.to_path_buf());
    let record = repo.create(&fake_java_spec(id, env)).unwrap();
    let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));
    (repo, record, plan)
}

#[test]
fn successful_run_streams_output_and_records_invocation() {
    let dir = tempfile::tempdir().unwrap();
    let record_file = dir.path().join("record.json");
    let env = BTreeMap::from([
        (
            "FAKE_JAVA_STDOUT".to_string(),
            "hello out\nsecond line".to_string(),
        ),
        ("FAKE_JAVA_STDERR".to_string(), "warn line".to_string()),
        (
            "FAKE_JAVA_RECORD".to_string(),
            record_file.display().to_string(),
        ),
    ]);
    let (_repo, _record, plan) = plan_for(dir.path(), "success", env);
    let events = launch(&plan, LaunchOptions::default()).unwrap().wait();

    assert!(matches!(events[0], LaunchEvent::Starting { .. }));
    assert!(matches!(events[1], LaunchEvent::Started { .. }));
    assert!(matches!(
        events.last(),
        Some(LaunchEvent::Exited { code: Some(0), .. })
    ));
    let stdout: Vec<&str> = events
        .iter()
        .filter_map(|e| match e {
            LaunchEvent::Stdout { line, .. } => Some(line.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(stdout, ["hello out", "second line"]);
    let stderr: Vec<&str> = events
        .iter()
        .filter_map(|e| match e {
            LaunchEvent::Stderr { line, .. } => Some(line.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(stderr, ["warn line"]);

    // The child received exactly the plan's argv, env, and working dir.
    let recorded: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&record_file).unwrap()).unwrap();
    let args: Vec<String> = recorded["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(args, plan.command_arguments());
    assert_eq!(recorded["env"]["FAKE_JAVA_STDERR"], "warn line");
    let cwd = recorded["cwd"].as_str().unwrap();
    assert_eq!(
        std::fs::canonicalize(cwd).unwrap(),
        std::fs::canonicalize(&plan.working_dir).unwrap()
    );

    // The run lock is gone after success.
    assert!(!plan.working_dir.join(".packwand-run.lock").exists());
}

#[test]
fn nonzero_exit_code_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let env = BTreeMap::from([("FAKE_JAVA_EXIT_CODE".to_string(), "7".to_string())]);
    let (_repo, _record, plan) = plan_for(dir.path(), "exit7", env);
    let events = launch(&plan, LaunchOptions::default()).unwrap().wait();
    assert!(matches!(
        events.last(),
        Some(LaunchEvent::Exited { code: Some(7), .. })
    ));
}

#[test]
fn missing_executable_emits_failed_and_releases_lock() {
    let dir = tempfile::tempdir().unwrap();
    let repo = FsInstanceRepository::new(dir.path().to_path_buf());
    let mut spec = fake_java_spec("missing", BTreeMap::new());
    spec.java_executable = dir.path().join("does-not-exist").join("java");
    let record = repo.create(&spec).unwrap();
    let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));
    let events = launch(&plan, LaunchOptions::default()).unwrap().wait();
    assert!(matches!(events[0], LaunchEvent::Starting { .. }));
    assert!(matches!(events.last(), Some(LaunchEvent::Failed { .. })));
    assert!(!plan.working_dir.join(".packwand-run.lock").exists());
}

#[test]
fn cancellation_before_spawn_yields_cancelled_only() {
    let dir = tempfile::tempdir().unwrap();
    let (_repo, _record, plan) = plan_for(dir.path(), "precancel", BTreeMap::new());
    let cancel = CancellationToken::new();
    cancel.cancel();
    let events = launch(
        &plan,
        LaunchOptions {
            cancel: Some(cancel),
            ..Default::default()
        },
    )
    .unwrap()
    .wait();
    assert!(matches!(events[0], LaunchEvent::Starting { .. }));
    assert!(matches!(events[1], LaunchEvent::Cancelled { .. }));
    assert_eq!(events.len(), 2);
    // Lock released: an immediate relaunch is allowed.
    assert!(launch(&plan, LaunchOptions::default()).is_ok());
}

#[test]
fn cancellation_during_run_kills_the_whole_process_tree() {
    let dir = tempfile::tempdir().unwrap();
    let heartbeat = dir.path().join("heartbeat.txt");
    let env = BTreeMap::from([
        (
            "FAKE_JAVA_WAIT_FOR_FILE".to_string(),
            dir.path().join("never-created").display().to_string(),
        ),
        (
            "FAKE_JAVA_SPAWN_HEARTBEAT".to_string(),
            heartbeat.display().to_string(),
        ),
    ]);
    let (_repo, _record, plan) = plan_for(dir.path(), "cancelrun", env);
    let handle = launch(&plan, LaunchOptions::default()).unwrap();
    assert!(matches!(
        next_event(handle.events(), 10),
        LaunchEvent::Starting { .. }
    ));
    assert!(matches!(
        next_event(handle.events(), 10),
        LaunchEvent::Started { .. }
    ));
    // Wait until the grandchild demonstrably runs.
    wait_until("heartbeat file to grow", 10, || file_len(&heartbeat) > 2);

    handle.cancel();
    let events = handle.wait();
    assert!(matches!(events.last(), Some(LaunchEvent::Cancelled { .. })));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, LaunchEvent::Exited { .. })),
        "cancelled run must not also report Exited"
    );

    // The grandchild must be dead too: the heartbeat stops growing.
    let len_after_cancel = file_len(&heartbeat);
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert_eq!(
        file_len(&heartbeat),
        len_after_cancel,
        "heartbeat kept growing: grandchild survived cancellation"
    );

    // Lock released after cancellation.
    assert!(!plan.working_dir.join(".packwand-run.lock").exists());
}

#[test]
fn cancellation_after_exit_is_a_noop() {
    let dir = tempfile::tempdir().unwrap();
    let (_repo, _record, plan) = plan_for(dir.path(), "postcancel", BTreeMap::new());
    let handle = launch(&plan, LaunchOptions::default()).unwrap();
    let cancel = handle.cancel_token();
    let events = handle.wait();
    assert!(matches!(
        events.last(),
        Some(LaunchEvent::Exited { code: Some(0), .. })
    ));
    // Cancelling after completion must not panic or emit anything further.
    cancel.cancel();
}

#[test]
fn duplicate_launch_of_same_instance_is_rejected_unless_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let release = dir.path().join("release.txt");
    let env = BTreeMap::from([(
        "FAKE_JAVA_WAIT_FOR_FILE".to_string(),
        release.display().to_string(),
    )]);
    let (_repo, _record, plan) = plan_for(dir.path(), "dup", env);

    let first = launch(&plan, LaunchOptions::default()).unwrap();
    assert!(matches!(
        next_event(first.events(), 10),
        LaunchEvent::Starting { .. }
    ));

    // Second run of the same instance: rejected while the first holds the lock.
    assert!(matches!(
        launch(&plan, LaunchOptions::default()),
        Err(LaunchError::AlreadyRunning(_))
    ));

    // Explicitly allowed concurrent run of the same instance succeeds.
    let second = launch(
        &plan,
        LaunchOptions {
            allow_concurrent: true,
            ..Default::default()
        },
    )
    .unwrap();

    std::fs::write(&release, b"go").unwrap();
    assert!(matches!(
        first.wait().last(),
        Some(LaunchEvent::Exited { code: Some(0), .. })
    ));
    assert!(matches!(
        second.wait().last(),
        Some(LaunchEvent::Exited { code: Some(0), .. })
    ));
}

#[test]
fn two_different_instances_run_concurrently() {
    let dir = tempfile::tempdir().unwrap();
    let release_a = dir.path().join("release-a.txt");
    let release_b = dir.path().join("release-b.txt");
    let env_a = BTreeMap::from([(
        "FAKE_JAVA_WAIT_FOR_FILE".to_string(),
        release_a.display().to_string(),
    )]);
    let env_b = BTreeMap::from([(
        "FAKE_JAVA_WAIT_FOR_FILE".to_string(),
        release_b.display().to_string(),
    )]);
    let repo = FsInstanceRepository::new(dir.path().to_path_buf());
    let record_a = repo.create(&fake_java_spec("inst-a", env_a)).unwrap();
    let record_b = repo.create(&fake_java_spec("inst-b", env_b)).unwrap();
    let plan_a = build_launch_plan(&record_a, &repo.instance_paths("inst-a"));
    let plan_b = build_launch_plan(&record_b, &repo.instance_paths("inst-b"));

    let handle_a = launch(&plan_a, LaunchOptions::default()).unwrap();
    let handle_b = launch(&plan_b, LaunchOptions::default()).unwrap();

    // Both must reach Started while the other is still running.
    for handle in [&handle_a, &handle_b] {
        assert!(matches!(
            next_event(handle.events(), 10),
            LaunchEvent::Starting { .. }
        ));
        assert!(matches!(
            next_event(handle.events(), 10),
            LaunchEvent::Started { .. }
        ));
    }

    std::fs::write(&release_a, b"go").unwrap();
    std::fs::write(&release_b, b"go").unwrap();
    assert!(matches!(
        handle_a.wait().last(),
        Some(LaunchEvent::Exited { code: Some(0), .. })
    ));
    assert!(matches!(
        handle_b.wait().last(),
        Some(LaunchEvent::Exited { code: Some(0), .. })
    ));
}

#[test]
fn token_like_output_passes_through_and_never_reaches_the_plan() {
    let token_like = "access_token=ghp_FAKE123NOTREALLYASECRET";
    let dir = tempfile::tempdir().unwrap();
    let env = BTreeMap::from([("FAKE_JAVA_STDOUT".to_string(), token_like.to_string())]);
    let (_repo, _record, plan) = plan_for(dir.path(), "tokens", env);

    // Child output resembling a token is forwarded verbatim: it is the
    // child's output, not a secret the supervisor holds.
    let events = launch(&plan, LaunchOptions::default()).unwrap().wait();
    assert!(events
        .iter()
        .any(|e| matches!(e, LaunchEvent::Stdout { line, .. } if line == token_like)));

    // Events never grow a session/secret payload: only the child's own
    // output lines are forwarded, and the plan's session map stays empty
    // for a spec without session placeholders.
    assert!(plan.session.is_empty());
}
