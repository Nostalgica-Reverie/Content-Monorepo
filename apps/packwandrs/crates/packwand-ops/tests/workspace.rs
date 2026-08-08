use std::collections::BTreeMap;
use std::fs;

use packwand_ops::Workspace;
use packwand_pack::{Download, HashFormat, Index, Mod, ModOption, Pack, hash_bytes};
use packwand_providers::{
    DEFAULT_ASSET_PATTERN, ProjectType, ProviderKind, ReleaseChannel, RepositoryRelease,
    ResolvedFile, ResolvedProject, ResolvedVersion,
};

fn fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("pack.toml"),
        concat!(
            "name = \"Fixture\"\n",
            "pack-format = \"packwand:27\"\n\n",
            "[index]\n",
            "file = \"index.json\"\n",
            "hash-format = \"sha512\"\n",
            "hash = \"stale\"\n\n",
            "[versions]\n",
            "fabric = \"0.16.0\"\n",
            "minecraft = \"1.21.1\"\n",
        ),
    )
    .unwrap();
    fs::write(
        directory.path().join("index.json"),
        r#"{"hash-format": "sha512", "files": []}"#,
    )
    .unwrap();
    directory
}

fn metadata(filename: &str, hash: &str) -> Mod {
    let mut update = packwand_pack::UpdateTable::new();
    update.insert("mod-id".into(), "project".into());
    update.insert("version".into(), "version".into());
    Mod {
        name: "Example".into(),
        filename: filename.into(),
        side: "both".into(),
        download: Download {
            url: "https://example.test/example.jar".into(),
            hash_format: "sha512".into(),
            hash: hash.into(),
            ..Download::default()
        },
        update: BTreeMap::from([("modrinth".into(), update)]),
        ..Mod::default()
    }
}

fn resolved(version: &str, filename: &str) -> ResolvedProject {
    ResolvedProject {
        provider: ProviderKind::Modrinth,
        id: "project".into(),
        slug: "example".into(),
        title: "Provider Name".into(),
        project_type: ProjectType::Mod,
        side: "both".into(),
        repository_release: None,
        version: ResolvedVersion {
            id: version.into(),
            name: version.into(),
            number: version.into(),
            channel: ReleaseChannel::Release,
            file: ResolvedFile {
                filename: filename.into(),
                url: Some(format!("https://example.test/{filename}")),
                size: 10,
                hashes: BTreeMap::from([("sha512".into(), format!("hash-{version}"))]),
            },
        },
    }
}

fn github_metadata() -> Mod {
    let mut update = packwand_pack::UpdateTable::new();
    update.insert("slug".into(), "owner/example".into());
    update.insert("tag".into(), "v1".into());
    update.insert("branch".into(), "main".into());
    update.insert("regex".into(), DEFAULT_ASSET_PATTERN.into());
    Mod {
        name: "User Repository Name".into(),
        filename: "one.jar".into(),
        side: "client".into(),
        download: Download {
            url: "https://downloads.test/one.jar".into(),
            hash_format: "sha512".into(),
            hash: "one".into(),
            ..Download::default()
        },
        update: BTreeMap::from([("github".into(), update)]),
        ..Mod::default()
    }
}

fn github_resolved(tag: &str, filename: &str) -> ResolvedProject {
    ResolvedProject {
        provider: ProviderKind::GitHub,
        id: "owner/example".into(),
        slug: "example".into(),
        title: "Provider Repository Name".into(),
        project_type: ProjectType::Mod,
        side: "both".into(),
        repository_release: Some(RepositoryRelease {
            instance: None,
            branch: "main".into(),
            asset_pattern: DEFAULT_ASSET_PATTERN.into(),
        }),
        version: ResolvedVersion {
            id: tag.into(),
            name: tag.into(),
            number: tag.into(),
            channel: ReleaseChannel::Release,
            file: ResolvedFile {
                filename: filename.into(),
                url: Some(format!("https://downloads.test/{filename}")),
                size: 12,
                hashes: BTreeMap::from([("sha512".into(), format!("hash-{tag}"))]),
            },
        },
    }
}

