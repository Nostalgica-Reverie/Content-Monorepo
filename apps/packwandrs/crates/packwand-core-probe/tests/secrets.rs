//! Spawn-time secret resolution: the child process receives the raw
//! value, while records, plans, and events only ever carry the
//! `${secret:<name>}` placeholder.

mod common;

use std::collections::BTreeMap;

use packwand_auth::SecretString;
use packwand_instance::{FsInstanceRepository, InstanceRepository, InstanceSpec, MemoryLimits};
use packwand_launch::{LaunchEvent, LaunchOptions, build_launch_plan, launch};

fn secret_spec(id: &str, record_path: &std::path::Path) -> InstanceSpec {
    InstanceSpec {
        id: id.to_string(),
        name: format!("Secret fixture {id}"),
        java_executable: common::fake_java(),
        jvm_args: vec![],
        main_class: "fixture.Main".to_string(),
        classpath: vec![],
        game_args: vec![
            "--accessToken".to_string(),
            "${secret:auth_access_token}".to_string(),
        ],
        env: BTreeMap::from([(
            "FAKE_JAVA_RECORD".to_string(),
            record_path.display().to_string(),
        )]),
        memory: MemoryLimits::default(),
        session_placeholders: vec!["auth_access_token".to_string()],
    }
}

#[test]
fn secret_reaches_child_but_never_plan_or_events() {
    let dir = tempfile::tempdir().unwrap();
    let record_path = dir.path().join("child-record.json");
    let repo = FsInstanceRepository::new(dir.path().join("root"));
    let record = repo
        .create(&secret_spec("secret-e2e", &record_path))
        .unwrap();
    let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));

    // The plan (what GUIs inspect and serialize) holds only the placeholder.
    let plan_json = serde_json::to_string(&plan).unwrap();
    assert!(plan_json.contains("${secret:auth_access_token}"));
    assert!(!plan_json.contains("tok-super-secret"));

    let options = LaunchOptions {
        secrets: BTreeMap::from([(
            "auth_access_token".to_string(),
            SecretString::new("tok-super-secret"),
        )]),
        ..LaunchOptions::default()
    };
    let events = launch(&plan, options).unwrap().wait();
    assert!(
        matches!(
            events.last(),
            Some(LaunchEvent::Exited { code: Some(0), .. })
        ),
        "unexpected events: {events:?}"
    );
    // Lifecycle events never carry the secret.
    let events_json = serde_json::to_string(&events).unwrap();
    assert!(!events_json.contains("tok-super-secret"));

    // The child's recorded argv received the resolved value.
    let child_record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
    let args: Vec<String> = child_record["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect();
    assert!(
        args.windows(2)
            .any(|w| w[0] == "--accessToken" && w[1] == "tok-super-secret"),
        "child argv: {args:?}"
    );
    assert!(!args.iter().any(|a| a.contains("${secret:")));
}

#[test]
fn unresolved_secret_placeholder_refuses_to_spawn() {
    let dir = tempfile::tempdir().unwrap();
    let record_path = dir.path().join("never-written.json");
    let repo = FsInstanceRepository::new(dir.path().join("root"));
    let record = repo
        .create(&secret_spec("secret-missing", &record_path))
        .unwrap();
    let plan = build_launch_plan(&record, &repo.instance_paths(&record.id));

    let events = launch(&plan, LaunchOptions::default()).unwrap().wait();
    assert!(
        matches!(
            events.last(),
            Some(LaunchEvent::Failed { error, .. }) if error.contains("auth_access_token")
        ),
        "unexpected events: {events:?}"
    );
    assert!(!record_path.exists(), "the child must never have spawned");
}
