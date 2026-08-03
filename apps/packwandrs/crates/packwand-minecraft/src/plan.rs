//! Install planning: turning version metadata into an inspectable list of
//! downloads, extractions, and copies before anything touches the network
//! or the filesystem (acquisition plans are separate from
//! application").

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use serde::Serialize;

use crate::MinecraftError;
use crate::model::{AssetIndex, Library, LoggingConfig, VersionDoc, maven_artifact_path};
use crate::rules::{Host, rules_allow};

/// Shared directories an installation writes into.
#[derive(Debug, Clone)]
pub struct InstallLayout {
    /// `<root>/versions`: client jars and version JSONs.
    pub versions_dir: PathBuf,
    /// `<root>/libraries`: maven-layout library store.
    pub libraries_dir: PathBuf,
    /// `<root>/assets`: `indexes/`, `objects/`, `virtual/`.
    pub assets_dir: PathBuf,
    /// Per-instance natives directory.
    pub natives_dir: PathBuf,
    /// Per-instance `resources` directory for very old versions whose
    /// asset index sets `map_to_resources`.
    pub resources_dir: Option<PathBuf>,
}

/// One file to fetch. `sha1`/`size` absent means the source publishes no
/// checksum; the installer then verifies what it can (size, if known).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DownloadAction {
    pub url: String,
    pub target: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Unpack a natives jar into the natives directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtractAction {
    pub archive: PathBuf,
    pub dest: PathBuf,
    /// Entry-name prefixes to skip (Mojang uses `META-INF/`).
    pub excludes: Vec<String>,
}

/// Materialize one downloaded asset object as a real file (legacy
/// `virtual` / `map_to_resources` asset layouts).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CopyAction {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Everything required to make one version launchable. Downloads may run
/// in any order; extractions and copies run after their sources exist.
#[derive(Debug, Clone, Default, Serialize)]
pub struct InstallPlan {
    pub version_id: String,
    pub downloads: Vec<DownloadAction>,
    pub extractions: Vec<ExtractAction>,
    pub copies: Vec<CopyAction>,
    /// Ordered classpath: libraries first, client jar last.
    pub classpath: Vec<PathBuf>,
    /// Directory legacy versions expect as `${game_assets}`, when the
    /// index uses a materialized layout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_assets_dir: Option<PathBuf>,
}

impl InstallPlan {
    fn push_download(&mut self, action: DownloadAction) {
        if !self.downloads.iter().any(|d| d.target == action.target) {
            self.downloads.push(action);
        }
    }

    /// Total bytes of all downloads with a known size.
    pub fn known_download_bytes(&self) -> u64 {
        self.downloads.iter().filter_map(|d| d.size).sum()
    }
}

/// Joins a metadata-supplied relative path onto `base`, rejecting absolute
/// paths and parent traversal so remote metadata can never write outside
/// the managed directories.
pub fn safe_join(base: &Path, relative: &str) -> Result<PathBuf, MinecraftError> {
    let rel = Path::new(relative);
    let mut out = base.to_path_buf();
    let mut pushed = false;
    for component in rel.components() {
        match component {
            Component::Normal(part) => {
                out.push(part);
                pushed = true;
            }
            Component::CurDir => {}
            _ => {
                return Err(MinecraftError::UnsafePath(relative.to_string()));
            }
        }
    }
    if !pushed {
        return Err(MinecraftError::UnsafePath(relative.to_string()));
    }
    Ok(out)
}

fn library_download_action(
    library: &Library,
    libraries_dir: &Path,
) -> Result<Option<DownloadAction>, MinecraftError> {
    let artifact = library.downloads.as_ref().and_then(|d| d.artifact.as_ref());
    if let Some(artifact) = artifact {
        let rel = match &artifact.path {
            Some(path) => path.clone(),
            None => maven_artifact_path(&library.name)
                .ok_or_else(|| MinecraftError::BadLibraryName(library.name.clone()))?,
        };
        let target = safe_join(libraries_dir, &rel)?;
        return Ok(Some(DownloadAction {
            url: artifact.url.clone(),
            target,
            sha1: artifact.sha1.clone(),
            size: artifact.size,
        }));
    }
    if let Some(base_url) = &library.url {
        let rel = maven_artifact_path(&library.name)
            .ok_or_else(|| MinecraftError::BadLibraryName(library.name.clone()))?;
        let target = safe_join(libraries_dir, &rel)?;
        return Ok(Some(DownloadAction {
            url: format!("{}/{rel}", base_url.trim_end_matches('/')),
            target,
            sha1: library.sha1.clone(),
            size: library.size,
        }));
    }
    Ok(None)
}

