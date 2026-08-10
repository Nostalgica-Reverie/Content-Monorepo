use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use packwand_diagnostics::{ContentRegistry, Issue, Severity, ValidationReport};
use serde::Serialize;
use tauri::State;
use walkdir::{DirEntry, WalkDir};

use crate::commands::off_thread;
use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::fsutil::safe_join;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionAsset {
	pub path: String,
	pub name: String,
	pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeAsset {
	pub path: String,
	pub name: String,
	pub kind: String,
	pub namespace: String,
	pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldgenAsset {
	pub path: String,
	pub name: String,
	pub kind: String,
	pub namespace: String,
	pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageFile {
	pub locale: String,
	pub namespace: String,
	pub path: String,
	pub keys: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageGap {
	pub locale: String,
	pub namespace: String,
	pub key: String,
	pub reference_locale: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageSnapshot {
	pub files: Vec<LanguageFile>,
	pub gaps: Vec<LanguageGap>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackGraphNode {
	pub id: String,
	pub name: String,
	pub path: String,
	pub kind: String,
	pub provider: String,
	pub side: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackGraphEdge {
	pub from: String,
	pub to: String,
	pub relation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackGraphSnapshot {
	pub nodes: Vec<PackGraphNode>,
	pub edges: Vec<PackGraphEdge>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorDiagnostic {
	pub path: String,
	pub severity: String,
	pub message: String,
	pub start_line: usize,
	pub start_column: usize,
	pub end_line: usize,
	pub end_column: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSymbol {
	pub id: String,
	pub kind: String,
	pub registry: String,
	pub path: String,
	pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorLanguageSnapshot {
	pub version: String,
	pub diagnostics: Vec<EditorDiagnostic>,
	pub symbols: Vec<EditorSymbol>,
}

fn should_descend(entry: &DirEntry) -> bool {
	entry.depth() == 0
		|| !entry.file_type().is_dir()
		|| !matches!(
			entry.file_name().to_str(),
			Some(".git" | "target" | "node_modules" | "build")
		)
}

fn relative(root: &Path, path: &Path) -> String {
	path.strip_prefix(root)
		.unwrap_or(path)
		.to_string_lossy()
		.replace('\\', "/")
}

fn assets_matching(
	root: &Path,
	kind: &str,
	predicate: impl Fn(&Path) -> bool,
) -> CommandResult<Vec<ExtensionAsset>> {
	let mut assets = Vec::new();
	for entry in WalkDir::new(root)
		.follow_links(false)
		.into_iter()
		.filter_entry(should_descend)
	{
		let entry =
			entry.map_err(|error| SerializableError::new("asset_scan", error.to_string()))?;
		if !entry.file_type().is_file() || !predicate(entry.path()) {
			continue;
		}
		assets.push(ExtensionAsset {
			path: relative(root, entry.path()),
			name: entry.file_name().to_string_lossy().into_owned(),
			kind: kind.into(),
		});
	}
	assets.sort_by(|left, right| left.path.cmp(&right.path));
	Ok(assets)
}

fn extension(path: &Path) -> String {
	path.extension()
		.map(|value| value.to_string_lossy().to_ascii_lowercase())
		.unwrap_or_default()
}

#[tauri::command]
pub async fn extension_kubejs_scripts(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<Vec<ExtensionAsset>> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || {
		assets_matching(&root, "kubejs-script", |path| {
			let relative = relative(&root, path);
			relative
				.split('/')
				.any(|part| part.eq_ignore_ascii_case("kubejs"))
				&& matches!(extension(path).as_str(), "js" | "ts")
		})
	})
	.await
}

#[tauri::command]
pub async fn extension_kubejs_validate(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<ValidationReport> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || {
		let scripts = assets_matching(&root, "kubejs-script", |path| {
			let relative = relative(&root, path);
			relative
				.split('/')
				.any(|part| part.eq_ignore_ascii_case("kubejs"))
				&& matches!(extension(path).as_str(), "js" | "ts")
		})?;
		let mut report = ValidationReport {
			checked: scripts.len(),
			issues: Vec::new(),
		};
		for script in scripts {
			let path = safe_join(&root, &script.path)?;
			match fs::read_to_string(&path) {
				Ok(source) => validate_delimiters(&path, &source, &mut report.issues),
				Err(error) => report.issues.push(Issue {
					severity: Severity::Error,
					path,
					message: format!("could not read KubeJS script: {error}"),
				}),
			}
		}
		Ok(report)
	})
	.await
}

fn data_asset_parts(relative_path: &str) -> Option<(String, String, String)> {
	let parts = relative_path.split('/').collect::<Vec<_>>();
	let data = parts.iter().position(|part| *part == "data")?;
	let namespace = parts.get(data + 1)?.to_string();
	let category = parts.get(data + 2)?.to_string();
	let tail = parts.get(data + 3..)?.join("/");
	Some((namespace, category, tail))
}

#[tauri::command]
pub async fn extension_recipes(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<Vec<RecipeAsset>> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || {
		let mut recipes = Vec::new();
		for asset in assets_matching(&root, "recipe", |path| extension(path) == "json")? {
			let Some((namespace, category, tail)) = data_asset_parts(&asset.path) else {
				continue;
			};
			if !matches!(category.as_str(), "recipe" | "recipes") {
				continue;
			}
			let recipe_id = tail.strip_suffix(".json").unwrap_or(&tail);
			recipes.push(RecipeAsset {
				path: asset.path,
				name: asset.name,
				kind: "recipe".into(),
				namespace: namespace.clone(),
				id: format!("{namespace}:{recipe_id}"),
			});
		}
		recipes.sort_by(|left, right| left.id.cmp(&right.id));
		Ok(recipes)
	})
	.await
}

#[tauri::command]
pub async fn extension_worldgen_assets(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<Vec<WorldgenAsset>> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || {
		let mut assets = Vec::new();
		for asset in assets_matching(&root, "worldgen", |path| extension(path) == "json")? {
			let parts = asset.path.split('/').collect::<Vec<_>>();
			let Some(data) = parts.iter().position(|part| *part == "data") else {
				continue;
			};
			if parts.get(data + 2) != Some(&"worldgen") || parts.len() <= data + 4 {
				continue;
			}
			let namespace = parts[data + 1];
			let kind = parts[data + 3];
			let tail = parts[data + 4..].join("/");
			let worldgen_id = tail.strip_suffix(".json").unwrap_or(&tail);
			assets.push(WorldgenAsset {
				path: asset.path.clone(),
				name: asset.name,
				kind: kind.into(),
				namespace: namespace.to_string(),
				id: format!("{namespace}:{worldgen_id}"),
			});
		}
		assets.sort_by(|left, right| (&left.kind, &left.id).cmp(&(&right.kind, &right.id)));
		Ok(assets)
	})
	.await
}

#[tauri::command]
pub async fn extension_language_files(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<LanguageSnapshot> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || {
		let mut files = Vec::new();
		let mut keysets: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
		for asset in assets_matching(&root, "language", |path| extension(path) == "json")? {
			let parts = asset.path.split('/').collect::<Vec<_>>();
			let Some(assets_index) = parts.iter().position(|part| *part == "assets") else {
				continue;
			};
			if parts.get(assets_index + 2) != Some(&"lang") || parts.len() != assets_index + 4 {
				continue;
			}
			let namespace = parts[assets_index + 1].to_owned();
			let locale = parts[assets_index + 3]
				.strip_suffix(".json")
				.unwrap_or(parts[assets_index + 3])
				.to_owned();
			let path = safe_join(&root, &asset.path)?;
			let document: serde_json::Value = serde_json::from_str(
				&fs::read_to_string(&path)
					.map_err(|error| SerializableError::new("language_read", error.to_string()))?,
			)
			.map_err(|error| {
				SerializableError::new("language_json", format!("{}: {error}", asset.path))
			})?;
			let keys = document
				.as_object()
				.map(|object| object.keys().cloned().collect::<BTreeSet<_>>())
				.unwrap_or_default();
			files.push(LanguageFile {
				locale: locale.clone(),
				namespace: namespace.clone(),
				path: asset.path,
				keys: keys.len(),
			});
			keysets.insert((namespace, locale), keys);
		}
		files.sort_by(|left, right| {
			(&left.namespace, &left.locale).cmp(&(&right.namespace, &right.locale))
		});

		let namespaces = files
			.iter()
			.map(|file| file.namespace.clone())
			.collect::<BTreeSet<_>>();
		let mut gaps = Vec::new();
		for namespace in namespaces {
			let locales = keysets
				.keys()
				.filter(|(candidate, _)| candidate == &namespace)
				.map(|(_, locale)| locale.clone())
				.collect::<Vec<_>>();
			let Some(reference_locale) = locales
				.iter()
				.find(|locale| locale.as_str() == "en_us")
				.or_else(|| locales.first())
				.cloned()
			else {
				continue;
			};
			let Some(reference_keys) = keysets.get(&(namespace.clone(), reference_locale.clone()))
			else {
				continue;
			};
			for locale in locales {
				let Some(keys) = keysets.get(&(namespace.clone(), locale.clone())) else {
					continue;
				};
				for key in reference_keys.difference(keys) {
					gaps.push(LanguageGap {
						locale: locale.clone(),
						namespace: namespace.clone(),
						key: key.clone(),
						reference_locale: reference_locale.clone(),
					});
				}
			}
		}
		Ok(LanguageSnapshot { files, gaps })
	})
	.await
}

fn toml_string(value: Option<&toml::Value>) -> Option<String> {
	match value {
		Some(toml::Value::String(value)) => Some(value.clone()),
		Some(toml::Value::Integer(value)) => Some(value.to_string()),
		_ => None,
	}
}

fn dependency_names(value: &toml::Value) -> Vec<String> {
	let Some(table) = value.get("dependencies").and_then(toml::Value::as_table) else {
		return Vec::new();
	};
	table.keys().cloned().collect()
}

#[tauri::command]
pub async fn extension_pack_graph(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<PackGraphSnapshot> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || {
		let metadata = assets_matching(&root, "package", |path| {
			path.file_name()
				.and_then(|name| name.to_str())
				.is_some_and(|_| packwand_pack::metafile::is_metafile(path))
		})?;
		let mut nodes = Vec::new();
		let mut dependencies: BTreeMap<String, Vec<String>> = BTreeMap::new();
		let mut aliases: BTreeMap<String, String> = BTreeMap::new();
		for asset in metadata {
			let path = safe_join(&root, &asset.path)?;
			let document = fs::read_to_string(&path)
				.map_err(|error| SerializableError::new("graph_read", error.to_string()))?
				.parse::<toml::Value>()
				.map_err(|error| {
					SerializableError::new("graph_toml", format!("{}: {error}", asset.path))
				})?;
			let name = toml_string(document.get("name")).unwrap_or_else(|| asset.name.clone());
			let side = toml_string(document.get("side")).unwrap_or_else(|| "both".into());
			let modrinth_id = document
				.get("update")
				.and_then(|value| value.get("modrinth"))
				.and_then(|value| toml_string(value.get("mod-id")));
			let curseforge_id = document
				.get("update")
				.and_then(|value| value.get("curseforge"))
				.and_then(|value| toml_string(value.get("project-id")));
			let (provider, provider_id) = if let Some(project) = modrinth_id {
				("modrinth".to_owned(), project)
			} else if let Some(project) = curseforge_id {
				("curseforge".to_owned(), project)
			} else {
				("direct".to_owned(), asset.path.clone())
			};
			let node_id = asset.path.clone();
			let kind = asset.path.split('/').next().unwrap_or("package").to_owned();
			aliases.insert(name.to_ascii_lowercase(), node_id.clone());
			aliases.insert(provider_id.to_ascii_lowercase(), node_id.clone());
			dependencies.insert(node_id.clone(), dependency_names(&document));
			nodes.push(PackGraphNode {
				id: node_id,
				name,
				path: asset.path,
				kind,
				provider,
				side,
			});
		}
		nodes.sort_by(|left, right| (&left.kind, &left.name).cmp(&(&right.kind, &right.name)));
		let mut edges = Vec::new();
		for (from, names) in dependencies {
			for name in names {
				if let Some(to) = aliases.get(&name.to_ascii_lowercase()) {
					edges.push(PackGraphEdge {
						from: from.clone(),
						to: to.clone(),
						relation: "requires".into(),
					});
				}
			}
		}
		edges.sort_by(|left, right| (&left.from, &left.to).cmp(&(&right.from, &right.to)));
		Ok(PackGraphSnapshot { nodes, edges })
	})
	.await
}
/// Produces the small, serializable language-service view used by Monaco.
/// The editor receives pack-relative facts, never Rust objects or fs handles.
#[tauri::command]
pub async fn extension_language_snapshot(
	id: String,
	enabled: Vec<String>,
	state: State<'_, AppState>,
) -> CommandResult<EditorLanguageSnapshot> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || {
		let datapack_enabled = enabled
			.iter()
			.any(|id| matches!(id.as_str(), "game-studio" | "worldgen-pw" | "datapack-pw"));
		let resourcepack_enabled = enabled
			.iter()
			.any(|id| matches!(id.as_str(), "game-studio" | "lang-pw" | "resourcepack-pw"));
		let kubejs_enabled = enabled.iter().any(|id| id == "packwand-js");
		let mut registries = packwand_diagnostics::build_all_registries(&root)
			.map_err(|error| SerializableError::new("registry", error.to_string()))?;
		registries.retain(|registry| match registry.kind {
			packwand_diagnostics::RegistryKind::Datapack => datapack_enabled,
			packwand_diagnostics::RegistryKind::Resourcepack => resourcepack_enabled,
			packwand_diagnostics::RegistryKind::Kubejs => kubejs_enabled,
			packwand_diagnostics::RegistryKind::Config => false,
		});
		let mut issues = if datapack_enabled || resourcepack_enabled {
			packwand_diagnostics::content_lint(&root)
				.issues
				.into_iter()
				.filter(|issue| {
					let path = relative(&root, &issue.path);
					(datapack_enabled && (path.contains("/data/") || path.starts_with("data/")))
						|| (resourcepack_enabled
							&& (path.contains("/assets/") || path.starts_with("assets/")))
						|| ((datapack_enabled || resourcepack_enabled)
							&& path.ends_with("pack.mcmeta"))
				})
				.collect()
		} else {
			Vec::new()
		};

		let scripts = if kubejs_enabled {
			assets_matching(&root, "kubejs-script", |path| {
				let relative = relative(&root, path);
				relative
					.split('/')
					.any(|part| part.eq_ignore_ascii_case("kubejs"))
					&& matches!(extension(path).as_str(), "js" | "ts")
			})?
		} else {
			Vec::new()
		};
		for script in scripts {
			let path = safe_join(&root, &script.path)?;
			match fs::read_to_string(&path) {
				Ok(source) => validate_delimiters(&path, &source, &mut issues),
				Err(error) => issues.push(Issue {
					severity: Severity::Error,
					path,
					message: format!("could not read KubeJS script: {error}"),
				}),
			}
		}

		let version = registries
			.iter()
			.map(|registry| registry.version.as_str())
			.collect::<Vec<_>>()
			.join(":");
		let mut symbols = registries
			.iter()
			.flat_map(registry_symbols)
			.collect::<Vec<_>>();
		symbols.sort_by(|left, right| {
			(&left.id, &left.registry, &left.kind).cmp(&(&right.id, &right.registry, &right.kind))
		});

		let mut diagnostics = issues
			.into_iter()
			.filter_map(|issue| {
				let path = issue.path.strip_prefix(&root).ok()?;
				let line = line_from_message(&issue.message).unwrap_or(1);
				Some(EditorDiagnostic {
					path: relative(&root, &root.join(path)),
					severity: match issue.severity {
						Severity::Error => "error".into(),
						Severity::Warning => "warning".into(),
					},
					message: issue.message,
					start_line: line,
					start_column: 1,
					end_line: line,
					end_column: 1,
				})
			})
			.collect::<Vec<_>>();
		diagnostics.sort_by(|left, right| {
			(&left.path, left.start_line, &left.message).cmp(&(
				&right.path,
				right.start_line,
				&right.message,
			))
		});

		Ok(EditorLanguageSnapshot {
			version,
			diagnostics,
			symbols,
		})
	})
	.await
}

#[tauri::command]
pub async fn extension_content_lint(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<ValidationReport> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || Ok(packwand_diagnostics::content_lint(root))).await
}

#[tauri::command]
pub async fn extension_registries(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<Vec<ContentRegistry>> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || {
		packwand_diagnostics::build_all_registries(root)
			.map_err(|error| SerializableError::new("registry", error.to_string()))
	})
	.await
}

fn registry_symbols(registry: &ContentRegistry) -> impl Iterator<Item = EditorSymbol> + '_ {
	registry.entries.iter().map(|entry| {
		let path = match (entry.origin.as_str(), entry.path.as_str()) {
			(_, "") => String::new(),
			("" | ".", path) => path.to_owned(),
			(origin, path) => format!("{origin}/{path}"),
		};
		EditorSymbol {
			id: entry.id.clone(),
			kind: entry.kind.clone(),
			registry: registry.kind.to_string(),
			path,
			detail: if entry.owner.is_empty() {
				format!("{} · {}", entry.kind, entry.origin)
			} else {
				format!("{} · owned by {}", entry.kind, entry.owner)
			},
		}
	})
}

