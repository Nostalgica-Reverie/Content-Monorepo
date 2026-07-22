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
