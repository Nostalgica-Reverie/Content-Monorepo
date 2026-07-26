//! Serde models for Mojang's launcher metadata and Fabric-style loader
//! profiles. Unknown fields are tolerated everywhere: this metadata is
//! provider-owned and gains fields over time.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// `version_manifest_v2.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<ManifestVersion>,
}

/// The latest release and snapshot version IDs.
#[derive(Debug, Clone, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

/// A single version entry from the version manifest.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    #[serde(default)]
    pub sha1: Option<String>,
}

impl VersionManifest {
    /// Finds a version by ID, supporting `latest-release` and `latest-snapshot` aliases.
    pub fn find(&self, id: &str) -> Option<&ManifestVersion> {
        let id = match id {
            "latest-release" => &self.latest.release,
            "latest-snapshot" => &self.latest.snapshot,
            other => other,
        };
        self.versions.iter().find(|v| v.id == id)
    }
}

/// One version document (`<id>.json`), or a loader profile inheriting from
/// one. Loader profiles usually carry only `id`, `inheritsFrom`,
/// `mainClass`, extra `libraries`, and extra `arguments`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDoc {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    /// Modern (1.13+) argument lists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,
    /// Legacy (pre-1.13) single-string game arguments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub libraries: Vec<Library>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_index: Option<AssetIndexRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub downloads: BTreeMap<String, DownloadRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_version: Option<JavaVersionRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingConfigSet>,
}

/// Modern (1.13+) JVM and game arguments from a version document.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Arguments {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub game: Vec<Argument>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub jvm: Vec<Argument>,
}

/// A plain argument or a rule-guarded one.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
    Plain(String),
    Conditional {
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

/// An argument value: either a single string or a list of strings.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    One(String),
    Many(Vec<String>),
}

impl ArgumentValue {
    /// Views this value as a string slice.
    pub fn as_slice(&self) -> &[String] {
        match self {
            ArgumentValue::One(v) => std::slice::from_ref(v),
            ArgumentValue::Many(v) => v,
        }
    }
}

/// A conditional rule that applies an argument based on OS and feature checks.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub action: RuleAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<OsRule>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub features: BTreeMap<String, bool>,
}

/// Whether a conditional rule allows or disallows an argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    Allow,
    Disallow,
}

/// OS-specific conditions for a rule: name, version, and architecture.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OsRule {
    /// `windows`, `osx`, or `linux`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Regex matched against the host OS version (Mojang uses `^10\.`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// `x86`, `x86_64`, `arm64`, ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

/// A library dependency (JAR) for the game, including natives and extraction rules.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Library {
    /// Maven coordinates `group:artifact:version[:classifier]`.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,
    /// Fabric/Quilt style: a maven repository base URL; the artifact path
    /// is derived from `name`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Legacy natives: os name -> classifier key (may contain `${arch}`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub natives: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extract: Option<ExtractRule>,
}

/// Download information for a library artifact and optional classifiers.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LibraryDownloads {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<Artifact>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub classifiers: BTreeMap<String, Artifact>,
}

/// Download metadata for a file: URL, size, and SHA-1 hash.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifact {
    /// Repository-relative path (`com/foo/bar/1.0/bar-1.0.jar`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Defines which paths to exclude when extracting a library.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExtractRule {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

/// Reference to an asset index document.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndexRef {
    pub id: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_size: Option<u64>,
}

/// Reference to a downloadable file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DownloadRef {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Java version requirement for this game version.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersionRef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    pub major_version: u32,
}

/// Available logging configurations (client and/or server).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LoggingConfigSet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<LoggingConfig>,
}

/// Log4j configuration details.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub argument: String,
    pub file: LoggingFile,
}

/// Download reference for a logging configuration file.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingFile {
    pub id: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// The asset index document (`assets/indexes/<id>.json`).
#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub objects: BTreeMap<String, AssetObject>,
    /// Pre-1.7.3 layout: objects are additionally materialized as real
    /// files under `assets/virtual/<id>`.
    #[serde(default)]
    pub virtual_: Option<bool>,
    #[serde(default)]
    pub map_to_resources: Option<bool>,
}

