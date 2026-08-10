//! Persistent, user-owned Minecraft instances.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use packwand_instance::{
	FsUserInstanceRepository, InstallStage, Instance, InstanceSettings, InstanceSource,
};
use packwand_launch::LaunchEvent;
use packwand_providers::{
	CurseForgeClient, ProviderResolver, ResolveRequest, UreqTransport, configured_api_key,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::RwLock;
use walkdir::WalkDir;

use crate::commands::jobs::JobRecord;
use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::events::emit_instance_status;
use crate::state::AppState;

fn domain_error(kind: &str, error: impl ToString) -> SerializableError {
	SerializableError::new(kind, error.to_string())
}

fn now_ms() -> u64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap_or_default()
		.as_millis() as u64
}

fn repository(app: &AppHandle) -> CommandResult<FsUserInstanceRepository> {
	let root = app
		.path()
		.app_data_dir()
		.map_err(|error| domain_error("path", error))?;
	Ok(FsUserInstanceRepository::new(root))
}

fn backing_pack(repo: &FsUserInstanceRepository, instance: &Instance) -> CommandResult<PathBuf> {
	match &instance.source {
		InstanceSource::Linked { pack_dir } => Ok(pack_dir.clone()),
		InstanceSource::Owned => repo
			.owned_pack_dir(&instance.id)
			.map_err(|error| domain_error("instance", error)),
	}
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatusPayload {
	pub id: String,
	pub phase: String,
	pub message: Option<String>,
	pub job_id: Option<String>,
	pub exit_code: Option<i32>,
}

#[derive(Clone, Default)]
pub struct InstanceRegistry {
	entries: Arc<RwLock<HashMap<String, InstanceStatusPayload>>>,
}

impl InstanceRegistry {
	pub async fn set(&self, payload: InstanceStatusPayload) {
		self.entries
			.write()
			.await
			.insert(payload.id.clone(), payload);
	}
	pub async fn list(&self) -> Vec<InstanceStatusPayload> {
		self.entries.read().await.values().cloned().collect()
	}
	pub async fn job_id_for(&self, id: &str) -> Option<String> {
		self.entries
			.read()
			.await
			.get(id)
			.filter(|entry| matches!(entry.phase.as_str(), "starting" | "running"))
			.and_then(|entry| entry.job_id.clone())
	}
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateInstanceSource {
	Linked,
	Owned,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceSpec {
	pub name: String,
	pub source: CreateInstanceSource,
	pub pack_id: Option<String>,
	pub game_version: Option<String>,
	pub loader: Option<String>,
	pub loader_version: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveFormat {
	Modrinth,
	CurseForge,
}

fn create_owned_pack(
	directory: &Path,
	name: &str,
	game_version: &str,
	loader: &str,
	loader_version: Option<&str>,
) -> CommandResult<()> {
	fs::create_dir_all(directory)?;
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
	let pack_toml = toml::to_string_pretty(&pack).map_err(|error| domain_error("pack", error))?;
	fs::write(directory.join("pack.toml"), pack_toml)?;
	fs::write(
		directory.join(packwand_pack::metafile::INDEX_FILE),
		serde_json::to_vec_pretty(&packwand_pack::Index::default())?,
	)?;
	fs::write(directory.join(".packwizignore"), "logs\n*.zip\n*.mrpack\n")?;
	Ok(())
}

#[tauri::command]
pub async fn instances_list(app: AppHandle) -> CommandResult<Vec<Instance>> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || {
		repo.list().map_err(|error| domain_error("instance", error))
	})
	.await
}

#[tauri::command]
pub async fn instances_get(id: String, app: AppHandle) -> CommandResult<Instance> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || {
		repo.get(&id)
			.map_err(|error| domain_error("instance", error))
	})
	.await
}

#[tauri::command]
pub async fn instances_icon(id: String, app: AppHandle) -> CommandResult<Option<Vec<u8>>> {
	instances_image_inner(id, InstanceImageKind::Icon, app).await
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceImageKind {
	Icon,
	Background,
}

fn read_instance_image(
	repo: &FsUserInstanceRepository,
	id: &str,
	kind: InstanceImageKind,
) -> CommandResult<Option<Vec<u8>>> {
	let instance = repo
		.get(id)
		.map_err(|error| domain_error("instance", error))?;
	let instance_root = repo
		.instance_dir(id)
		.map_err(|error| domain_error("instance", error))?;
	let pack_root = backing_pack(repo, &instance)?;
	let mut candidates = Vec::new();
	if matches!(kind, InstanceImageKind::Icon)
		&& let Some(icon) = &instance.icon
	{
		candidates.push(safe_content_path(&instance_root, icon)?);
	}
	candidates.push(instance_root.join(match kind {
		InstanceImageKind::Icon => "icon.png",
		InstanceImageKind::Background => "bg.png",
	}));
	candidates.push(pack_root.join(match kind {
		InstanceImageKind::Icon => "icon.png",
		InstanceImageKind::Background => "bg.png",
	}));
	for path in candidates {
		match fs::read(&path) {
			Ok(bytes) => return Ok(Some(bytes)),
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
			Err(error) => return Err(error.into()),
		}
	}
	Ok(None)
}

async fn instances_image_inner(
	id: String,
	kind: InstanceImageKind,
	app: AppHandle,
) -> CommandResult<Option<Vec<u8>>> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || read_instance_image(&repo, &id, kind)).await
}

