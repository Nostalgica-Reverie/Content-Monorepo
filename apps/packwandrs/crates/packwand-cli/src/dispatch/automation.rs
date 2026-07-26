//! The `automation` command group: effective settings, full-auto
//! discovery, and the unattended release pipeline.

use super::*;

pub(super) fn automation(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    match args.subcommand() {
        Some(("get", sub)) => {
            let project_root = absolute(required(sub, "pack-dir")?)?;
            if !project_root.is_dir() {
                return Err(format!("{} is not a directory", project_root.display()).into());
            }
            // A project directory without a manifest simply has no automation
            // overrides. Report the effective defaults rather than failing, so
            // callers can ask about any pack directory uniformly.
            let automation = if project_root.join("manifest.json").is_file() {
                packwand_workspace::read_project(&root, &project_root)?
                    .manifest
                    .automation()
            } else {
                packwand_workspace::Automation::default()
            };
            println!("{}", serde_json::to_string_pretty(&automation)?);
            Ok(())
        }
        Some(("list-full-auto", _)) => {
            for project in packwand_workspace::discover(root)? {
                if project
                    .manifest
                    .automation
                    .as_ref()
                    .and_then(|automation| automation.full_auto.as_ref())
                    .and_then(|value| value.get("enabled"))
                    .and_then(serde_json::Value::as_bool)
                    == Some(true)
                {
                    println!("{}", project.manifest.id);
                }
            }
            Ok(())
        }
        Some(("run", sub)) => automation_run(&root, sub),
        _ => Err("automation requires get, run, or list-full-auto".into()),
    }
}

#[derive(Serialize)]
struct AutomationStep {
    name: &'static str,
    status: &'static str,
    detail: String,
}

#[derive(Serialize)]
struct AutomationReport {
    pack_dir: String,
    pack_id: String,
    old_version: String,
    new_version: String,
    dry_run: bool,
    status: &'static str,
    steps: Vec<AutomationStep>,
}

fn automation_run(root: &Path, args: &ArgMatches) -> Result {
    let project_root = absolute(required(args, "pack-dir")?)?;
    let project = packwand_workspace::read_project(root, &project_root)?;
    let full_auto = project
        .manifest
        .automation
        .as_ref()
        .and_then(|value| value.full_auto.as_ref());
    if full_auto
        .and_then(|value| value.get("enabled"))
        .and_then(serde_json::Value::as_bool)
        != Some(true)
    {
        return Err("automation.full_auto.enabled is not true".into());
    }
    if project.subdirs.is_empty() {
        return Err("automation target has no pack subdirectories".into());
    }
    let dry_run = args.get_flag("dry-run");
    let validation = packwand_diagnostics::validate_projects(root)?;
    if !validation.valid() {
        return Err("pre-automation manifest validation failed".into());
    }
    let mut steps = vec![AutomationStep {
        name: "validate",
        status: "ok",
        detail: format!("{} document(s) checked", validation.checked),
    }];
    let mut changed = 0usize;
    for subdir in &project.subdirs {
        let records = update_workspace(subdir, None, true, dry_run)?;
        let failures = records
            .iter()
            .filter(|record| {
                record
                    .error
                    .as_deref()
                    .is_some_and(|error| error != "pinned")
            })
            .count();
        if failures > 0 {
            return Err(format!(
                "{failures} provider update(s) failed in {}",
                subdir.display()
            )
            .into());
        }
        changed += records.iter().filter(|record| record.changed).count();
    }
    steps.push(AutomationStep {
        name: "update",
        status: "ok",
        detail: format!(
            "{changed} compatible update(s){}",
            if dry_run { " found" } else { " applied" }
        ),
    });
    let sync = packwand_workspace::sync_performance_bases(root, dry_run)?;
    steps.push(AutomationStep {
        name: "sync",
        status: "ok",
        detail: format!("{} copied, {} deleted", sync.copied, sync.deleted),
    });
    let tests = full_auto
        .and_then(|value| value.get("tests"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    if dry_run {
        steps.push(AutomationStep {
            name: "tests",
            status: "skipped",
            detail: format!("would run {} configured test(s)", tests.len()),
        });
    } else {
        for test in &tests {
            #[cfg(windows)]
            let status = std::process::Command::new("cmd")
                .args(["/C", test])
                .current_dir(&project.root)
                .status()?;
            #[cfg(not(windows))]
            let status = std::process::Command::new("sh")
                .args(["-c", test])
                .current_dir(&project.root)
                .status()?;
            if !status.success() {
                return Err(format!("automation test {test:?} failed with {status}").into());
            }
        }
        steps.push(AutomationStep {
            name: "tests",
            status: if tests.is_empty() { "skipped" } else { "ok" },
            detail: format!("{} configured test(s)", tests.len()),
        });
    }
    let next = next_calver(&project.manifest.version);
    if dry_run {
        steps.push(AutomationStep {
            name: "bump",
            status: "skipped",
            detail: format!("would bump {} -> {next}", project.manifest.version),
        });
    } else {
        packwand_workspace::bump(root, &project.manifest.id, &next)?;
        steps.push(AutomationStep {
            name: "bump",
            status: "ok",
            detail: format!("{} -> {next}", project.manifest.version),
        });
    }
    let report = AutomationReport {
        pack_dir: project.root.to_string_lossy().into_owned(),
        pack_id: project.manifest.id,
        old_version: project.manifest.version,
        new_version: next,
        dry_run,
        status: if changed == 0 {
            "no_changes"
        } else {
            "ready_to_publish"
        },
        steps,
    };
    let encoded = serde_json::to_vec_pretty(&report)?;
    if let Some(path) = args.get_one::<String>("report") {
        let path = absolute(path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &encoded)?;
    }
    if args.get_flag("json") {
        println!("{}", String::from_utf8(encoded)?);
    } else {
        println!("automation {}: {}", report.pack_id, report.status);
        for step in &report.steps {
            println!("  {:<10} {:<8} {}", step.name, step.status, step.detail);
        }
    }
    Ok(())
}

fn next_calver(current: &str) -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let z = seconds.div_euclid(86_400) + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    let cycle = format!("{:02}.{month:02}", year % 100);
    let mut parts = current.split('.');
    let previous = match (parts.next(), parts.next()) {
        (Some(year), Some(month)) => format!("{year}.{month}"),
        _ => String::new(),
    };
    if previous != cycle {
        return cycle;
    }
    let patch = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0)
        + 1;
    format!("{cycle}.{patch}")
}
