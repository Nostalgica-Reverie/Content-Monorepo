//! Read-only validation, lint and MR/CF parity reports.

#![forbid(unsafe_code)]

mod conventions;
mod registry;

pub use conventions::{CHECKS, conventions_lint};
pub use registry::{
	ContentRegistry, RegistryEntry, RegistryKind, build_all_registries, build_all_registries_with,
	build_registry, build_registry_with,
};

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use packwand_pack::Mod;
use packwand_parallel::Jobs;
use packwand_workspace::{Manifest, Project};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
	} else if packwand_pack::metafile::is_metafile(path) {
		serde_json::from_str::<Mod>(&source)
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

/// The content categories a workspace-wide lint covers.
const LINT_CATEGORIES: [&str; 4] = ["mods", "modpacks", "datapacks", "resourcepacks"];

fn is_lintable(path: &Path) -> bool {
	path.extension()
		.is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
		|| packwand_pack::metafile::is_metafile(path)
}

/// The directories a workspace lint should walk.
///
/// At a repository root that is the content categories, not the repository
/// itself: a repo-wide walk also descends into vendored third-party trees,
/// which are checked in (so `.gitignore` does not exclude them) and whose
/// JSONC config files are correctly-formed JSONC but invalid JSON. Anywhere
/// else — a single pack root, as preflight passes — the root is the scope.
fn lint_roots(root: &Path) -> Vec<PathBuf> {
	let categories: Vec<PathBuf> = LINT_CATEGORIES
		.iter()
		.map(|category| root.join(category))
		.filter(|path| path.is_dir())
		.collect();
	if categories.is_empty() {
		vec![root.to_path_buf()]
	} else {
		categories
	}
}

pub fn lint_workspace(root: impl AsRef<Path>) -> ValidationReport {
	let mut report = ValidationReport::default();
	for scope in lint_roots(root.as_ref()) {
		for entry in WalkDir::new(&scope)
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
			if is_lintable(entry.path()) {
				report.checked += 1;
				report.issues.extend(lint_file(entry.path()));
			}
		}
	}
	report
}

/// Lints only the files touched by the HEAD commit. This is what `lint` with
/// no arguments does, so a pre-commit or CI syntax gate costs a handful of
/// file reads instead of a full-repository walk.
pub fn lint_changed(root: impl AsRef<Path>) -> ValidationReport {
	let root = root.as_ref();
	let mut report = ValidationReport::default();
	for relative in git_changed_files(root) {
		let path = root.join(&relative);
		// Skip paths the commit deleted or renamed away.
		if !is_lintable(&path) || !path.is_file() {
			continue;
		}
		report.checked += 1;
		report.issues.extend(lint_file(&path));
	}
	report
}

