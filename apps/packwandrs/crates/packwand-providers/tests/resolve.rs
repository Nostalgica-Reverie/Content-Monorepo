use std::collections::VecDeque;
use std::sync::Mutex;

use packwand_providers::{
	CurseForgeClient, ForgejoClient, GitHubClient, GitLabClient, HttpRequest, ModrinthClient,
	ProviderResolver, ReleaseChannel, ResolveRequest, Transport, TransportError, parse_file_url,
};

struct FixtureTransport {
	responses: Mutex<VecDeque<Vec<u8>>>,
	requests: Mutex<Vec<HttpRequest>>,
	post_bodies: Mutex<Vec<Vec<u8>>>,
}

impl FixtureTransport {
	fn new(responses: impl IntoIterator<Item = &'static str>) -> Self {
		Self {
			responses: Mutex::new(
				responses
					.into_iter()
					.map(|response| response.as_bytes().to_vec())
					.collect(),
			),
			requests: Mutex::new(Vec::new()),
			post_bodies: Mutex::new(Vec::new()),
		}
	}
}

impl Transport for &FixtureTransport {
	fn get(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError> {
		self.requests.lock().unwrap().push(request);
		self.responses
			.lock()
			.unwrap()
			.pop_front()
			.ok_or_else(|| TransportError {
				url: "fixture".into(),
				message: "no response left".into(),
				status: None,
				body_snippet: None,
			})
	}

	fn post_json(&self, request: HttpRequest, body: &[u8]) -> Result<Vec<u8>, TransportError> {
		self.requests.lock().unwrap().push(request);
		self.post_bodies.lock().unwrap().push(body.to_vec());
		self.responses
			.lock()
			.unwrap()
			.pop_front()
			.ok_or_else(|| TransportError {
				url: "fixture".into(),
				message: "no response left".into(),
				status: None,
				body_snippet: None,
			})
	}
}

fn request(project: &str) -> ResolveRequest {
	ResolveRequest {
		project: project.to_string(),
		version_id: None,
		version_filename: None,
		game_versions: vec!["1.21.1".into()],
		loaders: vec!["fabric".into()],
		channels: vec![ReleaseChannel::Release],
		branch: None,
		asset_pattern: None,
	}
}

#[test]
fn curseforge_file_url_preserves_the_project_and_exact_file() {
	assert_eq!(
		parse_file_url("https://www.curseforge.com/minecraft/mc-mods/sodium/files/8396428"),
		Some(("sodium".into(), "8396428".into()))
	);
	assert_eq!(parse_file_url("https://example.test/files/8396428"), None);
	assert_eq!(
		parse_file_url("https://www.curseforge.com/minecraft/mc-mods/sodium"),
		None
	);
}

#[test]
fn curseforge_matches_fingerprints_with_a_json_post() {
	let transport = FixtureTransport::new([r#"{
      "data": {
        "exactMatches": [{
          "id": 42,
          "file": {
            "id": 99,
            "fileName": "example.jar",
            "displayName": "Example",
            "releaseType": 1,
            "fileFingerprint": 1234
          }
        }],
        "partialMatches": [2345],
        "unmatchedFingerprints": [3456]
      }
    }"#]);
	let client =
		CurseForgeClient::with_api_base(&transport, "secret", "https://example.test/v1/").unwrap();

	let matches = client.match_fingerprints(&[1234, 2345, 3456]).unwrap();

	assert_eq!(matches.exact[0].fingerprint, 1234);
	assert_eq!(matches.exact[0].project_id, 42);
	assert_eq!(matches.exact[0].file_id, 99);
	assert_eq!(matches.partial, vec![2345]);
	assert_eq!(matches.unmatched, vec![3456]);
	let requests = transport.requests.lock().unwrap();
	assert_eq!(requests[0].url, "https://example.test/v1/fingerprints");
	assert!(
		requests[0]
			.headers
			.iter()
			.any(|(name, value)| name == "X-API-Key" && value == "secret")
	);
	let bodies = transport.post_bodies.lock().unwrap();
	assert_eq!(
		serde_json::from_slice::<serde_json::Value>(&bodies[0]).unwrap(),
		serde_json::json!({"fingerprints": [1234, 2345, 3456]})
	);
}

#[test]
fn modrinth_resolves_primary_file_and_go_compatible_metadata() {
	let transport = FixtureTransport::new([
		r#"{
          "id":"AANobbMI","slug":"sodium","title":"Sodium","project_type":"mod",
          "client_side":"required","server_side":"unsupported"
        }"#,
		r#"[{
          "id":"version-1","name":"Sodium 1.0","version_number":"1.0.0","version_type":"release",
          "files":[
            {"hashes":{"sha1":"old"},"url":"https://cdn/secondary.jar","filename":"secondary.jar","primary":false,"size":2},
            {"hashes":{"sha1":"abc","sha512":"def"},"url":"https://cdn/sodium.jar","filename":"sodium.jar","primary":true,"size":42}
          ]
        }]"#,
	]);
	let client = ModrinthClient::with_api_base(&transport, "https://example.test/v2/").unwrap();
	let resolved = client.resolve(&request("sodium")).unwrap();
	assert_eq!(resolved.metadata_path(), "mods/sodium.pw.json");
	assert_eq!(resolved.side, "client");
	assert_eq!(resolved.version.file.filename, "sodium.jar");

	let metadata = resolved.into_mod().unwrap();
	assert_eq!(metadata.download.hash_format, "sha512");
	assert_eq!(metadata.download.hash, "def");
	assert_eq!(metadata.download.extra_hashes["sha1"], "abc");
	assert_eq!(metadata.download.size, 42);
	assert_eq!(
		metadata.update["modrinth"]["mod-id"].as_str(),
		Some("AANobbMI")
	);
	assert_eq!(
		metadata.update["modrinth"]["version"].as_str(),
		Some("version-1")
	);

	let requests = transport.requests.lock().unwrap();
	assert_eq!(requests.len(), 2);
	assert!(requests[0].url.ends_with("/v2/project/sodium"));
	assert!(requests[1].url.contains("game_versions"));
	assert!(requests[1].url.contains("loaders"));
}

