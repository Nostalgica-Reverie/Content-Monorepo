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

const DEFAULT_INSTANCE: &str = "codeberg.org";

pub struct ForgejoClient<T> {
	transport: T,
	instance: String,
	api_base: Option<Url>,
	token: String,
}

impl<T> ForgejoClient<T> {
	pub fn new(transport: T, token: impl Into<String>) -> Self {
		Self {
			transport,
			instance: DEFAULT_INSTANCE.into(),
			api_base: None,
			token: token.into(),
		}
	}

	pub fn for_instance(
		transport: T,
		instance: impl Into<String>,
		token: impl Into<String>,
	) -> Self {
		Self {
			transport,
			instance: instance.into(),
			api_base: None,
			token: token.into(),
		}
	}

	pub fn with_api_base(
		transport: T,
		instance: impl Into<String>,
		token: impl Into<String>,
		api_base: &str,
	) -> Result<Self, ProviderError> {
		Ok(Self {
			transport,
			instance: instance.into(),
			api_base: Some(
				Url::parse(api_base)
					.map_err(|error| ProviderError::InvalidUrl(error.to_string()))?,
			),
			token: token.into(),
		})
	}

	fn api_base(&self, instance: &str) -> Result<Url, ProviderError> {
		if let Some(base) = &self.api_base {
			if instance != self.instance {
				return Err(ProviderError::InvalidUrl(format!(
					"repository instance {instance} does not match configured instance {}",
					self.instance
				)));
			}
			return Ok(base.clone());
		}
		Url::parse(&format!("https://{instance}/api/v1/"))
			.map_err(|error| ProviderError::InvalidUrl(error.to_string()))
	}

	fn endpoint(&self, instance: &str, segments: &[&str]) -> Result<Url, ProviderError> {
		let mut url = self.api_base(instance)?;
		let invalid_url = url.to_string();
		url.path_segments_mut()
			.map_err(|_| ProviderError::InvalidUrl(invalid_url))?
			.pop_if_empty()
			.extend(segments);
		Ok(url)
	}

	fn api_request(&self, url: Url) -> HttpRequest {
		let mut request = HttpRequest::get(url.to_string()).header("Accept", "application/json");
		if !self.token.is_empty() {
			request = request.header("Authorization", format!("token {}", self.token));
		}
		request
	}

	fn get_json<R: for<'de> Deserialize<'de>>(&self, url: Url) -> Result<R, ProviderError>
	where
		T: Transport,
	{
		let bytes = self.transport.get_large(self.api_request(url))?;
		serde_json::from_slice(&bytes).map_err(|error| ProviderError::Decode {
			provider: "Forgejo",
			message: error.to_string(),
		})
	}
}

impl<T: Transport> ProviderResolver for ForgejoClient<T> {
	fn resolve(&self, request: &ResolveRequest) -> Result<ResolvedProject, ProviderError> {
		if !release_channel_allowed(&request.channels) {
			return Err(ProviderError::NoCompatibleVersion);
		}
		let (instance, requested_slug) =
			repository_reference(&request.project, &self.instance, None)?;
		let parts: Vec<_> = requested_slug.split('/').collect();
		let repo: RepoResponse =
			self.get_json(self.endpoint(&instance, &["repos", parts[0], parts[1]])?)?;
		let releases: Vec<ReleaseResponse> =
			self.get_json(self.endpoint(&instance, &["repos", parts[0], parts[1], "releases"])?)?;
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
			release.attachments.iter().map(|asset| asset.name.clone()),
			&pattern,
		)?;
		let asset = &release.attachments[index];
		let download_url = Url::parse(asset.download_url())
			.map_err(|error| ProviderError::InvalidUrl(error.to_string()))?;
		let bytes = self
			.transport
			.get(HttpRequest::get(download_url.to_string()))?;
		let hashes =
			BTreeMap::from([("sha512".to_string(), hash_bytes(HashFormat::Sha512, &bytes))]);
		let tag = release.tag_name;
		Ok(ResolvedProject {
			provider: ProviderKind::Forgejo,
			id: repo.full_name,
			slug: slugify_name(&repo.name),
			title: repo.name,
			project_type: ProjectType::Mod,
			side: "both".into(),
			repository_release: Some(RepositoryRelease {
				instance: Some(instance),
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
					url: Some(asset.download_url().to_string()),
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
	#[serde(default, alias = "target")]
	target_commitish: String,
	name: Option<String>,
	#[serde(default)]
	attachments: Vec<AttachmentResponse>,
}

#[derive(Deserialize)]
struct AttachmentResponse {
	name: String,
	#[serde(alias = "download_url")]
	browser_download_url: String,
}

impl AttachmentResponse {
	fn download_url(&self) -> &str {
		&self.browser_download_url
	}
}
