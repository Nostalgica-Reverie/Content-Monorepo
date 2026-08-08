//! The `diff` command: mod changes between two git refs.

use super::*;

#[derive(Default, Serialize)]
struct DiffResult {
    old_ref: String,
    new_ref: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    path_prefix: String,
    subdirs: Vec<DiffSubdir>,
    total_added: usize,
    total_removed: usize,
    total_updated: usize,
}

#[derive(Serialize)]
struct DiffSubdir {
    subdir: String,
    added: usize,
    removed: usize,
    updated: usize,
    mods: Vec<DiffMod>,
}

#[derive(Serialize)]
struct DiffMod {
    slug: String,
    path: String,
    change: &'static str,
    #[serde(skip_serializing_if = "String::is_empty")]
    old_filename: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    new_filename: String,
}

pub(super) fn diff_command(args: &ArgMatches) -> Result {
    let old_ref = required(args, "old-ref")?;
    let new_ref = required(args, "new-ref")?;
    let path_prefix = args
        .get_one::<String>("path-prefix")
        .cloned()
        .unwrap_or_default();
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", &format!("{old_ref}..{new_ref}")])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    let mut changed = String::from_utf8(output.stdout)?
        .lines()
        .map(str::trim)
        .filter(|path| packwand_pack::metafile::is_metafile(path))
        .filter(|path| path_prefix.is_empty() || path.starts_with(&path_prefix))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    changed.sort();
    let mut grouped = std::collections::BTreeMap::<String, Vec<String>>::new();
    for path in changed {
        let subdir = Path::new(&path)
            .parent()
            .and_then(Path::parent)
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        grouped.entry(subdir).or_default().push(path);
    }
    let mut result = DiffResult {
        old_ref: old_ref.into(),
        new_ref: new_ref.into(),
        path_prefix,
        ..DiffResult::default()
    };
    for (subdir, paths) in grouped {
        let mut summary = DiffSubdir {
            subdir,
            added: 0,
            removed: 0,
            updated: 0,
            mods: Vec::new(),
        };
        for path in paths {
            let old = git_show(old_ref, &path)?;
            let new = git_show(new_ref, &path)?;
            let old_filename = old.as_deref().map(metadata_filename).unwrap_or_default();
            let new_filename = new.as_deref().map(metadata_filename).unwrap_or_default();
            let change = match (old.is_some(), new.is_some()) {
                (false, true) => {
                    summary.added += 1;
                    result.total_added += 1;
                    "added"
                }
                (true, false) => {
                    summary.removed += 1;
                    result.total_removed += 1;
                    "removed"
                }
                _ => {
                    summary.updated += 1;
                    result.total_updated += 1;
                    "updated"
                }
            };
            summary.mods.push(DiffMod {
                slug: Path::new(&path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(&path)
                    .trim_end_matches(".pw.json")
                    .into(),
                path,
                change,
                old_filename,
                new_filename,
            });
        }
        result.subdirs.push(summary);
    }
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if result.subdirs.is_empty() {
        println!("no .pw.json changes between {old_ref} and {new_ref}");
    } else {
        for subdir in &result.subdirs {
            println!("{}:", subdir.subdir);
            for change in &subdir.mods {
                match change.change {
                    "added" => println!("  + {:38} {}", change.slug, change.new_filename),
                    "removed" => println!("  - {:38} {}", change.slug, change.old_filename),
                    _ if change.old_filename != change.new_filename => println!(
                        "  ~ {:38} {} -> {}",
                        change.slug, change.old_filename, change.new_filename
                    ),
                    _ => println!("  ~ {}", change.slug),
                }
            }
            println!(
                "  +{} -{} ~{}\n",
                subdir.added, subdir.removed, subdir.updated
            );
        }
        println!(
            "{old_ref}..{new_ref}: +{} added  -{} removed  ~{} updated",
            result.total_added, result.total_removed, result.total_updated
        );
    }
    Ok(())
}