#[test]
fn curseforge_filters_files_and_preserves_numeric_update_ids() {
	let transport = FixtureTransport::new([r#"{
      "data": {
        "id": 238222, "name": "Just Enough Items", "slug": "jei", "classId": 6,
        "latestFiles": [
          {"id": 9, "fileName":"jei-beta.jar", "displayName":"JEI beta", "releaseType":2,
           "gameVersions":["1.21.1","Fabric"], "fileFingerprint":123, "hashes":[]},
          {"id": 10, "fileName":"jei.jar", "displayName":"JEI 1.0", "releaseType":1,
           "gameVersions":["1.21.1","Fabric"], "fileFingerprint":456,
           "hashes":[{"value":"sha-one","algo":1},{"value":"md-five","algo":2}]}
        ]
      }
    }"#]);
	let client =
		CurseForgeClient::with_api_base(&transport, "test-key", "https://example.test/v1/")
			.unwrap();
	let resolved = client.resolve(&request("238222")).unwrap();
	assert_eq!(resolved.metadata_path(), "mods/jei.pw.json");
	assert_eq!(resolved.version.id, "10");
	assert!(resolved.version.file.url.is_none());

	let metadata = resolved.into_mod().unwrap();
	assert_eq!(metadata.download.hash_format, "sha1");
	assert_eq!(metadata.download.hash, "sha-one");
	assert_eq!(metadata.download.mode, "metadata:curseforge");
	assert!(metadata.download.url.is_empty());
	assert!(metadata.download.extra_hashes.is_empty());
	assert_eq!(
		metadata.update["curseforge"]["project-id"].as_i64(),
		Some(238222)
	);
	assert_eq!(metadata.update["curseforge"]["file-id"].as_i64(), Some(10));

	let requests = transport.requests.lock().unwrap();
	assert_eq!(requests.len(), 1);
	assert!(
		requests[0]
			.headers
			.iter()
			.any(|(name, value)| name == "X-API-Key" && value == "test-key")
	);
}

#[test]
fn curseforge_falls_back_to_latest_files_indexes_when_latest_files_lacks_the_loader() {
	// Regression test: `latestFiles` is a short, arbitrarily-picked list of
	// recent uploads and can omit a build for the requested loader even
	// though CurseForge has one — only `latestFilesIndexes` reliably has one
	// entry per (gameVersion, modLoader) pair. Here `latestFiles` only has a
	// NeoForge build; the Fabric build must be found via the index and
	// fetched by file ID.
	let transport = FixtureTransport::new([
		r#"{
      "data": {
        "id": 455508, "name": "Iris Shaders", "slug": "irisshaders", "classId": 6,
        "latestFiles": [
          {"id": 88, "fileName":"iris-neoforge.jar", "displayName":"Iris NeoForge", "releaseType":1,
           "gameVersions":["26.2","NeoForge"], "fileFingerprint":111, "hashes":[]}
        ],
        "latestFilesIndexes": [
          {"fileId": 88, "gameVersion": "26.2", "modLoader": 6, "releaseType": 1},
          {"fileId": 77, "gameVersion": "26.2", "modLoader": 4, "releaseType": 1}
        ]
      }
    }"#,
		r#"{
      "data": {"id": 77, "fileName":"iris-fabric.jar", "displayName":"Iris Fabric", "releaseType":1,
       "gameVersions":["26.2","Fabric"], "fileFingerprint":222, "hashes":[]}
    }"#,
	]);
	let client =
		CurseForgeClient::with_api_base(&transport, "test-key", "https://example.test/v1/")
			.unwrap();
	let mut req = request("455508");
	req.game_versions = vec!["26.2".into()];
	let resolved = client.resolve(&req).unwrap();

	assert_eq!(resolved.version.id, "77");
	assert_eq!(resolved.version.file.filename, "iris-fabric.jar");

	let requests = transport.requests.lock().unwrap();
	assert_eq!(requests.len(), 2);
	assert!(requests[1].url.ends_with("/mods/455508/files/77"));
}