#[test]
fn add_writes_metadata_and_updates_generated_documents() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    let outcome = workspace
        .add_metadata(
            "mods/example.pw.json",
            metadata("example.jar", "download"),
            false,
        )
        .unwrap();
    assert_eq!(outcome.metadata_path, "mods/example.pw.json");
    assert!(!outcome.replaced);

    let metadata_bytes = fs::read(directory.path().join("mods/example.pw.json")).unwrap();
    let index_bytes = fs::read(directory.path().join("index.json")).unwrap();
    let index: Index = serde_json::from_str(std::str::from_utf8(&index_bytes).unwrap()).unwrap();
    assert_eq!(index.files.len(), 1);
    assert_eq!(index.files[0].file, "mods/example.pw.json");
    assert!(index.files[0].metafile);
    assert_eq!(
        index.files[0].hash,
        hash_bytes(HashFormat::Sha512, &metadata_bytes)
    );
    let pack: Pack =
        toml::from_str(&fs::read_to_string(directory.path().join("pack.toml")).unwrap()).unwrap();
    assert_eq!(
        pack.index.hash,
        hash_bytes(HashFormat::Sha512, &index_bytes)
    );
    assert_eq!(workspace.index(), &index);
}

#[test]
fn add_refuses_overwrite_unless_replace_is_explicit() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    workspace
        .add_metadata("mods/example.pw.json", metadata("one.jar", "one"), false)
        .unwrap();
    let before = fs::read(directory.path().join("mods/example.pw.json")).unwrap();
    assert!(
        workspace
            .add_metadata("mods/example.pw.json", metadata("two.jar", "two"), false)
            .is_err()
    );
    assert_eq!(
        fs::read(directory.path().join("mods/example.pw.json")).unwrap(),
        before
    );

    let outcome = workspace
        .add_metadata("mods/example.pw.json", metadata("two.jar", "two"), true)
        .unwrap();
    assert!(outcome.replaced);
    assert_eq!(outcome.filename, "two.jar");
}

#[test]
fn remove_deletes_metadata_and_its_index_entry_together() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    workspace
        .add_metadata(
            "mods/example.pw.json",
            metadata("example.jar", "hash"),
            false,
        )
        .unwrap();
    workspace.remove_metadata("mods/example.pw.json").unwrap();
    assert!(!directory.path().join("mods/example.pw.json").exists());
    let reopened = Workspace::open(directory.path()).unwrap();
    assert!(reopened.index().files.is_empty());
}

#[test]
fn refresh_rehashes_an_edited_metadata_file() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    workspace
        .add_metadata("mods/example.pw.json", metadata("one.jar", "one"), false)
        .unwrap();
    let edited = metadata("two.jar", "two").to_json_bytes().unwrap();
    fs::write(directory.path().join("mods/example.pw.json"), &edited).unwrap();
    let hash = workspace.refresh_metadata("mods/example.pw.json").unwrap();
    assert_eq!(hash, hash_bytes(HashFormat::Sha512, &edited));
    assert_eq!(workspace.index().files[0].hash, hash);
}

#[test]
fn refresh_index_discovers_changes_and_removes_missing_metadata() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    let first = metadata("one.jar", "one").to_json_bytes().unwrap();
    let second = metadata("two.jar", "two").to_json_bytes().unwrap();
    fs::create_dir_all(directory.path().join("mods")).unwrap();
    fs::write(directory.path().join("mods/first.pw.json"), first).unwrap();
    fs::write(directory.path().join("mods/second.pw.json"), second).unwrap();

    let added = workspace.refresh_metadata_index().unwrap();
    assert_eq!(added.added, 2);
    assert_eq!(added.updated, 0);
    assert_eq!(added.removed, 0);

    let edited = metadata("changed.jar", "changed").to_json_bytes().unwrap();
    fs::write(directory.path().join("mods/first.pw.json"), edited).unwrap();
    fs::remove_file(directory.path().join("mods/second.pw.json")).unwrap();
    let changed = workspace.refresh_metadata_index().unwrap();
    assert_eq!(changed.added, 0);
    assert_eq!(changed.updated, 1);
    assert_eq!(changed.removed, 1);
    assert_eq!(workspace.index().files.len(), 1);
    assert_eq!(workspace.index().files[0].file, "mods/first.pw.json");
}

#[test]
fn refresh_index_reconciles_renamed_ordinary_files() {
    let directory = fixture();
    fs::create_dir_all(directory.path().join("config")).unwrap();
    fs::write(directory.path().join("config/renamed.json"), "new").unwrap();
    fs::write(
        directory.path().join("index.json"),
        r#"{"hash-format": "sha512", "files": [
            {"file": "config/old.json", "hash": "stale"}
        ]}"#,
    )
    .unwrap();

    let mut workspace = Workspace::open(directory.path()).unwrap();
    let report = workspace.refresh_metadata_index().unwrap();

    assert_eq!(report.added, 1);
    assert_eq!(report.removed, 1);
    assert_eq!(workspace.index().files.len(), 1);
    assert_eq!(workspace.index().files[0].file, "config/renamed.json");
    assert!(!workspace.index().files[0].metafile);
}