fn line_from_message(message: &str) -> Option<usize> {
	let tail = message.split_once("line ")?.1;
	let digits = tail
		.chars()
		.take_while(char::is_ascii_digit)
		.collect::<String>();
	digits.parse().ok()
}

fn validate_delimiters(path: &Path, source: &str, issues: &mut Vec<Issue>) {
	let mut stack: Vec<(char, usize)> = Vec::new();
	let mut chars = source.chars().peekable();
	let mut line = 1usize;
	let mut quote = None;
	let mut escaped = false;
	let mut line_comment = false;
	let mut block_comment = false;

	while let Some(character) = chars.next() {
		if character == '\n' {
			line += 1;
			line_comment = false;
		}
		if line_comment {
			continue;
		}
		if block_comment {
			if character == '*' && chars.peek() == Some(&'/') {
				chars.next();
				block_comment = false;
			}
			continue;
		}
		if let Some(active) = quote {
			if escaped {
				escaped = false;
			} else if character == '\\' {
				escaped = true;
			} else if character == active {
				quote = None;
			}
			continue;
		}
		if character == '/' && chars.peek() == Some(&'/') {
			chars.next();
			line_comment = true;
			continue;
		}
		if character == '/' && chars.peek() == Some(&'*') {
			chars.next();
			block_comment = true;
			continue;
		}
		if matches!(character, '\'' | '"' | '`') {
			quote = Some(character);
			continue;
		}
		if matches!(character, '(' | '[' | '{') {
			stack.push((character, line));
		} else if matches!(character, ')' | ']' | '}') {
			let expected = match character {
				')' => '(',
				']' => '[',
				_ => '{',
			};
			match stack.pop() {
				Some((opening, _)) if opening == expected => {}
				_ => {
					issues.push(Issue {
						severity: Severity::Error,
						path: path.to_path_buf(),
						message: format!("unexpected '{character}' on line {line}"),
					});
					return;
				}
			}
		}
	}

	if let Some(active) = quote {
		issues.push(Issue {
			severity: Severity::Error,
			path: path.to_path_buf(),
			message: format!("unterminated {active} string"),
		});
	} else if block_comment {
		issues.push(Issue {
			severity: Severity::Error,
			path: path.to_path_buf(),
			message: "unterminated block comment".into(),
		});
	} else if let Some((opening, opening_line)) = stack.last() {
		issues.push(Issue {
			severity: Severity::Error,
			path: path.to_path_buf(),
			message: format!("unclosed '{opening}' opened on line {opening_line}"),
		});
	}
}