#[test]
fn curseforge_uses_the_newest_compatible_file_index() {
	// CurseForge's short `latestFiles` list can contain a stale compatible
	// build. The index is the authoritative latest selection for the exact
	// Minecraft version and loader, and its newest ID wins across channels.
	let transport = FixtureTransport::new([
		r#"{
      "data": {
        "id": 394468, "name": "Sodium", "slug": "sodium", "classId": 6,
        "latestFiles": [
          {"id": 8378327, "fileName":"sodium-fabric-0.9.1-beta.4+mc26.2.jar",
           "displayName":"Sodium 0.9.1", "releaseType":2,
           "gameVersions":["26.2","Fabric"], "fileFingerprint":111, "hashes":[]}
        ],
        "latestFilesIndexes": [
          {"fileId": 8378327, "gameVersion": "26.2", "modLoader": 4, "releaseType": 2},
          {"fileId": 9000000, "gameVersion": "26.2", "modLoader": 4, "releaseType": 1}
        ]
      }
    }"#,
		r#"{
      "data": {"id": 9000000, "fileName":"sodium-fabric-newer+mc26.2.jar",
       "displayName":"Sodium newer", "releaseType":2,
       "gameVersions":["26.2","Fabric"], "fileFingerprint":222, "hashes":[]}
    }"#,
	]);
	let client =
		CurseForgeClient::with_api_base(&transport, "test-key", "https://example.test/v1/")
			.unwrap();
	let mut req = request("394468");
	req.game_versions = vec!["26.2".into()];
	req.channels = vec![
		ReleaseChannel::Release,
		ReleaseChannel::Beta,
		ReleaseChannel::Alpha,
	];

	let resolved = client.resolve(&req).unwrap();

	assert_eq!(resolved.version.id, "9000000");
	assert_eq!(
		resolved.version.file.filename,
		"sodium-fabric-newer+mc26.2.jar"
	);
	let requests = transport.requests.lock().unwrap();
	assert_eq!(requests.len(), 2);
	assert!(requests[1].url.ends_with("/mods/394468/files/9000000"));
}