#[tauri::command]
pub async fn instances_image(
	id: String,
	kind: InstanceImageKind,
	app: AppHandle,
) -> CommandResult<Option<Vec<u8>>> {
	instances_image_inner(id, kind, app).await
}

#[tauri::command]
pub async fn instances_create(
	spec: CreateInstanceSpec,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<Instance> {
	let repo = repository(&app)?;
	let workspace = state.workspace()?;
	crate::commands::off_thread(move || {
		let name = spec.name.trim();
		if name.is_empty() {
			return Err(domain_error(
				"validation",
				"instance name must not be empty",
			));
		}
		let id = repo.available_id(name);
		let (source, target) = match spec.source {
			CreateInstanceSource::Linked => {
				let pack_id = spec.pack_id.ok_or_else(|| {
					domain_error("validation", "a linked instance requires packId")
				})?;
				let pack_dir = pack_root(&workspace, &pack_id)?;
				let target = packwand_devboot::resolve_pack_target(&pack_dir.join("pack.toml"))
					.map_err(|error| domain_error("pack", error))?;
				(InstanceSource::Linked { pack_dir }, target)
			}
			CreateInstanceSource::Owned => {
				let game_version = spec.game_version.ok_or_else(|| {
					domain_error("validation", "a standalone instance requires gameVersion")
				})?;
				let loader = spec.loader.unwrap_or_else(|| "vanilla".to_owned());
				let target = packwand_devboot::PackTarget {
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
		repo.create(&instance)
			.map_err(|error| domain_error("instance", error))?;
		if matches!(instance.source, InstanceSource::Owned) {
			let pack_dir = repo
				.owned_pack_dir(&id)
				.map_err(|error| domain_error("instance", error))?;
			if let Err(error) = create_owned_pack(
				&pack_dir,
				&instance.name,
				&target.minecraft,
				&loader,
				target.loader_version.as_deref(),
			) {
				let _ = repo.delete(&id, true);
				return Err(error);
			}
		}
		Ok(instance)
	})
	.await
}

/// A field-aware patch: missing leaves a value alone, `null` clears it.
#[derive(Debug, Default)]
pub enum Patch<T> {
	#[default]
	Missing,
	Value(Option<T>),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Patch<T> {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		Option::<T>::deserialize(deserializer).map(Patch::Value)
	}
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct InstanceSettingsPatch {
	java_path: Patch<PathBuf>,
	memory_min_mb: Patch<u32>,
	memory_max_mb: Patch<u32>,
	extra_jvm_args: Patch<Vec<String>>,
	extra_game_args: Patch<Vec<String>>,
	env: Patch<BTreeMap<String, String>>,
	window_width: Patch<u32>,
	window_height: Patch<u32>,
	fullscreen: Patch<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct InstancePatch {
	name: Patch<String>,
	icon: Patch<String>,
	group: Patch<String>,
	settings: Option<InstanceSettingsPatch>,
}

fn apply_patch<T>(target: &mut Option<T>, patch: Patch<T>) {
	if let Patch::Value(value) = patch {
		*target = value;
	}
}

#[tauri::command]
pub async fn instances_edit(
	id: String,
	patch: InstancePatch,
	app: AppHandle,
) -> CommandResult<Instance> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || {
		let mut instance = repo
			.get(&id)
			.map_err(|error| domain_error("instance", error))?;
		if let Patch::Value(Some(name)) = patch.name {
			if name.trim().is_empty() {
				return Err(domain_error(
					"validation",
					"instance name must not be empty",
				));
			}
			instance.name = name;
		}
		apply_patch(&mut instance.icon, patch.icon);
		apply_patch(&mut instance.group, patch.group);
		if let Some(settings) = patch.settings {
			apply_patch(&mut instance.settings.java_path, settings.java_path);
			apply_patch(&mut instance.settings.memory_min_mb, settings.memory_min_mb);
			apply_patch(&mut instance.settings.memory_max_mb, settings.memory_max_mb);
			apply_patch(
				&mut instance.settings.extra_jvm_args,
				settings.extra_jvm_args,
			);
			apply_patch(
				&mut instance.settings.extra_game_args,
				settings.extra_game_args,
			);
			apply_patch(&mut instance.settings.env, settings.env);
			apply_patch(&mut instance.settings.window_width, settings.window_width);
			apply_patch(&mut instance.settings.window_height, settings.window_height);
			apply_patch(&mut instance.settings.fullscreen, settings.fullscreen);
		}
		repo.write(&instance)
			.map_err(|error| domain_error("instance", error))?;
		Ok(instance)
	})
	.await
}

#[tauri::command]
pub async fn instances_delete(
	id: String,
	delete_files: bool,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	if state.instances.job_id_for(&id).await.is_some() {
		return Err(domain_error(
			"instance_running",
			"stop the instance before deleting it",
		));
	}
	let repo = repository(&app)?;
	crate::commands::off_thread(move || {
		repo.delete(&id, delete_files)
			.map_err(|error| domain_error("instance", error))
	})
	.await
}

fn bundled_installer_candidates(resource_dir: &Path) -> Vec<PathBuf> {
	let executable = if cfg!(windows) {
		"packwand-installer.exe"
	} else {
		"packwand-installer"
	};
	vec![
		resource_dir.join("resources").join(executable),
		resource_dir.join(executable),
	]
}

fn find_installer_binary(app: &AppHandle) -> Option<PathBuf> {
	let resource_dir = app.path().resource_dir().ok()?;
	bundled_installer_candidates(&resource_dir)
		.into_iter()
		.find(|path| path.is_file())
}

fn disabled_files(root: &Path) -> Vec<PathBuf> {
	WalkDir::new(root)
		.follow_links(false)
		.into_iter()
		.filter_map(Result::ok)
		.filter(|entry| entry.file_type().is_file())
		.map(|entry| entry.into_path())
		.filter(|path| {
			path.extension()
				.is_some_and(|extension| extension == "disabled")
		})
		.collect()
}

fn restore_disabled(root: &Path, disabled: &[PathBuf]) -> CommandResult<()> {
	for path in disabled {
		if !path.starts_with(root) || !path.is_file() {
			continue;
		}
		let enabled = path.with_extension("");
		if enabled.is_file() {
			fs::remove_file(enabled)?;
		}
	}
	Ok(())
}

fn install_instance(
	repo: &FsUserInstanceRepository,
	id: &str,
	installer_binary: Option<&Path>,
) -> CommandResult<Instance> {
	install_instance_with(repo, id, |pack_dir, game_dir| {
		packwand_build::install_with_native_installer(pack_dir, installer_binary, game_dir)
			.map(|_| ())
			.map_err(|error| domain_error("installer", error))
	})
}

fn install_instance_with(
	repo: &FsUserInstanceRepository,
	id: &str,
	install: impl FnOnce(&Path, &Path) -> CommandResult<()>,
) -> CommandResult<Instance> {
	let mut instance = repo
		.get(id)
		.map_err(|error| domain_error("instance", error))?;
	instance.stage = InstallStage::Installing;
	repo.write(&instance)
		.map_err(|error| domain_error("instance", error))?;
	let game_dir = repo
		.instance_dir(id)
		.map_err(|error| domain_error("instance", error))?;
	let pack_dir = backing_pack(repo, &instance)?;
	let disabled = disabled_files(&game_dir);
	let result = install(&pack_dir, &game_dir).and_then(|_| restore_disabled(&game_dir, &disabled));
	match result {
		Ok(()) => instance.stage = InstallStage::Ready,
		Err(error) => {
			instance.stage = InstallStage::Failed {
				message: error.message.clone(),
			};
			let _ = repo.write(&instance);
			return Err(error);
		}
	}
	repo.write(&instance)
		.map_err(|error| domain_error("instance", error))?;
	Ok(instance)
}

#[tauri::command]
pub async fn instances_install(
	id: String,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let repo = repository(&app)?;
	repo.get(&id)
		.map_err(|error| domain_error("instance", error))?;
	let installer = find_installer_binary(&app);
	let label = format!("Install {id}");
	let registry = state.instances.clone();
	let install_app = app.clone();
	let install_id = id.clone();
	let job = state
		.jobs
		.spawn(app, "instance.install", label, move |context| async move {
			let started = InstanceStatusPayload {
				id: install_id.clone(),
				phase: "starting".to_owned(),
				message: Some("Installing pack content".to_owned()),
				job_id: Some(context.id().to_owned()),
				exit_code: None,
			};
			registry.set(started.clone()).await;
			let _ = emit_instance_status(&install_app, started);
			context
				.progress(0.05, Some("Installing pack content".to_owned()))
				.await;
			let result = tokio::task::spawn_blocking(move || {
				install_instance(&repo, &id, installer.as_deref())
			})
			.await
			.map_err(|error| domain_error("task", error))?;
			if let Err(error) = result {
				let failed = InstanceStatusPayload {
					id: install_id.clone(),
					phase: "error".to_owned(),
					message: Some(error.message.clone()),
					job_id: Some(context.id().to_owned()),
					exit_code: None,
				};
				registry.set(failed.clone()).await;
				let _ = emit_instance_status(&install_app, failed);
				return Err(error);
			}
			context
				.progress(1.0, Some("Instance ready".to_owned()))
				.await;
			let ready = InstanceStatusPayload {
				id: install_id,
				phase: "stopped".to_owned(),
				message: Some("Ready".to_owned()),
				job_id: Some(context.id().to_owned()),
				exit_code: None,
			};
			registry.set(ready.clone()).await;
			let _ = emit_instance_status(&install_app, ready);
			Ok(())
		})
		.await;
	Ok(job)
}

/// A CurseForge file the author has excluded from third-party distribution
/// — the install still finishes, but this one mod needs a human.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingManualDownload {
	name: String,
	/// Instance-relative, matching every other content path in this file.
	target: String,
	page_url: Option<String>,
}

fn manual_pending_list(
	repo: &FsUserInstanceRepository,
	id: &str,
) -> CommandResult<Vec<PendingManualDownload>> {
	repo.get(id).map_err(|error| domain_error("instance", error))?;
	let game_dir = repo
		.instance_dir(id)
		.map_err(|error| domain_error("instance", error))?;
	let pending =
		packwand_build::manual_pending(&game_dir).map_err(|error| domain_error("installer", error))?;
	Ok(pending
		.into_iter()
		.map(|entry| PendingManualDownload {
			name: entry.name,
			target: normalized(entry.target.strip_prefix(&game_dir).unwrap_or(&entry.target)),
			page_url: entry.page_url,
		})
		.collect())
}

/// Mods left over from the last install that CurseForge's API won't serve
/// (third-party distribution disabled by the author). The install itself
/// already succeeded; these still need a human to place by hand.
#[tauri::command]
pub async fn instances_manual_pending(
	id: String,
	app: AppHandle,
) -> CommandResult<Vec<PendingManualDownload>> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || manual_pending_list(&repo, &id)).await
}

/// Prism-style "I already downloaded it": opens a native file picker, and if
/// the user selects a file, verifies it against the pending mod's expected
/// hash and copies it into place. Returns `false` if the dialog was
/// cancelled rather than erroring.
#[tauri::command]
pub async fn instances_manual_provide(
	id: String,
	target: String,
	app: AppHandle,
) -> CommandResult<bool> {
	let repo = repository(&app)?;
	let game_dir = repo
		.instance_dir(&id)
		.map_err(|error| domain_error("instance", error))?;
	safe_content_path(&game_dir, &target)?;
	let selected = app.dialog().file().blocking_pick_file();
	let Some(selected) = selected else {
		return Ok(false);
	};
	let source = selected
		.into_path()
		.map_err(|error| domain_error("invalid_path", error))?;
	crate::commands::off_thread(move || {
		let pending = packwand_build::manual_pending(&game_dir)
			.map_err(|error| domain_error("installer", error))?;
		let entry = pending
			.into_iter()
			.find(|entry| {
				normalized(entry.target.strip_prefix(&game_dir).unwrap_or(&entry.target)) == target
			})
			.ok_or_else(|| domain_error("not_found", "no pending manual download at that path"))?;
		packwand_build::provide_manual_download(&source, &entry)
			.map_err(|error| domain_error("installer", error))?;
		Ok(true)
	})
	.await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceContent {
	path: String,
	name: String,
	enabled: bool,
	pack_sourced: bool,
	bytes: u64,
}

fn normalized(path: &Path) -> String {
	path.to_string_lossy().replace('\\', "/")
}

fn pack_content_paths(pack_dir: &Path) -> CommandResult<BTreeSet<String>> {
	let pack: packwand_pack::Pack =
		toml::from_str(&fs::read_to_string(pack_dir.join("pack.toml"))?)?;
	let index: packwand_pack::Index =
		serde_json::from_slice(&fs::read(pack_dir.join(&pack.index.file))?)?;
	let mut paths = BTreeSet::new();
	for entry in index.files {
		if entry.metafile && packwand_pack::metafile::is_metafile(&entry.file) {
			let metadata: packwand_pack::Mod =
				serde_json::from_slice(&fs::read(pack_dir.join(&entry.file))?)?;
			let parent = Path::new(&entry.file)
				.parent()
				.unwrap_or_else(|| Path::new(""));
			paths.insert(normalized(&parent.join(metadata.filename)));
		} else {
			paths.insert(normalized(Path::new(&entry.file)));
		}
	}
	Ok(paths)
}

fn content_list(repo: &FsUserInstanceRepository, id: &str) -> CommandResult<Vec<InstanceContent>> {
	let instance = repo
		.get(id)
		.map_err(|error| domain_error("instance", error))?;
	let root = repo
		.instance_dir(id)
		.map_err(|error| domain_error("instance", error))?;
	let sourced = pack_content_paths(&backing_pack(repo, &instance)?)?;
	let mut items = Vec::new();
	for directory in ["mods", "resourcepacks", "shaderpacks", "datapacks"] {
		let start = root.join(directory);
		if !start.is_dir() {
			continue;
		}
		for entry in WalkDir::new(start)
			.follow_links(false)
			.into_iter()
			.filter_map(Result::ok)
		{
			if !entry.file_type().is_file() {
				continue;
			}
			let relative = entry.path().strip_prefix(&root).unwrap_or(entry.path());
			let raw = normalized(relative);
			let enabled = !raw.ends_with(".disabled");
			let logical = raw.strip_suffix(".disabled").unwrap_or(&raw).to_owned();
			items.push(InstanceContent {
				path: raw,
				name: Path::new(&logical)
					.file_name()
					.unwrap_or_default()
					.to_string_lossy()
					.into_owned(),
				enabled,
				pack_sourced: sourced.contains(&logical),
				bytes: entry.metadata().map(|metadata| metadata.len()).unwrap_or(0),
			});
		}
	}
	items.sort_by(|left, right| left.path.cmp(&right.path));
	Ok(items)
}

#[tauri::command]
pub async fn instances_content_list(
	id: String,
	app: AppHandle,
) -> CommandResult<Vec<InstanceContent>> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || content_list(&repo, &id)).await
}

