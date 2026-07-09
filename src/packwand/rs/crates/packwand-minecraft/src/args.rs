//! Resolution of a version document's argument lists into the values an
//! instance record stores.
//!
//! Three kinds of placeholder are handled differently:
//!
//! - **Identity/static values** (`${version_name}`, `${auth_player_name}`,
//!   ...) are substituted here, at bootstrap time.
//! - **Layout paths** (`${game_directory}`, `${natives_directory}`, ...)
//!   are left in place: `packwand-launch` substitutes them when the plan is
//!   built, which keeps instance records relocatable.
//! - **Secrets** (`${auth_access_token}`, legacy `${auth_session}`) are
//!   rewritten to `${secret:auth_access_token}` placeholders that the
//!   supervisor resolves at spawn time; the raw value never enters the
//!   record or the plan.

use crate::model::{Argument, VersionDoc};
use crate::rules::{rules_allow, Host};
use crate::MinecraftError;

/// Bootstrap-time values for identity and static placeholders.
#[derive(Debug, Clone)]
pub struct LaunchContext {
    pub version_id: String,
    pub version_type: String,
    pub assets_index_name: String,
    pub player_name: String,
    pub player_uuid: String,
    pub user_type: String,
    pub launcher_name: String,
    pub launcher_version: String,
    /// Absolute `${game_assets}` directory for legacy materialized-asset
    /// versions; `None` leaves the placeholder for the caller to reject.
    pub game_assets_dir: Option<String>,
}

/// The argument surfaces of one version, resolved for storage in an
/// instance record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedArgs {
    pub main_class: String,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    /// Names of `${secret:<name>}` placeholders the arguments reference.
    pub session_placeholders: Vec<String>,
}

/// Placeholders `packwand-launch` resolves at plan-build time; they must
/// survive bootstrap untouched.
const PLAN_TIME_VARS: &[&str] = &[
    "game_directory",
    "natives_directory",
    "assets_root",
    "library_directory",
    "classpath",
    "classpath_separator",
    "game_dir",
    "natives_dir",
    "assets_dir",
    "libraries_dir",
    "logs_dir",
    "instance_id",
];

fn substitute(arg: &str, ctx: &LaunchContext, session_placeholders: &mut Vec<String>) -> String {
    let mut out = arg.to_string();
    // Secrets first, so the static pass below can never see them.
    for secret_var in ["auth_access_token", "auth_session"] {
        let placeholder = format!("${{{secret_var}}}");
        if out.contains(&placeholder) {
            out = out.replace(&placeholder, "${secret:auth_access_token}");
            if !session_placeholders.contains(&"auth_access_token".to_string()) {
                session_placeholders.push("auth_access_token".to_string());
            }
        }
    }
    let statics: [(&str, &str); 12] = [
        ("version_name", &ctx.version_id),
        ("version_type", &ctx.version_type),
        ("assets_index_name", &ctx.assets_index_name),
        ("auth_player_name", &ctx.player_name),
        ("auth_uuid", &ctx.player_uuid),
        ("user_type", &ctx.user_type),
        ("launcher_name", &ctx.launcher_name),
        ("launcher_version", &ctx.launcher_version),
        // Legacy placeholders old versions still reference.
        ("user_properties", "{}"),
        ("auth_xuid", ""),
        ("clientid", ""),
        ("profile_name", &ctx.player_name),
    ];
    for (name, value) in statics {
        out = out.replace(&format!("${{{name}}}"), value);
    }
    if let Some(game_assets) = &ctx.game_assets_dir {
        out = out.replace("${game_assets}", game_assets);
    }
    out
}

fn logging_argument(doc: &VersionDoc) -> Option<String> {
    let logging = doc.logging.as_ref()?.client.as_ref()?;
    Some(logging.argument.replace(
        "${path}",
        &format!("${{assets_root}}/log_configs/{}", logging.file.id),
    ))
}

/// Flattens one modern argument list, applying rules and substitutions.
fn resolve_list(
    arguments: &[Argument],
    host: &Host,
    ctx: &LaunchContext,
    session_placeholders: &mut Vec<String>,
) -> Vec<String> {
    let mut out = Vec::new();
    for argument in arguments {
        match argument {
            Argument::Plain(value) => out.push(substitute(value, ctx, session_placeholders)),
            Argument::Conditional { rules, value } => {
                if rules_allow(rules, host) {
                    for v in value.as_slice() {
                        out.push(substitute(v, ctx, session_placeholders));
                    }
                }
            }
        }
    }
    out
}

