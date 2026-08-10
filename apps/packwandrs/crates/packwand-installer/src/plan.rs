use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use packwand_pack::Mod;
use packwand_providers::{CurseForgeClient, Transport, configured_api_key};
use serde::{Deserialize, Serialize};

use crate::InstallerError;
use crate::index::{self, RemotePack, decode_mod, safe_relative};

/// Content side selected by the launcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallSide {
	Client,
	Server,
}

impl FromStr for InstallSide {
	type Err = InstallerError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			"client" => Ok(Self::Client),
			"server" => Ok(Self::Server),
			_ => Err(InstallerError::Decode(format!(
				"unknown install side {value:?}"
			))),
		}
	}
}

/// Existing-file policy copied from an index entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverwriteMode {
	Replace,
	Preserve,
}

/// One transactional filesystem action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlanAction {
	Download {
		url: String,
		target: PathBuf,
		hash_format: String,
		hash: String,
		overwrite: OverwriteMode,
	},
	Remove {
		target: PathBuf,
	},
}

/// A mod that couldn't be fetched automatically and needs a human to place
/// it — currently only produced when a CurseForge author has disabled
/// third-party distribution for that file. Prism and the legacy Java
/// installer handle this the same way: install everything else, then point
/// the user at the file's page so they can download and drop it in
/// themselves. Re-running install recognizes a correctly-hashed file already
/// at `target` and drops it from this list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualDownload {
	pub name: String,
	pub target: PathBuf,
	pub page_url: Option<String>,
	/// Expected hash of the file the user places at `target`, so a GUI can
	/// verify a manually supplied file before accepting it — the same check
	/// [`already_satisfied`] uses to recognize one that's already there.
	pub hash_format: String,
	pub hash: String,
}

/// Fully resolved pack application plan.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallPlan {
	pub actions: Vec<PlanAction>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub manual: Vec<ManualDownload>,
}

pub fn build(
	remote: &RemotePack,
	instance: &Path,
	side: InstallSide,
	transport: &dyn Transport,
) -> Result<InstallPlan, InstallerError> {
	let mut actions = Vec::new();
	let mut manual = Vec::new();
	for entry in &remote.index.files {
		let format = entry
			.hash_format
			.as_deref()
			.unwrap_or(&remote.index.hash_format);
		let target = instance.join(safe_relative(
			entry.alias.as_deref().unwrap_or(&entry.file),
		)?);
		let overwrite = if entry.preserve {
			OverwriteMode::Preserve
		} else {
			OverwriteMode::Replace
		};
		if entry.metafile {
			let metadata = remote.entry(&entry.file, format, &entry.hash, transport)?;
			let metadata = decode_mod(&metadata)?;
			// `metadata.filename` is a bare name (e.g. "fabric-api.jar"); the
			// mod lands next to its metafile (e.g. `mods/`), not at the
			// instance root.
			let metafile_dir = target.parent().unwrap_or(instance);
			let target = metafile_dir.join(safe_relative(&metadata.filename)?);
			if !correct_side(&metadata, side) || optional_disabled(&metadata) {
				actions.push(PlanAction::Remove { target });
				continue;
			}
			match resolve_url(&metadata, transport) {
				Ok(url) => actions.push(PlanAction::Download {
					url,
					target,
					hash_format: metadata.download.hash_format,
					hash: metadata.download.hash,
					overwrite,
				}),
				Err(InstallerError::ManualDownloadRequired { name, page_url }) => {
					if !already_satisfied(
						&target,
						&metadata.download.hash_format,
						&metadata.download.hash,
					) {
						manual.push(ManualDownload {
							name,
							target,
							page_url,
							hash_format: metadata.download.hash_format,
							hash: metadata.download.hash,
						});
					}
				}
				Err(error) => return Err(error),
			}
		} else {
			let url = remote
				.base
				.join(&entry.file.replace('\\', "/"))
				.map_err(|error| InstallerError::InvalidUrl(error.to_string()))?
				.to_string();
			actions.push(PlanAction::Download {
				url,
				target,
				hash_format: format.into(),
				hash: entry.hash.clone(),
				overwrite,
			});
		}
	}
	Ok(InstallPlan { actions, manual })
}