fn safe_content_path(root: &Path, relative: &str) -> CommandResult<PathBuf> {
	let relative = Path::new(relative);
	if relative.as_os_str().is_empty()
		|| relative.is_absolute()
		|| relative
			.components()
			.any(|part| !matches!(part, Component::Normal(_)))
	{
		return Err(domain_error(
			"unsafe_path",
			"content path must be instance-relative",
		));
	}
	let target = root.join(relative);
	if !target.starts_with(root) {
		return Err(domain_error(
			"unsafe_path",
			"content path leaves the instance",
		));
	}
	Ok(target)
}

#[tauri::command]
pub async fn instances_content_toggle(
	id: String,
	path: String,
	app: AppHandle,
) -> CommandResult<String> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || {
		repo.get(&id)
			.map_err(|error| domain_error("instance", error))?;
		let root = repo
			.instance_dir(&id)
			.map_err(|error| domain_error("instance", error))?;
		let source = safe_content_path(&root, &path)?;
		if !source.is_file() {
			return Err(domain_error("not_found", "content file was not found"));
		}
		let target = if source
			.extension()
			.is_some_and(|extension| extension == "disabled")
		{
			source.with_extension("")
		} else {
			PathBuf::from(format!("{}.disabled", source.display()))
		};
		fs::rename(&source, &target)?;
		Ok(normalized(target.strip_prefix(&root).unwrap_or(&target)))
	})
	.await
}

