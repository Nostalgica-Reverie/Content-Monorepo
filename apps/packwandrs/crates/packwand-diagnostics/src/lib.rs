//! Read-only validation, lint and MR/CF parity reports.

#![forbid(unsafe_code)]

mod registry;

pub use registry::{
    ContentRegistry, RegistryEntry, RegistryKind, build_all_registries, build_registry,
};

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use packwand_pack::Mod;
use packwand_workspace::{Manifest, Project};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub severity: Severity,
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub checked: usize,
    pub issues: Vec<Issue>,
}

impl ValidationReport {
    pub fn valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|issue| issue.severity == Severity::Error)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariantParityReport {
    pub pack: String,
    pub variant: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only_mr: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only_cf: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_drift: Vec<String>,
    pub mr_count: usize,
    pub cf_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing_side: Option<String>,
}

impl VariantParityReport {
    pub fn drifted(&self) -> bool {
        !self.only_mr.is_empty() || !self.only_cf.is_empty() || !self.file_drift.is_empty()
    }
}

pub fn lint_file(path: impl AsRef<Path>) -> Vec<Issue> {
    let path = path.as_ref();
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => return vec![error_issue(path, format!("could not read: {error}"))],
    };
    let result = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        serde_json::from_str::<serde_json::Value>(&source)
            .map(|_| ())
            .map_err(|error| error.to_string())
    } else if path.to_string_lossy().ends_with(".pw.toml") {
        toml::from_str::<Mod>(&source)
            .map(|_| ())
            .map_err(|error| error.to_string())
    } else if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
    {
        toml::from_str::<toml::Value>(&source)
            .map(|_| ())
            .map_err(|error| error.to_string())
    } else {
        return Vec::new();
    };
    result
        .err()
        .map(|message| vec![error_issue(path, message)])
        .unwrap_or_default()
}

pub fn lint_workspace(root: impl AsRef<Path>) -> ValidationReport {
    let root = root.as_ref();
    let mut report = ValidationReport::default();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0
                || !entry.file_type().is_dir()
                || !matches!(
                    entry.file_name().to_str(),
                    Some(".git" | "target" | "node_modules")
                )
        })
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let relevant = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            || path.to_string_lossy().ends_with(".pw.toml");
        if relevant {
            report.checked += 1;
            report.issues.extend(lint_file(path));
        }
    }
    report
}

/// Lint Minecraft pack content, including case collisions, namespaces, model
/// and texture references, and function-tag targets.
pub fn content_lint(root: impl AsRef<Path>) -> ValidationReport {
    let root = root.as_ref();
    let mut report = ValidationReport::default();
    let mut paths = BTreeMap::<String, PathBuf>::new();
    let mut json_documents = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0
                || !entry.file_type().is_dir()
                || !matches!(
                    entry.file_name().to_str(),
                    Some(".git" | "target" | "node_modules")
                )
        })
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        report.checked += 1;
        let relative = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        let key = relative.to_ascii_lowercase();
        if let Some(previous) = paths.insert(key, entry.path().to_path_buf())
            && previous != entry.path()
        {
            report.issues.push(error_issue(
                entry.path(),
                format!("case-colliding path also exists at {}", previous.display()),
            ));
        }
        validate_namespace_path(entry.path(), &relative, &mut report.issues);
        if entry.path().extension().is_some_and(|extension| {
            extension.eq_ignore_ascii_case("json") || extension.eq_ignore_ascii_case("mcmeta")
        }) {
            match fs::read_to_string(entry.path())
                .map_err(|error| error.to_string())
                .and_then(|source| {
                    serde_json::from_str::<serde_json::Value>(&source)
                        .map_err(|error| error.to_string())
                }) {
                Ok(value) => json_documents.push((entry.path().to_path_buf(), relative, value)),
                Err(message) => report.issues.push(error_issue(entry.path(), message)),
            }
        }
    }
    if !root.join("pack.mcmeta").is_file() {
        report.issues.push(Issue {
            severity: Severity::Warning,
            path: root.join("pack.mcmeta"),
            message: "pack.mcmeta is missing from the content root".into(),
        });
    }
    let known = paths.keys().cloned().collect::<BTreeSet<_>>();
    for (path, relative, value) in json_documents {
        validate_content_references(&path, &relative, &value, &known, &mut report.issues);
    }
    report
}

