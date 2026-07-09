//! Fetching and resolving version metadata: Mojang's manifest and version
//! documents, asset indexes, and loader profiles.

use std::io::{Cursor, Read};

use regex_lite::Regex;
use serde::Deserialize;

use crate::http::HttpClient;
use crate::merge::merge_inherited;
use crate::model::{
    AssetIndex, AssetIndexRef, Library, ManifestVersion, VersionDoc, VersionManifest,
};
use crate::MinecraftError;

/// Remote endpoints, parameterized so tests can point at fixtures.
#[derive(Debug, Clone)]
pub struct MetadataEndpoints {
    pub version_manifest_url: String,
    pub fabric_meta_url: String,
    pub quilt_meta_url: String,
    pub forge_maven_url: String,
    pub neoforge_maven_url: String,
    pub resources_url: String,
}

impl Default for MetadataEndpoints {
    fn default() -> Self {
        Self {
            version_manifest_url: "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
                .to_string(),
            fabric_meta_url: "https://meta.fabricmc.net/v2".to_string(),
            quilt_meta_url: "https://meta.quiltmc.org/v3".to_string(),
            forge_maven_url: "https://files.minecraftforge.net/maven".to_string(),
            neoforge_maven_url: "https://maven.neoforged.net/releases".to_string(),
            resources_url: crate::plan::DEFAULT_RESOURCES_URL.to_string(),
        }
    }
}

/// A fetched document plus its raw bytes, so callers can persist exactly
/// what was verified.
#[derive(Debug)]
pub struct Fetched<T> {
    pub value: T,
    pub bytes: Vec<u8>,
}

/// Runtime-launch metadata extracted from a Forge/NeoForge installer jar.
#[derive(Debug)]
pub struct InstallerProfile {
    pub version: VersionDoc,
    pub libraries: Vec<Library>,
}

pub struct MetadataClient<'a> {
    http: &'a dyn HttpClient,
    pub endpoints: MetadataEndpoints,
}

fn json_error(url: &str) -> impl FnOnce(serde_json::Error) -> MinecraftError + '_ {
    move |source| MinecraftError::Json {
        context: url.to_string(),
        message: source.to_string(),
    }
}

fn xml_error(context: &str, message: impl Into<String>) -> MinecraftError {
    MinecraftError::Xml {
        context: context.to_string(),
        message: message.into(),
    }
}

fn verify_sha1(url: &str, bytes: &[u8], expected: Option<&str>) -> Result<(), MinecraftError> {
    use sha1::{Digest, Sha1};
    let Some(expected) = expected else {
        return Ok(());
    };
    let actual: String = Sha1::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(MinecraftError::ChecksumMismatch {
            url: url.to_string(),
            expected: expected.to_string(),
            actual,
        })
    }
}

fn parse_xml_versions(bytes: &[u8], context: &str) -> Result<Vec<String>, MinecraftError> {
    let text = std::str::from_utf8(bytes).map_err(|e| xml_error(context, e.to_string()))?;
    let re = Regex::new(r"(?s)<version>([^<]+)</version>").expect("valid regex");
    let versions = re
        .captures_iter(text)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
        .collect::<Vec<_>>();
    if versions.is_empty() {
        Err(xml_error(context, "no <version> entries were found"))
    } else {
        Ok(versions)
    }
}

fn parse_xml_scalar(bytes: &[u8], tag: &str) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?;
    let re = Regex::new(&format!(r"(?s)<{tag}>([^<]+)</{tag}>")).ok()?;
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
}

fn select_maven_version(
    bytes: &[u8],
    context: &str,
    filter: impl Fn(&str) -> bool,
) -> Result<String, MinecraftError> {
    let versions = parse_xml_versions(bytes, context)?;
    let filtered = versions
        .iter()
        .filter(|version| filter(version))
        .cloned()
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return Err(xml_error(context, "no compatible versions were found"));
    }
    for tag in ["release", "latest"] {
        if let Some(value) = parse_xml_scalar(bytes, tag) {
            if filter(&value) {
                return Ok(value);
            }
        }
    }
    filtered
        .last()
        .cloned()
        .ok_or_else(|| xml_error(context, "no compatible versions were found"))
}

fn forge_style_matches(version: &str, game_version: &str) -> bool {
    version
        .split_once('-')
        .is_some_and(|(prefix, _)| prefix == game_version)
}