#[tauri::command]
pub async fn instances_content_remove(
	id: String,
	path: String,
	app: AppHandle,
) -> CommandResult<()> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || {
		repo.get(&id)
			.map_err(|error| domain_error("instance", error))?;
		let root = repo
			.instance_dir(&id)
			.map_err(|error| domain_error("instance", error))?;
		let target = safe_content_path(&root, &path)?;
		if !target.is_file() {
			return Err(domain_error("not_found", "content file was not found"));
		}
		fs::remove_file(target)?;
		Ok(())
	})
	.await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceExportResult {
	path: PathBuf,
	files: usize,
	bytes: u64,
	excluded_hand_added: usize,
}

#[tauri::command]
pub async fn instances_export(
	id: String,
	format: ArchiveFormat,
	output: Option<PathBuf>,
	app: AppHandle,
) -> CommandResult<InstanceExportResult> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || {
		let instance = repo
			.get(&id)
			.map_err(|error| domain_error("instance", error))?;
		let pack = backing_pack(&repo, &instance)?;
		let excluded = content_list(&repo, &id)?
			.into_iter()
			.filter(|item| !item.pack_sourced)
			.count();
		let format = match format {
			ArchiveFormat::Modrinth => packwand_build::ExportFormat::Modrinth,
			ArchiveFormat::CurseForge => packwand_build::ExportFormat::CurseForge,
		};
		let destination = output.unwrap_or_else(|| {
			repo.root()
				.join("exports")
				.join(format!("{}.{}", id, format.extension()))
		});
		let artifact =
			packwand_build::export_pack(&pack, format, Some(&destination), Default::default())
				.map_err(|error| domain_error("export", error))?;
		Ok(InstanceExportResult {
			path: artifact.path,
			files: artifact.files,
			bytes: artifact.bytes,
			excluded_hand_added: excluded,
		})
	})
	.await
}