/// Strips the `-cp ${classpath}` pair from resolved JVM arguments: the
/// classpath lives in the instance record's `classpath` field and
/// `LaunchPlan::command_arguments` re-adds the flag.
fn strip_classpath_flag(jvm_args: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(jvm_args.len());
    let mut iter = jvm_args.into_iter().peekable();
    while let Some(arg) = iter.next() {
        if (arg == "-cp" || arg == "-classpath")
            && iter
                .peek()
                .is_some_and(|next| next.contains("${classpath}"))
        {
            iter.next();
            continue;
        }
        out.push(arg);
    }
    out
}

/// Resolves the launchable argument surfaces of one merged version document.
pub fn resolve_launch_args(
    doc: &VersionDoc,
    host: &Host,
    ctx: &LaunchContext,
) -> Result<ResolvedArgs, MinecraftError> {
    let main_class = doc
        .main_class
        .clone()
        .ok_or_else(|| MinecraftError::MissingMainClass(doc.id.clone()))?;
    let mut session_placeholders = Vec::new();

    let (mut jvm_args, game_args) = if let Some(arguments) = &doc.arguments {
        (
            resolve_list(&arguments.jvm, host, ctx, &mut session_placeholders),
            resolve_list(&arguments.game, host, ctx, &mut session_placeholders),
        )
    } else if let Some(legacy) = &doc.minecraft_arguments {
        // Pre-1.13: game arguments are one string; the JVM argument list
        // is implied. Provide the natives path the way modern documents do.
        let game = legacy
            .split_whitespace()
            .map(|arg| substitute(arg, ctx, &mut session_placeholders))
            .collect();
        (
            vec!["-Djava.library.path=${natives_directory}".to_string()],
            game,
        )
    } else {
        return Err(MinecraftError::MissingArguments(doc.id.clone()));
    };

    if let Some(logging_arg) = logging_argument(doc) {
        jvm_args.push(substitute(&logging_arg, ctx, &mut session_placeholders));
    }

    Ok(ResolvedArgs {
        main_class,
        jvm_args: strip_classpath_flag(jvm_args),
        game_args,
        session_placeholders,
    })
}

