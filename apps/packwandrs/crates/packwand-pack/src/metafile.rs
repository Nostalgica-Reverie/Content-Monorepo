//! Where the per-mod metadata file extension is defined.
//!
//! Generation 27 moved this metadata from TOML to JSON: `sodium.pw.toml`
//! became `sodium.pw.json`. Before that change the suffix was a bare string
//! literal in roughly twenty-five places across a dozen files, and every one of
//! them had to agree — a single missed literal produces a pack whose files are
//! silently invisible to one command and present to another, which is far
//! harder to notice than a build error. So the suffix lives here and nowhere
//! else.
//!
//! [`LEGACY_EXTENSION`] is retained on purpose: `packwand migrate format` has
//! to *find* generation-26 files in order to convert them, and diagnostics want
//! to say "this pack has not been migrated" rather than "this pack is empty".
//! Nothing outside migration and diagnostics should reference it.

use std::path::Path;

/// The metadata suffix packwand writes and reads.
pub const EXTENSION: &str = ".pw.json";

/// The generation-26 suffix. Recognized only so it can be migrated away.
pub const LEGACY_EXTENSION: &str = ".pw.toml";

/// Is this a current-generation metadata file?
#[must_use]
pub fn is_metafile(path: impl AsRef<Path>) -> bool {
	has_suffix(path.as_ref(), EXTENSION)
}

/// Is this a generation-26 metadata file that still needs migrating?
#[must_use]
pub fn is_legacy_metafile(path: impl AsRef<Path>) -> bool {
	has_suffix(path.as_ref(), LEGACY_EXTENSION)
}

/// Either generation — used when a command only needs to know that a path
/// *is* mod metadata, regardless of which format it is stored in.
#[must_use]
pub fn is_any_metafile(path: impl AsRef<Path>) -> bool {
	let path = path.as_ref();
	is_metafile(path) || is_legacy_metafile(path)
}

/// The metadata filename for a slug: `sodium` becomes `sodium.pw.json`.
#[must_use]
pub fn name_for(slug: &str) -> String {
	format!("{slug}{EXTENSION}")
}

/// Is this a generation-26 index filename?
///
/// Used by migration to decide whether a pack's `[index] file` still needs
/// repointing. Compared as a filename rather than a full path because the
/// value comes from `pack.toml`, where it is pack-relative.
#[must_use]
pub fn is_legacy_index(file: &str) -> bool {
	file.rsplit(['/', '\\']).next() == Some(LEGACY_INDEX_FILE)
}

/// The generated index filename packwand writes.
pub const INDEX_FILE: &str = "index.json";

/// The generation-26 index filename.
pub const LEGACY_INDEX_FILE: &str = "index.toml";

/// The current-generation path a legacy metadata file migrates to.
///
/// Returns `None` for anything that is not a legacy metadata path, so callers
/// cannot accidentally "migrate" an unrelated file.
#[must_use]
pub fn migrated_path(path: &Path) -> Option<std::path::PathBuf> {
	let name = path.file_name()?.to_str()?;
	let slug = name.strip_suffix(LEGACY_EXTENSION)?;
	Some(path.with_file_name(name_for(slug)))
}

/// Compares against the whole filename rather than `Path::extension`, which
/// would only ever see `json`/`toml` — the suffix here is two components.
fn has_suffix(path: &Path, suffix: &str) -> bool {
	path.file_name()
		.and_then(|name| name.to_str())
		.is_some_and(|name| name.ends_with(suffix))
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	#[test]
	fn recognizes_current_metadata_files() {
		assert!(is_metafile("mods/sodium.pw.json"));
		assert!(is_metafile(PathBuf::from(r"mods\sodium.pw.json")));
		assert!(!is_metafile("mods/sodium.pw.toml"));
	}

	/// `Path::extension` returns `json` for both `sodium.pw.json` and a plain
	/// `config.json`, so matching on it would sweep ordinary pack content into
	/// the metadata set.
	#[test]
	fn a_plain_json_file_is_not_metadata() {
		assert!(!is_metafile("config/sodium.json"));
		assert!(!is_metafile("pack.mcmeta"));
		assert!(!is_any_metafile("config/some.json"));
	}

	#[test]
	fn recognizes_legacy_metadata_for_migration() {
		assert!(is_legacy_metafile("mods/sodium.pw.toml"));
		assert!(!is_legacy_metafile("mods/sodium.pw.json"));
		assert!(is_any_metafile("mods/sodium.pw.toml"));
		assert!(is_any_metafile("mods/sodium.pw.json"));
	}

	#[test]
	fn builds_names_from_slugs() {
		assert_eq!(name_for("sodium"), "sodium.pw.json");
	}

	#[test]
	fn migration_maps_legacy_paths_and_refuses_everything_else() {
		assert_eq!(
			migrated_path(Path::new("mods/sodium.pw.toml")),
			Some(PathBuf::from("mods/sodium.pw.json"))
		);
		// Already migrated, or never metadata: no target.
		assert_eq!(migrated_path(Path::new("mods/sodium.pw.json")), None);
		assert_eq!(migrated_path(Path::new("pack.toml")), None);
		assert_eq!(migrated_path(Path::new("mods")), None);
	}

	/// Slugs containing dots are common (`appleskin.pw.toml`, `do-a-barrel-roll`),
	/// so the suffix strip must take the whole two-part suffix, not the last dot.
	#[test]
	fn slugs_containing_dots_survive_the_round_trip() {
		assert_eq!(
			migrated_path(Path::new("mods/some.mod.name.pw.toml")),
			Some(PathBuf::from("mods/some.mod.name.pw.json"))
		);
	}
}