#[tauri::command]
pub async fn instances_import(
	archive: PathBuf,
	format: ArchiveFormat,
	app: AppHandle,
) -> CommandResult<Instance> {
	let repo = repository(&app)?;
	let installer = find_installer_binary(&app);
	crate::commands::off_thread(move || {
		if !archive.is_file() {
			return Err(domain_error("not_found", "archive was not found"));
		}
		let base = archive
			.file_stem()
			.and_then(|value| value.to_str())
			.unwrap_or("imported-instance");
		let id = repo.available_id(base);
		let pack_dir = repo
			.owned_pack_dir(&id)
			.map_err(|error| domain_error("instance", error))?;
		let imported = match format {
			ArchiveFormat::Modrinth => packwand_build::import_modrinth_archive(&archive, &pack_dir),
			ArchiveFormat::CurseForge => {
				let client = CurseForgeClient::new(UreqTransport::new(), configured_api_key());
				packwand_build::import_curseforge_archive(
					&archive,
					&pack_dir,
					|project_id, file_id| {
						let mut request = ResolveRequest::new(project_id.to_string());
						request.version_id = Some(file_id.to_string());
						let resolved = client
							.resolve(&request)
							.map_err(|error| error.to_string())?;
						let path = resolved.metadata_path();
						let metadata = resolved.into_mod().map_err(|error| error.to_string())?;
						Ok((path, metadata))
					},
				)
			}
		}
		.map_err(|error| domain_error("import", error))?;
		let game_version = imported
			.minecraft_version
			.ok_or_else(|| domain_error("import", "archive has no Minecraft version"))?;
		let loader = imported.loader.unwrap_or_else(|| "vanilla".to_owned());
		let target = packwand_devboot::resolve_pack_target(&pack_dir.join("pack.toml"))
			.map_err(|error| domain_error("import", error))?;
		let instance = Instance::new(
			id,
			imported.name,
			InstanceSource::Owned,
			game_version,
			loader,
			target.loader_version,
			now_ms(),
		);
		repo.create(&instance)
			.map_err(|error| domain_error("instance", error))?;
		install_instance(&repo, &instance.id, installer.as_deref())
	})
	.await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceFileEntry {
	path: String,
	name: String,
	directory: bool,
	size: u64,
}

fn safe_instance_file(root: &Path, relative: &str) -> CommandResult<PathBuf> {
	let target = safe_content_path(root, relative)?;
	let first = Path::new(relative).components().next();
	if relative == "instance.json"
		|| first.is_some_and(|component| component.as_os_str() == ".pack")
	{
		return Err(domain_error(
			"protected_path",
			"instance metadata and the backing pack are not editable game files",
		));
	}
	Ok(target)
}

#[tauri::command]
pub async fn instances_files_list(
	id: String,
	app: AppHandle,
) -> CommandResult<Vec<InstanceFileEntry>> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || {
		repo.get(&id)
			.map_err(|error| domain_error("instance", error))?;
		let root = repo
			.instance_dir(&id)
			.map_err(|error| domain_error("instance", error))?;
		let mut entries = Vec::new();
		for entry in WalkDir::new(&root)
			.min_depth(1)
			.follow_links(false)
			.into_iter()
			.filter_map(Result::ok)
		{
			let relative = normalized(entry.path().strip_prefix(&root).unwrap_or(entry.path()));
			if relative == "instance.json" || relative.starts_with(".pack/") {
				continue;
			}
			let metadata = entry.metadata().ok();
			entries.push(InstanceFileEntry {
				name: entry.file_name().to_string_lossy().into_owned(),
				path: relative,
				directory: entry.file_type().is_dir(),
				size: metadata.map(|value| value.len()).unwrap_or(0),
			});
		}
		entries.sort_by(|left, right| {
			right
				.directory
				.cmp(&left.directory)
				.then_with(|| left.path.cmp(&right.path))
		});
		Ok(entries)
	})
	.await
}

