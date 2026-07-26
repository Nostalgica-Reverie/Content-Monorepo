//! The `migrate` command group and loader version resolution.

use super::*;

pub(super) fn migrate(args: &ArgMatches) -> Result {
    let root = std::env::current_dir()?;
    let mut workspace = Workspace::open(root)?;
    match args.subcommand() {
        Some(("format", _)) => {
            let (old, new) = workspace.migrate_format()?;
            println!("pack format: {old} -> {new}");
            Ok(())
        }
        Some(("minecraft", sub)) => {
            let version = required(sub, "version")?;
            let old = workspace
                .set_version("minecraft", version)?
                .unwrap_or_else(|| "<unset>".into());
            println!("minecraft: {old} -> {version}");
            Ok(())
        }
        Some(("loader", sub)) => {
            let requested = required(sub, "version")?;
            let loaders = workspace
                .pack()
                .versions
                .keys()
                .filter(|key| key.as_str() != "minecraft")
                .cloned()
                .collect::<Vec<_>>();
            if loaders.is_empty() {
                return Err("pack has no configured loader".into());
            }
            for loader in loaders {
                let version = if matches!(requested, "latest" | "recommended") {
                    let minecraft = workspace
                        .pack()
                        .versions
                        .get("minecraft")
                        .ok_or("pack has no Minecraft version")?;
                    resolve_loader_version(&loader, minecraft, requested)?
                } else {
                    requested.to_owned()
                };
                let old = workspace
                    .set_version(&loader, &version)?
                    .unwrap_or_else(|| "<unset>".into());
                println!("{loader}: {old} -> {version}");
            }
            Ok(())
        }
        _ => Err("migrate requires format, minecraft, or loader".into()),
    }
}

pub(super) fn resolve_loader_version(
    loader: &str,
    minecraft: &str,
    channel: &str,
) -> Result<String> {
    let transport = UreqTransport::new();
    let value = match loader {
        "fabric" => {
            let bytes = transport.get(packwand_providers::HttpRequest::get(format!(
                "https://meta.fabricmc.net/v2/versions/loader/{minecraft}"
            )))?;
            serde_json::from_slice::<serde_json::Value>(&bytes)?
        }
        "quilt" => {
            let bytes = transport.get(packwand_providers::HttpRequest::get(format!(
                "https://meta.quiltmc.org/v3/versions/loader/{minecraft}"
            )))?;
            serde_json::from_slice::<serde_json::Value>(&bytes)?
        }
        "forge" => {
            let bytes = transport.get(packwand_providers::HttpRequest::get(
                "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json",
            ))?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)?;
            let promos = value
                .get("promos")
                .and_then(serde_json::Value::as_object)
                .ok_or("Forge promotions response has no promos")?;
            let key = format!(
                "{minecraft}-{}",
                if channel == "recommended" {
                    "recommended"
                } else {
                    "latest"
                }
            );
            return promos
                .get(&key)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| format!("Forge has no {channel} loader for {minecraft}").into());
        }
        "neoforge" => {
            let bytes = transport.get(packwand_providers::HttpRequest::get(
                "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge",
            ))?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)?;
            let versions = value
                .get("versions")
                .and_then(serde_json::Value::as_array)
                .ok_or("NeoForge version response has no versions")?;
            let prefix = neoforge_version_prefix(minecraft);
            return versions
                .iter()
                .filter_map(serde_json::Value::as_str)
                .rfind(|version| {
                    prefix
                        .as_ref()
                        .is_none_or(|prefix| version.starts_with(prefix))
                })
                .map(str::to_owned)
                .ok_or_else(|| format!("NeoForge has no loader for {minecraft}").into());
        }
        _ => return Err(format!("unsupported loader {loader:?}").into()),
    };
    let releases = value
        .as_array()
        .ok_or_else(|| format!("{loader} loader response is not an array"))?;
    releases
        .iter()
        .filter(|release| {
            channel != "recommended"
                || release
                    .get("loader")
                    .and_then(|loader| loader.get("stable"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true)
        })
        .find_map(|release| {
            release
                .get("loader")
                .and_then(|loader| loader.get("version"))
                .and_then(serde_json::Value::as_str)
        })
        .map(str::to_owned)
        .ok_or_else(|| format!("{loader} has no {channel} loader for {minecraft}").into())
}

fn neoforge_version_prefix(minecraft: &str) -> Option<String> {
    let mut parts = minecraft.split('.');
    let major = parts.next()?;
    let minor = parts.next()?;
    let patch = parts.next();
    if major == "1" {
        Some(match patch {
            Some(patch) => format!("{minor}.{patch}."),
            None => format!("{minor}."),
        })
    } else {
        Some(format!("{major}."))
    }
}
