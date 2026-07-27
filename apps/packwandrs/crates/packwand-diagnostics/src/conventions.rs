//! Opt-in per-mod convention checks.
//!
//! Packwand requires no files of its own. Every check here is dormant until a
//! pack names it in `manifest.json`'s `conventions` object, so adding this
//! module changes nothing for a pack that does not opt in:
//!
//! ```jsonc
//! "conventions": {
//!   "bcc": true,
//!   "ftbquests": { "level": "warn" },
//!   "options": { "path": "config/modpack_defaults/options.txt" }
//! }
//! ```
//!
//! The value of these checks is less "does the file exist" than "does it still
//! agree with the manifest" — a Better Compatibility Checker config left at last
//! release's version ships silently and breaks update detection for every user.

use std::fs;
use std::path::{Path, PathBuf};

use packwand_workspace::{ConventionCheck, Manifest};

use crate::{Issue, Severity, ValidationReport};

/// Every known check id, in report order.
pub const CHECKS: [&str; 5] = [
    "options",
    "resourcepackoverrides",
    "modpackupdatechecker",
    "bcc",
    "ftbquests",
];

/// Where `options.txt` legitimately lives. Several mods provide default-options
/// handling and packs in the wild use all of them, so presence anywhere in this
/// list satisfies the check.
const OPTIONS_LOCATIONS: [&str; 5] = [
    "config/modpack_defaults/options.txt",
    "configureddefaults/options.txt",
    "config/defaultoptions/options.txt",
    "config/yosbr/options.txt",
    "options.txt",
];

/// Runs the checks this pack opted into. An empty `conventions` object — the
/// default — yields an empty report without touching the filesystem.
pub fn conventions_lint(root: impl AsRef<Path>, manifest: &Manifest) -> ValidationReport {
    let root = root.as_ref();
    let conventions = manifest.conventions();
    let mut report = ValidationReport::default();
    for id in CHECKS {
        let Some(check) = conventions.check(id) else {
            continue;
        };
        report.checked += 1;
        let level = severity_for(check);
        match id {
            "options" => check_options(root, check, level, &mut report.issues),
            "resourcepackoverrides" => {
                check_resourcepackoverrides(root, check, level, &mut report.issues);
            }
            "modpackupdatechecker" => {
                check_named_config(
                    root,
                    check,
                    level,
                    manifest,
                    "modpackupdatechecker",
                    &[
                        "config/modpackupdatechecker/modpackupdatechecker.json",
                        "config/modpackupdatechecker/config.json",
                        "config/modpackupdatechecker.json",
                    ],
                    &mut report.issues,
                );
            }
            "bcc" => {
                check_named_config(
                    root,
                    check,
                    level,
                    manifest,
                    "bcc",
                    &["config/bcc.json", "config/bcc/bcc.json"],
                    &mut report.issues,
                );
            }
            "ftbquests" => check_ftbquests(root, check, level, &mut report.issues),
            _ => {}
        }
    }
    report
}

/// A declared `level` overrides the default; anything unrecognised is reported
/// so a typo in a manifest cannot quietly disable a check.
fn severity_for(check: &ConventionCheck) -> Severity {
    match check.level() {
        Some(level) if level.eq_ignore_ascii_case("warn") => Severity::Warning,
        Some(level) if level.eq_ignore_ascii_case("warning") => Severity::Warning,
        _ => Severity::Error,
    }
}

fn issue(severity: Severity, path: &Path, message: impl Into<String>) -> Issue {
    Issue {
        severity,
        path: path.to_path_buf(),
        message: message.into(),
    }
}

/// Resolves a pack-relative path, also accepting the mirrored tree that
/// default-options mods keep under `config/modpack_defaults/`. Real packs in
/// this repo put `resourcepackoverrides.json` at
/// `config/modpack_defaults/config/resourcepackoverrides.json`, so a lookup
/// that only tried `config/` would miss it.
fn locate(root: &Path, relative: &str) -> Option<PathBuf> {
    let direct = root.join(relative);
    if direct.exists() {
        return Some(direct);
    }
    let mirrored = root.join("config/modpack_defaults").join(relative);
    if mirrored.exists() {
        return Some(mirrored);
    }
    None
}

fn check_options(root: &Path, check: &ConventionCheck, level: Severity, issues: &mut Vec<Issue>) {
    if let Some(explicit) = check.path() {
        if !root.join(explicit).is_file() {
            issues.push(issue(
                level,
                &root.join(explicit),
                "options.txt is missing at the path this pack declares",
            ));
        }
        return;
    }
    if OPTIONS_LOCATIONS
        .iter()
        .any(|candidate| root.join(candidate).is_file())
    {
        return;
    }
    issues.push(issue(
        level,
        root,
        format!(
            "no options.txt found; expected one of {}",
            OPTIONS_LOCATIONS.join(", ")
        ),
    ));
}