#[tauri::command]
pub async fn instances_file_read(
	id: String,
	path: String,
	app: AppHandle,
) -> CommandResult<String> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || {
		repo.get(&id)
			.map_err(|error| domain_error("instance", error))?;
		let root = repo
			.instance_dir(&id)
			.map_err(|error| domain_error("instance", error))?;
		let target = safe_instance_file(&root, &path)?;
		let metadata = fs::metadata(&target)?;
		if metadata.len() > 4 * 1024 * 1024 {
			return Err(domain_error(
				"file_too_large",
				"files larger than 4 MiB cannot be edited",
			));
		}
		fs::read_to_string(target).map_err(Into::into)
	})
	.await
}

#[tauri::command]
pub async fn instances_file_write(
	id: String,
	path: String,
	content: String,
	app: AppHandle,
) -> CommandResult<()> {
	let repo = repository(&app)?;
	crate::commands::off_thread(move || {
		repo.get(&id)
			.map_err(|error| domain_error("instance", error))?;
		let root = repo
			.instance_dir(&id)
			.map_err(|error| domain_error("instance", error))?;
		let target = safe_instance_file(&root, &path)?;
		if let Some(parent) = target.parent() {
			fs::create_dir_all(parent)?;
		}
		fs::write(target, content)?;
		Ok(())
	})
	.await
}