fn old_neoforge_matches(version: &str, game_version: &str) -> bool {
    let mut parts = game_version.split('.');
    let Some(_one) = parts.next() else {
        return false;
    };
    let Some(major) = parts.next() else {
        return false;
    };
    let minor = parts.next().unwrap_or("0");
    version.starts_with(&format!("{major}.{minor}."))
}

fn new_neoforge_matches(version: &str, game_version: &str) -> bool {
    let parts = game_version.splitn(3, '.').collect::<Vec<_>>();
    if parts.len() < 2 {
        return false;
    }
    let year = parts[0];
    let mut major = parts[1];
    let mut minor = "0";
    let mut prerelease = "";
    if parts.len() == 3 {
        if let Some((parsed_minor, parsed_prerelease)) = parts[2].split_once('-') {
            minor = parsed_minor;
            prerelease = parsed_prerelease;
        } else {
            minor = parts[2];
        }
    } else if let Some((parsed_major, parsed_prerelease)) = parts[1].split_once('-') {
        major = parsed_major;
        prerelease = parsed_prerelease;
    }
    let required_prefix = format!("{year}.{major}.{minor}");
    version.starts_with(&required_prefix) && version.ends_with(prerelease)
}

fn installer_url(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn installer_json_entry(
    archive_url: &str,
    archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
    name: &str,
) -> Result<Vec<u8>, MinecraftError> {
    let mut entry = archive.by_name(name).map_err(|e| match e {
        zip::result::ZipError::FileNotFound => MinecraftError::InstallerEntryMissing {
            url: archive_url.to_string(),
            entry: name.to_string(),
        },
        other => MinecraftError::InstallerArchive {
            url: archive_url.to_string(),
            message: other.to_string(),
        },
    })?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|e| MinecraftError::InstallerArchive {
            url: archive_url.to_string(),
            message: e.to_string(),
        })?;
    Ok(bytes)
}

#[derive(Debug, Deserialize)]
struct QuiltLoaderEntry {
    loader: QuiltLoaderVersion,
}

#[derive(Debug, Deserialize)]
struct QuiltLoaderVersion {
    version: String,
}

#[derive(Debug, Deserialize)]
struct InstallerProfileJson {
    #[serde(default)]
    libraries: Vec<Library>,
}

impl<'a> MetadataClient<'a> {
    pub fn new(http: &'a dyn HttpClient, endpoints: MetadataEndpoints) -> Self {
        Self { http, endpoints }
    }

    pub fn fetch_manifest(&self) -> Result<VersionManifest, MinecraftError> {
        let url = &self.endpoints.version_manifest_url;
        let bytes = self.http.get(url)?;
        serde_json::from_slice(&bytes).map_err(json_error(url))
    }

    /// Fetches and checksum-verifies one version document.
    pub fn fetch_version(
        &self,
        entry: &ManifestVersion,
    ) -> Result<Fetched<VersionDoc>, MinecraftError> {
        let bytes = self.http.get(&entry.url)?;
        verify_sha1(&entry.url, &bytes, entry.sha1.as_deref())?;
        let value = serde_json::from_slice(&bytes).map_err(json_error(&entry.url))?;
        Ok(Fetched { value, bytes })
    }

    pub fn fetch_asset_index(
        &self,
        reference: &AssetIndexRef,
    ) -> Result<Fetched<AssetIndex>, MinecraftError> {
        let bytes = self.http.get(&reference.url)?;
        verify_sha1(&reference.url, &bytes, reference.sha1.as_deref())?;
        let value = AssetIndex::from_slice(&bytes).map_err(json_error(&reference.url))?;
        Ok(Fetched { value, bytes })
    }

    /// Resolves a Fabric loader version: an explicit one is used as-is,
    /// otherwise the newest stable loader for the game version.
    pub fn resolve_fabric_loader(
        &self,
        game_version: &str,
        loader_version: Option<&str>,
    ) -> Result<String, MinecraftError> {
        if let Some(version) = loader_version {
            return Ok(version.to_string());
        }
        #[derive(Deserialize)]
        struct LoaderEntry {
            loader: LoaderInfo,
        }
        #[derive(Deserialize)]
        struct LoaderInfo {
            version: String,
            #[serde(default)]
            stable: bool,
        }
        let url = format!(
            "{}/versions/loader/{game_version}",
            self.endpoints.fabric_meta_url
        );
        let bytes = self.http.get(&url)?;
        let entries: Vec<LoaderEntry> = serde_json::from_slice(&bytes).map_err(json_error(&url))?;
        entries
            .iter()
            .find(|e| e.loader.stable)
            .or(entries.first())
            .map(|e| e.loader.version.clone())
            .ok_or_else(|| MinecraftError::NoLoaderVersion(game_version.to_string()))
    }

