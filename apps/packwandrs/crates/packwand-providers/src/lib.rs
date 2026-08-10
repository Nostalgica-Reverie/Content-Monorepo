//! Provider clients and provider-neutral resolved project metadata.

#![forbid(unsafe_code)]

mod browse;
mod curseforge;
mod forgejo;
mod github;
mod gitlab;
mod modrinth;
mod repository;
mod transport;

use std::collections::BTreeMap;

use packwand_pack::{Download, Mod};
use serde::{Deserialize, Serialize};

pub use browse::{
	BodyFormat, BrowseDetail, BrowsePage, BrowseProject, BrowseQuery, CreatorProfile, GalleryImage,
	ProviderBrowser,
};
pub use curseforge::{
	CurseForgeClient, CurseForgeDownload, FingerprintMatch, FingerprintMatches, configured_api_key,
	parse_file_url,
};
pub use forgejo::ForgejoClient;
pub use github::GitHubClient;
pub use gitlab::GitLabClient;
pub use modrinth::{ModrinthClient, search_loaders as modrinth_search_loaders};
pub use repository::DEFAULT_ASSET_PATTERN;
pub use transport::{HttpRequest, Transport, TransportError, UreqTransport};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
	Modrinth,
	CurseForge,
	Forgejo,
	GitHub,
	GitLab,
}