#[tauri::command]
pub async fn extension_krita_assets(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<Vec<ExtensionAsset>> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || {
		assets_matching(&root, "image", |path| {
			matches!(extension(path).as_str(), "png" | "jpg" | "jpeg" | "kra")
		})
	})
	.await
}

#[tauri::command]
pub async fn extension_blockbench_assets(
	id: String,
	state: State<'_, AppState>,
) -> CommandResult<Vec<ExtensionAsset>> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || {
		assets_matching(&root, "model", |path| {
			extension(path) == "bbmodel"
				|| (extension(path) == "json"
					&& path.components().any(|part| part.as_os_str() == "models"))
		})
	})
	.await
}

fn launch_approved(tool: &str, path: &Path) -> CommandResult<()> {
	let fallback = match tool {
		"krita" if cfg!(windows) => PathBuf::from("krita.exe"),
		"krita" => PathBuf::from("krita"),
		"blockbench" if cfg!(windows) => PathBuf::from("Blockbench.exe"),
		"blockbench" => PathBuf::from("blockbench"),
		_ => {
			return Err(SerializableError::new(
				"external_tool",
				"tool is not approved",
			));
		}
	};
	let mut candidates = Vec::new();
	if cfg!(windows) {
		if let Some(program_files) = std::env::var_os("ProgramFiles").map(PathBuf::from) {
			match tool {
				"krita" => {
					candidates.push(program_files.join("Krita (x64)/bin/krita.exe"));
					candidates.push(program_files.join("Krita/bin/krita.exe"));
				}
				"blockbench" => candidates.push(program_files.join("Blockbench/Blockbench.exe")),
				_ => {}
			}
		}
		if let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
			match tool {
				"krita" => candidates.push(local.join("Programs/Krita/bin/krita.exe")),
				"blockbench" => {
					candidates.push(local.join("Programs/blockbench/Blockbench.exe"));
				}
				_ => {}
			}
		}
	}
	let executable = candidates
		.into_iter()
		.find(|candidate| candidate.is_file())
		.unwrap_or(fallback);
	Command::new(executable)
		.arg(path)
		.spawn()
		.map(|_| ())
		.map_err(|error| SerializableError::new(format!("external_{tool}"), error.to_string()))
}

