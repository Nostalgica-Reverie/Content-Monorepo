use std::collections::BTreeMap;

use packwand_pack::{HashFormat, hash_bytes};
use serde::Deserialize;
use url::Url;

use crate::repository::{
	asset_pattern, release_channel_allowed, repository_reference, selected_asset, slugify_name,
};
use crate::{
	HttpRequest, ProjectType, ProviderError, ProviderKind, ProviderResolver, ReleaseChannel,
	RepositoryRelease, ResolveRequest, ResolvedFile, ResolvedProject, ResolvedVersion, Transport,
};

const DEFAULT_API: &str = "https://api.github.com/";

pub struct GitHubClient<T> {
	transport: T,
	api_base: Url,
	token: String,
}

impl<T> GitHubClient<T> {
	pub fn new(transport: T, token: impl Into<String>) -> Self {
		Self {
			transport,
			api_base: Url::parse(DEFAULT_API).expect("valid GitHub API URL"),
			token: token.into(),
		}
	}

	pub fn with_api_base(
		transport: T,
		token: impl Into<String>,
		api_base: &str,
	) -> Result<Self, ProviderError> {
		Ok(Self {
			transport,
			api_base: Url::parse(api_base)
				.map_err(|error| ProviderError::InvalidUrl(error.to_string()))?,
			token: token.into(),
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

	fn request(&self, url: Url) -> HttpRequest {
		let mut request =
			HttpRequest::get(url.to_string()).header("Accept", "application/vnd.github+json");
		if !self.token.is_empty() {
			request = request.header("Authorization", format!("Bearer {}", self.token));
		}
		request
	}

	fn get_json<R: for<'de> Deserialize<'de>>(&self, url: Url) -> Result<R, ProviderError>
	where
		T: Transport,
	{
		let bytes = self.transport.get(self.request(url))?;
		serde_json::from_slice(&bytes).map_err(|error| ProviderError::Decode {
			provider: "GitHub",
			message: error.to_string(),
		})
	}
}

impl<T: Transport> ProviderResolver for GitHubClient<T> {
	fn resolve(&self, request: &ResolveRequest) -> Result<ResolvedProject, ProviderError> {
		if !release_channel_allowed(&request.channels) {
			return Err(ProviderError::NoCompatibleVersion);
		}
		let (_, requested_slug) =
			repository_reference(&request.project, "github.com", Some("github.com"))?;
		let parts: Vec<_> = requested_slug.split('/').collect();
		let repo: RepoResponse = self.get_json(self.endpoint(&["repos", parts[0], parts[1]])?)?;
		if repo.full_name.is_empty() {
			return Err(ProviderError::InvalidResponse(
				"GitHub repository has no full_name".into(),
			));
		}
		let releases: Vec<ReleaseResponse> =
			self.get_json(self.endpoint(&["repos", parts[0], parts[1], "releases"])?)?;
		let release = releases
			.into_iter()
			.find(|release| {
				request
					.branch
					.as_deref()
					.is_none_or(|branch| release.target_commitish == branch)
			})
			.ok_or(ProviderError::NoCompatibleVersion)?;
		let pattern = asset_pattern(request.asset_pattern.as_deref());
		let index = selected_asset(
			release.assets.iter().map(|asset| asset.name.clone()),
			&pattern,
		)?;
		let asset = &release.assets[index];
		let bytes = self.transport.get_large(
			self.request(
				Url::parse(&asset.browser_download_url)
					.map_err(|error| ProviderError::InvalidUrl(error.to_string()))?,
			),
		)?;
		let hashes =
			BTreeMap::from([("sha512".to_string(), hash_bytes(HashFormat::Sha512, &bytes))]);
		let tag = release.tag_name;
		Ok(ResolvedProject {
			provider: ProviderKind::GitHub,
			id: repo.full_name,
			slug: slugify_name(&repo.name),
			title: repo.name,
			project_type: ProjectType::Mod,
			side: "both".into(),
			repository_release: Some(RepositoryRelease {
				instance: None,
				branch: request.branch.clone().unwrap_or_default(),
				asset_pattern: pattern,
			}),
			version: ResolvedVersion {
				id: tag.clone(),
				name: release.name.unwrap_or_else(|| tag.clone()),
				number: tag,
				channel: ReleaseChannel::Release,
				file: ResolvedFile {
					filename: asset.name.clone(),
					url: Some(asset.browser_download_url.clone()),
					size: bytes.len() as u64,
					hashes,
				},
			},
		})
	}
}

#[derive(Deserialize)]
struct RepoResponse {
	name: String,
	full_name: String,
}

#[derive(Deserialize)]
struct ReleaseResponse {
	tag_name: String,
	#[serde(default)]
	target_commitish: String,
	name: Option<String>,
	#[serde(default)]
	assets: Vec<AssetResponse>,
}

#[derive(Deserialize)]
struct AssetResponse {
	name: String,
	browser_download_url: String,
}