/// Whether a file the user placed manually already matches what the pack
/// expects, so a restart doesn't keep nagging for a file that's already
/// there.
fn already_satisfied(target: &Path, hash_format: &str, hash: &str) -> bool {
	if hash.is_empty() {
		return target.is_file();
	}
	let Ok(bytes) = fs::read(target) else {
		return false;
	};
	index::verify(&target.display().to_string(), hash_format, hash, &bytes).is_ok()
}

fn correct_side(metadata: &Mod, side: InstallSide) -> bool {
	match metadata.side.as_str() {
		"client" | "client-only" => side == InstallSide::Client,
		"server" | "server-only" => side == InstallSide::Server,
		_ => true,
	}
}

fn optional_disabled(metadata: &Mod) -> bool {
	metadata
		.option
		.as_ref()
		.is_some_and(|option| option.optional && !option.default)
}

fn resolve_url(metadata: &Mod, transport: &dyn Transport) -> Result<String, InstallerError> {
	if !metadata.download.url.is_empty() {
		return Ok(metadata.download.url.clone());
	}
	if metadata.download.mode != "metadata:curseforge" {
		return Err(InstallerError::Provider(format!(
			"{} has no download URL",
			metadata.name
		)));
	}
	let update = metadata
		.update
		.get("curseforge")
		.ok_or_else(|| InstallerError::Provider("missing CurseForge update metadata".into()))?;
	let project = update
		.get("project-id")
		.and_then(serde_json::Value::as_u64)
		.ok_or_else(|| InstallerError::Provider("missing CurseForge project-id".into()))?;
	let file = update
		.get("file-id")
		.and_then(serde_json::Value::as_u64)
		.ok_or_else(|| InstallerError::Provider("missing CurseForge file-id".into()))?;
	let key = configured_api_key();
	if key.is_empty() {
		return Err(InstallerError::Provider(
			"CurseForge content requires CURSEFORGE_API_KEY or PACKWAND_CURSEFORGE_API_KEY".into(),
		));
	}
	let client = CurseForgeClient::new(TransportRef(transport), key);
	// `ProviderResolver::resolve` deliberately nulls the download URL (it
	// feeds persisted pack metadata, and CurseForge's terms forbid storing
	// this URL). An installer needs the live URL to stream the download
	// right now, so it goes through the dedicated install-time lookup
	// instead and never writes the result anywhere.
	let resolved = client
		.download_url(project as u32, file as u32)
		.map_err(|error| InstallerError::Provider(format!("{}: {error}", metadata.name)))?;
	resolved
		.url
		.ok_or(InstallerError::ManualDownloadRequired {
			name: metadata.name.clone(),
			page_url: resolved.page_url,
		})
}

struct TransportRef<'a>(&'a dyn Transport);

impl Transport for TransportRef<'_> {
	fn get(
		&self,
		request: packwand_providers::HttpRequest,
	) -> Result<Vec<u8>, packwand_providers::TransportError> {
		self.0.get(request)
	}

	fn get_large(
		&self,
		request: packwand_providers::HttpRequest,
	) -> Result<Vec<u8>, packwand_providers::TransportError> {
		self.0.get_large(request)
	}
}

#[cfg(test)]
mod tests {
	use super::{InstallSide, correct_side, optional_disabled};
	use packwand_pack::{Mod, ModOption};

	#[test]
	fn side_and_optional_rules_match_launcher_expectations() {
		let mut metadata = Mod {
			side: "client".into(),
			..Mod::default()
		};
		assert!(correct_side(&metadata, InstallSide::Client));
		assert!(!correct_side(&metadata, InstallSide::Server));
		metadata.option = Some(ModOption {
			optional: true,
			default: false,
			description: String::new(),
		});
		assert!(optional_disabled(&metadata));
	}
}