/// Paths changed by the HEAD commit, relative to the repository root. Returns
/// nothing outside a git checkout, which callers treat as "nothing to lint".
fn git_changed_files(root: &Path) -> Vec<String> {
	let Ok(output) = std::process::Command::new("git")
		.args(["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
		.current_dir(root)
		.output()
	else {
		return Vec::new();
	};
	if !output.status.success() {
		return Vec::new();
	}
	String::from_utf8_lossy(&output.stdout)
		.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty())
		.map(str::to_owned)
		.collect()
}

/// Lint Minecraft pack content, including case collisions, namespaces, model
/// and texture references, and function-tag targets.
pub fn content_lint(root: impl AsRef<Path>) -> ValidationReport {
	content_lint_with(root, true)
}

/// As [`content_lint`], with an explicit worker count for the hashing pass.
pub fn content_lint_jobs(root: impl AsRef<Path>, hygiene: bool, jobs: Jobs) -> ValidationReport {
	content_lint_inner(root, hygiene, jobs)
}

/// `hygiene` enables the archive-hygiene rules — duplicate files and
/// resource-location charset. Those belong to the `content-lint` command;
/// preflight's reference gate leaves them out, so a pre-launch check does not
/// fail on packaging problems that do not stop the pack from loading.
pub fn content_lint_with(root: impl AsRef<Path>, hygiene: bool) -> ValidationReport {
	content_lint_inner(root, hygiene, packwand_parallel::configured())
}

fn content_lint_inner(root: impl AsRef<Path>, hygiene: bool, jobs: Jobs) -> ValidationReport {
	let root = root.as_ref();
	let mut report = ValidationReport::default();
	let mut paths = BTreeMap::<String, PathBuf>::new();
	// Paths of the JSON/mcmeta files, collected during the walk and read and
	// parsed in one parallel pass afterwards. The walk itself stays
	// sequential: it maintains the `paths` map that the case-collision check
	// reads and writes as it goes.
	let mut json_paths: Vec<(PathBuf, String)> = Vec::new();
	let mut content_files: Vec<(PathBuf, String)> = Vec::new();
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
		content_files.push((entry.path().to_path_buf(), relative.clone()));
		if entry.path().extension().is_some_and(|extension| {
			extension.eq_ignore_ascii_case("json") || extension.eq_ignore_ascii_case("mcmeta")
		}) {
			json_paths.push((entry.path().to_path_buf(), relative));
		}
	}

	// Reading and parsing the JSON is the bulk of a content lint on a real
	// pack — tens of thousands of small documents — and each one is
	// independent. Results come back in walk order, so the issue list this
	// produces is identical to the sequential pass.
	let parsed = packwand_parallel::map(&json_paths, jobs, |(path, _)| {
		fs::read_to_string(path)
			.map_err(|error| error.to_string())
			.and_then(|source| {
				serde_json::from_str::<serde_json::Value>(&source)
					.map_err(|error| error.to_string())
			})
	});
	let mut json_documents = Vec::with_capacity(json_paths.len());
	for ((path, relative), outcome) in json_paths.into_iter().zip(parsed) {
		match outcome {
			Ok(value) => json_documents.push((path, relative, value)),
			Err(message) => report.issues.push(error_issue(&path, message)),
		}
	}
	if !root.join("pack.mcmeta").is_file() {
		report.issues.push(Issue {
			severity: Severity::Warning,
			path: root.join("pack.mcmeta"),
			message: "pack.mcmeta is missing from the content root".into(),
		});
	}
	if hygiene {
		lint_path_charsets(&content_files, &mut report.issues);
		lint_duplicate_files(&content_files, jobs, &mut report.issues);
	}
	let known = paths.keys().cloned().collect::<BTreeSet<_>>();
	for (path, relative, value) in json_documents {
		validate_content_references(&path, &relative, &value, &known, &mut report.issues);
	}
	report
}

fn is_resource_segment(segment: &str) -> bool {
	!segment.is_empty()
		&& segment.bytes().all(|byte| {
			byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
		})
}

/// Flags path segments under `data/` or `assets/` that leave the
/// resource-location character set. Uppercase letters and spaces resolve fine
/// on Windows and macOS and then fail on a case-sensitive server filesystem,
/// so this is an error rather than advice.
///
/// Matched from the `data`/`assets` component rather than the start of the
/// path, because a project root holds its content under version directories.
fn lint_path_charsets(files: &[(PathBuf, String)], issues: &mut Vec<Issue>) {
	for (path, relative) in files {
		let parts = relative.split('/').collect::<Vec<_>>();
		let Some(position) = parts
			.iter()
			.position(|part| matches!(*part, "assets" | "data"))
		else {
			continue;
		};
		for segment in &parts[position + 1..] {
			if !is_resource_segment(segment) {
				issues.push(error_issue(
                    path,
                    format!(
                        "path segment {segment:?} violates resource-location charset [a-z0-9_.-] (breaks on case-sensitive filesystems)"
                    ),
                ));
			}
		}
	}
}