#[test]
fn refresh_index_honors_packwizignore_for_ordinary_files() {
    let directory = fixture();
    fs::create_dir_all(directory.path().join("config")).unwrap();
    fs::write(directory.path().join("config/kept.json"), "kept").unwrap();
    fs::write(directory.path().join("archive.mrpack"), "ignored").unwrap();
    fs::write(directory.path().join(".packwizignore"), "*.mrpack\n").unwrap();

    let mut workspace = Workspace::open(directory.path()).unwrap();
    workspace.refresh_metadata_index().unwrap();

    assert_eq!(workspace.index().files.len(), 1);
    assert_eq!(workspace.index().files[0].file, "config/kept.json");
}

#[test]
fn operations_reject_paths_outside_the_pack_root() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    assert!(
        workspace
            .add_metadata("../escape.pw.json", metadata("bad.jar", "bad"), false)
            .is_err()
    );
    assert!(
        !directory
            .path()
            .parent()
            .unwrap()
            .join("escape.pw.json")
            .exists()
    );
}

#[test]
fn update_preserves_user_fields_and_detects_no_op_versions() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    let mut initial = metadata("one.jar", "one");
    initial.name = "User Name".into();
    initial.side = "client".into();
    initial.option = Some(ModOption {
        optional: true,
        description: "User choice".into(),
        default: false,
    });
    initial
        .update
        .get_mut("modrinth")
        .unwrap()
        .insert("custom".into(), "keep".into());
    workspace
        .add_metadata("mods/example.pw.json", initial, false)
        .unwrap();

    let outcome = workspace
        .update_resolved("mods/example.pw.json", resolved("version-2", "two.jar"))
        .unwrap();
    assert!(outcome.changed);
    assert_eq!(outcome.old_filename, "one.jar");
    assert_eq!(outcome.new_filename, "two.jar");
    let updated: Mod = serde_json::from_str(
        &fs::read_to_string(directory.path().join("mods/example.pw.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(updated.name, "User Name");
    assert_eq!(updated.side, "client");
    assert_eq!(updated.option.unwrap().description, "User choice");
    assert_eq!(updated.update["modrinth"]["custom"].as_str(), Some("keep"));
    assert_eq!(
        updated.update["modrinth"]["version"].as_str(),
        Some("version-2")
    );

    let no_op = workspace
        .update_resolved("mods/example.pw.json", resolved("version-2", "ignored.jar"))
        .unwrap();
    assert!(!no_op.changed);
    assert_eq!(no_op.new_filename, "two.jar");
}

#[test]
fn update_honors_pinned_metadata() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    let mut initial = metadata("one.jar", "one");
    initial.pin = true;
    workspace
        .add_metadata("mods/example.pw.json", initial, false)
        .unwrap();
    let before = fs::read(directory.path().join("mods/example.pw.json")).unwrap();
    assert!(
        workspace
            .update_resolved("mods/example.pw.json", resolved("version-2", "two.jar"))
            .is_err()
    );
    assert_eq!(
        fs::read(directory.path().join("mods/example.pw.json")).unwrap(),
        before
    );
}

#[test]
fn update_rejects_a_different_provider_project() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    workspace
        .add_metadata("mods/example.pw.json", metadata("one.jar", "one"), false)
        .unwrap();
    let before = fs::read(directory.path().join("mods/example.pw.json")).unwrap();
    let mut wrong_project = resolved("version-2", "two.jar");
    wrong_project.id = "different-project".into();

    assert!(
        workspace
            .update_resolved("mods/example.pw.json", wrong_project)
            .is_err()
    );
    assert_eq!(
        fs::read(directory.path().join("mods/example.pw.json")).unwrap(),
        before
    );
}

