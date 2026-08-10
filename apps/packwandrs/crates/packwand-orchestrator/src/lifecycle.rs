use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::boot::PackTarget;
use packwand_instance::{FsUserInstanceRepository, Instance, InstanceSource};
use serde::{Deserialize, Deserializer};

use crate::error::{OrchestratorError, Result};
use crate::paths::now_ms;

/// Where a new instance's content comes from.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateSource {
	/// Points at a workspace pack, which stays the source of truth.
	Linked,
	/// Owns a private pack under the instance directory.
	Owned,
}

/// The request behind "New instance".
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpec {
	pub name: String,
	pub source: CreateSource,
	pub pack_id: Option<String>,
	pub game_version: Option<String>,
	pub loader: Option<String>,
	pub loader_version: Option<String>,
}

/// A field-aware patch: missing leaves a value alone, `null` clears it.
///
/// `Option<T>` alone cannot express this — serde cannot tell an absent field
/// from an explicit `null`, so clearing an override would be indistinguishable
/// from not mentioning it.
#[derive(Debug, Default)]
pub enum Patch<T> {
	#[default]
	Missing,
	Value(Option<T>),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Patch<T> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
		Option::<T>::deserialize(deserializer).map(Patch::Value)
	}
}

/// Per-field edits to an instance's launch settings.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SettingsPatch {
	pub java_path: Patch<PathBuf>,
	pub memory_min_mb: Patch<u32>,
	pub memory_max_mb: Patch<u32>,
	pub extra_jvm_args: Patch<Vec<String>>,
	pub extra_game_args: Patch<Vec<String>>,
	pub env: Patch<BTreeMap<String, String>>,
	pub window_width: Patch<u32>,
	pub window_height: Patch<u32>,
	pub fullscreen: Patch<bool>,
	pub download_jobs: Patch<usize>,
}

/// Per-field edits to an instance.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct InstancePatch {
	pub name: Patch<String>,
	pub icon: Patch<String>,
	pub group: Patch<String>,
	pub settings: Option<SettingsPatch>,
}

fn apply<T>(target: &mut Option<T>, patch: Patch<T>) {
	if let Patch::Value(value) = patch {
		*target = value;
	}
}

/// Writes the minimal pack a standalone instance owns, so the whole content
/// pipeline — add, remove, refresh, export — works on it unchanged.
pub fn create_owned_pack(
	directory: &Path,
	name: &str,
	game_version: &str,
	loader: &str,
	loader_version: Option<&str>,
) -> Result<()> {
	std::fs::create_dir_all(directory)?;
	let mut versions = BTreeMap::from([("minecraft".to_owned(), game_version.to_owned())]);
	if loader != "vanilla" && !loader.is_empty() {
		versions.insert(
			loader.to_owned(),
			loader_version.unwrap_or("latest").to_owned(),
		);
	}
	let pack = packwand_pack::Pack {
		name: name.to_owned(),
		version: "1.0.0".to_owned(),
		pack_format: packwand_pack::CURRENT_PACK_FORMAT.to_owned(),
		versions,
		..Default::default()
	};
	let pack_toml =
		toml::to_string_pretty(&pack).map_err(|error| OrchestratorError::new("pack", error))?;
	std::fs::write(directory.join("pack.toml"), pack_toml)?;
	std::fs::write(
		directory.join(packwand_pack::metafile::INDEX_FILE),
		serde_json::to_vec_pretty(&packwand_pack::Index::default())?,
	)?;
	std::fs::write(directory.join(".packwizignore"), "logs\n*.zip\n*.mrpack\n")?;
	Ok(())
}

/// Creates an instance record, and for a standalone one its backing pack.
///
/// `resolve_pack` turns a workspace pack id into a directory; the caller owns
/// that lookup because it needs the workspace, which this crate deliberately
/// does not know about.
pub fn create(
	repo: &FsUserInstanceRepository,
	spec: CreateSpec,
	resolve_pack: impl FnOnce(&str) -> Result<PathBuf>,
) -> Result<Instance> {
	let name = spec.name.trim();
	if name.is_empty() {
		return Err(OrchestratorError::new(
			"validation",
			"instance name must not be empty",
		));
	}
	let id = repo.available_id(name);
	let (source, target) = match spec.source {
		CreateSource::Linked => {
			let pack_id = spec.pack_id.ok_or_else(|| {
				OrchestratorError::new("validation", "a linked instance requires packId")
			})?;
			let pack_dir = resolve_pack(&pack_id)?;
			let target = crate::boot::resolve_pack_target(&pack_dir.join("pack.toml"))
				.map_err(|error| OrchestratorError::new("pack", error))?;
			(InstanceSource::Linked { pack_dir }, target)
		}
		CreateSource::Owned => {
			let game_version = spec.game_version.ok_or_else(|| {
				OrchestratorError::new("validation", "a standalone instance requires gameVersion")
			})?;
			let loader = spec.loader.unwrap_or_else(|| "vanilla".to_owned());
			let target = PackTarget {
				minecraft: game_version,
				loader: (loader != "vanilla").then_some(loader),
				loader_version: spec.loader_version,
			};
			(InstanceSource::Owned, target)
		}
	};
	let loader = target
		.loader
		.clone()
		.unwrap_or_else(|| "vanilla".to_owned());
	let instance = Instance::new(
		id.clone(),
		name.to_owned(),
		source,
		target.minecraft.clone(),
		loader.clone(),
		target.loader_version.clone(),
		now_ms(),
	);
	repo.create(&instance)?;
	if matches!(instance.source, InstanceSource::Owned) {
		let pack_dir = repo.owned_pack_dir(&id)?;
		if let Err(error) = create_owned_pack(
			&pack_dir,
			&instance.name,
			&target.minecraft,
			&loader,
			target.loader_version.as_deref(),
		) {
			// A record whose pack failed to materialize is not usable, and
			// leaving it would show a permanently broken card.
			let _ = repo.delete(&id, true);
			return Err(error);
		}
	}
	Ok(instance)
}

