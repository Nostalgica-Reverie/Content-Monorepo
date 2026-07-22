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
            "pack-format = \"packwand:26\"\n\n",
            "[index]\n",
            "file = \"index.toml\"\n",
            "hash-format = \"sha512\"\n",
            "hash = \"stale\"\n\n",
            "[versions]\n",
            "fabric = \"0.16.0\"\n",
            "minecraft = \"1.21.1\"\n",
        ),
    )
    .unwrap();
    fs::write(
        directory.path().join("index.toml"),
        "hash-format = \"sha512\"\n",
    )
    .unwrap();
    directory
}

fn metadata(filename: &str, hash: &str) -> Mod {
    let mut update = toml::Table::new();
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
    let mut update = toml::Table::new();
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
            "mods/example.pw.toml",
            metadata("example.jar", "download"),
            false,
        )
        .unwrap();
    assert_eq!(outcome.metadata_path, "mods/example.pw.toml");
    assert!(!outcome.replaced);

    let metadata_bytes = fs::read(directory.path().join("mods/example.pw.toml")).unwrap();
    let index_bytes = fs::read(directory.path().join("index.toml")).unwrap();
    let index: Index = toml::from_str(std::str::from_utf8(&index_bytes).unwrap()).unwrap();
    assert_eq!(index.files.len(), 1);
    assert_eq!(index.files[0].file, "mods/example.pw.toml");
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
        .add_metadata("mods/example.pw.toml", metadata("one.jar", "one"), false)
        .unwrap();
    let before = fs::read(directory.path().join("mods/example.pw.toml")).unwrap();
    assert!(
        workspace
            .add_metadata("mods/example.pw.toml", metadata("two.jar", "two"), false)
            .is_err()
    );
    assert_eq!(
        fs::read(directory.path().join("mods/example.pw.toml")).unwrap(),
        before
    );

    let outcome = workspace
        .add_metadata("mods/example.pw.toml", metadata("two.jar", "two"), true)
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
            "mods/example.pw.toml",
            metadata("example.jar", "hash"),
            false,
        )
        .unwrap();
    workspace.remove_metadata("mods/example.pw.toml").unwrap();
    assert!(!directory.path().join("mods/example.pw.toml").exists());
    let reopened = Workspace::open(directory.path()).unwrap();
    assert!(reopened.index().files.is_empty());
}

#[test]
fn refresh_rehashes_an_edited_metadata_file() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    workspace
        .add_metadata("mods/example.pw.toml", metadata("one.jar", "one"), false)
        .unwrap();
    let edited = metadata("two.jar", "two").to_toml().unwrap();
    fs::write(directory.path().join("mods/example.pw.toml"), &edited).unwrap();
    let hash = workspace.refresh_metadata("mods/example.pw.toml").unwrap();
    assert_eq!(hash, hash_bytes(HashFormat::Sha512, edited.as_bytes()));
    assert_eq!(workspace.index().files[0].hash, hash);
}

#[test]
fn refresh_index_discovers_changes_and_removes_missing_metadata() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    let first = metadata("one.jar", "one").to_toml().unwrap();
    let second = metadata("two.jar", "two").to_toml().unwrap();
    fs::create_dir_all(directory.path().join("mods")).unwrap();
    fs::write(directory.path().join("mods/first.pw.toml"), first).unwrap();
    fs::write(directory.path().join("mods/second.pw.toml"), second).unwrap();

    let added = workspace.refresh_metadata_index().unwrap();
    assert_eq!(added.added, 2);
    assert_eq!(added.updated, 0);
    assert_eq!(added.removed, 0);

    let edited = metadata("changed.jar", "changed").to_toml().unwrap();
    fs::write(directory.path().join("mods/first.pw.toml"), edited).unwrap();
    fs::remove_file(directory.path().join("mods/second.pw.toml")).unwrap();
    let changed = workspace.refresh_metadata_index().unwrap();
    assert_eq!(changed.added, 0);
    assert_eq!(changed.updated, 1);
    assert_eq!(changed.removed, 1);
    assert_eq!(workspace.index().files.len(), 1);
    assert_eq!(workspace.index().files[0].file, "mods/first.pw.toml");
}

