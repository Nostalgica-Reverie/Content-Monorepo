//! The `json` and `modlist` commands: minification and the
//! crash-assistant mod list.

use super::*;

pub(super) fn json(args: &ArgMatches) -> Result {
    let Some(("minify", sub)) = args.subcommand() else {
        return Err("json requires minify".into());
    };
    let check = sub.get_flag("check");
    let strict = sub.get_flag("strict");
    let mut files = Vec::new();
    for raw in strings(sub, "paths") {
        let path = absolute(raw)?;
        if path.is_dir() {
            files.extend(
                walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_entry(|entry| {
                        entry.depth() == 0
                            || !entry.file_type().is_dir()
                            || !matches!(entry.file_name().to_str(), Some(".git" | "node_modules"))
                    })
                    .flatten()
                    .filter(|entry| entry.file_type().is_file() && is_json_path(entry.path()))
                    .map(|entry| entry.into_path()),
            );
        } else if is_json_path(&path) {
            files.push(path);
        }
    }
    let mut changed = 0;
    let mut skipped = 0;
    let mut saved = 0usize;
    for path in &files {
        let source = fs::read(path)?;
        if let Err(error) = serde_json::from_slice::<serde_json::Value>(&source) {
            if strict {
                return Err(format!("{} is not valid JSON: {error}", path.display()).into());
            }
            skipped += 1;
            continue;
        }
        let compact = compact_json(&source);
        if compact.len() < source.len() {
            changed += 1;
            saved += source.len() - compact.len();
            if !check {
                fs::write(path, compact)?;
            }
        }
    }
    println!(
        "{} {changed} of {} JSON file(s), saving {saved} bytes; {skipped} skipped",
        if check { "would minify" } else { "minified" },
        files.len()
    );
    if check && changed > 0 {
        Err(format!("{changed} file(s) require minification").into())
    } else {
        Ok(())
    }
}

fn is_json_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext, "json" | "mcmeta"))
}

fn compact_json(source: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(source.len());
    let mut quoted = false;
    let mut escaped = false;
    for &byte in source {
        if quoted {
            output.push(byte);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else if byte == b'"' {
            quoted = true;
            output.push(byte);
        } else if !byte.is_ascii_whitespace() {
            output.push(byte);
        }
    }
    output
}

pub(super) fn modlist(args: &ArgMatches) -> Result {
    let subdir = absolute(required(args, "subdir")?)?;
    let mods_dir = subdir.join("mods");
    if !mods_dir.is_dir() {
        return Err(format!("no mods directory at {}", mods_dir.display()).into());
    }
    let mut entries = std::collections::BTreeMap::new();
    let mut parsed = 0;
    for entry in fs::read_dir(&mods_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file()
            || !entry.file_name().to_string_lossy().ends_with(".pw.toml")
        {
            continue;
        }
        let metadata: Mod = toml::from_str(&fs::read_to_string(entry.path())?)?;
        parsed += 1;
        let modrinth_id = metadata
            .update
            .get("modrinth")
            .and_then(|table| table.get("mod-id"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let cf_file = metadata
            .update
            .get("curseforge")
            .and_then(|table| table.get("file-id"))
            .and_then(toml::Value::as_integer);
        let mr_hash = (modrinth_id.is_some()
            && metadata.download.hash_format == "sha1"
            && !metadata.download.hash.is_empty())
        .then(|| metadata.download.hash.clone());
        entries.insert(
            metadata.filename.clone(),
            CrashMod {
                jar_name: metadata.filename.clone(),
                mod_id: modrinth_id,
                name: metadata.name,
                version: metadata.filename.trim_end_matches(".jar").to_owned(),
                curse_forge_hash: cf_file,
                modrinth_hash: mr_hash,
            },
        );
    }
    let output_dir = subdir.join("config/crash_assistant");
    fs::create_dir_all(&output_dir)?;
    let output = output_dir.join("modlist.json");
    let mut data = serde_json::to_vec_pretty(&entries)?;
    data.push(b'\n');
    fs::write(&output, data)?;
    if args.get_flag("json") {
        println!(
            "{}",
            serde_json::json!({"subdir":subdir,"out_path":output,"mod_count":parsed})
        );
    } else {
        println!("wrote {} ({parsed} mods)", output.display());
    }
    Ok(())
}