fn check_resourcepackoverrides(
    root: &Path,
    check: &ConventionCheck,
    level: Severity,
    issues: &mut Vec<Issue>,
) {
    let relative = check.path().unwrap_or("config/resourcepackoverrides.json");
    let Some(path) = locate(root, relative) else {
        issues.push(issue(
            level,
            &root.join(relative),
            "resourcepackoverrides config is missing",
        ));
        return;
    };
    // Presence is the whole check: the file carries pack lists, not a name or
    // version, so there is nothing to cross-reference against the manifest.
    if let Err(message) = read_json(&path) {
        issues.push(issue(Severity::Warning, &path, message));
    }
}

fn check_ftbquests(root: &Path, check: &ConventionCheck, level: Severity, issues: &mut Vec<Issue>) {
    let relative = check.path().unwrap_or("config/ftbquests/quests");
    let path = root.join(relative);
    if path.is_dir() {
        return;
    }
    // A file named `quests` where a directory belongs is a distinct failure
    // worth naming: the pack looks populated but FTB Quests loads nothing.
    let message = if path.exists() {
        "config/ftbquests/quests exists but is not a directory"
    } else {
        "config/ftbquests/quests directory is missing"
    };
    issues.push(issue(level, &path, message));
}

/// Shared shape for configs that embed the modpack's own name (and, for BCC,
/// its version). Drift here is the failure mode actually worth gating on.
fn check_named_config(
    root: &Path,
    check: &ConventionCheck,
    level: Severity,
    manifest: &Manifest,
    label: &str,
    candidates: &[&str],
    issues: &mut Vec<Issue>,
) {
    let located = match check.path() {
        Some(explicit) => locate(root, explicit),
        None => candidates.iter().find_map(|candidate| locate(root, candidate)),
    };
    let Some(path) = located else {
        let expected = check.path().map(str::to_owned).unwrap_or_else(|| candidates.join(", "));
        issues.push(issue(
            level,
            &root.join(candidates.first().copied().unwrap_or(label)),
            format!("{label} config is missing; expected {expected}"),
        ));
        return;
    };
    let value = match read_json(&path) {
        Ok(value) => value,
        Err(message) => {
            issues.push(issue(Severity::Warning, &path, message));
            return;
        }
    };

    let expected_name = check.expect_name().unwrap_or_else(|| manifest.effective_name());
    match find_string(&value, &["modpackName", "modpack_name", "name", "packName"]) {
        Some(found) if found.trim().eq_ignore_ascii_case(expected_name.trim()) => {}
        Some(found) => issues.push(issue(
            level,
            &path,
            format!("{label} modpack name is {found:?} but the manifest says {expected_name:?}"),
        )),
        // An unrecognised layout is a warning: a third-party format we cannot
        // read must not hard-block a release.
        None => issues.push(issue(
            Severity::Warning,
            &path,
            format!("{label} config has no recognisable modpack name field"),
        )),
    }

    // Only BCC carries the pack version, and it is the field that silently goes
    // stale after a release bump.
    if label != "bcc" {
        return;
    }
    if manifest.version.trim().is_empty() {
        return;
    }
    match find_string(&value, &["modpackVersion", "modpack_version", "version"]) {
        Some(found) if found.trim() == manifest.version.trim() => {}
        Some(found) => issues.push(issue(
            level,
            &path,
            format!(
                "{label} modpack version is {found:?} but the manifest says {:?}",
                manifest.version
            ),
        )),
        None => issues.push(issue(
            Severity::Warning,
            &path,
            format!("{label} config has no recognisable modpack version field"),
        )),
    }
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let source = fs::read_to_string(path).map_err(|error| format!("could not read: {error}"))?;
    serde_json::from_str(&source).map_err(|error| error.to_string())
}