#[test]
fn operations_reject_paths_outside_the_pack_root() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    assert!(
        workspace
            .add_metadata("../escape.pw.toml", metadata("bad.jar", "bad"), false)
            .is_err()
    );
    assert!(
        !directory
            .path()
            .parent()
            .unwrap()
            .join("escape.pw.toml")
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
        .add_metadata("mods/example.pw.toml", initial, false)
        .unwrap();

    let outcome = workspace
        .update_resolved("mods/example.pw.toml", resolved("version-2", "two.jar"))
        .unwrap();
    assert!(outcome.changed);
    assert_eq!(outcome.old_filename, "one.jar");
    assert_eq!(outcome.new_filename, "two.jar");
    let updated: Mod =
        toml::from_str(&fs::read_to_string(directory.path().join("mods/example.pw.toml")).unwrap())
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
        .update_resolved("mods/example.pw.toml", resolved("version-2", "ignored.jar"))
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
        .add_metadata("mods/example.pw.toml", initial, false)
        .unwrap();
    let before = fs::read(directory.path().join("mods/example.pw.toml")).unwrap();
    assert!(
        workspace
            .update_resolved("mods/example.pw.toml", resolved("version-2", "two.jar"))
            .is_err()
    );
    assert_eq!(
        fs::read(directory.path().join("mods/example.pw.toml")).unwrap(),
        before
    );
}

#[test]
fn update_rejects_a_different_provider_project() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    workspace
        .add_metadata("mods/example.pw.toml", metadata("one.jar", "one"), false)
        .unwrap();
    let before = fs::read(directory.path().join("mods/example.pw.toml")).unwrap();
    let mut wrong_project = resolved("version-2", "two.jar");
    wrong_project.id = "different-project".into();

    assert!(
        workspace
            .update_resolved("mods/example.pw.toml", wrong_project)
            .is_err()
    );
    assert_eq!(
        fs::read(directory.path().join("mods/example.pw.toml")).unwrap(),
        before
    );
}

#[test]
fn update_supports_repository_release_metadata() {
    let directory = fixture();
    let mut workspace = Workspace::open(directory.path()).unwrap();
    workspace
        .add_metadata("mods/example.pw.toml", github_metadata(), false)
        .unwrap();

    let outcome = workspace
        .update_resolved("mods/example.pw.toml", github_resolved("v2", "two.jar"))
        .unwrap();
    assert!(outcome.changed);
    let updated: Mod =
        toml::from_str(&fs::read_to_string(directory.path().join("mods/example.pw.toml")).unwrap())
            .unwrap();
    assert_eq!(updated.name, "User Repository Name");
    assert_eq!(updated.side, "client");
    assert_eq!(updated.filename, "two.jar");
    assert_eq!(updated.update["github"]["tag"].as_str(), Some("v2"));
    assert_eq!(updated.update["github"]["branch"].as_str(), Some("main"));

    let no_op = workspace
        .update_resolved("mods/example.pw.toml", github_resolved("v2", "ignored.jar"))
        .unwrap();
    assert!(!no_op.changed);
}

#[test]
fn fingerprint_replacement_removes_the_local_file_and_index_entry() {
    let directory = fixture();
    fs::create_dir_all(directory.path().join("mods")).unwrap();
    fs::write(directory.path().join("mods/local.jar"), b"local jar").unwrap();
    fs::write(
        directory.path().join("index.toml"),
        concat!(
            "hash-format = \"sha512\"\n\n",
            "[[files]]\n",
            "file = \"mods/local.jar\"\n",
            "hash = \"old\"\n",
        ),
    )
    .unwrap();
    let mut workspace = Workspace::open(directory.path()).unwrap();

    let outcome = workspace
        .replace_local_with_resolved("mods/local.jar", resolved("v2", "example.jar"))
        .unwrap();

    assert_eq!(outcome.metadata_path, "mods/example.pw.toml");
    assert!(!directory.path().join("mods/local.jar").exists());
    assert!(directory.path().join("mods/example.pw.toml").is_file());
    let index: Index =
        toml::from_str(&fs::read_to_string(directory.path().join("index.toml")).unwrap()).unwrap();
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
            .any(|entry| entry.file == "mods/example.pw.toml" && entry.metafile)
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
            "mods/example.pw.toml",
            metadata("example.jar", "hash"),
            false,
        )
        .unwrap();
    let mut workspace = Workspace::open(destination.path()).unwrap();

    let report = workspace.merge_imported_pack(imported.path()).unwrap();

    assert_eq!(report.files, 1);
    assert_eq!(report.metadata_files, 1);
    assert!(destination.path().join("mods/example.pw.toml").is_file());
    let merged = Workspace::open(destination.path()).unwrap();
    assert_eq!(merged.pack().name, "Fixture");
    assert_eq!(merged.pack().versions["minecraft"], "1.20.1");
    assert_eq!(merged.pack().versions["forge"], "47.3.0");
}