enum LaunchSignal {
	Log(String),
	Progress(f64, Option<String>),
	Status(&'static str, Option<String>, Option<i32>),
}

fn apply_launch_settings(plan: &mut packwand_launch::LaunchPlan, settings: &InstanceSettings) {
	if let Some(java) = &settings.java_path {
		plan.java_executable.clone_from(java);
	}
	plan.memory.initial_mb = settings.memory_min_mb;
	plan.memory.max_mb = settings.memory_max_mb;
	if let Some(args) = &settings.extra_jvm_args {
		plan.jvm_args.extend(args.iter().cloned());
	}
	if let Some(args) = &settings.extra_game_args {
		plan.game_args.extend(args.iter().cloned());
	}
	if let Some(env) = &settings.env {
		plan.env.extend(env.clone());
	}
	if let (Some(width), Some(height)) = (settings.window_width, settings.window_height) {
		plan.game_args.extend([
			"--width".to_owned(),
			width.to_string(),
			"--height".to_owned(),
			height.to_string(),
		]);
	}
	if settings.fullscreen == Some(true) {
		plan.game_args.push("--fullscreen".to_owned());
	}
}

fn run_instance_launch<F: Fn() -> bool>(
	repo: &FsUserInstanceRepository,
	id: &str,
	installer: Option<&Path>,
	managed_root: &Path,
	effective_settings: &InstanceSettings,
	is_cancelled: &F,
	tx: &tokio::sync::mpsc::UnboundedSender<LaunchSignal>,
) -> CommandResult<()> {
	let send = |signal| {
		let _ = tx.send(signal);
	};
	send(LaunchSignal::Status(
		"starting",
		Some("Installing pack contents".into()),
		None,
	));
	let mut instance = install_instance(repo, id, installer)?;
	if is_cancelled() {
		return Err(domain_error("cancelled", "job was cancelled"));
	}
	let pack_dir = backing_pack(repo, &instance)?;
	let game_dir = repo
		.instance_dir(id)
		.map_err(|error| domain_error("instance", error))?;
	let session =
		packwand_devboot::default_offline_session().map_err(|error| domain_error("auth", error))?;
	let progress = tx.clone();
	let mut booted = packwand_devboot::boot_pack(
		managed_root,
		&pack_dir,
		&game_dir,
		&session,
		effective_settings.java_path.clone(),
		move |update| {
			let fraction = if update.total_downloads == 0 {
				0.0
			} else {
				update.finished_downloads as f64 / update.total_downloads as f64
			};
			let _ = progress.send(LaunchSignal::Progress(
				fraction,
				Some(format!(
					"{}/{} downloads",
					update.finished_downloads, update.total_downloads
				)),
			));
		},
	)
	.map_err(|error| domain_error("bootstrap", error))?;
	apply_launch_settings(&mut booted.plan, effective_settings);
	let handle = packwand_launch::launch(
		&booted.plan,
		packwand_launch::LaunchOptions {
			secrets: booted.secrets,
			..Default::default()
		},
	)
	.map_err(|error| domain_error("launch", error))?;
	instance.last_played_ms = Some(now_ms());
	repo.write(&instance)
		.map_err(|error| domain_error("instance", error))?;
	loop {
		match handle.events().recv_timeout(Duration::from_millis(250)) {
			Ok(LaunchEvent::Started { pid, .. }) => send(LaunchSignal::Status(
				"running",
				Some(format!("Running (pid {pid})")),
				None,
			)),
			Ok(LaunchEvent::Stdout { line, .. } | LaunchEvent::Stderr { line, .. }) => {
				send(LaunchSignal::Log(line))
			}
			Ok(LaunchEvent::Exited { code, .. }) => {
				let okay = code == Some(0);
				send(LaunchSignal::Status(
					if okay { "stopped" } else { "error" },
					Some(format!("Exited with code {code:?}")),
					code,
				));
				if !okay {
					return Err(domain_error(
						"exit_code",
						format!("Minecraft exited with code {code:?}"),
					));
				}
				break;
			}
			Ok(LaunchEvent::Failed { error, .. }) => return Err(domain_error("launch", error)),
			Ok(LaunchEvent::Cancelled { .. }) => {
				return Err(domain_error("cancelled", "job was cancelled"));
			}
			Ok(LaunchEvent::Starting { .. }) => {}
			Err(RecvTimeoutError::Timeout) if is_cancelled() => handle.cancel(),
			Err(RecvTimeoutError::Timeout) => {}
			Err(RecvTimeoutError::Disconnected) => break,
		}
	}
	Ok(())
}

#[tauri::command]
pub async fn instances_status_list(
	state: State<'_, AppState>,
) -> CommandResult<Vec<InstanceStatusPayload>> {
	Ok(state.instances.list().await)
}

#[tauri::command]
pub async fn instances_stop(id: String, state: State<'_, AppState>) -> CommandResult<bool> {
	match state.instances.job_id_for(&id).await {
		Some(job) => Ok(state.jobs.cancel(&job).await),
		None => Ok(false),
	}
}

#[tauri::command]
pub async fn instances_launch(
	id: String,
	app: AppHandle,
	state: State<'_, AppState>,
) -> CommandResult<JobRecord> {
	let repo = repository(&app)?;
	let instance = repo
		.get(&id)
		.map_err(|error| domain_error("instance", error))?;
	let app_settings = state.settings()?;
	let inherited = InstanceSettings {
		java_path: app_settings
			.java_defaults
			.get(&instance.game_version)
			.map(PathBuf::from),
		memory_max_mb: Some(app_settings.memory_mb),
		..Default::default()
	};
	let effective_settings = instance.settings.merged(&inherited);
	let managed_root = packwand_devboot::default_managed_root(repo.root());
	let installer = find_installer_binary(&app);
	let registry = state.instances.clone();
	let launch_app = app.clone();
	let launch_id = id.clone();
	let job = state
		.jobs
		.spawn(
			app,
			"instance.launch",
			format!("Launch {id}"),
			move |context| async move {
				let job_id = context.id().to_owned();
				let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
				let cancel = context.clone();
				let blocking = tokio::task::spawn_blocking(move || {
					run_instance_launch(
						&repo,
						&id,
						installer.as_deref(),
						&managed_root,
						&effective_settings,
						&|| cancel.is_cancelled(),
						&tx,
					)
				});
				while let Some(signal) = rx.recv().await {
					match signal {
						LaunchSignal::Log(line) => context.log(line).await,
						LaunchSignal::Progress(value, message) => {
							context.progress(value, message).await
						}
						LaunchSignal::Status(phase, message, exit_code) => {
							let payload = InstanceStatusPayload {
								id: launch_id.clone(),
								phase: phase.into(),
								message,
								job_id: Some(job_id.clone()),
								exit_code,
							};
							registry.set(payload.clone()).await;
							let _ = emit_instance_status(&launch_app, payload);
						}
					}
				}
				let outcome = blocking
					.await
					.map_err(|error| domain_error("task", error))?;
				if let Err(error) = &outcome {
					let phase = if error.kind == "cancelled" {
						"stopped"
					} else {
						"error"
					};
					let payload = InstanceStatusPayload {
						id: launch_id,
						phase: phase.to_owned(),
						message: Some(error.message.clone()),
						job_id: Some(job_id),
						exit_code: None,
					};
					registry.set(payload.clone()).await;
					let _ = emit_instance_status(&launch_app, payload);
				}
				outcome
			},
		)
		.await;
	Ok(job)
}

#[cfg(test)]
mod tests {
	use std::cell::Cell;

