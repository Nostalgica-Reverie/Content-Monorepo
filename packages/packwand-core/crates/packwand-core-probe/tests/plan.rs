//! Launch-plan fixtures: determinism, ordering, substitution, separators,
//! Unicode/space paths, and secret redaction.

mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use packwand_instance::{FsInstanceRepository, InstanceRepository, InstanceSpec, MemoryLimits};
use packwand_launch::{build_launch_plan, host_classpath_separator};

/// A root inside a directory whose name contains spaces and non-ASCII
/// characters, exercising the host OS's path fixture from the matrix.
fn unicode_root(dir: &tempfile::TempDir) -> PathBuf {
    let root = dir.path().join("päck wand ünï t").join("root");
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn rich_spec(root: &Path) -> InstanceSpec {
    InstanceSpec {
        id: "rich".to_string(),
        name: "Rich fixture".to_string(),
        java_executable: common::fake_java(),
        jvm_args: vec![
            "-Djava.library.path=${natives_dir}".to_string(),
            "-Dwand.id=${instance_id}".to_string(),
        ],
        main_class: "fixture.Main".to_string(),
        // Deliberately not alphabetical: order must be preserved.
        classpath: vec![root.join("libs/z.jar"), root.join("libs/a.jar")],
        game_args: vec![
            "--gameDir".to_string(),
            "${game_dir}".to_string(),
            "--assetsDir".to_string(),
            "${assets_dir}".to_string(),
        ],
        env: BTreeMap::from([("FIXTURE_KEY".to_string(), "fixture-value".to_string())]),
        memory: MemoryLimits {
            initial_mb: Some(512),
            max_mb: Some(1024),
        },
        session_placeholders: vec!["access_token".to_string(), "xuid".to_string()],
    }
}

#[test]
fn plan_substitutes_and_preserves_order() {
    let dir = tempfile::tempdir().unwrap();
    let root = unicode_root(&dir);
    let repo = FsInstanceRepository::new(root.clone());
    let record = repo.create(&rich_spec(&root)).unwrap();
    let paths = repo.instance_paths(&record.id);
    let plan = build_launch_plan(&record, &paths);

    assert_eq!(
        plan.classpath,
        vec![root.join("libs/z.jar"), root.join("libs/a.jar")]
    );
    assert_eq!(plan.classpath_separator, host_classpath_separator());
    assert_eq!(
        plan.jvm_args[0],
        format!("-Djava.library.path={}", paths.natives_dir.display())
    );
    assert_eq!(plan.jvm_args[1], "-Dwand.id=rich");
    assert_eq!(plan.game_args[1], paths.game_dir.display().to_string());
    assert_eq!(plan.game_args[3], paths.assets_dir.display().to_string());
    assert_eq!(plan.working_dir, paths.game_dir);
    assert_eq!(plan.paths.game_data, paths.game_dir);
    assert_eq!(plan.paths.logs, paths.game_dir.join("logs"));

    // Session values are redacted placeholders, never secrets.
    assert_eq!(plan.session["access_token"], "${secret:access_token}");
    assert_eq!(plan.session["xuid"], "${secret:xuid}");

    // Full argv ordering contract: jvm args, memory, classpath, main class,
    // game args.
    let sep = host_classpath_separator();
    let expected_cp = format!(
        "{}{sep}{}",
        root.join("libs/z.jar").display(),
        root.join("libs/a.jar").display()
    );
    assert_eq!(
        plan.command_arguments(),
        vec![
            plan.jvm_args[0].clone(),
            "-Dwand.id=rich".to_string(),
            "-Xms512m".to_string(),
            "-Xmx1024m".to_string(),
            "-cp".to_string(),
            expected_cp,
            "fixture.Main".to_string(),
            "--gameDir".to_string(),
            paths.game_dir.display().to_string(),
            "--assetsDir".to_string(),
            paths.assets_dir.display().to_string(),
        ]
    );
}

#[test]
fn plan_json_is_deterministic_across_20_runs() {
    let dir = tempfile::tempdir().unwrap();
    let root = unicode_root(&dir);
    let repo = FsInstanceRepository::new(root.clone());
    repo.create(&rich_spec(&root)).unwrap();

    let mut outputs = Vec::new();
    for _ in 0..20 {
        let output = Command::new(common::probe_bin())
            .args(["launch", "plan", "--json", "--instance", "rich", "--root"])
            .arg(&root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "plan failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        outputs.push(output.stdout);
    }
    for output in &outputs[1..] {
        assert_eq!(
            output, &outputs[0],
            "launch plan JSON differed between runs"
        );
    }

    // The plan never contains raw secret material, only placeholders.
    let text = String::from_utf8(outputs[0].clone()).unwrap();
    assert!(text.contains("${secret:access_token}"));
}