/// Warns about byte-identical files — usually a copy-paste leftover that
/// bloats the shipped archive. Grouped by size first so only genuine
/// candidates are read and hashed.
fn lint_duplicate_files(files: &[(PathBuf, String)], jobs: Jobs, issues: &mut Vec<Issue>) {
	let mut by_size: BTreeMap<u64, Vec<&(PathBuf, String)>> = BTreeMap::new();
	for file in files {
		let Ok(metadata) = fs::metadata(&file.0) else {
			continue;
		};
		if metadata.len() == 0 {
			continue;
		}
		by_size.entry(metadata.len()).or_default().push(file);
	}
	// Only size-matched candidates are read, and each digest is independent,
	// so the reads and hashes run concurrently. Findings are grouped and
	// sorted afterwards, so the output does not depend on completion order.
	let candidates: Vec<&(PathBuf, String)> = by_size
		.values()
		.filter(|group| group.len() >= 2)
		.flat_map(|group| group.iter().copied())
		.collect();
	let digests = packwand_parallel::map(&candidates, jobs, |file| {
		fs::read(&file.0)
			.ok()
			.map(|bytes| format!("{:x}", Sha256::digest(&bytes)))
	});
	let digest_of: BTreeMap<&Path, String> = candidates
		.iter()
		.zip(digests)
		.filter_map(|(file, digest)| digest.map(|digest| (file.0.as_path(), digest)))
		.collect();

	for group in by_size.values() {
		if group.len() < 2 {
			continue;
		}
		let mut by_hash: BTreeMap<String, Vec<&(PathBuf, String)>> = BTreeMap::new();
		for file in group {
			let Some(digest) = digest_of.get(file.0.as_path()) else {
				continue;
			};
			by_hash.entry(digest.clone()).or_default().push(file);
		}
		for duplicates in by_hash.values() {
			if duplicates.len() < 2 {
				continue;
			}
			let mut sorted = duplicates.clone();
			sorted.sort_by(|left, right| left.1.cmp(&right.1));
			let names = sorted
				.iter()
				.map(|file| file.1.as_str())
				.collect::<Vec<_>>()
				.join(" == ");
			issues.push(Issue {
				severity: Severity::Warning,
				path: sorted[0].0.clone(),
				message: format!("duplicate content: {names}"),
			});
		}
	}
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
		}) {
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
	let root = root.as_ref();
	let projects = packwand_workspace::discover(root)?;
	let ids = projects
		.iter()
		.map(|project| project.manifest.id.as_str())
		.collect::<BTreeSet<_>>();
	let mut report = ValidationReport {
		checked: projects.len(),
		issues: Vec::new(),
	};
	let schema = ManifestSchema::load(root);
	for project in &projects {
		validate_project(project, &ids, &mut report.issues);
		if let Some(schema) = schema.as_ref() {
			schema.validate(project, &mut report.issues);
		}
	}
	Ok(report)
}

/// The repository's manifest JSON Schema, when it is available. Manifests are
/// also checked field-by-field above; this catches the things a typed struct
/// cannot see — unknown properties, `oneOf` shape rules, and value patterns —
/// and keeps the schema that editors consume honest about what actually ships.
struct ManifestSchema {
	validator: jsonschema::Validator,
}

impl ManifestSchema {
	/// Returns `None` when the schema is absent or unusable. A missing schema
	/// is not a validation failure: the file is developer tooling, and pack
	/// authors run `validate` from checkouts that may not include it.
	fn load(root: &Path) -> Option<Self> {
		let source = fs::read_to_string(root.join("scripts").join("schema.json")).ok()?;
		let document: serde_json::Value = serde_json::from_str(&source).ok()?;
		jsonschema::validator_for(&document)
			.ok()
			.map(|validator| Self { validator })
	}

	fn validate(&self, project: &Project, issues: &mut Vec<Issue>) {
		let path = project.root.join("manifest.json");
		let Ok(source) = fs::read_to_string(&path) else {
			return;
		};
		let Ok(document) = serde_json::from_str::<serde_json::Value>(&source) else {
			return; // Malformed JSON is already reported by the syntax lint.
		};
		for error in self.validator.iter_errors(&document) {
			let location = error.instance_path().to_string();
			let where_ = if location.is_empty() {
				"manifest".to_owned()
			} else {
				location.trim_start_matches('/').replace('/', ".")
			};
			issues.push(error_issue(&path, format!("schema: {where_}: {error}")));
		}
	}
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
	validate_environment(project, issues);
	validate_automation(project, issues);
}

