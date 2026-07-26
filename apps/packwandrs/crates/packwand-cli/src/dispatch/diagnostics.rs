//! Diagnostics commands: lint, content-lint, preflight, registry,
//! ci-local, validate, and parity.

use super::*;

pub(super) fn lint(args: &ArgMatches) -> Result {
    let files = strings(args, "files");
    let report = if files.is_empty() {
        // No arguments lints what the HEAD commit touched, not the whole
        // repository.
        packwand_diagnostics::lint_changed(std::env::current_dir()?)
    } else {
        let mut report = packwand_diagnostics::ValidationReport::default();
        for file in files {
            report.checked += 1;
            report.issues.extend(packwand_diagnostics::lint_file(file));
        }
        report
    };
    for issue in &report.issues {
        eprintln!("{}: {}", issue.path.display(), issue.message);
    }
    if report.checked == 0 {
        println!("no JSON or .pw.toml files to lint.");
        return Ok(());
    }
    if report.valid() {
        println!("{} file(s) lint clean", report.checked);
        Ok(())
    } else {
        Err(format!(
            "{} of {} file(s) failed lint",
            report.issues.len(),
            report.checked
        )
        .into())
    }
}

/// A datapack ships its content under `data/`. Each version directory is its
/// own content root, so the check runs per version rather than at the project
/// root, which only holds those directories.
fn datapack_content_roots(project_root: &Path) -> Vec<PathBuf> {
    // A pack.mcmeta at the top means the project is laid out flat rather than
    // one directory per version.
    if project_root.join("pack.mcmeta").is_file() {
        return vec![project_root.to_path_buf()];
    }
    let Ok(entries) = fs::read_dir(project_root) else {
        return Vec::new();
    };
    let mut roots: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir() && (path.join("pack.mcmeta").is_file() || path.join("data").is_dir())
        })
        .collect();
    roots.sort();
    roots
}

fn datapack_structure_issues(project_root: &Path) -> Vec<packwand_diagnostics::Issue> {
    let roots = datapack_content_roots(project_root);
    if roots.is_empty() {
        return vec![packwand_diagnostics::Issue {
            severity: packwand_diagnostics::Severity::Error,
            path: project_root.to_path_buf(),
            message: format!(
                "no content root found in {} (no pack.mcmeta at root or in a version directory)",
                project_root.display()
            ),
        }];
    }
    roots
        .into_iter()
        .filter(|root| !root.join("data").is_dir())
        .map(|root| packwand_diagnostics::Issue {
            severity: packwand_diagnostics::Severity::Error,
            path: root.clone(),
            message: format!("datapack has no data/ directory under {}", root.display()),
        })
        .collect()
}