    /// Fetches the Fabric profile for one game+loader pair. The result
    /// still carries `inheritsFrom`; resolve it with [`Self::resolve_inheritance`].
    pub fn fetch_fabric_profile(
        &self,
        game_version: &str,
        loader_version: &str,
    ) -> Result<VersionDoc, MinecraftError> {
        let url = format!(
            "{}/versions/loader/{game_version}/{loader_version}/profile/json",
            self.endpoints.fabric_meta_url
        );
        let bytes = self.http.get(&url)?;
        serde_json::from_slice(&bytes).map_err(json_error(&url))
    }

    pub fn resolve_quilt_loader(
        &self,
        game_version: &str,
        loader_version: Option<&str>,
    ) -> Result<String, MinecraftError> {
        let url = format!(
            "{}/versions/loader/{game_version}",
            self.endpoints.quilt_meta_url
        );
        let bytes = self.http.get(&url)?;
        let entries: Vec<QuiltLoaderEntry> =
            serde_json::from_slice(&bytes).map_err(json_error(&url))?;
        if let Some(requested) = loader_version {
            return entries
                .into_iter()
                .find(|entry| entry.loader.version == requested)
                .map(|entry| entry.loader.version)
                .ok_or_else(|| MinecraftError::LoaderVersionNotFound {
                    game_version: game_version.to_string(),
                    loader: requested.to_string(),
                });
        }
        entries
            .first()
            .map(|entry| entry.loader.version.clone())
            .ok_or_else(|| MinecraftError::NoLoaderVersion(game_version.to_string()))
    }

    pub fn fetch_quilt_profile(
        &self,
        game_version: &str,
        loader_version: &str,
    ) -> Result<VersionDoc, MinecraftError> {
        let url = format!(
            "{}/versions/loader/{game_version}/{loader_version}/profile/json",
            self.endpoints.quilt_meta_url
        );
        let bytes = self.http.get(&url)?;
        serde_json::from_slice(&bytes).map_err(json_error(&url))
    }

    pub fn resolve_forge_loader(
        &self,
        game_version: &str,
        loader_version: Option<&str>,
    ) -> Result<String, MinecraftError> {
        if let Some(version) = loader_version {
            return Ok(version.to_string());
        }
        let url = installer_url(
            &self.endpoints.forge_maven_url,
            "net/minecraftforge/forge/maven-metadata.xml",
        );
        let bytes = self.http.get(&url)?;
        select_maven_version(&bytes, &url, |version| {
            forge_style_matches(version, game_version)
        })
        .map(|version| {
            version
                .rsplit_once('-')
                .map(|(_, loader)| loader.to_string())
                .unwrap_or(version)
        })
    }

    pub fn fetch_forge_profile(
        &self,
        game_version: &str,
        loader_version: &str,
    ) -> Result<InstallerProfile, MinecraftError> {
        let url = installer_url(
            &self.endpoints.forge_maven_url,
            &format!(
                "net/minecraftforge/forge/{game_version}-{loader_version}/forge-{game_version}-{loader_version}-installer.jar"
            ),
        );
        self.fetch_installer_profile(&url)
    }

    pub fn resolve_neoforge_loader(
        &self,
        game_version: &str,
        loader_version: Option<&str>,
    ) -> Result<String, MinecraftError> {
        if let Some(version) = loader_version {
            return Ok(version.to_string());
        }
        if game_version == "1.20.1" {
            let url = installer_url(
                &self.endpoints.neoforge_maven_url,
                "net/neoforged/forge/maven-metadata.xml",
            );
            let bytes = self.http.get(&url)?;
            return select_maven_version(&bytes, &url, |version| {
                forge_style_matches(version, game_version)
            })
            .map(|version| {
                version
                    .rsplit_once('-')
                    .map(|(_, loader)| loader.to_string())
                    .unwrap_or(version)
            });
        }
        let url = installer_url(
            &self.endpoints.neoforge_maven_url,
            "net/neoforged/neoforge/maven-metadata.xml",
        );
        let bytes = self.http.get(&url)?;
        if game_version.starts_with("1.") {
            select_maven_version(&bytes, &url, |version| {
                old_neoforge_matches(version, game_version)
            })
        } else {
            select_maven_version(&bytes, &url, |version| {
                new_neoforge_matches(version, game_version)
            })
        }
    }