#[test]
fn update_supports_repository_release_metadata() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    workspace
        .add_metadata("mods/example.pw.json", github_metadata(), false)
        .unwrap();

    let outcome = workspace
        .update_resolved("mods/example.pw.json", github_resolved("v2", "two.jar"))
        .unwrap();
    assert!(outcome.changed);
    let updated: Mod = serde_json::from_str(
        &fs::read_to_string(directory.path().join("mods/example.pw.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(updated.name, "User Repository Name");
    assert_eq!(updated.side, "client");
    assert_eq!(updated.filename, "two.jar");
    assert_eq!(updated.update["github"]["tag"].as_str(), Some("v2"));
    assert_eq!(updated.update["github"]["branch"].as_str(), Some("main"));

    let no_op = workspace
        .update_resolved("mods/example.pw.json", github_resolved("v2", "ignored.jar"))
        .unwrap();
    assert!(!no_op.changed);
}

#[test]
fn fingerprint_replacement_removes_the_local_file_and_index_entry() {
    let directory = fixture();
    fs::create_dir_all(directory.path().join("mods")).unwrap();
    fs::write(directory.path().join("mods/local.jar"), b"local jar").unwrap();
    fs::write(
        directory.path().join("index.json"),
        r#"{"hash-format": "sha512", "files": [
            {"file": "mods/local.jar", "hash": "old"}
        ]}"#,
    )
    .unwrap();
    let mut workspace = Workspace::open(directory.path()).unwrap();

    let outcome = workspace
        .replace_local_with_resolved("mods/local.jar", resolved("v2", "example.jar"))
        .unwrap();

    assert_eq!(outcome.metadata_path, "mods/example.pw.json");
    assert!(!directory.path().join("mods/local.jar").exists());
    assert!(directory.path().join("mods/example.pw.json").is_file());
    let index: Index =
        serde_json::from_str(&fs::read_to_string(directory.path().join("index.json")).unwrap())
            .unwrap();
    assert!(
        !index
            .files
            .iter()
            .any(|entry| entry.file == "mods/local.jar")
    );
    assert!(
        index
            .files
            .iter()
            .any(|entry| entry.file == "mods/example.pw.json" && entry.metafile)
    );
}

#[test]
fn imported_pack_merge_preserves_identity_and_merges_versions_and_files() {
    let destination = fixture();
    let imported = fixture();
    let mut imported_pack: Pack =
        toml::from_str(&fs::read_to_string(imported.path().join("pack.toml")).unwrap()).unwrap();
    imported_pack.name = "Imported".into();
    imported_pack
        .versions
        .insert("minecraft".into(), "1.20.1".into());
    imported_pack
        .versions
        .insert("forge".into(), "47.3.0".into());
    fs::write(
        imported.path().join("pack.toml"),
        imported_pack.to_toml().unwrap(),
    )
    .unwrap();
    Workspace::open(imported.path())
        .unwrap()
        .add_metadata(
            "mods/example.pw.json",
            metadata("example.jar", "hash"),
            false,
        )
        .unwrap();
    let mut workspace = Workspace::open(destination.path()).unwrap();

    let report = workspace.merge_imported_pack(imported.path()).unwrap();

    assert_eq!(report.files, 1);
    assert_eq!(report.metadata_files, 1);
    assert!(destination.path().join("mods/example.pw.json").is_file());
    let merged = Workspace::open(destination.path()).unwrap();
    assert_eq!(merged.pack().name, "Fixture");
    assert_eq!(merged.pack().versions["minecraft"], "1.20.1");
    assert_eq!(merged.pack().versions["forge"], "47.3.0");
}

/// A generation-26 pack on disk: TOML metadata, TOML index, old format string.
fn legacy_fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("pack.toml"),
        concat!(
            "name = \"Legacy\"\n",
            "pack-format = \"packwand:26\"\n\n",
            "[index]\n",
            "file = \"index.toml\"\n",
            "hash-format = \"sha512\"\n\n",
            "[versions]\n",
            "minecraft = \"1.21.1\"\n",
        ),
    )
    .unwrap();
    fs::write(
        directory.path().join("index.toml"),
        "hash-format = \"sha512\"\n",
    )
    .unwrap();
    fs::create_dir_all(directory.path().join("mods")).unwrap();
    for slug in ["alpha", "beta"] {
        // Written as literal TOML rather than serialized: `Mod` only encodes
        // JSON now, and this fixture has to be what generation 26 actually
        // left on disk.
        fs::write(
            directory.path().join(format!("mods/{slug}.pw.toml")),
            format!(
                concat!(
                    "name = \"Example\"\n",
                    "filename = \"{slug}.jar\"\n",
                    "side = \"both\"\n\n",
                    "[download]\n",
                    "url = \"https://example.test/{slug}.jar\"\n",
                    "hash-format = \"sha512\"\n",
                    "hash = \"{slug}\"\n\n",
                    "[update]\n",
                    "[update.modrinth]\n",
                    "mod-id = \"project\"\n",
                    "version = \"version\"\n",
                ),
                slug = slug
            ),
        )
        .unwrap();
    }
    fs::write(directory.path().join("config.json"), "{}").unwrap();
    directory
}

