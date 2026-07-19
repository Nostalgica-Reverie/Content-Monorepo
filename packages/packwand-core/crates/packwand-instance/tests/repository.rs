//! Fixture tests for the filesystem instance repository, including the
//! corrupt and future-schema-version records from the spike fixture matrix.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use packwand_instance::{
    FsInstanceRepository, InstanceError, InstanceRepository, InstanceSpec, ListEntry, MemoryLimits,
    SCHEMA_VERSION,
};

fn spec(id: &str) -> InstanceSpec {
    InstanceSpec {
        id: id.to_string(),
        name: format!("Instance {id}"),
        java_executable: PathBuf::from("/opt/java/bin/java"),
        jvm_args: vec!["-Dsome=flag".to_string()],
        main_class: "net.minecraft.client.main.Main".to_string(),
        classpath: vec![PathBuf::from("a.jar"), PathBuf::from("b.jar")],
        game_args: vec!["--demo".to_string()],
        env: BTreeMap::from([("KEY".to_string(), "value".to_string())]),
        memory: MemoryLimits {
            initial_mb: Some(512),
            max_mb: Some(2048),
        },
        session_placeholders: vec!["access_token".to_string()],
    }
}

#[test]
fn create_get_roundtrip() {
    let root = tempfile::tempdir().unwrap();
    let repo = FsInstanceRepository::new(root.path().to_path_buf());
    let created = repo.create(&spec("alpha")).unwrap();
    assert_eq!(created.schema_version, SCHEMA_VERSION);
    let loaded = repo.get("alpha").unwrap();
    assert_eq!(created, loaded);
    assert!(
        root.path()
            .join("instances")
            .join("alpha")
            .join("instance.json")
            .is_file()
    );
}

#[test]
fn create_duplicate_fails() {
    let root = tempfile::tempdir().unwrap();
    let repo = FsInstanceRepository::new(root.path().to_path_buf());
    repo.create(&spec("alpha")).unwrap();
    assert!(matches!(
        repo.create(&spec("alpha")),
        Err(InstanceError::AlreadyExists(_))
    ));
}

#[test]
fn get_missing_reports_not_found() {
    let root = tempfile::tempdir().unwrap();
    let repo = FsInstanceRepository::new(root.path().to_path_buf());
    assert!(matches!(repo.get("nope"), Err(InstanceError::NotFound(_))));
}

#[test]
fn list_empty_root_is_empty() {
    let root = tempfile::tempdir().unwrap();
    let repo = FsInstanceRepository::new(root.path().to_path_buf());
    assert!(repo.list().unwrap().is_empty());
}

#[test]
fn list_is_sorted_and_reports_corrupt_and_future_records() {
    let root = tempfile::tempdir().unwrap();
    let repo = FsInstanceRepository::new(root.path().to_path_buf());
    repo.create(&spec("beta")).unwrap();
    repo.create(&spec("alpha")).unwrap();

    let corrupt_dir = root.path().join("instances").join("corrupt");
    fs::create_dir_all(&corrupt_dir).unwrap();
    fs::write(corrupt_dir.join("instance.json"), b"{ not json").unwrap();

    let future_dir = root.path().join("instances").join("future");
    fs::create_dir_all(&future_dir).unwrap();
    fs::write(
        future_dir.join("instance.json"),
        format!("{{\"schema_version\": {}}}", SCHEMA_VERSION + 41),
    )
    .unwrap();

    let entries = repo.list().unwrap();
    let ids: Vec<&str> = entries.iter().map(|e| e.id()).collect();
    assert_eq!(ids, ["alpha", "beta", "corrupt", "future"]);

    assert!(matches!(&entries[0], ListEntry::Ok { .. }));
    assert!(matches!(&entries[1], ListEntry::Ok { .. }));
    let ListEntry::Error { error, .. } = &entries[2] else {
        panic!("corrupt record should be an error entry");
    };
    assert!(error.contains("corrupt"), "unexpected error: {error}");
    let ListEntry::Error { error, .. } = &entries[3] else {
        panic!("future-version record should be an error entry");
    };
    assert!(
        error.contains("schema version"),
        "unexpected error: {error}"
    );

    // The listing serializes with explicit statuses for CLI/adapter output.
    let json = serde_json::to_value(&entries).unwrap();
    assert_eq!(json[0]["status"], "ok");
    assert_eq!(json[2]["status"], "error");
}

#[test]
fn invalid_ids_are_rejected_before_touching_disk() {
    let root = tempfile::tempdir().unwrap();
    let repo = FsInstanceRepository::new(root.path().to_path_buf());
    for bad in ["", ".", "..", "a/b", "a\\b", "a:b"] {
        let mut s = spec("ok");
        s.id = bad.to_string();
        assert!(
            matches!(repo.create(&s), Err(InstanceError::InvalidId(_))),
            "expected id {bad:?} to be rejected"
        );
    }
    assert!(matches!(
        repo.get("../escape"),
        Err(InstanceError::InvalidId(_))
    ));
}

#[test]
fn update_overwrites_an_existing_record_in_place() {
    let root = tempfile::tempdir().unwrap();
    let repo = FsInstanceRepository::new(root.path().to_path_buf());
    let created = repo.create(&spec("alpha")).unwrap();

    let mut changed = created.clone();
    changed.game_args = vec!["--different".to_string()];
    repo.update("alpha", &changed).unwrap();

    let loaded = repo.get("alpha").unwrap();
    assert_eq!(loaded.game_args, vec!["--different".to_string()]);
    assert_eq!(loaded.schema_version, created.schema_version);
}

#[test]
fn update_rejects_invalid_ids_before_touching_disk() {
    let root = tempfile::tempdir().unwrap();
    let repo = FsInstanceRepository::new(root.path().to_path_buf());
    let record = repo.create(&spec("alpha")).unwrap();
    assert!(matches!(
        repo.update("../escape", &record),
        Err(InstanceError::InvalidId(_))
    ));
}

#[test]
fn spec_with_unknown_field_is_rejected() {
    let json = r#"{
        "id": "x", "name": "X", "java_executable": "java",
        "main_class": "Main", "unknown_field": true
    }"#;
    assert!(serde_json::from_str::<InstanceSpec>(json).is_err());
}
