//! The `cache` command group: reading the shared download cache index
//! and pruning entries no pack references.

use super::*;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct CacheIndex {
	version: u32,
	hashes: std::collections::BTreeMap<String, Vec<String>>,
}

#[derive(Serialize)]
struct CachePruneEntry {
	hash: String,
	size_bytes: u64,
}

#[derive(Serialize)]
struct CachePruneResult {
	scanned_entries: usize,
	removed_entries: Vec<CachePruneEntry>,
	removed_bytes: u64,
	dry_run: bool,
}

pub(super) fn cache_command(args: &ArgMatches, root_args: &ArgMatches) -> Result {
	let Some(("prune", sub)) = args.subcommand() else {
		return Err("cache requires prune".into());
	};
	let cache = root_args
		.get_one::<String>("cache")
		.map(PathBuf::from)
		.unwrap_or_else(default_cache_path);
	let index_path = cache.join("index.json");
	let mut index = match fs::read(&index_path) {
		Ok(bytes) => serde_json::from_slice::<CacheIndex>(&bytes)?,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => CacheIndex {
			version: 2,
			..CacheIndex::default()
		},
		Err(error) => return Err(error.into()),
	};
	if index.version > 2 {
		return Err(format!(
			"cache index version {} is newer than supported version 2",
			index.version
		)
		.into());
	}
	let referenced = referenced_download_hashes(std::env::current_dir()?)?;
	let sha256 = index.hashes.get("sha256").cloned().unwrap_or_default();
	let mut removals = Vec::new();
	let mut remove_indices = Vec::new();
	for (position, hash) in sha256.iter().enumerate() {
		let used = index.hashes.values().any(|hashes| {
			hashes
				.get(position)
				.is_some_and(|candidate| referenced.contains(&candidate.to_ascii_lowercase()))
		});
		if used || hash.is_empty() {
			continue;
		}
		if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
			return Err(format!("unsafe sha256 cache key {hash:?}").into());
		}
		let path = cache.join(&hash[..2]).join(&hash[2..]);
		let size = fs::metadata(&path)
			.map(|metadata| metadata.len())
			.unwrap_or(0);
		removals.push(CachePruneEntry {
			hash: hash.clone(),
			size_bytes: size,
		});
		remove_indices.push(position);
		if !sub.get_flag("dry-run") {
			match fs::remove_file(&path) {
				Ok(()) => {}
				Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
				Err(error) => {
					return Err(format!("failed to remove {}: {error}", path.display()).into());
				}
			}
		}
	}
	let removed_bytes = removals.iter().map(|entry| entry.size_bytes).sum();
	let result = CachePruneResult {
		scanned_entries: sha256.len(),
		removed_entries: removals,
		removed_bytes,
		dry_run: sub.get_flag("dry-run"),
	};
	if !result.dry_run && !remove_indices.is_empty() {
		for hashes in index.hashes.values_mut() {
			let mut position = 0usize;
			hashes.retain(|_| {
				let keep = !remove_indices.contains(&position);
				position += 1;
				keep
			});
		}
		fs::create_dir_all(&cache)?;
		let mut bytes = serde_json::to_vec(&index)?;
		bytes.push(b'\n');
		let mut temporary = tempfile::NamedTempFile::new_in(&cache)?;
		use std::io::Write as _;
		temporary.write_all(&bytes)?;
		temporary.persist(&index_path)?;
	}
	if sub.get_flag("json") {
		println!("{}", serde_json::to_string_pretty(&result)?);
	} else {
		println!(
			"{} {}/{} cache entries ({:.1} MB)",
			if result.dry_run {
				"would remove"
			} else {
				"removed"
			},
			result.removed_entries.len(),
			result.scanned_entries,
			result.removed_bytes as f64 / 1_000_000.0
		);
	}
	Ok(())
}

fn default_cache_path() -> PathBuf {
	if let Some(path) = std::env::var_os("PACKWAND_CACHE") {
		return path.into();
	}
	#[cfg(windows)]
	if let Some(path) = std::env::var_os("LOCALAPPDATA") {
		return PathBuf::from(path).join("packwand/cache");
	}
	if let Some(path) = std::env::var_os("XDG_CACHE_HOME") {
		return PathBuf::from(path).join("packwand");
	}
	std::env::var_os("HOME")
		.map(PathBuf::from)
		.unwrap_or_else(|| PathBuf::from("."))
		.join(".cache/packwand")
}

fn referenced_download_hashes(root: PathBuf) -> Result<std::collections::BTreeSet<String>> {
	let mut hashes = std::collections::BTreeSet::new();
	for entry in walkdir::WalkDir::new(root)
		.follow_links(false)
		.into_iter()
		.filter_entry(|entry| {
			entry.depth() == 0
				|| !entry.file_type().is_dir()
				|| !matches!(
					entry.file_name().to_str(),
					Some(".git" | "target" | "node_modules")
				)
		}) {
		let entry = entry?;
		if !entry.file_type().is_file() || !packwand_pack::metafile::is_metafile(entry.path()) {
			continue;
		}
		let metadata: Mod = serde_json::from_str(&fs::read_to_string(entry.path())?)?;
		if !metadata.download.hash.is_empty() {
			hashes.insert(metadata.download.hash.to_ascii_lowercase());
		}
		hashes.extend(
			metadata
				.download
				.extra_hashes
				.values()
				.filter(|hash| !hash.is_empty())
				.map(|hash| hash.to_ascii_lowercase()),
		);
	}
	Ok(hashes)
}