#[test]
fn migrating_to_generation_27_converts_metadata_and_the_index() {
    let directory = legacy_fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    let (old, new, renames) = workspace.migrate_format_with(false).unwrap();

    assert_eq!(old, "packwand:26");
    assert_eq!(new, "packwand:27");
    assert_eq!(renames.len(), 2, "both mods must be renamed");

    // Every legacy file is gone and its JSON replacement parses.
    for slug in ["alpha", "beta"] {
        let legacy = directory.path().join(format!("mods/{slug}.pw.toml"));
        let migrated = directory.path().join(format!("mods/{slug}.pw.json"));
        assert!(!legacy.exists(), "{slug}: legacy metadata must be removed");
        assert!(migrated.is_file(), "{slug}: JSON metadata must be written");
        let parsed: Mod = serde_json::from_str(&fs::read_to_string(&migrated).unwrap()).unwrap();
        assert_eq!(parsed.filename, format!("{slug}.jar"));
    }

    // The index moved to JSON and the pack points at it.
    assert!(!directory.path().join("index.toml").exists());
    assert!(directory.path().join("index.json").is_file());
    let pack: Pack =
        toml::from_str(&fs::read_to_string(directory.path().join("pack.toml")).unwrap()).unwrap();
    assert_eq!(pack.index.file, "index.json");
    assert_eq!(pack.pack_format, "packwand:27");
    assert!(!pack.format().unwrap().needs_migration());

    // The regenerated index describes the new paths, not the old ones.
    let index: Index =
        serde_json::from_str(&fs::read_to_string(directory.path().join("index.json")).unwrap())
            .unwrap();
    assert!(
        index
            .files
            .iter()
            .any(|entry| entry.file == "mods/alpha.pw.json" && entry.metafile)
    );
    assert!(
        !index
            .files
            .iter()
            .any(|entry| entry.file.ends_with(".pw.toml")),
        "no generation-26 path may survive in the index"
    );
}

#[test]
fn a_dry_run_migration_reports_the_plan_and_changes_nothing() {
    let directory = legacy_fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    let before = fs::read(directory.path().join("mods/alpha.pw.toml")).unwrap();

    let (old, new, renames) = workspace.migrate_format_with(true).unwrap();
    assert_eq!(old, "packwand:26");
    assert_eq!(new, "packwand:27");
    assert_eq!(
        renames,
        vec![
            packwand_ops::MetadataRename {
                from: "mods/alpha.pw.toml".into(),
                to: "mods/alpha.pw.json".into(),
            },
            packwand_ops::MetadataRename {
                from: "mods/beta.pw.toml".into(),
                to: "mods/beta.pw.json".into(),
            },
        ],
        "the plan must be reported in sorted path order"
    );

    // Nothing on disk moved.
    assert_eq!(
        fs::read(directory.path().join("mods/alpha.pw.toml")).unwrap(),
        before
    );
    assert!(!directory.path().join("mods/alpha.pw.json").exists());
    assert!(directory.path().join("index.toml").exists());
    assert!(!directory.path().join("index.json").exists());
}

#[test]
fn migrating_an_already_current_pack_is_a_no_op() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    workspace
        .add_metadata("mods/example.pw.json", metadata("one.jar", "one"), false)
        .unwrap();
    let (old, new, renames) = workspace.migrate_format_with(false).unwrap();
    assert_eq!(old, "packwand:27");
    assert_eq!(new, "packwand:27");
    assert!(renames.is_empty());
    assert!(directory.path().join("mods/example.pw.json").is_file());
}

/// A pack that already has both generations of one mod is ambiguous: migrating
/// would have to choose which copy wins. Refusing is the only safe answer.
#[test]
fn migration_refuses_when_the_target_metadata_already_exists() {
    let directory = legacy_fixture();
    fs::write(
        directory.path().join("mods/alpha.pw.json"),
        metadata("conflict.jar", "conflict")
            .to_json_bytes()
            .unwrap(),
    )
    .unwrap();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    let error = workspace.migrate_format_with(false).unwrap_err();
    assert!(
        matches!(error, packwand_ops::OpsError::AlreadyExists(ref path) if path.contains("alpha")),
        "expected an AlreadyExists error naming the conflict, got {error:?}"
    );
    // The pack is untouched: the legacy file is still there and still legacy.
    assert!(directory.path().join("mods/alpha.pw.toml").exists());
    assert!(directory.path().join("index.toml").exists());
}
