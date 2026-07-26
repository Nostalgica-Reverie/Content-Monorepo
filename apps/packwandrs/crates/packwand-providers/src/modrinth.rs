use std::collections::BTreeMap;

use serde::Deserialize;
use url::Url;

use crate::{
    HttpRequest, ProjectType, ProviderError, ProviderKind, ProviderResolver, ReleaseChannel,
    ResolveRequest, ResolvedFile, ResolvedProject, ResolvedVersion, Transport,
};

const DEFAULT_API: &str = "https://api.modrinth.com/v2/";

pub struct ModrinthClient<T> {
    transport: T,
    api_base: Url,
}

impl<T> ModrinthClient<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            api_base: Url::parse(DEFAULT_API).expect("valid Modrinth API URL"),
        }
    }

    pub fn with_api_base(transport: T, api_base: &str) -> Result<Self, ProviderError> {
        let api_base =
            Url::parse(api_base).map_err(|error| ProviderError::InvalidUrl(error.to_string()))?;
        Ok(Self {
            transport,
            api_base,
        })
    }

    fn endpoint(&self, segments: &[&str]) -> Result<Url, ProviderError> {
        let mut url = self.api_base.clone();
        url.path_segments_mut()
            .map_err(|_| ProviderError::InvalidUrl(self.api_base.to_string()))?
            .pop_if_empty()
            .extend(segments);
        Ok(url)
    }

    fn get_json<R: for<'de> Deserialize<'de>>(&self, url: Url) -> Result<R, ProviderError>
    where
        T: Transport,
    {
        let bytes = self.transport.get(HttpRequest::get(url.to_string()))?;
        serde_json::from_slice(&bytes).map_err(|error| ProviderError::Decode {
            provider: "Modrinth",
            message: error.to_string(),
        })
    }
}

/// Loaders Modrinth accepts regardless of the pack's declared mod loader.
///
/// `minecraft` is the loader Modrinth files resource packs under and
/// `vanilla` covers core shaders, so omitting these makes every resource pack
/// and shader in a pack look like it has no compatible version.
pub const DEFAULT_SEARCH_LOADERS: [&str; 5] =
    ["canvas", "iris", "optifine", "vanilla", "minecraft"];

/// As [`DEFAULT_SEARCH_LOADERS`], plus the datapack loader, for packs that
/// keep datapacks in the pack tree.
pub const DATAPACK_SEARCH_LOADERS: [&str; 6] = [
    "canvas",
    "iris",
    "optifine",
    "vanilla",
    "minecraft",
    "datapack",
];

/// Appends the always-acceptable Modrinth loaders to a pack's own loaders.
#[must_use]
pub fn search_loaders(pack_loaders: &[String], with_datapacks: bool) -> Vec<String> {
    let defaults: &[&str] = if with_datapacks {
        &DATAPACK_SEARCH_LOADERS
    } else {
        &DEFAULT_SEARCH_LOADERS
    };
    let mut loaders = pack_loaders.to_vec();
    for extra in defaults {
        let extra = (*extra).to_owned();
        if !loaders.contains(&extra) {
            loaders.push(extra);
        }
    }
    loaders
}

/// The largest number of hashes sent in one `version_files/update` call.
/// Modrinth accepts more, but an oversized body is likelier to be rejected
/// outright than to be slow, and a failed batch costs every mod in it.
pub const UPDATE_BATCH: usize = 300;

impl<T: Transport> ModrinthClient<T> {
    /// Resolves the latest matching version for many installed files in a
    /// single request.
    ///
    /// This is the difference between one request per mod and one request per
    /// few hundred. Modrinth's budget is 300 requests/minute, so a workspace
    /// with thousands of mods is bound by that budget, not by bandwidth or
    /// cores — no amount of concurrency substitutes for asking once.
    ///
    /// Hashes Modrinth does not know, or for which nothing matches the
    /// filters, simply have no entry in the returned map.
    pub fn latest_versions_by_hash(
        &self,
        hashes: &[String],
        algorithm: &str,
        loaders: &[String],
        game_versions: &[String],
        channels: &[ReleaseChannel],
    ) -> Result<BTreeMap<String, ResolvedVersion>, ProviderError> {
        if hashes.is_empty() {
            return Ok(BTreeMap::new());
        }
        let mut resolved = BTreeMap::new();
        for batch in hashes.chunks(UPDATE_BATCH) {
            let body = serde_json::json!({
                "hashes": batch,
                "algorithm": algorithm,
                "loaders": loaders,
                "game_versions": game_versions,
            });
            let payload = serde_json::to_vec(&body).map_err(|error| ProviderError::Decode {
                provider: "Modrinth",
                message: error.to_string(),
            })?;
            let url = self.endpoint(&["version_files", "update"])?;
            let bytes = self
                .transport
                .post_json(HttpRequest::get(url.to_string()), &payload)?;
            let response: BTreeMap<String, VersionResponse> = serde_json::from_slice(&bytes)
                .map_err(|error| ProviderError::Decode {
                    provider: "Modrinth",
                    message: error.to_string(),
                })?;
            for (hash, version) in response {
                let channel = version.channel();
                if !channel_allowed(channel, channels) {
                    continue;
                }
                let Some(file) = primary_file(&version) else {
                    continue;
                };
                resolved.insert(
                    hash,
                    ResolvedVersion {
                        id: version.id,
                        name: version.name,
                        number: version.version_number,
                        channel,
                        file: ResolvedFile {
                            filename: file.filename,
                            url: Some(file.url),
                            size: file.size,
                            hashes: file.hashes,
                        },
                    },
                );
            }
        }
        Ok(resolved)
    }
}