    pub fn fetch_neoforge_profile(
        &self,
        game_version: &str,
        loader_version: &str,
    ) -> Result<InstallerProfile, MinecraftError> {
        let path = if game_version == "1.20.1" {
            format!(
                "net/neoforged/forge/{game_version}-{loader_version}/forge-{game_version}-{loader_version}-installer.jar"
            )
        } else {
            format!(
                "net/neoforged/neoforge/{loader_version}/neoforge-{loader_version}-installer.jar"
            )
        };
        let url = installer_url(&self.endpoints.neoforge_maven_url, &path);
        self.fetch_installer_profile(&url)
    }

    pub fn fetch_installer_profile(
        &self,
        archive_url: &str,
    ) -> Result<InstallerProfile, MinecraftError> {
        let bytes = self.http.get(archive_url)?;
        let cursor = Cursor::new(bytes);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| MinecraftError::InstallerArchive {
                url: archive_url.to_string(),
                message: e.to_string(),
            })?;
        let version_bytes = installer_json_entry(archive_url, &mut archive, "version.json")?;
        let profile_bytes =
            installer_json_entry(archive_url, &mut archive, "install_profile.json")?;
        let version = serde_json::from_slice(&version_bytes).map_err(json_error(archive_url))?;
        let profile: InstallerProfileJson =
            serde_json::from_slice(&profile_bytes).map_err(json_error(archive_url))?;
        Ok(InstallerProfile {
            version,
            libraries: profile.libraries,
        })
    }

    /// Follows `inheritsFrom` chains until a self-contained document
    /// remains. Parents are looked up in the manifest.
    pub fn resolve_inheritance(
        &self,
        manifest: &VersionManifest,
        mut doc: VersionDoc,
    ) -> Result<VersionDoc, MinecraftError> {
        // Chains deeper than a handful indicate a metadata cycle.
        for _ in 0..8 {
            let Some(parent_id) = doc.inherits_from.clone() else {
                return Ok(doc);
            };
            let entry = manifest
                .find(&parent_id)
                .ok_or_else(|| MinecraftError::VersionNotFound(parent_id.clone()))?;
            let parent = self.fetch_version(entry)?.value;
            doc = merge_inherited(&parent, &doc);
        }
        Err(MinecraftError::InheritanceTooDeep(doc.id))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::http::FixtureHttpClient;

    fn endpoints() -> MetadataEndpoints {
        MetadataEndpoints {
            version_manifest_url: "http://meta/manifest.json".to_string(),
            fabric_meta_url: "http://fabric/v2".to_string(),
            quilt_meta_url: "http://quilt/v3".to_string(),
            forge_maven_url: "http://forge-maven".to_string(),
            neoforge_maven_url: "http://neoforge-maven".to_string(),
            resources_url: "http://resources".to_string(),
        }
    }

    const MANIFEST: &str = r#"{
        "latest": {"release": "1.21", "snapshot": "24w40a"},
        "versions": [
            {"id": "1.21", "type": "release", "url": "http://meta/1.21.json", "sha1": null},
            {"id": "24w40a", "type": "snapshot", "url": "http://meta/24w40a.json"}
        ]
    }"#;

    fn installer_bytes(version_json: &str, profile_json: &str) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut cursor);
            let options = zip::write::SimpleFileOptions::default();
            writer.start_file("version.json", options).unwrap();
            writer.write_all(version_json.as_bytes()).unwrap();
            writer.start_file("install_profile.json", options).unwrap();
            writer.write_all(profile_json.as_bytes()).unwrap();
            writer.finish().unwrap();
        }
        cursor.into_inner()
    }

    #[test]
    fn manifest_lookup_and_latest_aliases() {
        let http = FixtureHttpClient::new([(
            "http://meta/manifest.json".to_string(),
            MANIFEST.as_bytes().to_vec(),
        )]);
        let client = MetadataClient::new(&http, endpoints());
        let manifest = client.fetch_manifest().unwrap();
        assert_eq!(manifest.find("1.21").unwrap().id, "1.21");
        assert_eq!(manifest.find("latest-release").unwrap().id, "1.21");
        assert_eq!(manifest.find("latest-snapshot").unwrap().id, "24w40a");
        assert!(manifest.find("9.9.9").is_none());
    }

    #[test]
    fn version_fetch_verifies_sha1() {
        let body = br#"{"id": "1.21"}"#.to_vec();
        let http = FixtureHttpClient::new([("http://meta/1.21.json".to_string(), body)]);
        let client = MetadataClient::new(&http, endpoints());
        let entry = ManifestVersion {
            id: "1.21".to_string(),
            kind: "release".to_string(),
            url: "http://meta/1.21.json".to_string(),
            sha1: Some("0000000000000000000000000000000000000000".to_string()),
        };
        let err = client.fetch_version(&entry).unwrap_err();
        assert!(err.to_string().contains("checksum"), "{err}");
    }

    #[test]
    fn fabric_loader_resolution_prefers_stable() {
        let listing = r#"[
            {"loader": {"version": "0.17.0-beta.1", "stable": false}},
            {"loader": {"version": "0.16.9", "stable": true}}
        ]"#;
        let http = FixtureHttpClient::new([(
            "http://fabric/v2/versions/loader/1.21".to_string(),
            listing.as_bytes().to_vec(),
        )]);
        let client = MetadataClient::new(&http, endpoints());
        assert_eq!(
            client.resolve_fabric_loader("1.21", None).unwrap(),
            "0.16.9"
        );
        assert_eq!(
            client
                .resolve_fabric_loader("1.21", Some("0.15.0"))
                .unwrap(),
            "0.15.0"
        );
    }

    #[test]
    fn quilt_loader_resolution_uses_listing_and_profile_endpoint() {
        let listing = r#"[
            {"loader": {"version": "0.24.0"}},
            {"loader": {"version": "0.23.1"}}
        ]"#;
        let profile = r#"{
            "id": "quilt-loader-0.24.0-1.21",
            "inheritsFrom": "1.21",
            "mainClass": "org.quiltmc.loader.impl.launch.knot.KnotClient",
            "libraries": [{"name": "org.quiltmc:quilt-loader:0.24.0", "url": "http://quilt-maven"}]
        }"#;
        let http = FixtureHttpClient::new([
            (
                "http://quilt/v3/versions/loader/1.21".to_string(),
                listing.as_bytes().to_vec(),
            ),
            (
                "http://quilt/v3/versions/loader/1.21/0.24.0/profile/json".to_string(),
                profile.as_bytes().to_vec(),
            ),
        ]);
        let client = MetadataClient::new(&http, endpoints());
        assert_eq!(client.resolve_quilt_loader("1.21", None).unwrap(), "0.24.0");
        let doc = client.fetch_quilt_profile("1.21", "0.24.0").unwrap();
        assert_eq!(doc.inherits_from.as_deref(), Some("1.21"));
        assert_eq!(doc.libraries[0].name, "org.quiltmc:quilt-loader:0.24.0");
    }

    #[test]
    fn forge_and_neoforge_loader_resolution_use_maven_metadata() {
        let forge_meta = r#"<metadata><versioning><release>1.20.1-47.4.5</release><versions><version>1.20.1-47.4.4</version><version>1.20.1-47.4.5</version></versions></versioning></metadata>"#;
        let neoforge_old_meta = r#"<metadata><versioning><versions><version>1.20.1-47.1.105</version><version>1.20.1-47.1.106</version></versions></versioning></metadata>"#;
        let neoforge_new_meta = r#"<metadata><versioning><versions><version>21.1.201</version><version>21.1.213</version></versions></versioning></metadata>"#;
        let http = FixtureHttpClient::new([
            (
                "http://forge-maven/net/minecraftforge/forge/maven-metadata.xml".to_string(),
                forge_meta.as_bytes().to_vec(),
            ),
            (
                "http://neoforge-maven/net/neoforged/forge/maven-metadata.xml".to_string(),
                neoforge_old_meta.as_bytes().to_vec(),
            ),
            (
                "http://neoforge-maven/net/neoforged/neoforge/maven-metadata.xml".to_string(),
                neoforge_new_meta.as_bytes().to_vec(),
            ),
        ]);
        let client = MetadataClient::new(&http, endpoints());
        assert_eq!(
            client.resolve_forge_loader("1.20.1", None).unwrap(),
            "47.4.5"
        );
        assert_eq!(
            client.resolve_neoforge_loader("1.20.1", None).unwrap(),
            "47.1.106"
        );
        assert_eq!(
            client.resolve_neoforge_loader("1.21.1", None).unwrap(),
            "21.1.213"
        );
    }

    #[test]
    fn installer_profiles_parse_version_and_extra_libraries() {
        let version_json = r#"{
            "id": "1.20.1-forge-47.4.5",
            "inheritsFrom": "1.20.1",
            "mainClass": "cpw.mods.bootstraplauncher.BootstrapLauncher",
            "arguments": {"jvm": ["-p", "${library_directory}/cpw/mods/bootstraplauncher/1.1.2/bootstraplauncher-1.1.2.jar${classpath_separator}${library_directory}/net/minecraftforge/JarJarFileSystems/0.3.19/JarJarFileSystems-0.3.19.jar"], "game": ["--launchTarget", "forgeclient"]},
            "libraries": [{"name": "cpw.mods:bootstraplauncher:1.1.2", "downloads": {"artifact": {"path": "cpw/mods/bootstraplauncher/1.1.2/bootstraplauncher-1.1.2.jar", "url": "http://maven/bootstrap.jar", "sha1": null, "size": 4}}}]
        }"#;
        let profile_json = r#"{
            "libraries": [
                {"name": "net.minecraftforge:javafmllanguage:1.20.1-47.4.5", "downloads": {"artifact": {"path": "net/minecraftforge/javafmllanguage/1.20.1-47.4.5/javafmllanguage-1.20.1-47.4.5.jar", "url": "http://maven/javafmllanguage.jar", "sha1": null, "size": 5}}}
            ]
        }"#;
        let http = FixtureHttpClient::new([(
            "http://forge-maven/net/minecraftforge/forge/1.20.1-47.4.5/forge-1.20.1-47.4.5-installer.jar".to_string(),
            installer_bytes(version_json, profile_json),
        )]);
        let client = MetadataClient::new(&http, endpoints());
        let profile = client.fetch_forge_profile("1.20.1", "47.4.5").unwrap();
        assert_eq!(profile.version.id, "1.20.1-forge-47.4.5");
        assert_eq!(profile.libraries.len(), 1);
        assert_eq!(
            profile.libraries[0].name,
            "net.minecraftforge:javafmllanguage:1.20.1-47.4.5"
        );
    }

    #[test]
    fn inheritance_resolves_through_the_manifest() {
        let parent = r#"{
            "id": "1.21",
            "mainClass": "net.minecraft.client.main.Main",
            "downloads": {"client": {"url": "http://data/client.jar", "sha1": null, "size": null}},
            "libraries": [{"name": "org.ow2.asm:asm:9.6"}]
        }"#;
        let http = FixtureHttpClient::new([
            (
                "http://meta/manifest.json".to_string(),
                MANIFEST.as_bytes().to_vec(),
            ),
            (
                "http://meta/1.21.json".to_string(),
                parent.as_bytes().to_vec(),
            ),
        ]);
        let client = MetadataClient::new(&http, endpoints());
        let manifest = client.fetch_manifest().unwrap();
        let child: VersionDoc = serde_json::from_str(
            r#"{
                "id": "fabric-loader-0.16.9-1.21",
                "inheritsFrom": "1.21",
                "mainClass": "net.fabricmc.loader.impl.launch.knot.KnotClient",
                "libraries": [{"name": "org.ow2.asm:asm:9.7", "url": "http://maven/"}]
            }"#,
        )
        .unwrap();
        let merged = client.resolve_inheritance(&manifest, child).unwrap();
        assert_eq!(
            merged.main_class.as_deref(),
            Some("net.fabricmc.loader.impl.launch.knot.KnotClient")
        );
        assert!(merged.downloads.contains_key("client"));
        let names: Vec<&str> = merged.libraries.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, vec!["org.ow2.asm:asm:9.7"]);
    }
}