fn validate_namespace_path(path: &Path, relative: &str, issues: &mut Vec<Issue>) {
    let parts = relative.split('/').collect::<Vec<_>>();
    let Some(position) = parts
        .iter()
        .position(|part| matches!(*part, "assets" | "data"))
    else {
        return;
    };
    let Some(namespace) = parts.get(position + 1) else {
        return;
    };
    if namespace.is_empty()
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
    {
        issues.push(error_issue(
            path,
            format!("invalid Minecraft namespace {namespace:?}"),
        ));
    }
}

fn validate_content_references(
    path: &Path,
    relative: &str,
    value: &serde_json::Value,
    known: &BTreeSet<String>,
    issues: &mut Vec<Issue>,
) {
    if (relative.contains("/tags/function/") || relative.contains("/tags/functions/"))
        && let Some(values) = value.get("values").and_then(serde_json::Value::as_array)
    {
        for entry in values {
            let Some(reference) = entry
                .as_str()
                .or_else(|| entry.get("id").and_then(serde_json::Value::as_str))
            else {
                continue;
            };
            let reference = reference.trim_start_matches('#');
            if let Some((namespace, name)) = split_identifier(reference) {
                let singular =
                    format!("data/{namespace}/function/{name}.mcfunction").to_ascii_lowercase();
                let plural =
                    format!("data/{namespace}/functions/{name}.mcfunction").to_ascii_lowercase();
                if !known.contains(&singular)
                    && !known.contains(&plural)
                    && !reference.starts_with("minecraft:")
                {
                    issues.push(error_issue(
                        path,
                        format!("missing function reference {reference}"),
                    ));
                }
            }
        }
    }
    if let Some(parent) = value.get("parent").and_then(serde_json::Value::as_str)
        && !parent.starts_with("builtin/")
    {
        validate_asset_reference(path, parent, "models", ".json", known, issues);
    }
    if let Some(textures) = value.get("textures").and_then(serde_json::Value::as_object) {
        for texture in textures.values().filter_map(serde_json::Value::as_str) {
            if !texture.starts_with('#') {
                validate_asset_reference(path, texture, "textures", ".png", known, issues);
            }
        }
    }
}

fn validate_asset_reference(
    path: &Path,
    reference: &str,
    folder: &str,
    extension: &str,
    known: &BTreeSet<String>,
    issues: &mut Vec<Issue>,
) {
    let Some((namespace, name)) = split_identifier(reference) else {
        return;
    };
    let candidate = format!("assets/{namespace}/{folder}/{name}{extension}").to_ascii_lowercase();
    if !known.contains(&candidate) && namespace != "minecraft" {
        issues.push(error_issue(
            path,
            format!("missing {folder} reference {reference}"),
        ));
    }
}

fn split_identifier(value: &str) -> Option<(&str, &str)> {
    let (namespace, path) = value.split_once(':').unwrap_or(("minecraft", value));
    (!namespace.is_empty()
        && !path.is_empty()
        && !path.contains("..")
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        }))
    .then_some((namespace, path))
}