	use super::*;

	#[test]
	fn disabled_content_wins_after_reinstall() {
		let root = tempfile::tempdir().unwrap();
		let disabled = root.path().join("mods/example.jar.disabled");
		fs::create_dir_all(disabled.parent().unwrap()).unwrap();
		fs::write(&disabled, b"disabled").unwrap();
		fs::write(root.path().join("mods/example.jar"), b"fresh").unwrap();
		restore_disabled(root.path(), std::slice::from_ref(&disabled)).unwrap();
		assert!(disabled.is_file());
		assert!(!root.path().join("mods/example.jar").exists());
	}

	#[test]
	fn content_paths_reject_traversal_and_absolute_paths() {
		let root = Path::new("instance");
		assert!(safe_content_path(root, "mods/a.jar").is_ok());
		assert!(safe_content_path(root, "../a.jar").is_err());
		assert!(safe_content_path(root, "C:\\a.jar").is_err());
	}

	#[test]
	fn linked_pack_art_is_used_and_instance_art_takes_precedence() {
		let root = tempfile::tempdir().unwrap();
		let pack = tempfile::tempdir().unwrap();
		fs::write(pack.path().join("icon.png"), b"pack icon").unwrap();
		fs::write(pack.path().join("bg.png"), b"pack background").unwrap();
		let repo = FsUserInstanceRepository::new(root.path().to_path_buf());
		let instance = Instance::new(
			"visual".into(),
			"Visual".into(),
			InstanceSource::Linked {
				pack_dir: pack.path().to_path_buf(),
			},
			"1.21.1".into(),
			"fabric".into(),
			None,
			0,
		);
		repo.create(&instance).unwrap();
		assert_eq!(
			read_instance_image(&repo, "visual", InstanceImageKind::Icon).unwrap(),
			Some(b"pack icon".to_vec())
		);
		assert_eq!(
			read_instance_image(&repo, "visual", InstanceImageKind::Background).unwrap(),
			Some(b"pack background".to_vec())
		);
		fs::write(
			repo.instance_dir("visual").unwrap().join("icon.png"),
			b"local icon",
		)
		.unwrap();
		assert_eq!(
			read_instance_image(&repo, "visual", InstanceImageKind::Icon).unwrap(),
			Some(b"local icon".to_vec())
		);
	}

	#[test]
	fn bundled_installer_candidates_are_native_executables() {
		let root = Path::new("bundle");
		let candidates = bundled_installer_candidates(root);
		assert_eq!(candidates.len(), 2);
		assert!(
			candidates
				.iter()
				.all(|path| path.to_string_lossy().contains("packwand-installer"))
		);
		assert!(
			candidates
				.iter()
				.all(|path| path.extension().is_none_or(|extension| extension == "exe"))
		);
	}

	#[test]
	fn native_installer_failure_blocks_ready_state_and_retry_can_recover() {
		let root = tempfile::tempdir().unwrap();
		let pack = tempfile::tempdir().unwrap();
		let repo = FsUserInstanceRepository::new(root.path().to_path_buf());
		let instance = Instance::new(
			"launch-contract".into(),
			"Launch Contract".into(),
			InstanceSource::Linked {
				pack_dir: pack.path().to_path_buf(),
			},
			"1.21.1".into(),
			"fabric".into(),
			None,
			0,
		);
		repo.create(&instance).unwrap();

		let error = install_instance_with(&repo, &instance.id, |_, _| {
			Err(domain_error("installer", "native installer failed"))
		})
		.unwrap_err();
		assert_eq!(error.kind, "installer");
		assert!(matches!(
			repo.get(&instance.id).unwrap().stage,
			InstallStage::Failed { .. }
		));

		let called = Cell::new(false);
		let recovered = install_instance_with(&repo, &instance.id, |pack_dir, game_dir| {
			called.set(true);
			assert_eq!(pack_dir, pack.path());
			assert_eq!(game_dir, repo.instance_dir(&instance.id).unwrap());
			Ok(())
		})
		.unwrap();
		assert!(called.get());
		assert_eq!(recovered.stage, InstallStage::Ready);
	}
}