fn logging_download_action(
    logging: &LoggingConfig,
    layout: &InstallLayout,
) -> Result<DownloadAction, MinecraftError> {
    Ok(DownloadAction {
        url: logging.file.url.clone(),
        target: safe_join(&layout.assets_dir.join("log_configs"), &logging.file.id)?,
        sha1: logging.file.sha1.clone(),
        size: logging.file.size,
    })
}

/// Download extra libraries referenced by installer metadata without
/// forcing them onto the vanilla launch classpath.
pub fn build_library_downloads(
    libraries: &[Library],
    layout: &InstallLayout,
) -> Result<Vec<DownloadAction>, MinecraftError> {
    let mut downloads = Vec::new();
    for library in libraries {
        if let Some(action) = library_download_action(library, &layout.libraries_dir)?
            && !downloads
                .iter()
                .any(|existing: &DownloadAction| existing.target == action.target)
        {
            downloads.push(action);
        }
    }
    Ok(downloads)
}

/// Substitutes `${arch}` in a natives classifier key (Mojang's legacy
/// 32/64-bit split).
fn natives_classifier(template: &str, host: &Host) -> String {
    let bits = if host.arch.contains("64") { "64" } else { "32" };
    template.replace("${arch}", bits)
}

/// Plans the client jar, libraries, and natives for one (already merged)
/// version document.
pub fn build_version_plan(
    doc: &VersionDoc,
    host: &Host,
    layout: &InstallLayout,
) -> Result<InstallPlan, MinecraftError> {
    let mut plan = InstallPlan {
        version_id: doc.id.clone(),
        ..InstallPlan::default()
    };

    for library in &doc.libraries {
        if !rules_allow(&library.rules, host) {
            continue;
        }
        if let Some(action) = library_download_action(library, &layout.libraries_dir)? {
            let target = action.target.clone();
            plan.push_download(action);
            if !plan.classpath.contains(&target) {
                plan.classpath.push(target);
            }
        }
        // Legacy natives classifier for this OS.
        if let Some(classifier_template) = library.natives.get(&host.os_name) {
            let classifier = natives_classifier(classifier_template, host);
            let classified = library
                .downloads
                .as_ref()
                .and_then(|d| d.classifiers.get(&classifier));
            if let Some(artifact) = classified {
                let rel = match &artifact.path {
                    Some(path) => path.clone(),
                    None => maven_artifact_path(&format!("{}:{classifier}", library.name))
                        .ok_or_else(|| MinecraftError::BadLibraryName(library.name.clone()))?,
                };
                let target = safe_join(&layout.libraries_dir, &rel)?;
                plan.push_download(DownloadAction {
                    url: artifact.url.clone(),
                    target: target.clone(),
                    sha1: artifact.sha1.clone(),
                    size: artifact.size,
                });
                plan.extractions.push(ExtractAction {
                    archive: target,
                    dest: layout.natives_dir.clone(),
                    excludes: library
                        .extract
                        .as_ref()
                        .map(|e| e.exclude.clone())
                        .unwrap_or_else(|| vec!["META-INF/".to_string()]),
                });
            }
        }
    }

    // Client jar last on the classpath, matching the official launcher.
    let client = doc
        .downloads
        .get("client")
        .ok_or_else(|| MinecraftError::MissingClientDownload(doc.id.clone()))?;
    let client_jar = safe_join(&layout.versions_dir, &format!("{0}/{0}.jar", doc.id))?;
    plan.push_download(DownloadAction {
        url: client.url.clone(),
        target: client_jar.clone(),
        sha1: client.sha1.clone(),
        size: client.size,
    });
    plan.classpath.push(client_jar);

    if let Some(logging) = doc
        .logging
        .as_ref()
        .and_then(|logging| logging.client.as_ref())
    {
        plan.push_download(logging_download_action(logging, layout)?);
    }

    Ok(plan)
}

/// Mojang's content-addressed asset store URL.
pub const DEFAULT_RESOURCES_URL: &str = "https://resources.download.minecraft.net";