#[test]
fn curseforge_uses_a_newer_compatible_latest_file_when_the_index_is_stale() {
	let transport = FixtureTransport::new([r#"{
      "data": {
        "id": 1460602, "name": "Fast Noise", "slug": "zfastnoise", "classId": 6,
        "latestFiles": [
          {"id":9000000, "fileName":"zfastnoise-1.0.40+26.2.jar",
           "displayName":"Fast Noise 1.0.40", "releaseType":1,
           "gameVersions":["26.2","Fabric"], "fileFingerprint":222, "hashes":[]}
        ],
        "latestFilesIndexes": [
          {"fileId": 8378327, "gameVersion": "26.2", "modLoader": 4, "releaseType": 2}
        ]
      }
    }"#]);
	let client =
		CurseForgeClient::with_api_base(&transport, "test-key", "https://example.test/v1/")
			.unwrap();
	let mut req = request("1460602");
	req.game_versions = vec!["26.2".into()];
	req.channels = vec![
		ReleaseChannel::Release,
		ReleaseChannel::Beta,
		ReleaseChannel::Alpha,
	];

	let resolved = client.resolve(&req).unwrap();

	assert_eq!(resolved.version.id, "9000000");
	assert_eq!(resolved.version.file.filename, "zfastnoise-1.0.40+26.2.jar");
	assert_eq!(transport.requests.lock().unwrap().len(), 1);
}

#[test]
fn curseforge_resolves_exact_slugs_through_search() {
	let transport = FixtureTransport::new([r#"{
      "data": [{
        "id": 238222, "name": "Just Enough Items", "slug": "jei", "classId": 6,
        "latestFiles": [{"id": 10, "fileName":"jei.jar", "displayName":"JEI 1.0", "releaseType":1,
          "gameVersions":["1.21.1","Fabric"], "fileFingerprint":456,
          "hashes":[{"value":"sha-one","algo":1}]}]
      }]
    }"#]);
	let client =
		CurseForgeClient::with_api_base(&transport, "test-key", "https://example.test/v1/")
			.unwrap();
	let resolved = client.resolve(&request("jei")).unwrap();
	assert_eq!(resolved.id, "238222");
	let requests = transport.requests.lock().unwrap();
	assert!(requests[0].url.contains("mods/search"));
	assert!(requests[0].url.contains("slug=jei"));
}

#[test]
fn github_resolves_a_branch_release_and_persists_update_options() {
	let transport = FixtureTransport::new([
		r#"{"name":"Example Mod","full_name":"owner/example"}"#,
		r#"[
          {"tag_name":"wrong","target_commitish":"dev","name":"Dev","assets":[]},
          {"tag_name":"v1.2.3","target_commitish":"main","name":"Release 1.2.3","assets":[
            {"name":"example-sources.jar","browser_download_url":"https://downloads/sources.jar"},
            {"name":"example.jar","browser_download_url":"https://downloads/example.jar"}
          ]}
        ]"#,
		"github artifact",
	]);
	let client =
		GitHubClient::with_api_base(&transport, "secret", "https://example.test/api/").unwrap();
	let mut resolve_request = request("https://github.com/owner/example/releases/latest");
	resolve_request.branch = Some("main".into());
	let resolved = client.resolve(&resolve_request).unwrap();
	assert_eq!(resolved.metadata_path(), "mods/example-mod.pw.json");
	assert_eq!(resolved.version.file.filename, "example.jar");

	let metadata = resolved.into_mod().unwrap();
	assert_eq!(metadata.download.hash_format, "sha512");
	assert_eq!(metadata.download.hash.len(), 128);
	assert_eq!(
		metadata.update["github"]["slug"].as_str(),
		Some("owner/example")
	);
	assert_eq!(metadata.update["github"]["tag"].as_str(), Some("v1.2.3"));
	assert_eq!(metadata.update["github"]["branch"].as_str(), Some("main"));
	assert!(metadata.update["github"].get("instance").is_none());

	let requests = transport.requests.lock().unwrap();
	assert_eq!(requests.len(), 3);
	assert!(requests[0].url.ends_with("/api/repos/owner/example"));
	assert!(
		requests[1]
			.url
			.ends_with("/api/repos/owner/example/releases")
	);
	assert!(requests.iter().all(|request| {
		request
			.headers
			.iter()
			.any(|(name, value)| name == "Authorization" && value == "Bearer secret")
	}));
}

#[test]
fn forgejo_resolves_an_attachment_and_preserves_instance_metadata() {
	let transport = FixtureTransport::new([
		r#"{"name":"Example Mod (Fabric)","full_name":"owner/example"}"#,
		r#"[{
          "tag_name":"v2","target_commitish":"main","name":"Version 2",
          "attachments":[
            {"name":"example-dev.jar","browser_download_url":"https://forge.test/dev.jar"},
            {"name":"example.jar","browser_download_url":"https://forge.test/example.jar"}
          ]
        }]"#,
		"forgejo artifact",
	]);
	let client = ForgejoClient::with_api_base(
		&transport,
		"forge.test",
		"forge-token",
		"https://example.test/api/v1/",
	)
	.unwrap();
	let mut resolve_request = request("owner/example");
	resolve_request.branch = Some("main".into());
	let resolved = client.resolve(&resolve_request).unwrap();
	assert_eq!(resolved.metadata_path(), "mods/example-mod.pw.json");

	let metadata = resolved.into_mod().unwrap();
	assert_eq!(
		metadata.update["forgejo"]["instance"].as_str(),
		Some("forge.test")
	);
	assert_eq!(
		metadata.update["forgejo"]["slug"].as_str(),
		Some("owner/example")
	);
	assert_eq!(metadata.update["forgejo"]["tag"].as_str(), Some("v2"));
	assert_eq!(metadata.update["forgejo"]["branch"].as_str(), Some("main"));

	let requests = transport.requests.lock().unwrap();
	assert_eq!(requests.len(), 3);
	assert!(requests[0].url.ends_with("/api/v1/repos/owner/example"));
	assert!(
		requests[0]
			.headers
			.iter()
			.any(|(name, value)| name == "Authorization" && value == "token forge-token")
	);
	assert!(requests[2].headers.is_empty());
}

