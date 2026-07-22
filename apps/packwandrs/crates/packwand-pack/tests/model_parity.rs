use std::fs;
use std::path::{Path, PathBuf};

use packwand_pack::{HashFormat, Index, Mod, Pack, PackFormat, hash_file};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}

fn fixture(relative: &str) -> String {
    fs::read_to_string(repository_root().join(relative)).unwrap()
}

fn visit_files(root: &Path, visit: &mut impl FnMut(&Path)) {
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            visit_files(&path, visit);
        } else {
            visit(&path);
        }
    }
}

#[test]
fn parses_a_real_packwand_26_pack() {
    let source = fixture("modpacks/vital/26.2-mr/pack.toml");
    let pack: Pack = toml::from_str(&source).unwrap();
    assert_eq!(pack.name, "Vital");
    assert_eq!(pack.index.file, "index.toml");
    assert_eq!(pack.index.hash_format, "sha512");
    assert_eq!(pack.versions["minecraft"], "26.2");
    assert_eq!(pack.format().unwrap(), PackFormat::Packwand(26));

    let encoded = pack.to_toml().unwrap();
    assert_eq!(toml::from_str::<Pack>(&encoded).unwrap(), pack);
    assert_eq!(
        encoded, source,
        "pack.toml must remain byte-for-byte stable"
    );
}

#[test]
fn parses_real_index_alias_and_metafile_fields() {
    let source = fixture("modpacks/vital/26.2-mr/index.toml");
    let index: Index = toml::from_str(&source).unwrap();
    assert_eq!(index.hash_format, "sha512");
    assert!(index.files.len() > 50);
    assert!(index.files.iter().any(|entry| entry.metafile));
    assert!(
        index
            .files
            .iter()
            .any(|entry| entry.file == "mods/sodium.pw.toml")
    );
    assert_eq!(
        toml::to_string(&index).unwrap(),
        source,
        "index.toml must remain byte-for-byte stable"
    );
}

#[test]
fn sha512_matches_a_real_go_generated_index_entry() {
    let pack_root = repository_root().join("modpacks/vital/26.2-mr");
    let index: Index =
        toml::from_str(&fs::read_to_string(pack_root.join("index.toml")).unwrap()).unwrap();
    let entry = index
        .files
        .iter()
        .find(|entry| entry.file == "mods/sodium.pw.toml")
        .unwrap();
    assert_eq!(
        hash_file(HashFormat::Sha512, &pack_root.join(&entry.file)).unwrap(),
        entry.hash
    );
}

#[test]
fn parses_provider_specific_mod_metadata_without_losing_it() {
    let source = fixture("modpacks/vital/26.2-mr/mods/sodium.pw.toml");
    let metadata: Mod = toml::from_str(&source).unwrap();
    assert_eq!(metadata.name, "Sodium");
    assert_eq!(metadata.side, "client");
    assert_eq!(metadata.download.hash_format, "sha512");
    assert_eq!(
        metadata.update["modrinth"]["mod-id"].as_str(),
        Some("AANobbMI")
    );

    let encoded = metadata.to_toml().unwrap();
    assert_eq!(toml::from_str::<Mod>(&encoded).unwrap(), metadata);
    assert_eq!(
        encoded, source,
        "provider metadata must remain byte-for-byte stable"
    );
}

#[test]
fn pack_format_compatibility_matches_current_go_rules() {
    assert_eq!(
        "packwiz:1.1.0".parse::<PackFormat>().unwrap(),
        PackFormat::Packwiz {
            major: 1,
            minor: 1,
            patch: 0,
        }
    );
    assert!("packwand:25".parse::<PackFormat>().is_err());
    assert!("packwiz:2.0.0".parse::<PackFormat>().is_err());
    assert!("unknown:1".parse::<PackFormat>().is_err());
}

#[test]
fn all_real_models_round_trip_with_required_fidelity() {
    let root = repository_root().join("modpacks");
    let mut packs = 0usize;
    let mut indexes = 0usize;
    let mut metadata_files = 0usize;
    visit_files(&root, &mut |path| {
        let name = path.file_name().unwrap().to_string_lossy();
        enum Kind {
            Pack,
            Index,
            Metadata,
        }
        let kind = if name == "pack.toml" {
            Kind::Pack
        } else if name == "index.toml" {
            Kind::Index
        } else if name.ends_with(".pw.toml") {
            Kind::Metadata
        } else {
            return;
        };
        let source = fs::read_to_string(path).unwrap();
        let encoded = match kind {
            Kind::Pack => {
                packs += 1;
                toml::from_str::<Pack>(&source).unwrap().to_toml().unwrap()
            }
            Kind::Index => {
                indexes += 1;
                toml::to_string(&toml::from_str::<Index>(&source).unwrap()).unwrap()
            }
            Kind::Metadata => {
                metadata_files += 1;
                let metadata = toml::from_str::<Mod>(&source).unwrap();
                let encoded = metadata.to_toml().unwrap();
                assert_eq!(
                    toml::from_str::<Mod>(&encoded).unwrap(),
                    metadata,
                    "semantic mismatch for {}",
                    path.display()
                );
                return;
            }
        };
        assert_eq!(encoded, source, "byte mismatch for {}", path.display());
    });
    assert!(packs > 5, "expected real pack fixtures, found {packs}");
    assert!(indexes > 5, "expected real index fixtures, found {indexes}");
    assert!(
        metadata_files > 100,
        "expected real metadata fixtures, found {metadata_files}"
    );
}