pub fn validate_projects(root: impl AsRef<Path>) -> packwand_workspace::Result<ValidationReport> {
    let projects = packwand_workspace::discover(root)?;
    let ids = projects
        .iter()
        .map(|project| project.manifest.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut report = ValidationReport {
        checked: projects.len(),
        issues: Vec::new(),
    };
    for project in &projects {
        validate_project(project, &ids, &mut report.issues);
    }
    Ok(report)
}

fn validate_project(project: &Project, ids: &BTreeSet<&str>, issues: &mut Vec<Issue>) {
    let manifest = &project.manifest;
    let path = project.root.join("manifest.json");
    for (name, value) in [
        ("id", manifest.id.as_str()),
        ("name", manifest.name.as_str()),
        ("type", manifest.project_type.as_str()),
        (
            "release_type",
            manifest.release_type.as_deref().unwrap_or(""),
        ),
    ] {
        if value.trim().is_empty() {
            issues.push(error_issue(
                &path,
                format!("manifest missing required field: {name}"),
            ));
        }
    }
    if !matches!(
        manifest.project_type.as_str(),
        "mod" | "modpack" | "datapack" | "resourcepack"
    ) {
        issues.push(error_issue(
            &path,
            format!("invalid type: {}", manifest.project_type),
        ));
    }
    if !matches!(
        manifest.release_type.as_deref(),
        Some("release" | "beta" | "alpha")
    ) {
        issues.push(error_issue(
            &path,
            "release_type must be release, beta, or alpha",
        ));
    }
    if !matches!(
        manifest.lifecycle(),
        "active" | "maintenance" | "archived" | "eol"
    ) {
        issues.push(error_issue(
            &path,
            format!("invalid lifecycle: {}", manifest.lifecycle()),
        ));
    }
    if manifest.role.is_none() {
        issues.push(error_issue(&path, "manifest missing required field: role"));
    }
    if manifest.mc_version.is_some() != manifest.variants.is_empty() {
        issues.push(error_issue(
            &path,
            "manifest must declare exactly one of mc_version or variants",
        ));
    }
    if manifest.project_type == "mod" && manifest.variants.is_empty() {
        issues.push(error_issue(
            &path,
            "mod manifests require at least one variant",
        ));
    }
    if manifest.project_type == "modpack"
        && manifest.variants.is_empty()
        && manifest.loader.as_deref().unwrap_or("").trim().is_empty()
    {
        issues.push(error_issue(
            &path,
            "modpack manifests must declare a loader",
        ));
    }
    let has_platform = [
        &manifest.modrinth_id,
        &manifest.curseforge_id,
        &manifest.github_id,
        &manifest.gitea_id,
        &manifest.gitlab_id,
    ]
    .iter()
    .any(|id| id.as_deref().is_some_and(|id| !id.trim().is_empty()));
    if !has_platform {
        issues.push(error_issue(
            &path,
            "manifest must set at least one platform id",
        ));
    }
    validate_role(manifest, ids, &path, issues);
    let changelog = project.root.join("changelog.md");
    match fs::read_to_string(&changelog) {
        Ok(content)
            if content.lines().any(|line| {
                let line = line.trim();
                !line.is_empty() && !line.starts_with('#')
            }) => {}
        Ok(_) => issues.push(error_issue(&changelog, "changelog has no content")),
        Err(_) => issues.push(error_issue(&changelog, "changelog.md is missing")),
    }
    validate_subdirs(project, issues);
}

fn validate_role(manifest: &Manifest, ids: &BTreeSet<&str>, path: &Path, issues: &mut Vec<Issue>) {
    let Some(role) = &manifest.role else { return };
    if let Some(label) = role.as_str() {
        if !matches!(label, "none" | "base") {
            issues.push(error_issue(path, format!("invalid role: {label}")));
        }
        if manifest.project_type == "mod" && label != "none" {
            issues.push(error_issue(path, "mods must use role none"));
        }
        return;
    }
    let base = role
        .pointer("/performance_base/pack")
        .and_then(serde_json::Value::as_str);
    let mappings = role
        .pointer("/performance_base/mappings")
        .and_then(serde_json::Value::as_array);
    if base.is_none() || mappings.is_none_or(Vec::is_empty) {
        issues.push(error_issue(
            path,
            "performance_base requires pack and mappings",
        ));
        return;
    }
    let base = base.unwrap();
    if base == manifest.id || !ids.contains(base) {
        issues.push(error_issue(
            path,
            format!("unknown or self-referencing performance base: {base}"),
        ));
    }
    for mapping in mappings.unwrap() {
        let source = mapping
            .get("source")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let target = mapping
            .get("target")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let platform = |value: &str| {
            if value.ends_with("-mr") {
                "mr"
            } else if value.ends_with("-cf") {
                "cf"
            } else {
                ""
            }
        };
        if platform(source).is_empty()
            || platform(target).is_empty()
            || platform(source) != platform(target)
        {
            issues.push(error_issue(
                path,
                format!("invalid or cross-platform mapping: {source} -> {target}"),
            ));
        }
    }
}

fn validate_subdirs(project: &Project, issues: &mut Vec<Issue>) {
    if project.manifest.project_type != "modpack" {
        return;
    }
    let path = project.root.join("manifest.json");
    let names = project
        .subdirs
        .iter()
        .filter_map(|path| path.file_name()?.to_str())
        .collect::<BTreeSet<_>>();
    let keys = if project.manifest.variants.is_empty() {
        project
            .manifest
            .mc_version
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    } else {
        project
            .manifest
            .variants
            .iter()
            .filter_map(|variant| variant.id.as_deref().or(variant.mc_version.as_deref()))
            .collect()
    };
    for key in keys {
        if project
            .manifest
            .modrinth_id
            .as_deref()
            .is_some_and(|id| !id.trim().is_empty())
            && !names.contains(format!("{key}-mr").as_str())
        {
            issues.push(error_issue(&path, format!("missing {key}-mr subdir")));
        }
        if project
            .manifest
            .curseforge_id
            .as_deref()
            .is_some_and(|id| !id.trim().is_empty())
            && !names.contains(format!("{key}-cf").as_str())
        {
            issues.push(error_issue(&path, format!("missing {key}-cf subdir")));
        }
    }
}

fn error_issue(path: &Path, message: impl Into<String>) -> Issue {
    Issue {
        severity: Severity::Error,
        path: path.to_path_buf(),
        message: message.into(),
    }
}

#[derive(Debug, Clone)]
struct PwMeta {
    name: String,
    file: String,
}

pub fn parity_project(project: &Project) -> Vec<VariantParityReport> {
    let mut pairs = BTreeMap::<String, (bool, bool)>::new();
    for subdir in &project.subdirs {
        let Some(name) = subdir.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if let Some(key) = name.strip_suffix("-mr") {
            pairs.entry(key.to_owned()).or_default().0 = true;
        }
        if let Some(key) = name.strip_suffix("-cf") {
            pairs.entry(key.to_owned()).or_default().1 = true;
        }
    }
    pairs
        .into_iter()
        .map(|(variant, (mr, cf))| parity_pair(project, variant, mr, cf))
        .collect()
}

pub fn parity_workspace(
    root: impl AsRef<Path>,
) -> packwand_workspace::Result<Vec<VariantParityReport>> {
    Ok(packwand_workspace::discover(root)?
        .iter()
        .filter(|project| project.category == "modpacks")
        .flat_map(parity_project)
        .collect())
}

fn parity_pair(
    project: &Project,
    variant: String,
    has_mr: bool,
    has_cf: bool,
) -> VariantParityReport {
    let mut report = VariantParityReport {
        pack: project.manifest.id.clone(),
        variant: variant.clone(),
        ..VariantParityReport::default()
    };
    if !has_mr || !has_cf {
        report.missing_side = Some(if has_mr { "cf" } else { "mr" }.to_owned());
        return report;
    }
    let mr = collect_metadata(&project.root.join(format!("{variant}-mr")));
    let cf = collect_metadata(&project.root.join(format!("{variant}-cf")));
    report.mr_count = mr.len();
    report.cf_count = cf.len();
    let mut matched_cf = BTreeSet::new();
    for (slug, mr_meta) in &mr {
        if let Some(cf_meta) = cf.get(slug) {
            matched_cf.insert(slug.clone());
            if mr_meta.file != cf_meta.file {
                report.file_drift.push(format!(
                    "{slug}: {} (mr) != {} (cf)",
                    mr_meta.file, cf_meta.file
                ));
            }
            continue;
        }
        let match_slug = cf
            .iter()
            .find(|(cf_slug, meta)| {
                !matched_cf.contains(*cf_slug)
                    && ((!mr_meta.name.is_empty()
                        && normalize_name(&mr_meta.name) == normalize_name(&meta.name))
                        || (!mr_meta.file.is_empty() && mr_meta.file == meta.file))
            })
            .map(|(slug, _)| slug.clone());
        if let Some(cf_slug) = match_slug {
            matched_cf.insert(cf_slug);
        } else {
            report.only_mr.push(slug.clone());
        }
    }
    for slug in cf.keys() {
        if !mr.contains_key(slug) && !matched_cf.contains(slug) {
            report.only_cf.push(slug.clone());
        }
    }
    report.only_mr.sort();
    report.only_cf.sort();
    report.file_drift.sort();
    report
}

fn collect_metadata(root: &Path) -> BTreeMap<String, PwMeta> {
    WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry.file_type().is_file() && entry.file_name().to_string_lossy().ends_with(".pw.toml")
        })
        .filter_map(|entry| {
            let metadata: Mod = toml::from_str(&fs::read_to_string(entry.path()).ok()?).ok()?;
            let slug = entry
                .file_name()
                .to_string_lossy()
                .trim_end_matches(".pw.toml")
                .to_owned();
            Some((
                slug,
                PwMeta {
                    name: metadata.name,
                    file: metadata.filename,
                },
            ))
        })
        .collect()
}