impl AssetIndex {
    /// Manual alias handling: the field is literally named `virtual`.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        #[derive(Deserialize)]
        struct Raw {
            objects: BTreeMap<String, AssetObject>,
            #[serde(default, rename = "virtual")]
            virtual_: Option<bool>,
            #[serde(default, rename = "map_to_resources")]
            map_to_resources: Option<bool>,
        }
        let raw: Raw = serde_json::from_slice(bytes)?;
        Ok(Self {
            objects: raw.objects,
            virtual_: raw.virtual_,
            map_to_resources: raw.map_to_resources,
        })
    }
}

/// An asset object: hash and size of a single asset file.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    #[serde(default)]
    pub size: Option<u64>,
}

/// Splits maven coordinates into a repository-relative artifact path:
/// `com.foo:bar:1.0` -> `com/foo/bar/1.0/bar-1.0.jar`,
/// `com.foo:bar:1.0:natives-linux` -> `com/foo/bar/1.0/bar-1.0-natives-linux.jar`.
pub fn maven_artifact_path(name: &str) -> Option<String> {
    let mut parts = name.split(':');
    let (group, artifact, version) = (parts.next()?, parts.next()?, parts.next()?);
    if group.is_empty() || artifact.is_empty() || version.is_empty() {
        return None;
    }
    // A version may carry an `@ext` suffix (e.g. `1.0@zip`).
    let (version, ext) = match version.split_once('@') {
        Some((v, ext)) => (v, ext),
        None => (version, "jar"),
    };
    let classifier = parts.next().map(|c| format!("-{c}")).unwrap_or_default();
    Some(format!(
        "{}/{artifact}/{version}/{artifact}-{version}{classifier}.{ext}",
        group.replace('.', "/")
    ))
}

/// The `group:artifact[:classifier]` key used to detect collisions when a
/// loader profile overrides one of the vanilla libraries.
pub fn maven_collision_key(name: &str) -> String {
    let mut parts = name.split(':');
    let group = parts.next().unwrap_or_default();
    let artifact = parts.next().unwrap_or_default();
    let _version = parts.next();
    match parts.next() {
        Some(classifier) => format!("{group}:{artifact}:{classifier}"),
        None => format!("{group}:{artifact}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maven_paths() {
        assert_eq!(
            maven_artifact_path("com.foo.baz:bar:1.0").unwrap(),
            "com/foo/baz/bar/1.0/bar-1.0.jar"
        );
        assert_eq!(
            maven_artifact_path("org.lwjgl:lwjgl:3.3.3:natives-windows").unwrap(),
            "org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-windows.jar"
        );
        assert_eq!(
            maven_artifact_path("g:a:1.0@zip").unwrap(),
            "g/a/1.0/a-1.0.zip"
        );
        assert!(maven_artifact_path("only:two").is_none());
    }

    #[test]
    fn collision_keys_ignore_version_but_keep_classifier() {
        assert_eq!(
            maven_collision_key("org.ow2.asm:asm:9.6"),
            "org.ow2.asm:asm"
        );
        assert_eq!(
            maven_collision_key("org.ow2.asm:asm:9.7"),
            maven_collision_key("org.ow2.asm:asm:9.6")
        );
        assert_eq!(
            maven_collision_key("org.lwjgl:lwjgl:3.3.3:natives-windows"),
            "org.lwjgl:lwjgl:natives-windows"
        );
    }

    #[test]
    fn argument_forms_deserialize() {
        let json = r#"{
            "game": [
                "--username",
                "${auth_player_name}",
                {"rules": [{"action": "allow", "features": {"is_demo_user": true}}], "value": "--demo"},
                {"rules": [{"action": "allow", "features": {"has_custom_resolution": true}}], "value": ["--width", "${resolution_width}"]}
            ],
            "jvm": [
                {"rules": [{"action": "allow", "os": {"name": "windows", "version": "^10\\."}}], "value": "-Dos.version=10.0"}
            ]
        }"#;
        let args: Arguments = serde_json::from_str(json).unwrap();
        assert_eq!(args.game.len(), 4);
        assert_eq!(args.jvm.len(), 1);
        match &args.game[3] {
            Argument::Conditional { value, .. } => assert_eq!(value.as_slice().len(), 2),
            other => panic!("expected conditional, got {other:?}"),
        }
    }

    #[test]
    fn asset_index_virtual_alias() {
        let index = AssetIndex::from_slice(
            br#"{"virtual": true, "objects": {"icons/a.png": {"hash": "0123456789abcdef0123456789abcdef01234567", "size": 3}}}"#,
        )
        .unwrap();
        assert_eq!(index.virtual_, Some(true));
        assert_eq!(index.objects.len(), 1);
    }
}
