use std::collections::BTreeMap;

use serde::Deserialize;
use url::Url;

use crate::{
	HttpRequest, ProjectType, ProviderError, ProviderKind, ProviderResolver, ReleaseChannel,
	ResolveRequest, ResolvedFile, ResolvedProject, ResolvedVersion, Transport,
};

const DEFAULT_API: &str = "https://api.curseforge.com/v1/";

// PackWand's client key. Runtime environment variables take precedence so
// releases can rotate it without requiring a new binary.
//
// Fallback if this one is ever revoked/rate-limited:
// $2a$10$sAYXjnU57A3JjsrbX3rUvOvQi64pKKpgCeilg5MC5PcJ/DXNiFYla
const DEFAULT_API_KEY: &str = "$2a$10$S2u1vjuHt8ITKW7df0D3ie0ualMc0UznUX/PYxzB8s90yIDRgU78S";

pub fn configured_api_key() -> String {
	[
		"PACKWAND_CURSEFORGE_API_KEY",
		"CURSEFORGE_API_KEY",
		"CF_API_KEY",
	]
	.into_iter()
	.filter_map(|name| std::env::var(name).ok())
	.map(|value| value.trim().to_owned())
	.find(|value| !value.is_empty())
	.unwrap_or_else(|| DEFAULT_API_KEY.to_owned())
}

/// Extracts the project slug and numeric file id from a CurseForge file page.
///
/// Project pages and file pages have different last path segments. Treating a
/// file page as a project URL used to search for a project named after the file
/// id, so a perfectly valid release URL could never be installed explicitly.
pub fn parse_file_url(value: &str) -> Option<(String, String)> {
	let url = Url::parse(value).ok()?;
	if !matches!(url.host_str()?, "curseforge.com" | "www.curseforge.com") {
		return None;
	}
	let segments = url
		.path_segments()?
		.filter(|segment| !segment.is_empty())
		.collect::<Vec<_>>();
	let files = segments.iter().position(|segment| *segment == "files")?;
	let slug = segments.get(files.checked_sub(1)?)?;
	let file_id = segments.get(files + 1)?;
	if file_id.parse::<u32>().is_err()
		|| !slug
			.bytes()
			.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
	{
		return None;
	}
	Some(((*slug).to_owned(), (*file_id).to_owned()))
}

/// Reports whether `key` is missing the "$<algo>$<cost>$" bcrypt-style prefix
/// CurseForge API keys always have. Both PowerShell and bash treat unescaped
/// $digits/$name inside double-quoted strings as variable interpolation,
/// which silently truncates a real key (e.g. "$2a$10$sAYX..." becomes
/// "a0/DXNiFYla") into something that will always be rejected.
fn looks_shell_mangled(key: &str) -> bool {
	!key.starts_with("$2")
}

fn auth_error(status: u16, api_key: &str, body_snippet: Option<&str>) -> ProviderError {
	// CurseForge's real API always answers with a JSON error payload. A
	// response that isn't JSON (typically an HTML page) never reached the
	// API at all — it was stopped at the edge by a CDN/WAF (e.g. CloudFront's
	// static "Request blocked" page), which returns the same status code
	// regardless of whether the key is valid.
	let looks_like_cdn_block = body_snippet.is_some_and(|body| !body.trim_start().starts_with('{'));

	let hint = if looks_like_cdn_block {
		format!(
			"the response body isn't JSON, which means the request never reached CurseForge's \
             API — it was blocked at the edge (CDN/WAF, e.g. CloudFront's \"Request blocked\" \
             page). This is unrelated to the key's validity; it usually means the network this \
             request came from is flagged (cloud/datacenter IP ranges, some VPNs). Body: {}",
			body_snippet
				.map(|body| body.chars().take(200).collect::<String>())
				.unwrap_or_default()
		)
	} else if looks_shell_mangled(api_key) {
		"the key does not start with the expected \"$2a$10$\"-style prefix, which usually \
         means it was set with double quotes and the shell/PowerShell interpolated the $ \
         characters — re-set it with single quotes instead"
			.to_owned()
	} else {
		let body = body_snippet
			.map(|body| format!(" CurseForge said: {body}"))
			.unwrap_or_default();
		format!(
			"set PACKWAND_CURSEFORGE_API_KEY, CURSEFORGE_API_KEY, or CF_API_KEY to override \
             it.{body}"
		)
	};
	ProviderError::CurseForgeAuthRejected { status, hint }
}

