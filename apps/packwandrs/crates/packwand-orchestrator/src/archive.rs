use std::path::{Path, PathBuf};

use packwand_instance::{FsUserInstanceRepository, Instance, InstanceSource};
use packwand_providers::{CurseForgeClient, ProviderResolver, ResolveRequest, UreqTransport};
use serde::{Deserialize, Serialize};

use crate::content;
use crate::error::{OrchestratorError, Result};
use crate::install;
use crate::paths::{backing_pack, now_ms};

/// The two pack archive formats an instance can be imported from or exported
/// to.
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveFormat {
	Modrinth,
	CurseForge,
}

impl From<ArchiveFormat> for packwand_build::ExportFormat {
	fn from(format: ArchiveFormat) -> Self {
		match format {
			ArchiveFormat::Modrinth => Self::Modrinth,
			ArchiveFormat::CurseForge => Self::CurseForge,
		}
	}
}

/// What an export produced.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
	pub path: PathBuf,
	pub files: usize,
	pub bytes: u64,
	/// Files present in the instance but absent from the backing pack, and so
	/// missing from the archive. Reported rather than silently dropped: an
	/// archive that quietly differs from what the user is running is worse
	/// than one that says what it left out.
	pub excluded_hand_added: usize,
}

/// Exports an instance's backing pack.
pub fn export(
	repo: &FsUserInstanceRepository,
	id: &str,
	format: ArchiveFormat,
	output: Option<PathBuf>,
) -> Result<ExportResult> {
	let instance = repo.get(id)?;
	let pack = backing_pack(repo, &instance)?;
	let excluded = content::list(repo, id)?
		.into_iter()
		.filter(|item| !item.pack_sourced)
		.count();
	let format = packwand_build::ExportFormat::from(format);
	let destination = output.unwrap_or_else(|| {
		repo.root()
			.join("exports")
			.join(format!("{}.{}", id, format.extension()))
	});
	let artifact =
		packwand_build::export_pack(&pack, format, Some(&destination), Default::default())
			.map_err(|error| OrchestratorError::new("export", error))?;
	Ok(ExportResult {
		path: artifact.path,
		files: artifact.files,
		bytes: artifact.bytes,
		excluded_hand_added: excluded,
	})
}

/// Imports an archive into a new standalone instance and installs it.
pub fn import(
	repo: &FsUserInstanceRepository,
	archive: &Path,
	format: ArchiveFormat,
	default_jobs: usize,
) -> Result<Instance> {
	if !archive.is_file() {
		return Err(OrchestratorError::new("not_found", "archive was not found"));
	}
	let base = archive
		.file_stem()
		.and_then(|value| value.to_str())
		.unwrap_or("imported-instance");
	let id = repo.available_id(base);
	let pack_dir = repo.owned_pack_dir(&id)?;
	let imported = match format {
		ArchiveFormat::Modrinth => packwand_build::import_modrinth_archive(archive, &pack_dir),
		ArchiveFormat::CurseForge => {
			let client = CurseForgeClient::new(
				UreqTransport::new(),
				packwand_providers::configured_api_key(),
			);
			packwand_build::import_curseforge_archive(archive, &pack_dir, |project_id, file_id| {
				let mut request = ResolveRequest::new(project_id.to_string());
				request.version_id = Some(file_id.to_string());
				let resolved = client
					.resolve(&request)
					.map_err(|error| error.to_string())?;
				let path = resolved.metadata_path();
				let metadata = resolved.into_mod().map_err(|error| error.to_string())?;
				Ok((path, metadata))
			})
		}
	}
	.map_err(|error| OrchestratorError::new("import", error))?;

	let game_version = imported
		.minecraft_version
		.ok_or_else(|| OrchestratorError::new("import", "archive has no Minecraft version"))?;
	let loader = imported.loader.unwrap_or_else(|| "vanilla".to_owned());
	let target = crate::boot::resolve_pack_target(&pack_dir.join("pack.toml"))
		.map_err(|error| OrchestratorError::new("import", error))?;
	let instance = Instance::new(
		id,
		imported.name,
		InstanceSource::Owned,
		game_version,
		loader,
		target.loader_version,
		now_ms(),
	);
	repo.create(&instance)?;
	install::install(repo, &instance.id, default_jobs)
}