impl<T: Transport> ModrinthClient<T> {
    /// Required-dependency project ids for a batch of installed version ids,
    /// fetched with Modrinth's bulk `/versions` endpoint rather than one
    /// request per mod — the same "ask once" budget concern as
    /// [`Self::latest_versions_by_hash`].
    ///
    /// Versions Modrinth does not know simply have no entry in the returned
    /// map; optional/incompatible/embedded dependencies are dropped, since a
    /// coverage check only cares about hard requirements.
    pub fn dependencies_by_version(
        &self,
        version_ids: &[String],
    ) -> Result<BTreeMap<String, Vec<String>>, ProviderError> {
        if version_ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let mut url = self.endpoint(&["versions"])?;
        url.query_pairs_mut().append_pair(
            "ids",
            &serde_json::to_string(version_ids).expect("string list serializes"),
        );
        let versions: Vec<VersionWithDependencies> = self.get_json(url)?;
        Ok(versions
            .into_iter()
            .map(|version| {
                let required = version
                    .dependencies
                    .into_iter()
                    .filter(|dependency| dependency.dependency_type == "required")
                    .filter_map(|dependency| dependency.project_id)
                    .collect();
                (version.id, required)
            })
            .collect())
    }
}

#[derive(Deserialize)]
struct VersionWithDependencies {
    id: String,
    #[serde(default)]
    dependencies: Vec<DependencyResponse>,
}

#[derive(Deserialize)]
struct DependencyResponse {
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    dependency_type: String,
}

/// The file a version actually ships: the one flagged primary, else the first.
fn primary_file(version: &VersionResponse) -> Option<FileResponse> {
    version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
        .cloned()
}

impl<T: Transport> ProviderResolver for ModrinthClient<T> {
    fn resolve(&self, request: &ResolveRequest) -> Result<ResolvedProject, ProviderError> {
        let project: ProjectResponse =
            self.get_json(self.endpoint(&["project", &request.project])?)?;
        let version = if let Some(version_id) = &request.version_id {
            self.get_json(self.endpoint(&["version", version_id])?)?
        } else {
            let mut versions_url = self.endpoint(&["project", &project.id, "version"])?;
            {
                let mut query = versions_url.query_pairs_mut();
                if !request.game_versions.is_empty() {
                    query.append_pair(
                        "game_versions",
                        &serde_json::to_string(&request.game_versions)
                            .expect("string list serializes"),
                    );
                }
                if !request.loaders.is_empty() {
                    query.append_pair(
                        "loaders",
                        &serde_json::to_string(&request.loaders).expect("string list serializes"),
                    );
                }
            }
            let versions: Vec<VersionResponse> = self.get_json(versions_url)?;
            versions
                .into_iter()
                .find(|version| channel_allowed(version.channel(), &request.channels))
                .ok_or(ProviderError::NoCompatibleVersion)?
        };
        let file = version
            .files
            .iter()
            .find(|file| {
                request
                    .version_filename
                    .as_ref()
                    .is_some_and(|name| name == &file.filename)
            })
            .or_else(|| {
                request
                    .version_filename
                    .is_none()
                    .then(|| version.files.iter().find(|file| file.primary))
                    .flatten()
            })
            .or_else(|| {
                request
                    .version_filename
                    .is_none()
                    .then(|| version.files.first())
                    .flatten()
            })
            .ok_or(ProviderError::NoFiles)?
            .clone();
        let project_type = project.project_type();
        let side = project.side();
        let channel = version.channel();
        Ok(ResolvedProject {
            provider: ProviderKind::Modrinth,
            id: project.id,
            slug: project.slug,
            title: project.title,
            project_type,
            side,
            repository_release: None,
            version: ResolvedVersion {
                id: version.id,
                name: version.name,
                number: version.version_number,
                channel,
                file: ResolvedFile {
                    filename: file.filename,
                    url: Some(file.url),
                    size: file.size,
                    hashes: file.hashes,
                },
            },
        })
    }
}

fn channel_allowed(channel: ReleaseChannel, allowed: &[ReleaseChannel]) -> bool {
    allowed.is_empty() || allowed.contains(&channel)
}

#[derive(Deserialize)]
struct ProjectResponse {
    id: String,
    slug: String,
    title: String,
    project_type: String,
    client_side: String,
    server_side: String,
}

impl ProjectResponse {
    fn project_type(&self) -> ProjectType {
        match self.project_type.as_str() {
            "mod" => ProjectType::Mod,
            "resourcepack" => ProjectType::ResourcePack,
            "shader" => ProjectType::Shader,
            "datapack" => ProjectType::DataPack,
            "plugin" => ProjectType::Plugin,
            _ => ProjectType::Other,
        }
    }

    fn side(&self) -> String {
        let client = matches!(self.client_side.as_str(), "required" | "optional");
        let server = matches!(self.server_side.as_str(), "required" | "optional");
        match (client, server) {
            (true, true) => "both",
            (true, false) => "client",
            (false, true) => "server",
            (false, false) => "both",
        }
        .to_string()
    }
}

#[derive(Deserialize)]
struct VersionResponse {
    id: String,
    name: String,
    version_number: String,
    version_type: String,
    files: Vec<FileResponse>,
}

impl VersionResponse {
    fn channel(&self) -> ReleaseChannel {
        match self.version_type.as_str() {
            "alpha" => ReleaseChannel::Alpha,
            "beta" => ReleaseChannel::Beta,
            _ => ReleaseChannel::Release,
        }
    }
}

#[derive(Clone, Deserialize)]
struct FileResponse {
    hashes: BTreeMap<String, String>,
    url: String,
    filename: String,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    size: u64,
}