/// Flags mods whose `side` cannot run in the environment their pack declares —
/// a client-only mod shipped in a server pack, or the reverse.
///
/// The side comparison is a warning, not an error. `side` is provider
/// metadata (Modrinth's `client_side`/`server_side`), and it is wrong often
/// enough that a hard gate would block packs that are actually correct;
/// `automation.environment_exempt` silences a slug that is mislabelled
/// upstream. An unrecognized `environment` value *is* an error, since that is
/// the pack's own declaration and a typo would silently disable the rule.
fn validate_environment(project: &Project, issues: &mut Vec<Issue>) {
	let manifest = &project.manifest;
	let manifest_path = project.root.join("manifest.json");
	let declared = std::iter::once((None, manifest.environment.as_deref())).chain(
		manifest
			.variants
			.iter()
			.map(|variant| (variant.key(), variant.environment.as_deref())),
	);
	for (key, value) in declared {
		if let Some(value) = value
			&& !packwand_workspace::ENVIRONMENTS.contains(&value)
		{
			let where_ = key.map_or_else(String::new, |key| format!("variant {key} "));
			issues.push(error_issue(
				&manifest_path,
				format!("invalid {where_}environment: {value} (expected client, server, or both)"),
			));
		}
	}
	if manifest.project_type != "modpack" {
		return;
	}
	let exempt = manifest
		.automation()
		.environment_exempt
		.into_iter()
		.collect::<BTreeSet<_>>();
	for subdir in &project.subdirs {
		let Some(name) = subdir.file_name().and_then(|name| name.to_str()) else {
			continue;
		};
		let key = name
			.strip_suffix("-mr")
			.or_else(|| name.strip_suffix("-cf"))
			.unwrap_or(name);
		let environment = manifest.environment_for(key);
		// "both" accepts everything, so there is nothing to compare against.
		let unrunnable = match environment {
			"server" => "client",
			"client" => "server",
			_ => continue,
		};
		for (slug, path, side) in mod_sides(subdir) {
			if side == unrunnable && !exempt.contains(&slug) {
				issues.push(warning_issue(
					&path,
					format!(
						"{slug} is side \"{side}\" but {name} declares environment \
                         \"{environment}\" — it will not run there; add it to \
                         automation.environment_exempt if the provider mislabelled it"
					),
				));
			}
		}
	}
}