/// True when an argument still carries an unresolved non-plan-time,
/// non-secret placeholder (typically a feature this port does not model).
pub fn has_unresolved_placeholder(arg: &str) -> bool {
    let mut rest = arg;
    while let Some(start) = rest.find("${") {
        let after = &rest[start + 2..];
        let Some(end) = after.find('}') else {
            return true;
        };
        let name = &after[..end];
        if !name.starts_with("secret:") && !PLAN_TIME_VARS.contains(&name) {
            return true;
        }
        rest = &after[end + 1..];
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> LaunchContext {
        LaunchContext {
            version_id: "fixture-1.0".to_string(),
            version_type: "release".to_string(),
            assets_index_name: "17".to_string(),
            player_name: "Steve".to_string(),
            player_uuid: "00000000-0000-3000-8000-000000000000".to_string(),
            user_type: "legacy".to_string(),
            launcher_name: "packwand".to_string(),
            launcher_version: "0.1.0".to_string(),
            game_assets_dir: None,
        }
    }

    fn windows_host() -> Host {
        Host {
            os_name: "windows".to_string(),
            arch: "x86_64".to_string(),
            os_version: "10.0".to_string(),
            features: Default::default(),
        }
    }

    #[test]
    fn modern_arguments_resolve() {
        let doc: crate::model::VersionDoc =
            serde_json::from_str(include_str!("../tests/fixtures/version-modern.json")).unwrap();
        let resolved = resolve_launch_args(&doc, &windows_host(), &ctx()).unwrap();
        assert_eq!(resolved.main_class, "net.minecraft.client.main.Main");

        // Identity and static values are baked in.
        let game = resolved.game_args.join(" ");
        assert!(game.contains("--username Steve"), "{game}");
        assert!(game.contains("--version fixture-1.0"), "{game}");
        assert!(game.contains("--assetIndex 17"), "{game}");
        // Secrets are rewritten, never resolved.
        assert!(
            game.contains("--accessToken ${secret:auth_access_token}"),
            "{game}"
        );
        assert_eq!(resolved.session_placeholders, vec!["auth_access_token"]);
        // Feature-gated demo flag is off by default.
        assert!(!game.contains("--demo"), "{game}");
        // Plan-time layout vars survive.
        assert!(game.contains("--gameDir ${game_directory}"), "{game}");
        assert!(game.contains("--assetsDir ${assets_root}"), "{game}");

        let jvm = resolved.jvm_args.join(" ");
        // Windows-only rule applies on the windows host.
        assert!(jvm.contains("HeapDumpPath"), "{jvm}");
        assert!(
            jvm.contains("-Djava.library.path=${natives_directory}"),
            "{jvm}"
        );
        assert!(jvm.contains("-Dminecraft.launcher.brand=packwand"), "{jvm}");
        // The -cp pair is stripped; the classpath field owns it.
        assert!(!resolved.jvm_args.contains(&"-cp".to_string()), "{jvm}");
        assert!(!jvm.contains("${classpath}"), "{jvm}");
    }

    #[test]
    fn windows_only_jvm_arg_is_dropped_elsewhere() {
        let doc: crate::model::VersionDoc =
            serde_json::from_str(include_str!("../tests/fixtures/version-modern.json")).unwrap();
        let linux = Host {
            os_name: "linux".to_string(),
            arch: "x86_64".to_string(),
            os_version: String::new(),
            features: Default::default(),
        };
        let resolved = resolve_launch_args(&doc, &linux, &ctx()).unwrap();
        assert!(!resolved.jvm_args.join(" ").contains("HeapDumpPath"));
    }

    #[test]
    fn legacy_minecraft_arguments_resolve() {
        let doc = crate::model::VersionDoc {
            id: "1.7.10".to_string(),
            main_class: Some("net.minecraft.client.main.Main".to_string()),
            minecraft_arguments: Some(
                "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${game_assets} --session ${auth_session}".to_string(),
            ),
            ..Default::default()
        };
        let mut context = ctx();
        context.game_assets_dir = Some("C:/root/assets/virtual/legacy".to_string());
        let resolved = resolve_launch_args(&doc, &windows_host(), &context).unwrap();
        let game = resolved.game_args.join(" ");
        assert!(game.contains("--username Steve"), "{game}");
        assert!(
            game.contains("--assetsDir C:/root/assets/virtual/legacy"),
            "{game}"
        );
        assert!(
            game.contains("--session ${secret:auth_access_token}"),
            "{game}"
        );
        assert_eq!(
            resolved.jvm_args,
            vec!["-Djava.library.path=${natives_directory}".to_string()]
        );
    }

    #[test]
    fn logging_argument_uses_plan_time_assets_root_placeholder() {
        let doc = crate::model::VersionDoc {
            id: "1.20.1-forge-47.4.5".to_string(),
            main_class: Some("cpw.mods.bootstraplauncher.BootstrapLauncher".to_string()),
            arguments: Some(crate::model::Arguments::default()),
            logging: Some(crate::model::LoggingConfigSet {
                client: Some(crate::model::LoggingConfig {
                    argument: "-Dlog4j.configurationFile=${path}".to_string(),
                    file: crate::model::LoggingFile {
                        id: "client-1.xml".to_string(),
                        url: "http://x/logging.xml".to_string(),
                        sha1: None,
                        size: None,
                    },
                }),
            }),
            ..Default::default()
        };
        let resolved = resolve_launch_args(&doc, &windows_host(), &ctx()).unwrap();
        assert!(resolved.jvm_args.contains(
            &"-Dlog4j.configurationFile=${assets_root}/log_configs/client-1.xml".to_string()
        ));
    }

    #[test]
    fn unresolved_placeholder_detection() {
        assert!(!has_unresolved_placeholder("--gameDir ${game_directory}"));
        assert!(!has_unresolved_placeholder("${secret:auth_access_token}"));
        assert!(!has_unresolved_placeholder("plain"));
        assert!(!has_unresolved_placeholder("${classpath_separator}"));
        assert!(has_unresolved_placeholder("--width ${resolution_width}"));
    }
}