pub(super) fn content_lint_command(args: &ArgMatches) -> Result {
    let current = std::env::current_dir()?;
    let mut roots = strings(args, "pack-dirs")
        .into_iter()
        .map(absolute)
        .collect::<Result<Vec<_>>>()?;
    if args.get_flag("all") {
        roots.extend(
            packwand_workspace::discover(&current)?
                .into_iter()
                .filter(|project| {
                    matches!(project.category.as_str(), "datapacks" | "resourcepacks")
                })
                .map(|project| project.root),
        );
    }
    if roots.is_empty() {
        roots.push(current.clone());
    }
    roots.sort();
    roots.dedup();
    // Which roots are datapacks is only knowable here, from the manifest
    // category — content_lint itself just sees a directory.
    let datapack_roots = packwand_workspace::discover(&current)
        .map(|projects| {
            projects
                .into_iter()
                .filter(|project| project.category == "datapacks")
                .map(|project| project.root)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut report = packwand_diagnostics::ValidationReport::default();
    for root in roots {
        let next = packwand_diagnostics::content_lint(&root);
        report.checked += next.checked;
        report.issues.extend(next.issues);
        if datapack_roots.contains(&root) {
            report.issues.extend(datapack_structure_issues(&root));
        }
    }
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for issue in &report.issues {
            eprintln!(
                "{}: {:?}: {}",
                issue.path.display(),
                issue.severity,
                issue.message
            );
        }
        println!(
            "checked {} content file(s); {} issue(s)",
            report.checked,
            report.issues.len()
        );
    }
    if report.valid() {
        Ok(())
    } else {
        Err(format!(
            "content lint found {} error(s)",
            report
                .issues
                .iter()
                .filter(|issue| matches!(issue.severity, packwand_diagnostics::Severity::Error))
                .count()
        )
        .into())
    }
}

#[derive(Debug, Serialize)]
struct PreflightIssue {
    level: &'static str,
    #[serde(skip_serializing_if = "String::is_empty")]
    path: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct PreflightStep {
    name: &'static str,
    errors: usize,
    warnings: usize,
    issues: Vec<PreflightIssue>,
}

#[derive(Debug, Serialize)]
struct PreflightResult {
    dir: String,
    steps: Vec<PreflightStep>,
    errors: usize,
    warnings: usize,
    ok: bool,
}

pub(super) fn preflight(args: &ArgMatches) -> Result {
    let root = args
        .get_one::<String>("dir")
        .map(absolute)
        .transpose()?
        .unwrap_or(std::env::current_dir()?);
    let report = run_preflight(&root);
    if args.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for step in &report.steps {
            println!(
                "{}: {} ({} error(s), {} warning(s))",
                step.name,
                if step.errors == 0 { "PASS" } else { "FAIL" },
                step.errors,
                step.warnings
            );
            for issue in &step.issues {
                let path = if issue.path.is_empty() {
                    String::new()
                } else {
                    format!("{}: ", issue.path)
                };
                eprintln!("  {}: {path}{}", issue.level, issue.message);
            }
        }
    }
    if report.ok {
        Ok(())
    } else {
        Err(format!("preflight failed with {} error(s)", report.errors).into())
    }
}

fn run_preflight(root: &Path) -> PreflightResult {
    let mut steps = Vec::new();

    let manifest_path = if root.join("manifest.json").is_file() {
        root.join("manifest.json")
    } else {
        root.parent()
            .map(|parent| parent.join("manifest.json"))
            .unwrap_or_else(|| root.join("manifest.json"))
    };
    let manifest_issues = match fs::read(&manifest_path) {
        Ok(bytes) => match serde_json::from_slice::<Manifest>(&bytes) {
            Ok(manifest) => {
                let mut issues = Vec::new();
                for (field, value) in [
                    ("id", manifest.id.as_str()),
                    ("name", manifest.name.as_str()),
                    ("type", manifest.project_type.as_str()),
                ] {
                    if value.trim().is_empty() {
                        issues.push(PreflightIssue {
                            level: "error",
                            path: manifest_path.to_string_lossy().into_owned(),
                            message: format!("manifest is missing required field {field:?}"),
                        });
                    }
                }
                if manifest.version.trim().is_empty() {
                    issues.push(PreflightIssue {
                        level: "warning",
                        path: manifest_path.to_string_lossy().into_owned(),
                        message: "manifest has no version".into(),
                    });
                }
                issues
            }
            Err(error) => vec![PreflightIssue {
                level: "error",
                path: manifest_path.to_string_lossy().into_owned(),
                message: error.to_string(),
            }],
        },
        Err(_) => vec![PreflightIssue {
            level: "error",
            path: manifest_path.to_string_lossy().into_owned(),
            message: "no manifest.json found in the pack directory or its parent".into(),
        }],
    };
    steps.push(preflight_step("manifest", manifest_issues));

    let syntax = packwand_diagnostics::lint_workspace(root);
    steps.push(preflight_step(
        "syntax",
        syntax
            .issues
            .into_iter()
            .map(|issue| PreflightIssue {
                level: match issue.severity {
                    packwand_diagnostics::Severity::Error => "error",
                    packwand_diagnostics::Severity::Warning => "warning",
                },
                path: issue.path.to_string_lossy().into_owned(),
                message: issue.message,
            })
            .collect(),
    ));

    // Reference checks only: duplicate-file and charset hygiene belong to
    // `content-lint`, not to a pre-launch gate.
    let mut reference_issues = packwand_diagnostics::content_lint_with(root, false)
        .issues
        .into_iter()
        .map(|issue| PreflightIssue {
            level: match issue.severity {
                packwand_diagnostics::Severity::Error => "error",
                packwand_diagnostics::Severity::Warning => "warning",
            },
            path: issue.path.to_string_lossy().into_owned(),
            message: issue.message,
        })
        .collect::<Vec<_>>();
    if root.join("pack.toml").is_file() {
        match Workspace::open(root.to_path_buf()) {
            Ok(workspace) => {
                for entry in &workspace.index().files {
                    let path = root.join(entry.file.replace('/', std::path::MAIN_SEPARATOR_STR));
                    if !path.is_file() {
                        reference_issues.push(PreflightIssue {
                            level: "error",
                            path: entry.file.clone(),
                            message: "indexed path is missing".into(),
                        });
                    }
                }
            }
            Err(error) => reference_issues.push(PreflightIssue {
                level: "error",
                path: root.join("pack.toml").to_string_lossy().into_owned(),
                message: error.to_string(),
            }),
        }
    }
    match packwand_diagnostics::build_all_registries(root) {
        Ok(registries) => {
            // A config file no installed mod claims is usually left behind by
            // a mod that was removed: the pack ships dead weight, and anyone
            // editing that file is tuning a mod that is not there.
            for registry in &registries {
                if registry.kind != packwand_diagnostics::RegistryKind::Config {
                    continue;
                }
                for entry in &registry.entries {
                    if entry.kind == "config_file" && entry.owner.is_empty() {
                        reference_issues.push(PreflightIssue {
                            level: "warning",
                            path: if entry.origin.is_empty() {
                                entry.path.clone()
                            } else {
                                format!("{}/{}", entry.origin, entry.path)
                            },
                            message: "config is not associated with an installed mod".into(),
                        });
                    }
                }
            }
        }
        Err(error) => reference_issues.push(PreflightIssue {
            level: "error",
            path: root.to_string_lossy().into_owned(),
            message: format!("registry build failed: {error}"),
        }),
    }
    steps.push(preflight_step("references", reference_issues));

    let errors = steps.iter().map(|step| step.errors).sum();
    let warnings = steps.iter().map(|step| step.warnings).sum();
    PreflightResult {
        dir: root.to_string_lossy().replace('\\', "/"),
        steps,
        errors,
        warnings,
        ok: errors == 0,
    }
}

fn preflight_step(name: &'static str, issues: Vec<PreflightIssue>) -> PreflightStep {
    PreflightStep {
        name,
        errors: issues.iter().filter(|issue| issue.level == "error").count(),
        warnings: issues
            .iter()
            .filter(|issue| issue.level == "warning")
            .count(),
        issues,
    }
}

pub(super) fn registry_command(args: &ArgMatches) -> Result {
    use std::str::FromStr as _;

    let root = args
        .get_one::<String>("dir")
        .map(absolute)
        .transpose()?
        .unwrap_or(std::env::current_dir()?);
    let kind = required(args, "kind")?;
    let registries = if kind.eq_ignore_ascii_case("all") {
        packwand_diagnostics::build_all_registries(&root)?
    } else {
        vec![packwand_diagnostics::build_registry(
            &root,
            packwand_diagnostics::RegistryKind::from_str(kind)?,
        )?]
    };
    if args.get_flag("json") {
        if registries.len() == 1 {
            println!("{}", serde_json::to_string_pretty(&registries[0])?);
        } else {
            println!("{}", serde_json::to_string_pretty(&registries)?);
        }
    } else {
        for registry in registries {
            println!(
                "{} registry: {} source(s), {} entries, version {:.12}",
                registry.kind,
                registry.sources.len(),
                registry.entries.len(),
                registry.version
            );
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct LocalCiStage {
    name: &'static str,
    ok: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    message: String,
}

pub(super) fn ci_local(args: &ArgMatches) -> Result {
    let root = args
        .get_one::<String>("dir")
        .map(absolute)
        .transpose()?
        .unwrap_or(std::env::current_dir()?);
    let preflight = run_preflight(&root);
    let registry = packwand_diagnostics::build_all_registries(&root);
    let stages = vec![
        LocalCiStage {
            name: "preflight",
            ok: preflight.ok,
            message: format!(
                "{} error(s), {} warning(s)",
                preflight.errors, preflight.warnings
            ),
        },
        LocalCiStage {
            name: "registry",
            ok: registry.is_ok(),
            message: registry
                .err()
                .map(|error| error.to_string())
                .unwrap_or_default(),
        },
    ];
    let ok = stages.iter().all(|stage| stage.ok);
    if args.get_flag("json") {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"dir":root.to_string_lossy().replace('\\', "/"),"stages":stages,"ok":ok})
            )?
        );
    } else {
        for stage in &stages {
            println!(
                "{}: {} {}",
                stage.name,
                if stage.ok { "PASS" } else { "FAIL" },
                stage.message
            );
        }
    }
    if ok {
        Ok(())
    } else {
        Err("localized CI failed".into())
    }
}