/// Looks up the first matching key, case-insensitively, at the top level and one
/// level down. These are third-party formats whose exact casing and nesting
/// vary between mod versions, so an exact-key lookup would be brittle.
fn find_string<'a>(value: &'a serde_json::Value, keys: &[&str]) -> Option<&'a str> {
    let object = value.as_object()?;
    for key in keys {
        if let Some(found) = object
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .and_then(|(_, value)| value.as_str())
            .filter(|found| !found.trim().is_empty())
        {
            return Some(found);
        }
    }
    object
        .values()
        .filter_map(serde_json::Value::as_object)
        .find_map(|nested| {
            keys.iter().find_map(|key| {
                nested
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(key))
                    .and_then(|(_, value)| value.as_str())
                    .filter(|found| !found.trim().is_empty())
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(json: &str) -> Manifest {
        serde_json::from_str(json).expect("manifest parses")
    }

    fn write(root: &Path, relative: &str, contents: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("has parent")).expect("create dirs");
        fs::write(path, contents).expect("write file");
    }

    /// The property that keeps this safe to ship: a pack that says nothing is
    /// never reported against, however incomplete it is.
    #[test]
    fn opting_out_is_the_default() {
        let root = tempfile::tempdir().expect("tempdir");
        let manifest = manifest(r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0"}"#);
        let report = conventions_lint(root.path(), &manifest);
        assert_eq!(report.checked, 0);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn disabled_check_does_not_run() {
        let root = tempfile::tempdir().expect("tempdir");
        let manifest = manifest(
            r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0",
                "conventions":{"bcc":false}}"#,
        );
        let report = conventions_lint(root.path(), &manifest);
        assert_eq!(report.checked, 0);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn options_accepts_every_known_mechanism() {
        let manifest = manifest(
            r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0",
                "conventions":{"options":true}}"#,
        );
        for location in OPTIONS_LOCATIONS {
            let root = tempfile::tempdir().expect("tempdir");
            write(root.path(), location, "fov:1.0\n");
            let report = conventions_lint(root.path(), &manifest);
            assert!(
                report.issues.is_empty(),
                "{location} should satisfy the options check"
            );
        }
    }

    #[test]
    fn options_missing_everywhere_is_reported() {
        let root = tempfile::tempdir().expect("tempdir");
        let manifest = manifest(
            r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0",
                "conventions":{"options":true}}"#,
        );
        let report = conventions_lint(root.path(), &manifest);
        assert_eq!(report.checked, 1);
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].severity, Severity::Error);
    }

    #[test]
    fn bcc_name_and_version_drift_are_caught() {
        let root = tempfile::tempdir().expect("tempdir");
        write(
            root.path(),
            "config/bcc.json",
            r#"{"modpackName":"Wrong Pack","modpackVersion":"0.9"}"#,
        );
        let manifest = manifest(
            r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0",
                "conventions":{"bcc":true}}"#,
        );
        let report = conventions_lint(root.path(), &manifest);
        assert_eq!(report.issues.len(), 2, "{:?}", report.issues);
        assert!(report.issues.iter().all(|i| i.severity == Severity::Error));
        assert!(report.issues.iter().any(|i| i.message.contains("name")));
        assert!(report.issues.iter().any(|i| i.message.contains("version")));
    }

    #[test]
    fn bcc_matching_config_is_clean() {
        let root = tempfile::tempdir().expect("tempdir");
        write(
            root.path(),
            "config/bcc.json",
            r#"{"modpackName":"Pack","modpackVersion":"1.0"}"#,
        );
        let manifest = manifest(
            r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0",
                "conventions":{"bcc":true}}"#,
        );
        assert!(conventions_lint(root.path(), &manifest).issues.is_empty());
    }

    /// An unreadable third-party format must not block a release.
    #[test]
    fn unrecognised_config_layout_warns_only() {
        let root = tempfile::tempdir().expect("tempdir");
        write(root.path(), "config/bcc.json", r#"{"unrelated":true}"#);
        let manifest = manifest(
            r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0",
                "conventions":{"bcc":true}}"#,
        );
        let report = conventions_lint(root.path(), &manifest);
        assert!(report.issues.iter().all(|i| i.severity == Severity::Warning));
    }

    #[test]
    fn level_override_downgrades_to_warning() {
        let root = tempfile::tempdir().expect("tempdir");
        let manifest = manifest(
            r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0",
                "conventions":{"ftbquests":{"level":"warn"}}}"#,
        );
        let report = conventions_lint(root.path(), &manifest);
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].severity, Severity::Warning);
    }

    #[test]
    fn quests_as_a_file_is_distinguished_from_missing() {
        let root = tempfile::tempdir().expect("tempdir");
        write(root.path(), "config/ftbquests/quests", "not a directory");
        let manifest = manifest(
            r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0",
                "conventions":{"ftbquests":true}}"#,
        );
        let report = conventions_lint(root.path(), &manifest);
        assert_eq!(report.issues.len(), 1);
        assert!(report.issues[0].message.contains("not a directory"));
    }

    /// Mirrors the real layout found in this repo, where the default-options mod
    /// carries a nested `config/` tree.
    #[test]
    fn mirrored_modpack_defaults_tree_is_searched() {
        let root = tempfile::tempdir().expect("tempdir");
        write(
            root.path(),
            "config/modpack_defaults/config/resourcepackoverrides.json",
            r#"{"schema_version":2}"#,
        );
        let manifest = manifest(
            r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0",
                "conventions":{"resourcepackoverrides":true}}"#,
        );
        assert!(conventions_lint(root.path(), &manifest).issues.is_empty());
    }

    #[test]
    fn explicit_path_override_is_honoured() {
        let root = tempfile::tempdir().expect("tempdir");
        write(root.path(), "custom/opts.txt", "fov:1.0\n");
        let manifest = manifest(
            r#"{"id":"p","name":"Pack","type":"modpack","version":"1.0",
                "conventions":{"options":{"path":"custom/opts.txt"}}}"#,
        );
        assert!(conventions_lint(root.path(), &manifest).issues.is_empty());
    }

    #[test]
    fn expect_name_overrides_the_manifest_name() {
        let root = tempfile::tempdir().expect("tempdir");
        write(
            root.path(),
            "config/bcc.json",
            r#"{"modpackName":"Shipped Name","modpackVersion":"1.0"}"#,
        );
        let manifest = manifest(
            r#"{"id":"p","name":"Internal Name","type":"modpack","version":"1.0",
                "conventions":{"bcc":{"expect_name":"Shipped Name"}}}"#,
        );
        assert!(conventions_lint(root.path(), &manifest).issues.is_empty());
    }
}
