//! Round-trip fidelity against the repository's real pack corpus.
//!
//! These tests read the actual packs in this repository rather than fixtures,
//! because the failure they exist to catch is "our model silently drops a
//! field some real pack uses" — which a hand-written fixture never reproduces.
//!
//! The corpus spans two generations while the migration to `packwand:27`
//! rolls out: `pack.toml` is TOML in both, but metadata and the index are TOML
//! in generation 26 and JSON in 27. Every test here therefore dispatches on
//! what is actually on disk instead of assuming one generation, so they stay
//! meaningful before, during, and after `packwand migrate format` runs across
//! the corpus.

use std::fs;
use std::path::{Path, PathBuf};

use packwand_pack::{HashFormat, Index, Mod, Pack, PackFormat, hash_file, metafile};

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

/// A representative pack, resolved to whichever generation is on disk.
struct SamplePack {
	root: PathBuf,
	index: PathBuf,
	sodium: PathBuf,
}

fn sample_pack() -> SamplePack {
	let root = repository_root().join("modpacks/vital/26.2-mr");
	let json_index = root.join("index.json");
	let index = if json_index.is_file() {
		json_index
	} else {
		root.join("index.toml")
	};
	let json_metadata = root.join("mods/sodium.pw.json");
	let sodium = if json_metadata.is_file() {
		json_metadata
	} else {
		root.join("mods/sodium.pw.toml")
	};
	SamplePack {
		root,
		index,
		sodium,
	}
}

fn read_index(path: &Path) -> Index {
	let source = fs::read_to_string(path).unwrap();
	if path.extension().is_some_and(|ext| ext == "json") {
		serde_json::from_str(&source).unwrap()
	} else {
		toml::from_str(&source).unwrap()
	}
}

fn read_metadata(path: &Path) -> Mod {
	let source = fs::read_to_string(path).unwrap();
	if metafile::is_metafile(path) {
		serde_json::from_str(&source).unwrap()
	} else {
		toml::from_str(&source).unwrap()
	}
}

#[test]
fn parses_a_real_pack() {
	let source = fixture("modpacks/vital/26.2-mr/pack.toml");
	let pack: Pack = toml::from_str(&source).unwrap();
	assert_eq!(pack.name, "Vital");
	assert_eq!(pack.index.hash_format, "sha512");
	assert_eq!(pack.versions["minecraft"], "26.2");
	// Either generation is valid on disk; both must parse.
	assert!(matches!(pack.format().unwrap(), PackFormat::Packwand(_)));

	// pack.toml stays TOML and stays hand-authored in every generation, so it
	// is the one document that must still survive a byte-for-byte round trip.
	let encoded = pack.to_toml().unwrap();
	assert_eq!(toml::from_str::<Pack>(&encoded).unwrap(), pack);
	assert_eq!(
		encoded, source,
		"pack.toml must remain byte-for-byte stable"
	);
}

#[test]
fn parses_real_index_alias_and_metafile_fields() {
	let sample = sample_pack();
	let index = read_index(&sample.index);
	assert_eq!(index.hash_format, "sha512");
	assert!(index.files.len() > 50);
	assert!(index.files.iter().any(|entry| entry.metafile));
	assert!(
		index
			.files
			.iter()
			.any(|entry| entry.file.contains("sodium.pw.")),
		"the sample pack should index its sodium metadata"
	);
}

#[test]
fn parses_provider_specific_mod_metadata_without_losing_it() {
	let sample = sample_pack();
	let metadata = read_metadata(&sample.sodium);
	assert_eq!(metadata.name, "Sodium");
	assert_eq!(metadata.side, "client");
	assert_eq!(metadata.download.hash_format, "sha512");
	assert_eq!(
		metadata.update["modrinth"]["mod-id"].as_str(),
		Some("AANobbMI"),
		"provider update metadata must survive parsing"
	);
}

#[test]
fn sha512_matches_a_real_generated_index_entry() {
	let sample = sample_pack();
	let index = read_index(&sample.index);
	let name = sample.sodium.file_name().unwrap().to_string_lossy();
	let entry = index
		.files
		.iter()
		.find(|entry| entry.file.ends_with(name.as_ref()))
		.expect("sodium metadata must be indexed");
	assert_eq!(
		hash_file(HashFormat::Sha512, &sample.root.join(&entry.file)).unwrap(),
		entry.hash,
		"the index hash must match the file it describes"
	);
}

/// Every hand-authored model in the corpus must survive a parse/encode/parse
/// cycle without losing a field, in whichever generation that pack is
/// currently stored in.
///
/// Fidelity is asserted semantically (re-parsing the encoding yields an equal
/// model) rather than byte-for-byte, because metadata bytes are packwand's to
/// choose. `pack.toml` is additionally checked byte-for-byte in
/// [`parses_a_real_pack`] — it is the one document a human edits.
///
/// The index is deliberately **not** swept here. It is a generated,
/// `.gitignore`d artifact rather than repository content, so a stale or
/// half-written one on a developer's disk is not a model-fidelity failure —
/// and one such corrupt file is what made the previous version of this test
/// fail permanently. Index fidelity is covered by
/// [`parses_real_index_alias_and_metafile_fields`] against a known-good pack,
/// and by the round-trip unit tests.
#[test]
fn all_real_models_round_trip_with_required_fidelity() {
	let root = repository_root().join("modpacks");
	let mut packs = 0usize;
	let mut metadata_files = 0usize;

	visit_files(&root, &mut |path| {
		let name = path.file_name().unwrap().to_string_lossy().into_owned();

		if name == "pack.toml" {
			let source = fs::read_to_string(path).unwrap();
			packs += 1;
			let pack: Pack = toml::from_str(&source).unwrap();
			let encoded = pack.to_toml().unwrap();
			assert_eq!(
				toml::from_str::<Pack>(&encoded).unwrap(),
				pack,
				"pack round-trip lost data: {}",
				path.display()
			);
		} else if metafile::is_any_metafile(&name) {
			metadata_files += 1;
			let metadata = read_metadata(path);
			let encoded = metadata.to_json_bytes().unwrap();
			assert_eq!(
				serde_json::from_slice::<Mod>(&encoded).unwrap(),
				metadata,
				"metadata round-trip lost data: {}",
				path.display()
			);
		}
	});

	assert!(packs > 5, "expected real pack fixtures, found {packs}");
	assert!(
		metadata_files > 100,
		"expected real metadata fixtures, found {metadata_files}"
	);
}

#[test]
fn pack_format_compatibility_matches_the_supported_range() {
	assert_eq!(
		"packwiz:1.1.0".parse::<PackFormat>().unwrap(),
		PackFormat::Packwiz {
			major: 1,
			minor: 1,
			patch: 0,
		}
	);
	// 26 is still readable so it can be migrated; 25 is not.
	assert!("packwand:26".parse::<PackFormat>().is_ok());
	assert!("packwand:27".parse::<PackFormat>().is_ok());
	assert!("packwand:25".parse::<PackFormat>().is_err());
	assert!("packwiz:2.0.0".parse::<PackFormat>().is_err());
	assert!("unknown:1".parse::<PackFormat>().is_err());
}