pub(super) fn validate(args: &ArgMatches) -> Result {
    if !args.get_flag("all") && strings(args, "manifests").is_empty() {
        return Err("provide manifest path(s) or use --all".into());
    }
    let report = packwand_diagnostics::validate_projects(std::env::current_dir()?)?;
    for issue in &report.issues {
        eprintln!("{}: {}", issue.path.display(), issue.message);
    }
    if report.valid() {
        println!("all {} manifest(s) OK", report.checked);
        Ok(())
    } else {
        Err(format!("{} validation issue(s)", report.issues.len()).into())
    }
}

pub(super) fn parity(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let reports = if strings(args, "pack-dirs").is_empty() {
        packwand_diagnostics::parity_workspace(root)?
    } else {
        let mut reports = Vec::new();
        for path in strings(args, "pack-dirs") {
            reports.extend(packwand_diagnostics::parity_project(
                &packwand_workspace::read_project(&root, &absolute(path)?)?,
            ));
        }
        reports
    };
    let drifted = reports.iter().filter(|report| report.drifted()).count();
    if args.get_flag("json") {
        println!("{}", serde_json::to_string(&reports)?);
    } else {
        for report in &reports {
            println!(
                "{}/{}: {}",
                report.pack,
                report.variant,
                if report.drifted() {
                    "drifted"
                } else if report.missing_side.is_some() {
                    "single-platform"
                } else {
                    "in sync"
                }
            );
        }
        println!("{drifted} of {} report(s) drifted", reports.len());
    }
    if args.get_flag("strict") && drifted > 0 {
        Err(format!("{drifted} variant pair(s) drifted").into())
    } else {
        Ok(())
    }
}