fn normalize_name(name: &str) -> String {
    let mut value = name
        .to_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>();
    for loader in ["fabric", "forge", "neoforge", "quilt"] {
        value = value.strip_suffix(loader).unwrap_or(&value).to_owned();
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use packwand_workspace::{NewProject, ProjectRole};

    fn create(root: &Path, id: &str) -> Project {
        packwand_workspace::create_project(
            root,
            &NewProject {
                category: "modpacks".to_owned(),
                id: id.to_owned(),
                name: None,
                minecraft_version: Some("1.21.1".to_owned()),
                loader: Some("fabric".to_owned()),
                variants: Vec::new(),
                role: ProjectRole::None,
            },
        )
        .unwrap()
    }

    fn metadata(name: &str, filename: &str) -> String {
        format!(
            "name = \"{name}\"\nfilename = \"{filename}\"\nside = \"both\"\n\n[download]\nhash-format = \"sha512\"\nhash = \"abc\"\n"
        )
    }

    #[test]
    fn lint_reports_json_and_metadata_syntax() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("bad.json"), "{").unwrap();
        fs::write(root.path().join("bad.pw.toml"), "not toml").unwrap();
        let report = lint_workspace(root.path());
        assert_eq!(report.checked, 2);
        assert_eq!(report.issues.len(), 2);
        assert!(!report.valid());
    }

    #[test]
    fn validates_a_native_scaffold() {
        let root = tempfile::tempdir().unwrap();
        let project = create(root.path(), "example");
        let mut manifest = project.manifest;
        manifest.modrinth_id = Some("example".to_owned());
        packwand_workspace::write_manifest(project.root, &manifest).unwrap();
        let report = validate_projects(root.path()).unwrap();
        assert!(report.valid(), "{:?}", report.issues);
    }

    #[test]
    fn parity_matches_different_slugs_by_display_name() {
        let root = tempfile::tempdir().unwrap();
        let project = create(root.path(), "example");
        let mr = project.root.join("1.21.1-mr/mods");
        let cf = project.root.join("1.21.1-cf/mods");
        fs::create_dir_all(&mr).unwrap();
        fs::create_dir_all(&cf).unwrap();
        fs::write(
            mr.join("ferrite-core.pw.toml"),
            metadata("FerriteCore (Fabric)", "ferrite.jar"),
        )
        .unwrap();
        fs::write(
            cf.join("ferritecore.pw.toml"),
            metadata("FerriteCore", "ferrite.jar"),
        )
        .unwrap();
        let reports =
            parity_project(&packwand_workspace::read_project(root.path(), &project.root).unwrap());
        assert_eq!(reports.len(), 1);
        assert!(!reports[0].drifted());
    }

    #[test]
    fn content_lint_finds_case_collisions_and_missing_assets() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("assets/example/models/item")).unwrap();
        fs::write(
            root.path().join("pack.mcmeta"),
            r#"{"pack":{"pack_format":34,"description":"test"}}"#,
        )
        .unwrap();
        fs::write(
            root.path().join("assets/example/models/item/tool.json"),
            r#"{"parent":"example:item/missing","textures":{"layer0":"example:item/missing"}}"#,
        )
        .unwrap();
        #[cfg(not(windows))]
        fs::write(
            root.path().join("assets/example/models/item/Tool.json"),
            "{}",
        )
        .unwrap();
        let report = content_lint(root.path());
        #[cfg(not(windows))]
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.message.contains("case-colliding"))
        );
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.message.contains("missing models"))
        );
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.message.contains("missing textures"))
        );
    }
}
