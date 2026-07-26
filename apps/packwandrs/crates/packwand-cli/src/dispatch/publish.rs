//! The `export` and `publish` command groups.

use super::*;

pub(super) fn export_local(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let filter = args.get_one::<String>("pack-name").map(String::as_str);
    if root.join("pack.toml").is_file() {
        let format = if root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("-cf"))
        {
            ExportFormat::CurseForge
        } else {
            ExportFormat::Modrinth
        };
        let artifact = export_pack(&root, format, None::<&Path>, ExportOptions::default())?;
        println!("exported {}", artifact.path.display());
        return Ok(());
    }
    let projects = packwand_workspace::discover(&root)?;
    let mut built = 0usize;
    for project in projects.into_iter().filter(|project| {
        filter.is_none_or(|value| {
            project.manifest.id == value
                || project
                    .root
                    .to_string_lossy()
                    .replace('\\', "/")
                    .ends_with(value)
        })
    }) {
        let output_dir = project.root.join("build");
        fs::create_dir_all(&output_dir)?;
        for subdir in project.subdirs {
            let name = subdir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("pack");
            let format = if name.ends_with("-cf") {
                ExportFormat::CurseForge
            } else {
                ExportFormat::Modrinth
            };
            let output = output_dir.join(format!(
                "{}-{}.{}",
                project.manifest.id,
                name,
                format.extension()
            ));
            let artifact = export_pack(&subdir, format, Some(&output), ExportOptions::default())?;
            println!("exported {}", artifact.path.display());
            built += 1;
        }
    }
    if built == 0 {
        Err("no exportable pack variants matched".into())
    } else {
        println!("built {built} archive(s)");
        Ok(())
    }
}

pub(super) fn publish_command(args: &ArgMatches) -> Result {
    match args.subcommand() {
        Some(("list", sub)) => {
            let manifests = strings(sub, "manifests")
                .into_iter()
                .map(absolute)
                .collect::<Result<Vec<_>>>()?;
            if manifests.is_empty() {
                return Err("publish list requires manifest path(s)".into());
            }
            println!(
                "{}",
                serde_json::to_string(
                    &packwand_build::list_publish_targets(manifests)
                        .map_err(|error| error.to_string())?,
                )?
            );
            Ok(())
        }
        Some(("build", sub)) => {
            let manifest = absolute(required(sub, "manifest")?)?;
            let target = packwand_build::build_publish_target(
                manifest,
                sub.get_one::<String>("variant").map(String::as_str),
            )
            .map_err(|error| error.to_string())?;
            println!("{}", serde_json::to_string_pretty(&target)?);
            Ok(())
        }
        Some(("upload", sub)) => {
            let manifest = absolute(required(sub, "manifest")?)?;
            let changelog = sub
                .get_one::<String>("changelog-file")
                .map(absolute)
                .transpose()?;
            let report = packwand_build::upload_publish_target(
                manifest,
                sub.get_one::<String>("variant").map(String::as_str),
                sub.get_flag("live"),
                changelog.as_deref(),
            )
            .map_err(|error| error.to_string())?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        Some(("verify", sub)) => {
            let manifest = absolute(required(sub, "manifest")?)?;
            let found = packwand_build::verify_publish_target(
                manifest,
                sub.get_one::<String>("variant").map(String::as_str),
                8,
                std::time::Duration::from_secs(15),
            )
            .map_err(|error| error.to_string())?;
            if found {
                println!("publish target is live");
                Ok(())
            } else {
                Err("publish target was not found after verification retries".into())
            }
        }
        Some(("plan", sub)) => {
            let root = std::env::current_dir()?;
            let from = sub
                .get_one::<String>("from")
                .map(String::as_str)
                .unwrap_or("HEAD^");
            let to = required(sub, "to")?;
            let output = std::process::Command::new("git")
                .args(["diff", "--name-only", &format!("{from}..{to}")])
                .current_dir(&root)
                .output()?;
            if !output.status.success() {
                return Err(format!(
                    "git diff failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                )
                .into());
            }
            let pack_filter = sub.get_one::<String>("pack").map(String::as_str);
            let changed = String::from_utf8(output.stdout)?
                .lines()
                .filter_map(|line| {
                    let mut parts = line.split('/');
                    Some((parts.next()?.to_owned(), parts.next()?.to_owned()))
                })
                .collect::<std::collections::BTreeSet<_>>();
            let candidates = packwand_workspace::discover(&root)?
                .into_iter()
                .filter(|project| {
                    pack_filter.is_none_or(|wanted| {
                        project.manifest.id == wanted
                            || project.root.file_name().is_some_and(|name| name == wanted)
                    })
                })
                .filter(|project| {
                    let directory = project
                        .root
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    pack_filter.is_some()
                        || changed.contains(&(project.category.clone(), directory))
                })
                .collect::<Vec<_>>();

            // A manifest that fails validation is dropped from the plan and
            // reported, rather than aborting the run: one bad manifest
            // elsewhere in the workspace must not stop everything else from
            // being publishable.
            let mut invalid: Vec<(PathBuf, usize)> = Vec::new();
            let mut manifests = Vec::new();
            let issues = if sub.get_flag("no-validate") {
                Vec::new()
            } else {
                packwand_diagnostics::validate_projects(&root)?.issues
            };
            for project in candidates {
                let count = issues
                    .iter()
                    .filter(|issue| {
                        matches!(issue.severity, packwand_diagnostics::Severity::Error)
                            && issue.path.starts_with(&project.root)
                    })
                    .count();
                if count > 0 {
                    invalid.push((project.root.clone(), count));
                } else {
                    manifests.push(project.root.join("manifest.json"));
                }
            }

            let plan = packwand_build::list_publish_targets(manifests)
                .map_err(|error| error.to_string())?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            for (project_root, count) in &invalid {
                eprintln!(
                    "plan: INVALID {} — {count} validation issue(s)",
                    slash_display(project_root, &root)
                );
            }
            if invalid.is_empty() {
                Ok(())
            } else {
                Err(format!("plan: {} manifest(s) failed validation", invalid.len()).into())
            }
        }
        _ => Err("publish requires plan, list, build, upload, or verify".into()),
    }
}