/// Applies a patch and persists the result.
pub fn edit(repo: &FsUserInstanceRepository, id: &str, patch: InstancePatch) -> Result<Instance> {
	let mut instance = repo.get(id)?;
	if let Patch::Value(Some(name)) = patch.name {
		if name.trim().is_empty() {
			return Err(OrchestratorError::new(
				"validation",
				"instance name must not be empty",
			));
		}
		instance.name = name;
	}
	apply(&mut instance.icon, patch.icon);
	apply(&mut instance.group, patch.group);
	if let Some(settings) = patch.settings {
		apply(&mut instance.settings.java_path, settings.java_path);
		apply(&mut instance.settings.memory_min_mb, settings.memory_min_mb);
		apply(&mut instance.settings.memory_max_mb, settings.memory_max_mb);
		apply(
			&mut instance.settings.extra_jvm_args,
			settings.extra_jvm_args,
		);
		apply(
			&mut instance.settings.extra_game_args,
			settings.extra_game_args,
		);
		apply(&mut instance.settings.env, settings.env);
		apply(&mut instance.settings.window_width, settings.window_width);
		apply(&mut instance.settings.window_height, settings.window_height);
		apply(&mut instance.settings.fullscreen, settings.fullscreen);
		apply(&mut instance.settings.download_jobs, settings.download_jobs);
	}
	repo.write(&instance)?;
	Ok(instance)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn an_absent_patch_field_differs_from_an_explicit_null() {
		let repo = FsUserInstanceRepository::new(tempfile::tempdir().unwrap().keep());
		let instance = create(
			&repo,
			CreateSpec {
				name: "Patchy".into(),
				source: CreateSource::Owned,
				pack_id: None,
				game_version: Some("1.21.1".into()),
				loader: Some("fabric".into()),
				loader_version: None,
			},
			|_| unreachable!("standalone instances resolve no pack"),
		)
		.unwrap();

		let set: InstancePatch = serde_json::from_str(r#"{"group":"Testing"}"#).unwrap();
		let edited = edit(&repo, &instance.id, set).unwrap();
		assert_eq!(edited.group.as_deref(), Some("Testing"));

		// Absent: leave it.
		let untouched: InstancePatch = serde_json::from_str(r#"{"name":"Renamed"}"#).unwrap();
		let edited = edit(&repo, &instance.id, untouched).unwrap();
		assert_eq!(edited.group.as_deref(), Some("Testing"));
		assert_eq!(edited.name, "Renamed");

		// Explicit null: clear it.
		let cleared: InstancePatch = serde_json::from_str(r#"{"group":null}"#).unwrap();
		assert_eq!(edit(&repo, &instance.id, cleared).unwrap().group, None);
	}

	#[test]
	fn a_standalone_instance_gets_a_usable_backing_pack() {
		let repo = FsUserInstanceRepository::new(tempfile::tempdir().unwrap().keep());
		let instance = create(
			&repo,
			CreateSpec {
				name: "Owned".into(),
				source: CreateSource::Owned,
				pack_id: None,
				game_version: Some("1.21.1".into()),
				loader: Some("fabric".into()),
				loader_version: Some("0.16.9".into()),
			},
			|_| unreachable!(),
		)
		.unwrap();
		let pack = repo.owned_pack_dir(&instance.id).unwrap();
		assert!(pack.join("pack.toml").is_file());
		assert!(pack.join(packwand_pack::metafile::INDEX_FILE).is_file());

		let target = crate::boot::resolve_pack_target(&pack.join("pack.toml")).unwrap();
		assert_eq!(target.minecraft, "1.21.1");
		assert_eq!(target.loader.as_deref(), Some("fabric"));
	}

	#[test]
	fn a_blank_name_is_refused_before_anything_is_written() {
		let repo = FsUserInstanceRepository::new(tempfile::tempdir().unwrap().keep());
		let error = create(
			&repo,
			CreateSpec {
				name: "   ".into(),
				source: CreateSource::Owned,
				pack_id: None,
				game_version: Some("1.21.1".into()),
				loader: None,
				loader_version: None,
			},
			|_| unreachable!(),
		)
		.unwrap_err();
		assert_eq!(error.kind, "validation");
		assert!(repo.list().unwrap().is_empty());
	}
}