#[test]
fn gitlab_encodes_project_paths_and_persists_instance_metadata() {
	let transport = FixtureTransport::new([
		r#"{"name":"GitLab Example","path_with_namespace":"owner/example"}"#,
		r#"[{
          "tag_name":"3.0","assets":{"links":[
            {"name":"example-api.jar","url":"https://gitlab.test/api.jar"},
            {"name":"example.jar","url":"https://gitlab.test/example.jar"}
          ]}
        }]"#,
		"gitlab artifact",
	]);
	let client = GitLabClient::with_api_base(
		&transport,
		"gitlab.test",
		"gitlab-token",
		"https://example.test/api/v4/",
	)
	.unwrap();
	let resolved = client.resolve(&request("owner/example")).unwrap();
	assert_eq!(resolved.version.id, "3.0");

	let metadata = resolved.into_mod().unwrap();
	assert_eq!(
		metadata.update["gitlab"]["instance"].as_str(),
		Some("gitlab.test")
	);
	assert_eq!(
		metadata.update["gitlab"]["slug"].as_str(),
		Some("owner/example")
	);
	assert_eq!(metadata.update["gitlab"]["tag"].as_str(), Some("3.0"));
	assert!(metadata.update["gitlab"].get("branch").is_none());

	let requests = transport.requests.lock().unwrap();
	assert_eq!(requests.len(), 3);
	assert!(requests[0].url.contains("projects/owner%2Fexample"));
	assert!(requests[1].url.contains("order_by=released_at"));
	assert!(
		requests[0]
			.headers
			.iter()
			.any(|(name, value)| name == "PRIVATE-TOKEN" && value == "gitlab-token")
	);
	assert!(requests[2].headers.is_empty());
}