/// Plans all asset objects of one index, plus the materialized copies for
/// legacy `virtual` / `map_to_resources` layouts.
pub fn build_asset_plan(
    index_id: &str,
    index: &AssetIndex,
    layout: &InstallLayout,
    resources_url: &str,
) -> Result<InstallPlan, MinecraftError> {
    let mut plan = InstallPlan::default();
    let objects_dir = layout.assets_dir.join("objects");

    let materialize_root = if index.map_to_resources == Some(true) {
        Some(
            layout
                .resources_dir
                .clone()
                .ok_or(MinecraftError::ResourcesDirRequired)?,
        )
    } else if index.virtual_ == Some(true) {
        Some(safe_join(&layout.assets_dir.join("virtual"), index_id)?)
    } else {
        None
    };

    let mut seen_hashes = BTreeSet::new();
    for (object_path, object) in &index.objects {
        let hash = &object.hash;
        if hash.len() < 2 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(MinecraftError::BadAssetHash {
                object: object_path.clone(),
                hash: hash.clone(),
            });
        }
        let prefix = &hash[..2];
        let stored = safe_join(&objects_dir, &format!("{prefix}/{hash}"))?;
        if seen_hashes.insert(hash.clone()) {
            plan.push_download(DownloadAction {
                url: format!("{}/{prefix}/{hash}", resources_url.trim_end_matches('/')),
                target: stored.clone(),
                sha1: Some(hash.clone()),
                size: object.size,
            });
        }
        if let Some(root) = &materialize_root {
            plan.copies.push(CopyAction {
                from: stored,
                to: safe_join(root, object_path)?,
            });
        }
    }
    plan.game_assets_dir = materialize_root;
    Ok(plan)
}