/// `(slug, path, side)` for every mod metadata file directly under `mods/`.
/// Unreadable and malformed files are skipped — the syntax lint reports those.
fn mod_sides(subdir: &Path) -> Vec<(String, PathBuf, String)> {
	let Ok(entries) = fs::read_dir(subdir.join("mods")) else {
		return Vec::new();
	};
	let mut sides = entries
		.flatten()
		.filter_map(|entry| {
			let path = entry.path();
			let slug = path
				.file_name()
				.and_then(|name| name.to_str())?
				.strip_suffix(".pw.json")?
				.to_owned();
			let metadata: Mod = serde_json::from_str(&fs::read_to_string(&path).ok()?).ok()?;
			Some((slug, path, metadata.side))
		})
		.collect::<Vec<_>>();
	sides.sort();
	sides
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
			.filter_map(packwand_workspace::Variant::key)
			.collect()
	};
	let has_mr = project
		.manifest
		.modrinth_id
		.as_deref()
		.is_some_and(|id| !id.trim().is_empty());
	let has_cf = project
		.manifest
		.curseforge_id
		.as_deref()
		.is_some_and(|id| !id.trim().is_empty());
	for key in keys {
		let mr = names.contains(format!("{key}-mr").as_str());
		let cf = names.contains(format!("{key}-cf").as_str());
		if has_mr && !mr {
			issues.push(error_issue(&path, format!("missing {key}-mr subdir")));
		}
		if has_cf && !cf {
			issues.push(error_issue(&path, format!("missing {key}-cf subdir")));
		}
		// The other direction: content that is present but has nowhere to go.
		// Not an error — the pack builds fine, it just never reaches that
		// platform, which is easy to not notice.
		if mr && !has_mr {
			issues.push(warning_issue(
				&path,
				format!("{key}-mr exists but modrinth_id is not set — it will not publish"),
			));
		}
		if cf && !has_cf {
			issues.push(warning_issue(
				&path,
				format!("{key}-cf exists but curseforge_id is not set — it will not publish"),
			));
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

fn warning_issue(path: &Path, message: impl Into<String>) -> Issue {
	Issue {
		severity: Severity::Warning,
		path: path.to_path_buf(),
		message: message.into(),
	}
}

/// Rules for the manifest's `automation` block and the legacy sidecar files
/// it replaced. These are warnings: the legacy layout still works, and a
/// stale freeze entry is a silent no-op rather than a broken pack.
fn validate_automation(project: &Project, issues: &mut Vec<Issue>) {
	for legacy in ["opt-out.json", "auto-update-ignore.json"] {
		let path = project.root.join(legacy);
		if path.is_file() {
			issues.push(warning_issue(
				&path,
				format!("{legacy} is deprecated — migrate into manifest.json \"automation\""),
			));
		}
	}
	for subdir in &project.subdirs {
		let path = subdir.join("sync-exclude.json");
		if path.is_file() {
			issues.push(warning_issue(
				&path,
				"sync-exclude.json is deprecated — migrate into manifest.json \"automation\".sync_exclude",
			));
		}
	}
	let Some(automation) = project.manifest.automation.as_ref() else {
		return;
	};
	let names = project
		.subdirs
		.iter()
		.filter_map(|path| path.file_name()?.to_str())
		.collect::<BTreeSet<_>>();
	let manifest_path = project.root.join("manifest.json");
	for subdir in automation.freeze.keys() {
		if !names.contains(subdir.as_str()) {
			issues.push(warning_issue(
				&manifest_path,
				format!(
					"automation.freeze names unknown subdir {subdir:?} — the freeze has no effect"
				),
			));
		}
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
			entry.file_type().is_file() && packwand_pack::metafile::is_metafile(entry.path())
		})
		.filter_map(|entry| {
			let metadata: Mod =
				serde_json::from_str(&fs::read_to_string(entry.path()).ok()?).ok()?;
			let slug = entry
				.file_name()
				.to_string_lossy()
				.trim_end_matches(".pw.json")
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
			r#"{{"name": "{name}", "filename": "{filename}", "side": "both",
                "download": {{"hash-format": "sha512", "hash": "abc"}}}}"#
		)
	}

	#[test]
	fn lint_reports_json_and_metadata_syntax() {
		let root = tempfile::tempdir().unwrap();
		fs::write(root.path().join("bad.json"), "{").unwrap();
		fs::write(root.path().join("bad.pw.json"), "not json").unwrap();
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

	/// Writes `slug` into `subdir`'s mod folder with the given side.
	fn write_mod(project: &Project, subdir: &str, slug: &str, side: &str) {
		let mods = project.root.join(subdir).join("mods");
		fs::create_dir_all(&mods).unwrap();
		fs::write(
			mods.join(format!("{slug}.pw.json")),
			format!(
				r#"{{"name": "{slug}", "filename": "{slug}.jar", "side": "{side}",
                    "download": {{"hash-format": "sha512", "hash": "abc"}}}}"#
			),
		)
		.unwrap();
	}

	fn environment_warnings(root: &Path) -> Vec<String> {
		validate_projects(root)
			.unwrap()
			.issues
			.into_iter()
			.filter(|issue| issue.message.contains("environment"))
			.map(|issue| issue.message)
			.collect()
	}

	#[test]
	fn flags_client_mods_in_a_server_pack_without_failing_validation() {
		let root = tempfile::tempdir().unwrap();
		let project = create(root.path(), "server-pack");
		let mut manifest = project.manifest.clone();
		manifest.modrinth_id = Some("server-pack".to_owned());
		manifest.environment = Some("server".to_owned());
		packwand_workspace::write_manifest(&project.root, &manifest).unwrap();
		write_mod(&project, "1.21.1-mr", "asynclogger", "client");
		write_mod(&project, "1.21.1-mr", "lithium", "both");
		write_mod(&project, "1.21.1-mr", "spark", "server");

		let warnings = environment_warnings(root.path());
		assert_eq!(warnings.len(), 1, "{warnings:?}");
		assert!(warnings[0].starts_with("asynclogger is side \"client\""));
		// Provider metadata is unreliable, so this must not fail the gate.
		assert!(validate_projects(root.path()).unwrap().valid());
	}

	#[test]
	fn environment_exempt_silences_a_mislabelled_slug() {
		let root = tempfile::tempdir().unwrap();
		let project = create(root.path(), "server-pack");
		let mut manifest = project.manifest.clone();
		manifest.modrinth_id = Some("server-pack".to_owned());
		manifest.environment = Some("server".to_owned());
		manifest.automation = Some(packwand_workspace::Automation {
			environment_exempt: vec!["asynclogger".to_owned()],
			..packwand_workspace::Automation::default()
		});
		packwand_workspace::write_manifest(&project.root, &manifest).unwrap();
		write_mod(&project, "1.21.1-mr", "asynclogger", "client");

		assert!(environment_warnings(root.path()).is_empty());
	}

	#[test]
	fn undeclared_environment_accepts_every_side() {
		let root = tempfile::tempdir().unwrap();
		let project = create(root.path(), "example");
		let mut manifest = project.manifest.clone();
		manifest.modrinth_id = Some("example".to_owned());
		packwand_workspace::write_manifest(&project.root, &manifest).unwrap();
		write_mod(&project, "1.21.1-mr", "sodium", "client");
		write_mod(&project, "1.21.1-mr", "spark", "server");

		assert!(environment_warnings(root.path()).is_empty());
	}

	#[test]
	fn a_misspelled_environment_is_an_error_not_a_silent_no_op() {
		let root = tempfile::tempdir().unwrap();
		let project = create(root.path(), "example");
		let mut manifest = project.manifest.clone();
		manifest.modrinth_id = Some("example".to_owned());
		manifest.environment = Some("serverside".to_owned());
		packwand_workspace::write_manifest(&project.root, &manifest).unwrap();

		let report = validate_projects(root.path()).unwrap();
		assert!(!report.valid());
		assert!(
			report
				.issues
				.iter()
				.any(|issue| issue.message.contains("invalid environment: serverside")),
			"{:?}",
			report.issues
		);
	}

	/// Writes a schema that rejects unknown properties, which the typed
	/// manifest struct silently tolerates.
	fn write_schema(root: &Path) {
		let schema = serde_json::json!({
			"$schema": "http://json-schema.org/draft-07/schema#",
			"type": "object",
			"required": ["id", "type"],
			"properties": {
				"id": { "type": "string" },
				"type": { "enum": ["mod", "modpack", "datapack", "resourcepack"] },
			},
			"additionalProperties": false,
		});
		let scripts = root.join("scripts");
		fs::create_dir_all(&scripts).unwrap();
		fs::write(
			scripts.join("schema.json"),
			serde_json::to_vec_pretty(&schema).unwrap(),
		)
		.unwrap();
	}

	#[test]
	fn schema_violations_are_reported() {
		let root = tempfile::tempdir().unwrap();
		let project = create(root.path(), "example");
		let mut manifest = project.manifest;
		manifest.modrinth_id = Some("example".to_owned());
		packwand_workspace::write_manifest(&project.root, &manifest).unwrap();
		write_schema(root.path());

		let report = validate_projects(root.path()).unwrap();
		let schema_issues = report
			.issues
			.iter()
			.filter(|issue| issue.message.starts_with("schema:"))
			.count();
		assert!(
			schema_issues > 0,
			"expected the strict schema to reject the scaffold, got {:?}",
			report.issues
		);
	}

	#[test]
	fn a_missing_schema_is_not_a_failure() {
		// No scripts/schema.json is written here.
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
			mr.join("ferrite-core.pw.json"),
			metadata("FerriteCore (Fabric)", "ferrite.jar"),
		)
		.unwrap();
		fs::write(
			cf.join("ferritecore.pw.json"),
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
