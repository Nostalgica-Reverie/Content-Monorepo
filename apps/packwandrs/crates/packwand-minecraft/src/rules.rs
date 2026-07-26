//! Evaluation of Mojang's allow/disallow rules against a host description.

use std::collections::BTreeMap;

use crate::model::{Rule, RuleAction};

/// The host properties rules are matched against. Captured explicitly so
/// tests can evaluate any platform from anywhere.
#[derive(Debug, Clone)]
pub struct Host {
    /// `windows`, `osx`, or `linux`.
    pub os_name: String,
    /// `x86`, `x86_64`, `aarch64`, ...
    pub arch: String,
    /// OS version string matched by rule regexes; empty when unknown.
    pub os_version: String,
    /// Launcher feature flags (`is_demo_user`, `has_custom_resolution`, ...).
    /// Absent flags are false.
    pub features: BTreeMap<String, bool>,
}

impl Host {
    /// The host this process runs on, with no features enabled.
    pub fn current() -> Self {
        let os_name = if cfg!(windows) {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else {
            "linux"
        };
        Self {
            os_name: os_name.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            // Mojang's only version rules target Windows 10+ (`^10\.`);
            // modern Windows reports a 10.x version consistently.
            os_version: if cfg!(windows) {
                "10.0".to_string()
            } else {
                String::new()
            },
            features: BTreeMap::new(),
        }
    }
}

fn rule_matches(rule: &Rule, host: &Host) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name
            && name != &host.os_name
        {
            return false;
        }
        if let Some(arch) = &os.arch {
            // Mojang uses "x86" to mean 32-bit x86 only.
            if arch != &host.arch {
                return false;
            }
        }
        if let Some(version) = &os.version {
            match regex_lite::Regex::new(version) {
                Ok(re) if re.is_match(&host.os_version) => {}
                _ => return false,
            }
        }
    }
    for (feature, required) in &rule.features {
        if host.features.get(feature).copied().unwrap_or(false) != *required {
            return false;
        }
    }
    true
}

/// Standard launcher-metadata semantics: no rules means allowed; otherwise
/// the last matching rule's action wins, defaulting to disallowed.
pub fn rules_allow(rules: &[Rule], host: &Host) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for rule in rules {
        if rule_matches(rule, host) {
            allowed = rule.action == RuleAction::Allow;
        }
    }
    allowed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::OsRule;

    fn host(os: &str, arch: &str) -> Host {
        Host {
            os_name: os.to_string(),
            arch: arch.to_string(),
            os_version: "10.0".to_string(),
            features: BTreeMap::new(),
        }
    }

    fn allow_os(name: Option<&str>, arch: Option<&str>, version: Option<&str>) -> Rule {
        Rule {
            action: RuleAction::Allow,
            os: Some(OsRule {
                name: name.map(String::from),
                arch: arch.map(String::from),
                version: version.map(String::from),
            }),
            features: BTreeMap::new(),
        }
    }

    #[test]
    fn no_rules_allows() {
        assert!(rules_allow(&[], &host("linux", "x86_64")));
    }

    #[test]
    fn os_allow_and_disallow() {
        // The classic LWJGL pattern: allow everywhere, disallow osx.
        let rules = vec![
            Rule {
                action: RuleAction::Allow,
                os: None,
                features: BTreeMap::new(),
            },
            Rule {
                action: RuleAction::Disallow,
                os: Some(OsRule {
                    name: Some("osx".to_string()),
                    ..OsRule::default()
                }),
                features: BTreeMap::new(),
            },
        ];
        assert!(rules_allow(&rules, &host("windows", "x86_64")));
        assert!(!rules_allow(&rules, &host("osx", "x86_64")));
    }

    #[test]
    fn unmatched_allow_defaults_to_disallow() {
        let rules = vec![allow_os(Some("windows"), None, None)];
        assert!(!rules_allow(&rules, &host("linux", "x86_64")));
    }

    #[test]
    fn arch_and_version_constraints() {
        let rules = vec![allow_os(Some("windows"), Some("x86"), None)];
        assert!(!rules_allow(&rules, &host("windows", "x86_64")));
        assert!(rules_allow(&rules, &host("windows", "x86")));

        let rules = vec![allow_os(Some("windows"), None, Some("^10\\."))];
        assert!(rules_allow(&rules, &host("windows", "x86_64")));
        let mut old = host("windows", "x86_64");
        old.os_version = "6.1".to_string();
        assert!(!rules_allow(&rules, &old));
    }

    #[test]
    fn feature_rules() {
        let rule = Rule {
            action: RuleAction::Allow,
            os: None,
            features: BTreeMap::from([("is_demo_user".to_string(), true)]),
        };
        assert!(!rules_allow(
            std::slice::from_ref(&rule),
            &host("linux", "x86_64")
        ));
        let mut demo = host("linux", "x86_64");
        demo.features.insert("is_demo_user".to_string(), true);
        assert!(rules_allow(&[rule], &demo));
    }
}