/// Concatenates two plans (typically version + assets).
pub fn merge_plans(mut base: InstallPlan, other: InstallPlan) -> InstallPlan {
    for download in other.downloads {
        base.push_download(download);
    }
    base.extractions.extend(other.extractions);
    base.copies.extend(other.copies);
    base.classpath.extend(other.classpath);
    if base.game_assets_dir.is_none() {
        base.game_assets_dir = other.game_assets_dir;
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AssetIndex;

    fn layout(root: &Path) -> InstallLayout {
        InstallLayout {
            versions_dir: root.join("versions"),
            libraries_dir: root.join("libraries"),
            assets_dir: root.join("assets"),
            natives_dir: root.join("natives"),
            resources_dir: None,
        }
    }

    #[test]
    fn safe_join_rejects_traversal_and_absolute() {
        let base = Path::new("base");
        assert!(safe_join(base, "a/b.jar").is_ok());
        assert!(safe_join(base, "../evil").is_err());
        assert!(safe_join(base, "a/../../evil").is_err());
        assert!(safe_join(base, "/abs").is_err());
        assert!(safe_join(base, "").is_err());
        if cfg!(windows) {
            assert!(safe_join(base, "C:\\evil").is_err());
        }
    }

    #[test]
    fn version_plan_from_fixture() {
        let doc: VersionDoc =
            serde_json::from_str(include_str!("../tests/fixtures/version-modern.json")).unwrap();
        let host = Host {
            os_name: "windows".to_string(),
            arch: "x86_64".to_string(),
            os_version: "10.0".to_string(),
            features: Default::default(),
        };
        let root = Path::new("root");
        let plan = build_version_plan(&doc, &host, &layout(root)).unwrap();
        // linux-only library is filtered; windows library, plain library,
        // fabric-style library and client jar remain.
        let targets: Vec<String> = plan
            .downloads
            .iter()
            .map(|d| d.target.display().to_string().replace('\\', "/"))
            .collect();
        assert!(
            targets
                .contains(&"root/libraries/com/mojang/brigadier/1.2/brigadier-1.2.jar".to_string())
        );
        assert!(
            targets.contains(
                &"root/libraries/net/fabricmc/fabric-loader/0.16.0/fabric-loader-0.16.0.jar"
                    .to_string()
            )
        );
        assert!(!targets.iter().any(|t| t.contains("linux-only")));
        assert_eq!(
            plan.classpath
                .last()
                .unwrap()
                .display()
                .to_string()
                .replace('\\', "/"),
            "root/versions/fixture-1.0/fixture-1.0.jar"
        );
        // The fabric-style library URL is derived from maven coordinates.
        let fabric = plan
            .downloads
            .iter()
            .find(|d| d.url.starts_with("https://maven.example"))
            .unwrap();
        assert_eq!(
            fabric.url,
            "https://maven.example/repo/net/fabricmc/fabric-loader/0.16.0/fabric-loader-0.16.0.jar"
        );
        // Natives jar is downloaded and queued for extraction.
        assert_eq!(plan.extractions.len(), 1);
        assert!(
            plan.extractions[0]
                .archive
                .display()
                .to_string()
                .contains("natives-windows")
        );
        assert_eq!(plan.extractions[0].excludes, vec!["META-INF/".to_string()]);
        // Natives jars are not classpath entries.
        assert!(
            !plan
                .classpath
                .iter()
                .any(|p| p.display().to_string().contains("natives-windows"))
        );
    }

    #[test]
    fn missing_client_download_is_an_error() {
        let doc = VersionDoc {
            id: "x".to_string(),
            ..VersionDoc::default()
        };
        let host = Host::current();
        let err = build_version_plan(&doc, &host, &layout(Path::new("r"))).unwrap_err();
        assert!(err.to_string().contains("client download"));
    }

    #[test]
    fn asset_plan_dedupes_and_verifies_hashes() {
        let index = AssetIndex::from_slice(
            br#"{"objects": {
                "a/one.png": {"hash": "aa00000000000000000000000000000000000001", "size": 10},
                "b/two.png": {"hash": "aa00000000000000000000000000000000000001", "size": 10},
                "c/three.png": {"hash": "bb00000000000000000000000000000000000002", "size": 5}
            }}"#,
        )
        .unwrap();
        let plan =
            build_asset_plan("17", &index, &layout(Path::new("r")), DEFAULT_RESOURCES_URL).unwrap();
        assert_eq!(plan.downloads.len(), 2, "duplicate hash downloads once");
        assert!(plan.copies.is_empty());
        assert_eq!(plan.known_download_bytes(), 15);
        assert!(
            plan.downloads[0]
                .url
                .starts_with("https://resources.download.minecraft.net/aa/")
        );

        let bad = AssetIndex::from_slice(br#"{"objects": {"x": {"hash": "not-hex!", "size": 1}}}"#)
            .unwrap();
        assert!(
            build_asset_plan("17", &bad, &layout(Path::new("r")), DEFAULT_RESOURCES_URL).is_err()
        );
    }

    #[test]
    fn virtual_assets_are_materialized() {
        let index = AssetIndex::from_slice(
            br#"{"virtual": true, "objects": {
                "icons/icon_16x16.png": {"hash": "cc00000000000000000000000000000000000003", "size": 1}
            }}"#,
        )
        .unwrap();
        let root = Path::new("r");
        let plan =
            build_asset_plan("legacy", &index, &layout(root), DEFAULT_RESOURCES_URL).unwrap();
        assert_eq!(plan.copies.len(), 1);
        let to = plan.copies[0].to.display().to_string().replace('\\', "/");
        assert_eq!(to, "r/assets/virtual/legacy/icons/icon_16x16.png");
        assert_eq!(
            plan.game_assets_dir
                .as_ref()
                .unwrap()
                .display()
                .to_string()
                .replace('\\', "/"),
            "r/assets/virtual/legacy"
        );
    }

    #[test]
    fn logging_config_is_downloaded_to_assets() {
        let doc = VersionDoc {
            id: "fixture".to_string(),
            main_class: Some("net.minecraft.client.main.Main".to_string()),
            downloads: [(
                "client".to_string(),
                crate::model::DownloadRef {
                    url: "http://x/client.jar".to_string(),
                    sha1: None,
                    size: Some(5),
                },
            )]
            .into_iter()
            .collect(),
            logging: Some(crate::model::LoggingConfigSet {
                client: Some(crate::model::LoggingConfig {
                    argument: "-Dlog4j.configurationFile=${path}".to_string(),
                    file: crate::model::LoggingFile {
                        id: "client-1.xml".to_string(),
                        url: "http://x/logging.xml".to_string(),
                        sha1: None,
                        size: Some(3),
                    },
                }),
            }),
            ..VersionDoc::default()
        };
        let plan = build_version_plan(&doc, &Host::current(), &layout(Path::new("root"))).unwrap();
        assert!(plan.downloads.iter().any(|download| {
            download.target.display().to_string().replace('\\', "/")
                == "root/assets/log_configs/client-1.xml"
        }));
    }

    #[test]
    fn installer_library_downloads_do_not_require_classpath_insertion() {
        let libraries = vec![Library {
            name: "net.minecraftforge:javafmllanguage:1.20.1-47.4.5".to_string(),
            downloads: Some(crate::model::LibraryDownloads {
                artifact: Some(crate::model::Artifact {
                    path: Some("net/minecraftforge/javafmllanguage/1.20.1-47.4.5/javafmllanguage-1.20.1-47.4.5.jar".to_string()),
                    url: "http://x/javafmllanguage.jar".to_string(),
                    sha1: None,
                    size: Some(7),
                }),
                classifiers: Default::default(),
            }),
            url: None,
            sha1: None,
            size: None,
            natives: Default::default(),
            rules: vec![],
            extract: None,
        }];
        let downloads = build_library_downloads(&libraries, &layout(Path::new("root"))).unwrap();
        assert_eq!(downloads.len(), 1);
        assert_eq!(
            downloads[0].target.display().to_string().replace('\\', "/"),
            "root/libraries/net/minecraftforge/javafmllanguage/1.20.1-47.4.5/javafmllanguage-1.20.1-47.4.5.jar"
        );
    }
}