/// Takes the resolved pack root rather than `State`, so the whole check-and-
/// launch can be moved onto a blocking thread — `State<'_, AppState>` is not
/// `'static` and cannot cross that boundary.
fn open_asset(tool: &str, root: &Path, relative_path: &str, allowed: &[&str]) -> CommandResult<()> {
	let path = safe_join(root, relative_path)?;
	if !path.is_file() || !allowed.contains(&extension(&path).as_str()) {
		return Err(SerializableError::new(
			"external_asset",
			"the selected file is not supported by this integration",
		));
	}
	launch_approved(tool, &path)
}

#[tauri::command]
pub async fn extension_krita_open(
	id: String,
	path: String,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	let root = pack_root(&state.workspace()?, &id)?;
	// `launch_approved` probes several install locations and then spawns the
	// process; both are blocking.
	off_thread(move || open_asset("krita", &root, &path, &["png", "jpg", "jpeg", "kra"])).await
}

#[tauri::command]
pub async fn extension_blockbench_open(
	id: String,
	path: String,
	state: State<'_, AppState>,
) -> CommandResult<()> {
	let root = pack_root(&state.workspace()?, &id)?;
	off_thread(move || open_asset("blockbench", &root, &path, &["json", "bbmodel"])).await
}

#[cfg(test)]
mod tests {
	use packwand_diagnostics::Issue;

	use super::validate_delimiters;

	#[test]
	fn kubejs_delimiter_check_ignores_strings_and_comments() {
		let mut issues: Vec<Issue> = Vec::new();
		validate_delimiters(
			std::path::Path::new("server_scripts/test.js"),
			"// }\nServerEvents.recipes(event => { event.remove({ output: 'x' }) })",
			&mut issues,
		);
		assert!(issues.is_empty());
	}

	#[test]
	fn kubejs_delimiter_check_reports_unclosed_blocks() {
		let mut issues: Vec<Issue> = Vec::new();
		validate_delimiters(
			std::path::Path::new("server_scripts/test.js"),
			"ServerEvents.recipes(event => {",
			&mut issues,
		);
		assert_eq!(issues.len(), 1);
	}
}