/// Either a URL ready to fetch, or a page to download the file from by hand.
/// See [`CurseForgeClient::download_url`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurseForgeDownload {
	pub url: Option<String>,
	pub page_url: Option<String>,
}

pub struct CurseForgeClient<T> {
	transport: T,
	api_base: Url,
	api_key: String,
}

impl<T> CurseForgeClient<T> {
	pub fn new(transport: T, api_key: impl Into<String>) -> Self {
		Self {
			transport,
			api_base: Url::parse(DEFAULT_API).expect("valid CurseForge API URL"),
			api_key: api_key.into(),
		}
	}

	pub fn with_api_base(
		transport: T,
		api_key: impl Into<String>,
		api_base: &str,
	) -> Result<Self, ProviderError> {
		let api_base =
			Url::parse(api_base).map_err(|error| ProviderError::InvalidUrl(error.to_string()))?;
		Ok(Self {
			transport,
			api_base,
			api_key: api_key.into(),
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
		let request = HttpRequest::get(url.to_string())
			.header("Accept", "application/json")
			.header("X-API-Key", &self.api_key);
		let bytes = self
			.transport
			.get(request)
			.map_err(|error| match error.status {
				Some(status @ (401 | 403)) => {
					auth_error(status, &self.api_key, error.body_snippet.as_deref())
				}
				_ => ProviderError::Transport(error),
			})?;
		serde_json::from_slice(&bytes).map_err(|error| ProviderError::Decode {
			provider: "CurseForge",
			message: error.to_string(),
		})
	}
}

impl<T: Transport> ProviderResolver for CurseForgeClient<T> {
	fn resolve(&self, request: &ResolveRequest) -> Result<ResolvedProject, ProviderError> {
		let project = match request.project.parse::<u32>() {
			Ok(project_id) => {
				let response: ProjectEnvelope =
					self.get_json(self.endpoint(&["mods", &project_id.to_string()])?)?;
				if response.data.id != project_id {
					return Err(ProviderError::InvalidResponse(format!(
						"expected project {project_id}, got {}",
						response.data.id
					)));
				}
				response.data
			}
			Err(_) => {
				let slug = curseforge_slug(&request.project)?;
				let mut url = self.endpoint(&["mods", "search"])?;
				url.query_pairs_mut()
					.append_pair("gameId", "432")
					.append_pair("slug", &slug);
				let response: SearchEnvelope = self.get_json(url)?;
				response
					.data
					.into_iter()
					.find(|project| project.slug.eq_ignore_ascii_case(&slug))
					.ok_or(ProviderError::InvalidCurseForgeProject)?
			}
		};
		let file = if let Some(file_id) = &request.version_id {
			let file_id = file_id.parse::<u32>().map_err(|_| {
				ProviderError::InvalidResponse(format!("invalid CurseForge file id {file_id:?}"))
			})?;
			let response: FileEnvelope = self.get_json(self.endpoint(&[
				"mods",
				&project.id.to_string(),
				"files",
				&file_id.to_string(),
			])?)?;
			response.data
		} else {
			let latest_index = project
				.latest_files_indexes
				.iter()
				.filter(|entry| entry.compatible(request))
				// The API can return one compatible entry per release channel.
				// File IDs are monotonic upload IDs, so choose the newest allowed
				// file rather than depending on the response order.
				.max_by_key(|entry| entry.file_id);
			let latest_file = project
				.latest_files
				.iter()
				.filter(|file| file.compatible(request))
				.max_by_key(|file| file.id);
			if let Some(index_entry) =
				latest_index.filter(|entry| latest_file.is_none_or(|file| entry.file_id >= file.id))
			{
				// `latestFiles` is only a short recent-upload list. It can retain
				// an older compatible file while its newer replacement is absent;
				// the indexes are CurseForge's authoritative latest-file mapping
				// for each game-version/loader/release-type combination.
				let response: FileEnvelope = self.get_json(self.endpoint(&[
					"mods",
					&project.id.to_string(),
					"files",
					&index_entry.file_id.to_string(),
				])?)?;
				response.data
			} else if let Some(file) = latest_file {
				file.clone()
			} else {
				return Err(ProviderError::NoCompatibleVersion);
			}
		};
		let hashes = file.hashes();
		if hashes.is_empty() {
			return Err(ProviderError::NoUsableHash);
		}
		Ok(ResolvedProject {
			provider: ProviderKind::CurseForge,
			id: project.id.to_string(),
			slug: project.slug,
			title: project.name,
			project_type: class_to_project_type(project.class_id),
			side: "both".to_string(),
			repository_release: None,
			version: ResolvedVersion {
				id: file.id.to_string(),
				name: file.display_name.clone(),
				number: file.display_name.clone(),
				channel: file.channel(),
				file: ResolvedFile {
					filename: file.file_name.clone(),
					// CurseForge API terms prohibit persisting the temporary URL.
					url: None,
					// The Go metadata contract currently omits CF file length.
					size: 0,
					hashes,
				},
			},
		})
	}
}

impl<T: Transport> CurseForgeClient<T> {
	/// The result of resolving one file's live download: either a URL ready
	/// to fetch, or (when the author has disabled third-party distribution,
	/// a real and permanent CurseForge state) a human page to download it
	/// from by hand.
	///
	/// Deliberately separate from [`ProviderResolver::resolve`], which nulls
	/// the URL field: CurseForge's terms forbid persisting it in pack
	/// metadata, but an installer needs it in memory just long enough to
	/// stream the download right now. Callers must never write `url` to disk.
	pub fn download_url(
		&self,
		project_id: u32,
		file_id: u32,
	) -> Result<CurseForgeDownload, ProviderError> {
		let response: FileEnvelope = self.get_json(self.endpoint(&[
			"mods",
			&project_id.to_string(),
			"files",
			&file_id.to_string(),
		])?)?;
		if let Some(url) = response.data.download_url {
			return Ok(CurseForgeDownload {
				url: Some(url),
				page_url: None,
			});
		}
		// Distribution disabled: only fetched on this rarer path, since it
		// costs an extra request the common (allowed) case doesn't need.
		let page_url = self.project_page_url(project_id, file_id).ok();
		Ok(CurseForgeDownload {
			url: None,
			page_url,
		})
	}

	fn project_page_url(&self, project_id: u32, file_id: u32) -> Result<String, ProviderError> {
		let response: ProjectEnvelope = self.get_json(self.endpoint(&["mods", &project_id.to_string()])?)?;
		Ok(format!(
			"https://www.curseforge.com/minecraft/mc-mods/{}/files/{file_id}",
			response.data.slug
		))
	}

	/// Match CurseForge's whitespace-normalized Murmur2 fingerprints to exact
	/// project/file pairs. Partial and unmatched fingerprints are retained so
	/// callers can report every local file without guessing.
	pub fn match_fingerprints(
		&self,
		fingerprints: &[u32],
	) -> Result<FingerprintMatches, ProviderError> {
		let url = self.endpoint(&["fingerprints"])?;
		let request = HttpRequest::get(url.to_string())
			.header("Accept", "application/json")
			.header("X-API-Key", &self.api_key);
		let body = serde_json::to_vec(&serde_json::json!({
			"fingerprints": fingerprints,
		}))
		.map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
		let bytes =
			self.transport
				.post_json(request, &body)
				.map_err(|error| match error.status {
					Some(status @ (401 | 403)) => {
						auth_error(status, &self.api_key, error.body_snippet.as_deref())
					}
					_ => ProviderError::Transport(error),
				})?;
		let response: FingerprintEnvelope =
			serde_json::from_slice(&bytes).map_err(|error| ProviderError::Decode {
				provider: "CurseForge",
				message: error.to_string(),
			})?;
		Ok(FingerprintMatches {
			exact: response
				.data
				.exact_matches
				.into_iter()
				.map(|item| FingerprintMatch {
					fingerprint: item.file.file_fingerprint,
					project_id: item.id,
					file_id: item.file.id,
				})
				.collect(),
			partial: response.data.partial_matches,
			unmatched: response.data.unmatched_fingerprints,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FingerprintMatches {
	pub exact: Vec<FingerprintMatch>,
	pub partial: Vec<u32>,
	pub unmatched: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FingerprintMatch {
	pub fingerprint: u32,
	pub project_id: u32,
	pub file_id: u32,
}

#[derive(Deserialize)]
struct FingerprintEnvelope {
	data: FingerprintResponse,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FingerprintResponse {
	#[serde(default)]
	exact_matches: Vec<FingerprintResponseMatch>,
	#[serde(default)]
	partial_matches: Vec<u32>,
	#[serde(default)]
	unmatched_fingerprints: Vec<u32>,
}

#[derive(Deserialize)]
struct FingerprintResponseMatch {
	id: u32,
	file: FileResponse,
}

fn class_to_project_type(class_id: u32) -> ProjectType {
	match class_id {
		5 => ProjectType::Plugin,
		6 => ProjectType::Mod,
		12 => ProjectType::ResourcePack,
		6552 => ProjectType::Shader,
		6945 => ProjectType::DataPack,
		_ => ProjectType::Other,
	}
}

#[derive(Deserialize)]
struct ProjectEnvelope {
	data: ProjectResponse,
}

#[derive(Deserialize)]
struct SearchEnvelope {
	data: Vec<ProjectResponse>,
}

#[derive(Deserialize)]
struct FileEnvelope {
	data: FileResponse,
}

fn curseforge_slug(value: &str) -> Result<String, ProviderError> {
	let slug = if let Ok(url) = Url::parse(value) {
		url.path_segments()
			.and_then(|mut segments| segments.rfind(|part| !part.is_empty()))
			.unwrap_or("")
			.to_owned()
	} else {
		value.to_owned()
	};
	if slug.is_empty()
		|| !slug
			.bytes()
			.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
	{
		Err(ProviderError::InvalidCurseForgeProject)
	} else {
		Ok(slug)
	}
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectResponse {
	id: u32,
	name: String,
	slug: String,
	#[serde(default)]
	class_id: u32,
	#[serde(default)]
	latest_files: Vec<FileResponse>,
	// `latestFiles` is a short, arbitrarily-picked recent-uploads list, not
	// guaranteed to contain an entry for every supported (game version,
	// loader) pair — popular mods that publish simultaneous multi-loader
	// builds routinely have the wanted loader's file missing from it even
	// though it exists. `latestFilesIndexes` is CurseForge's purpose-built
	// index with exactly one entry per (gameVersion, modLoader, releaseType)
	// combination and is the reliable source for this lookup.
	#[serde(default)]
	latest_files_indexes: Vec<FileIndexEntry>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileIndexEntry {
	file_id: u32,
	#[serde(default)]
	game_version: String,
	#[serde(default)]
	mod_loader: Option<u8>,
	#[serde(default)]
	release_type: u8,
}

impl FileIndexEntry {
	fn channel(&self) -> ReleaseChannel {
		match self.release_type {
			2 => ReleaseChannel::Beta,
			3 => ReleaseChannel::Alpha,
			_ => ReleaseChannel::Release,
		}
	}

	fn compatible(&self, request: &ResolveRequest) -> bool {
		let channel_matches =
			request.channels.is_empty() || request.channels.contains(&self.channel());
		let game_matches = request
			.game_versions
			.iter()
			.any(|wanted| wanted == &self.game_version);
		let loader_matches = request.loaders.is_empty()
			|| request
				.loaders
				.iter()
				.any(|wanted| mod_loader_id(wanted) == self.mod_loader);
		channel_matches && game_matches && loader_matches
	}
}

/// Maps a loader name (as used in pack.toml/CLI flags) to CurseForge's
/// numeric modLoaderType, matching the values CurseForge's own API expects
/// (see https://docs.curseforge.com/rest-api/#tocS_ModLoaderType).
fn mod_loader_id(name: &str) -> Option<u8> {
	match name.to_ascii_lowercase().as_str() {
		"forge" => Some(1),
		"cauldron" => Some(2),
		"liteloader" => Some(3),
		"fabric" => Some(4),
		"quilt" => Some(5),
		"neoforge" => Some(6),
		_ => None,
	}
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileResponse {
	id: u32,
	file_name: String,
	display_name: String,
	release_type: u8,
	#[serde(default)]
	game_versions: Vec<String>,
	#[serde(default)]
	file_fingerprint: u32,
	#[serde(default)]
	hashes: Vec<FileHash>,
	/// `null` when CurseForge's third-party-download restriction applies to
	/// this file. Only [`CurseForgeClient::download_url`] reads this field;
	/// [`ProviderResolver::resolve`] discards it regardless of the API's
	/// value, since that path feeds persisted pack metadata.
	#[serde(default)]
	download_url: Option<String>,
}

impl FileResponse {
	fn channel(&self) -> ReleaseChannel {
		match self.release_type {
			2 => ReleaseChannel::Beta,
			3 => ReleaseChannel::Alpha,
			_ => ReleaseChannel::Release,
		}
	}

	fn compatible(&self, request: &ResolveRequest) -> bool {
		let channel_matches =
			request.channels.is_empty() || request.channels.contains(&self.channel());
		let game_matches = request.game_versions.is_empty()
			|| request
				.game_versions
				.iter()
				.any(|wanted| self.game_versions.iter().any(|found| found == wanted));
		let loader_matches = request.loaders.is_empty()
			|| request.loaders.iter().any(|wanted| {
				self.game_versions
					.iter()
					.any(|found| found.eq_ignore_ascii_case(wanted))
			});
		channel_matches && game_matches && loader_matches
	}

	fn hashes(&self) -> BTreeMap<String, String> {
		let mut hashes = BTreeMap::new();
		if self.file_fingerprint != 0 {
			hashes.insert("murmur2".to_string(), self.file_fingerprint.to_string());
		}
		for hash in &self.hashes {
			let algorithm = match hash.algorithm {
				1 => "sha1",
				2 => "md5",
				_ => continue,
			};
			if !hash.value.is_empty() {
				hashes.insert(algorithm.to_string(), hash.value.clone());
			}
		}
		hashes
	}
}

#[derive(Clone, Deserialize)]
struct FileHash {
	value: String,
	#[serde(rename = "algo")]
	algorithm: u8,
}

/// CurseForge's `mods/search`, shaped into [`crate::BrowsePage`].
///
/// `gameId=432` is Minecraft, and `classId` selects the section — 6 is Mods,
/// which is the default here. Unlike Modrinth, CurseForge takes one loader and
/// one game version rather than a filter set, so only the first of each is
/// sent; narrowing further happens client-side on the returned page.
impl<T: Transport> crate::ProviderBrowser for CurseForgeClient<T> {
	fn search(&self, query: &crate::BrowseQuery) -> Result<crate::BrowsePage, ProviderError> {
		let mut url = self.endpoint(&["mods", "search"])?;
		{
			let mut pairs = url.query_pairs_mut();
			pairs.append_pair("gameId", "432");
			pairs.append_pair("classId", class_id_for(query.project_type.as_deref()));
			pairs.append_pair("index", &query.offset.to_string());
			pairs.append_pair("pageSize", &query.limit_or_default().to_string());
			if !query.text.trim().is_empty() {
				pairs.append_pair("searchFilter", query.text.trim());
			}
			if let Some(loader) = query.loaders.first()
				&& let Some(id) = mod_loader_id(loader)
			{
				pairs.append_pair("modLoaderType", &id.to_string());
			}
			if let Some(version) = query.game_versions.first() {
				pairs.append_pair("gameVersion", version);
			}
		}
		let response: BrowseEnvelope = self.get_json(url)?;
		Ok(crate::BrowsePage {
			projects: response.data.into_iter().map(Into::into).collect(),
			total: response
				.pagination
				.map(|page| page.total_count)
				.unwrap_or(0),
			offset: query.offset,
		})
	}

	fn project(&self, id: &str) -> Result<crate::BrowseDetail, ProviderError> {
		let envelope: BrowseDetailEnvelope = self.get_json(self.endpoint(&["mods", id])?)?;
		let links = envelope.data.links.clone();
		let screenshots = envelope.data.screenshots;
		let project: crate::BrowseProject = envelope.data.summary.into();
		// The description is a separate endpoint and arrives as rendered HTML;
		// CurseForge exposes no markdown source. A failure there is not fatal —
		// the rest of the page is still worth showing.
		let body = self
			.get_json::<DescriptionEnvelope>(self.endpoint(&["mods", id, "description"])?)
			.map(|envelope| envelope.data)
			.unwrap_or_default();
		Ok(crate::BrowseDetail {
			project,
			body,
			body_format: crate::BodyFormat::Html,
			gallery: screenshots
				.into_iter()
				.map(|shot| crate::GalleryImage {
					url: shot.url,
					title: shot.title.unwrap_or_default(),
					description: shot.description.unwrap_or_default(),
				})
				.collect(),
			source_url: links.as_ref().and_then(|links| links.source_url.clone()),
			issues_url: links.as_ref().and_then(|links| links.issues_url.clone()),
			wiki_url: links.as_ref().and_then(|links| links.wiki_url.clone()),
			discord_url: None,
		})
	}

	/// A reconstructed author view, because CurseForge has no user resource.
	///
	/// Their API exposes authors only as a `{ id, name, url }` triple embedded
	/// in each mod payload — there is no `/v1/users/{id}` to call, and no way
	/// to list a given author's mods. The closest available answer is a search
	/// for the author's name filtered down to exact author matches, which
	/// finds their popular work and misses the long tail.
	///
	/// This returns `partial: true` so the UI states the limitation instead of
	/// presenting a thin result as if it were the whole profile.
	fn creator(&self, handle: &str) -> Result<crate::CreatorProfile, ProviderError> {
		let query = crate::BrowseQuery {
			text: handle.to_owned(),
			limit: 50,
			..crate::BrowseQuery::default()
		};
		let page = self.search(&query)?;
		let wanted = handle.trim().to_ascii_lowercase();
		let projects = page
			.projects
			.into_iter()
			.filter(|project| project.author.trim().to_ascii_lowercase() == wanted)
			.collect();
		Ok(crate::CreatorProfile {
			handle: handle.to_owned(),
			name: handle.to_owned(),
			avatar_url: None,
			bio: String::new(),
			joined: None,
			page_url: None,
			projects,
			partial: true,
		})
	}
}

#[derive(Deserialize)]
struct BrowseDetailEnvelope {
	data: DetailResponse,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailResponse {
	#[serde(flatten)]
	summary: BrowseProjectResponse,
	#[serde(default)]
	screenshots: Vec<Screenshot>,
	#[serde(default)]
	links: Option<DetailLinks>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailLinks {
	#[serde(default)]
	source_url: Option<String>,
	#[serde(default)]
	issues_url: Option<String>,
	#[serde(default)]
	wiki_url: Option<String>,
}

#[derive(Deserialize)]
struct Screenshot {
	url: String,
	#[serde(default)]
	title: Option<String>,
	#[serde(default)]
	description: Option<String>,
}

#[derive(Default, Deserialize)]
struct DescriptionEnvelope {
	#[serde(default)]
	data: String,
}

/// CurseForge section ids. 6 is Mods, 12 Resource Packs, 17 Worlds,
/// 4471 Modpacks, 6552 Shaders.
fn class_id_for(project_type: Option<&str>) -> &'static str {
	match project_type {
		Some("resourcepack") => "12",
		Some("modpack") => "4471",
		Some("shader") => "6552",
		_ => "6",
	}
}

#[derive(Deserialize)]
struct BrowseEnvelope {
	#[serde(default)]
	data: Vec<BrowseProjectResponse>,
	#[serde(default)]
	pagination: Option<Pagination>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pagination {
	#[serde(default)]
	total_count: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowseProjectResponse {
	id: u32,
	name: String,
	slug: String,
	#[serde(default)]
	summary: String,
	#[serde(default)]
	logo: Option<Logo>,
	#[serde(default)]
	authors: Vec<Author>,
	#[serde(default)]
	download_count: f64,
	#[serde(default)]
	latest_files_indexes: Vec<FileIndexEntry>,
	#[serde(default)]
	links: Option<Links>,
}

#[derive(Deserialize)]
struct Logo {
	#[serde(default)]
	url: String,
}

#[derive(Deserialize)]
struct Author {
	#[serde(default)]
	name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Links {
	#[serde(default)]
	website_url: Option<String>,
}

impl From<BrowseProjectResponse> for crate::BrowseProject {
	fn from(project: BrowseProjectResponse) -> Self {
		let page_url = project
			.links
			.and_then(|links| links.website_url)
			.filter(|url| !url.is_empty())
			.unwrap_or_else(|| {
				format!(
					"https://www.curseforge.com/minecraft/mc-mods/{}",
					project.slug
				)
			});
		// Legacy CurseForge serves the same catalogue from a different host,
		// so the link is a host swap rather than a second lookup.
		let legacy_page_url = page_url
			.replace(
				"https://www.curseforge.com",
				"https://legacy.curseforge.com",
			)
			.replace("https://curseforge.com", "https://legacy.curseforge.com");
		let mut game_versions: Vec<String> = project
			.latest_files_indexes
			.iter()
			.map(|entry| entry.game_version.clone())
			.collect();
		game_versions.sort();
		game_versions.dedup();
		Self {
			id: project.id.to_string(),
			slug: project.slug,
			title: project.name,
			summary: project.summary,
			icon_url: project
				.logo
				.map(|logo| logo.url)
				.filter(|url| !url.is_empty()),
			author: project
				.authors
				.into_iter()
				.next()
				.map(|author| author.name)
				.unwrap_or_default(),
			// CurseForge reports download counts as a float.
			downloads: project.download_count.max(0.0) as u64,
			loaders: Vec::new(),
			game_versions,
			license: None,
			legacy_page_url: Some(legacy_page_url),
			page_url,
		}
	}
}
