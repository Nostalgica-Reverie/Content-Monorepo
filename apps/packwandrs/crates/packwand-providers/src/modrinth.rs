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
	/// A personal access token, when the user has linked their account.
	///
	/// Modrinth's read endpoints are public, so this is never required. What
	/// it buys is a higher rate limit and visibility of the user's own
	/// unlisted and draft projects, which are invisible anonymously — a
	/// pack author searching for their own unreleased mod otherwise gets no
	/// results and no explanation.
	token: Option<String>,
}

impl<T> ModrinthClient<T> {
	pub fn new(transport: T) -> Self {
		Self {
			transport,
			api_base: Url::parse(DEFAULT_API).expect("valid Modrinth API URL"),
			token: None,
		}
	}

	/// Attaches a token. An empty or blank string is treated as absent so
	/// callers can pass a config value straight through.
	#[must_use]
	pub fn with_token(mut self, token: Option<String>) -> Self {
		self.token = token
			.map(|value| value.trim().to_owned())
			.filter(|value| !value.is_empty());
		self
	}

	pub fn with_api_base(transport: T, api_base: &str) -> Result<Self, ProviderError> {
		let api_base =
			Url::parse(api_base).map_err(|error| ProviderError::InvalidUrl(error.to_string()))?;
		Ok(Self {
			transport,
			api_base,
			token: None,
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
		let mut request = HttpRequest::get(url.to_string());
		if let Some(token) = &self.token {
			request = request.header("Authorization", token);
		}
		let bytes = self.transport.get(request)?;
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

/// Modrinth's `/v2/search`, shaped into [`BrowsePage`].
///
/// Facets are Modrinth's filter syntax: an array of OR-groups that are ANDed
/// together, so `[["categories:fabric"],["versions:1.21.1"]]` means "fabric
/// AND 1.21.1". Building them here rather than in the UI keeps the encoding —
/// and the `modrinth_search_loaders` expansion that makes a Quilt pack also
/// match Fabric mods — in one place.
impl<T: Transport> crate::ProviderBrowser for ModrinthClient<T> {
	fn search(&self, query: &crate::BrowseQuery) -> Result<crate::BrowsePage, ProviderError> {
		let mut url = self.endpoint(&["search"])?;
		let mut facets: Vec<Vec<String>> = Vec::new();
		// Expanded, not passed through: a Quilt pack runs Fabric mods and a
		// NeoForge pack runs Forge mods, so searching for only the declared
		// loader would hide most of what the pack can actually use.
		let loaders = search_loaders(&query.loaders, false);
		if !loaders.is_empty() {
			facets.push(
				loaders
					.iter()
					.map(|loader| format!("categories:{loader}"))
					.collect(),
			);
		}
		if !query.game_versions.is_empty() {
			facets.push(
				query
					.game_versions
					.iter()
					.map(|version| format!("versions:{version}"))
					.collect(),
			);
		}
		facets.push(vec![format!(
			"project_type:{}",
			query.project_type.as_deref().unwrap_or("mod")
		)]);
		{
			let mut pairs = url.query_pairs_mut();
			pairs.append_pair("query", &query.text);
			pairs.append_pair("offset", &query.offset.to_string());
			pairs.append_pair("limit", &query.limit_or_default().to_string());
			pairs.append_pair(
				"facets",
				&serde_json::to_string(&facets).expect("facet list serializes"),
			);
		}
		let response: SearchResponse = self.get_json(url)?;
		Ok(crate::BrowsePage {
			projects: response.hits.into_iter().map(Into::into).collect(),
			total: response.total_hits,
			offset: query.offset,
		})
	}

	fn project(&self, id: &str) -> Result<crate::BrowseDetail, ProviderError> {
		let detail: ProjectDetailResponse = self.get_json(self.endpoint(&["project", id])?)?;
		Ok(crate::BrowseDetail {
			project: crate::BrowseProject {
				page_url: format!("https://modrinth.com/mod/{}", detail.slug),
				id: detail.id,
				slug: detail.slug,
				title: detail.title,
				summary: detail.description,
				icon_url: detail.icon_url.filter(|url| !url.is_empty()),
				// The project endpoint has no author field; the search hit
				// that led here carries it, so the UI keeps what it had.
				author: String::new(),
				downloads: detail.downloads,
				loaders: detail.loaders,
				game_versions: detail.game_versions,
				license: detail.license.map(|license| {
					if license.name.is_empty() {
						license.id
					} else {
						license.name
					}
				}),
				legacy_page_url: None,
			},
			body: detail.body,
			body_format: crate::BodyFormat::Markdown,
			gallery: detail
				.gallery
				.into_iter()
				.map(|image| crate::GalleryImage {
					url: image.url,
					title: image.title.unwrap_or_default(),
					description: image.description.unwrap_or_default(),
				})
				.collect(),
			source_url: detail.source_url.filter(|url| !url.is_empty()),
			issues_url: detail.issues_url.filter(|url| !url.is_empty()),
			wiki_url: detail.wiki_url.filter(|url| !url.is_empty()),
			discord_url: detail.discord_url.filter(|url| !url.is_empty()),
		})
	}

	fn creator(&self, handle: &str) -> Result<crate::CreatorProfile, ProviderError> {
		let user: UserResponse = self.get_json(self.endpoint(&["user", handle])?)?;
		// `/user/{id}/projects` returns full project records rather than search
		// hits, so it needs its own mapping.
		let projects: Vec<UserProjectResponse> =
			self.get_json(self.endpoint(&["user", handle, "projects"])?)?;
		Ok(crate::CreatorProfile {
			page_url: Some(format!("https://modrinth.com/user/{}", user.username)),
			handle: user.username.clone(),
			name: user
				.name
				.filter(|name| !name.is_empty())
				.unwrap_or(user.username),
			avatar_url: user.avatar_url.filter(|url| !url.is_empty()),
			bio: user.bio.unwrap_or_default(),
			joined: user.created,
			projects: projects.into_iter().map(Into::into).collect(),
			partial: false,
		})
	}
}

/// One Modrinth user, from `/v2/user/{id|username}`.
#[derive(Deserialize)]
struct UserResponse {
	username: String,
	#[serde(default)]
	name: Option<String>,
	#[serde(default)]
	avatar_url: Option<String>,
	#[serde(default)]
	bio: Option<String>,
	#[serde(default)]
	created: Option<String>,
}

/// One entry of `/v2/user/{id}/projects`.
///
/// Shaped like the project endpoint rather than a search hit: no `author`
/// field, and `description` rather than a summary. Kept separate from
/// [`ProjectDetailResponse`] because that one demands `body`, which this
/// listing omits.
#[derive(Deserialize)]
struct UserProjectResponse {
	id: String,
	slug: String,
	title: String,
	#[serde(default)]
	description: String,
	#[serde(default)]
	icon_url: Option<String>,
	#[serde(default)]
	downloads: u64,
	#[serde(default)]
	loaders: Vec<String>,
	#[serde(default)]
	game_versions: Vec<String>,
}

impl From<UserProjectResponse> for crate::BrowseProject {
	fn from(project: UserProjectResponse) -> Self {
		Self {
			page_url: format!("https://modrinth.com/mod/{}", project.slug),
			id: project.id,
			slug: project.slug,
			title: project.title,
			summary: project.description,
			icon_url: project.icon_url.filter(|url| !url.is_empty()),
			// The listing is already scoped to one user, so restating the
			// author on every row would be noise.
			author: String::new(),
			downloads: project.downloads,
			loaders: project.loaders,
			game_versions: project.game_versions,
			license: None,
			legacy_page_url: None,
		}
	}
}

#[derive(Deserialize)]
struct SearchResponse {
	#[serde(default)]
	hits: Vec<SearchHit>,
	#[serde(default)]
	total_hits: u64,
}

#[derive(Deserialize)]
struct SearchHit {
	project_id: String,
	slug: String,
	title: String,
	#[serde(default)]
	description: String,
	#[serde(default)]
	icon_url: Option<String>,
	#[serde(default)]
	author: String,
	#[serde(default)]
	downloads: u64,
	#[serde(default)]
	categories: Vec<String>,
	#[serde(default)]
	versions: Vec<String>,
	#[serde(default)]
	license: Option<String>,
}

impl From<SearchHit> for crate::BrowseProject {
	fn from(hit: SearchHit) -> Self {
		Self {
			page_url: format!("https://modrinth.com/mod/{}", hit.slug),
			id: hit.project_id,
			slug: hit.slug,
			title: hit.title,
			summary: hit.description,
			// Modrinth returns an empty string rather than omitting the field
			// when a project has no icon.
			icon_url: hit.icon_url.filter(|url| !url.is_empty()),
			author: hit.author,
			downloads: hit.downloads,
			loaders: hit.categories,
			game_versions: hit.versions,
			license: hit.license,
			legacy_page_url: None,
		}
	}
}

/// One Modrinth project, from `/v2/project/{id}`.
#[derive(Deserialize)]
struct ProjectDetailResponse {
	id: String,
	slug: String,
	title: String,
	#[serde(default)]
	description: String,
	#[serde(default)]
	body: String,
	#[serde(default)]
	icon_url: Option<String>,
	#[serde(default)]
	downloads: u64,
	#[serde(default)]
	loaders: Vec<String>,
	#[serde(default)]
	game_versions: Vec<String>,
	#[serde(default)]
	license: Option<LicenseResponse>,
	#[serde(default)]
	gallery: Vec<GalleryResponse>,
	#[serde(default)]
	source_url: Option<String>,
	#[serde(default)]
	issues_url: Option<String>,
	#[serde(default)]
	wiki_url: Option<String>,
	#[serde(default)]
	discord_url: Option<String>,
}

#[derive(Deserialize)]
struct LicenseResponse {
	#[serde(default)]
	id: String,
	#[serde(default)]
	name: String,
}

#[derive(Deserialize)]
struct GalleryResponse {
	url: String,
	#[serde(default)]
	title: Option<String>,
	#[serde(default)]
	description: Option<String>,
}