impl ProviderKind {
	pub const fn name(self) -> &'static str {
		match self {
			Self::Modrinth => "modrinth",
			Self::CurseForge => "curseforge",
			Self::Forgejo => "forgejo",
			Self::GitHub => "github",
			Self::GitLab => "gitlab",
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
	Release,
	Beta,
	Alpha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
	Mod,
	ResourcePack,
	Shader,
	DataPack,
	Plugin,
	Other,
}

impl ProjectType {
	pub const fn default_folder(self) -> &'static str {
		match self {
			Self::Mod => "mods",
			Self::ResourcePack => "resourcepacks",
			Self::Shader => "shaderpacks",
			Self::DataPack => "datapacks",
			Self::Plugin => "plugins",
			Self::Other => ".",
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolveRequest {
	pub project: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub version_id: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub version_filename: Option<String>,
	pub game_versions: Vec<String>,
	pub loaders: Vec<String>,
	pub channels: Vec<ReleaseChannel>,
	pub branch: Option<String>,
	pub asset_pattern: Option<String>,
}

impl ResolveRequest {
	pub fn new(project: impl Into<String>) -> Self {
		Self {
			project: project.into(),
			version_id: None,
			version_filename: None,
			game_versions: Vec::new(),
			loaders: Vec::new(),
			channels: Vec::new(),
			branch: None,
			asset_pattern: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryRelease {
	pub instance: Option<String>,
	pub branch: String,
	pub asset_pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedFile {
	pub filename: String,
	pub url: Option<String>,
	pub size: u64,
	pub hashes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedVersion {
	pub id: String,
	pub name: String,
	pub number: String,
	pub channel: ReleaseChannel,
	pub file: ResolvedFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedProject {
	pub provider: ProviderKind,
	pub id: String,
	pub slug: String,
	pub title: String,
	pub project_type: ProjectType,
	pub side: String,
	pub repository_release: Option<RepositoryRelease>,
	pub version: ResolvedVersion,
}

impl ResolvedProject {
	pub fn metadata_path(&self) -> String {
		format!(
			"{}/{}",
			self.project_type.default_folder(),
			packwand_pack::metafile::name_for(&self.slug)
		)
	}

	pub fn into_mod(self) -> Result<Mod, ProviderError> {
		let (hash_format, hash) = strongest_hash(&self.version.file.hashes)?;
		let mut provider_data = packwand_pack::UpdateTable::new();
		let (url, extra_hashes, size, mode) = match self.provider {
			ProviderKind::Modrinth => {
				provider_data.insert("mod-id".into(), self.id.clone().into());
				provider_data.insert("version".into(), self.version.id.clone().into());
				let mut extra_hashes = self.version.file.hashes.clone();
				extra_hashes.remove(hash_format);
				(
					self.version.file.url.clone().unwrap_or_default(),
					extra_hashes,
					self.version.file.size,
					String::new(),
				)
			}
			ProviderKind::CurseForge => {
				let project_id = self.id.parse::<i64>().map_err(|_| {
					ProviderError::InvalidResponse("CurseForge project id is not numeric".into())
				})?;
				let file_id = self.version.id.parse::<i64>().map_err(|_| {
					ProviderError::InvalidResponse("CurseForge file id is not numeric".into())
				})?;
				provider_data.insert("project-id".into(), project_id.into());
				provider_data.insert("file-id".into(), file_id.into());
				(
					String::new(),
					BTreeMap::new(),
					0,
					"metadata:curseforge".to_string(),
				)
			}
			ProviderKind::Forgejo | ProviderKind::GitHub | ProviderKind::GitLab => {
				let release = self.repository_release.as_ref().ok_or_else(|| {
					ProviderError::InvalidResponse("missing repository release metadata".into())
				})?;
				if matches!(self.provider, ProviderKind::Forgejo | ProviderKind::GitLab) {
					provider_data.insert(
						"instance".into(),
						release.instance.clone().unwrap_or_default().into(),
					);
				}
				provider_data.insert("slug".into(), self.id.clone().into());
				provider_data.insert("tag".into(), self.version.id.clone().into());
				if matches!(self.provider, ProviderKind::Forgejo | ProviderKind::GitHub) {
					provider_data.insert("branch".into(), release.branch.clone().into());
				}
				provider_data.insert("regex".into(), release.asset_pattern.clone().into());
				(
					self.version.file.url.clone().unwrap_or_default(),
					BTreeMap::new(),
					0,
					String::new(),
				)
			}
		};
		let provider_name = self.provider.name();
		Ok(Mod {
			name: self.title,
			filename: self.version.file.filename,
			side: self.side,
			download: Download {
				url,
				hash_format: hash_format.to_string(),
				hash,
				extra_hashes,
				size,
				mode,
			},
			update: BTreeMap::from([(provider_name.to_string(), provider_data)]),
			..Mod::default()
		})
	}
}

fn strongest_hash(hashes: &BTreeMap<String, String>) -> Result<(&str, String), ProviderError> {
	for format in ["sha512", "sha256", "sha1", "md5", "murmur2"] {
		if let Some(hash) = hashes.get(format).filter(|hash| !hash.is_empty()) {
			return Ok((format, hash.clone()));
		}
	}
	Err(ProviderError::NoUsableHash)
}

pub trait ProviderResolver {
	fn resolve(&self, request: &ResolveRequest) -> Result<ResolvedProject, ProviderError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
	#[error(transparent)]
	Transport(#[from] TransportError),
	#[error("failed to decode {provider} response: {message}")]
	Decode {
		provider: &'static str,
		message: String,
	},
	#[error("provider returned no compatible versions")]
	NoCompatibleVersion,
	#[error("resolved version has no downloadable files")]
	NoFiles,
	#[error("resolved file has no usable hash")]
	NoUsableHash,
	#[error("invalid provider response: {0}")]
	InvalidResponse(String),
	#[error("invalid provider URL: {0}")]
	InvalidUrl(String),
	#[error("CurseForge project ids must be numeric")]
	InvalidCurseForgeProject,
	#[error("invalid repository slug {0:?}; expected owner/repository")]
	InvalidRepository(String),
	#[error("invalid asset regular expression: {0}")]
	InvalidAssetPattern(String),
	#[error("release has {count} assets matching {pattern:?}; expected exactly one")]
	AmbiguousAssets { pattern: String, count: usize },
	#[error("CurseForge rejected the configured API key ({status}); {hint}")]
	CurseForgeAuthRejected { status: u16, hint: String },
}
