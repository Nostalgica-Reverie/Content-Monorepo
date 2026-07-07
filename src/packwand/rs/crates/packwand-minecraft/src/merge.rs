//! `inheritsFrom` resolution: overlaying a loader profile (Fabric, Quilt,
//! and NeoForge/Forge profiles use the same convention) onto its parent
//! vanilla version document.

use crate::model::{maven_collision_key, Arguments, VersionDoc};

/// Merges `child` (the inheriting loader profile) over `parent`.
///
/// - Scalars and objects present on the child win.
/// - Argument lists concatenate: parent first, child additions after.
/// - Libraries concatenate with the child taking precedence: a parent
///   library whose `group:artifact[:classifier]` collides with a child
///   library is dropped (the loader ships the newer artifact).
pub fn merge_inherited(parent: &VersionDoc, child: &VersionDoc) -> VersionDoc {
    let arguments = match (&parent.arguments, &child.arguments) {
        (None, None) => None,
        (Some(p), None) => Some(p.clone()),
        (None, Some(c)) => Some(c.clone()),
        (Some(p), Some(c)) => Some(Arguments {
            game: p.game.iter().chain(&c.game).cloned().collect(),
            jvm: p.jvm.iter().chain(&c.jvm).cloned().collect(),
        }),
    };

    let child_keys: Vec<String> = child
        .libraries
        .iter()
        .map(|l| maven_collision_key(&l.name))
        .collect();
    let mut libraries = child.libraries.clone();
    libraries.extend(
        parent
            .libraries
            .iter()
            .filter(|l| !child_keys.contains(&maven_collision_key(&l.name)))
            .cloned(),
    );

    VersionDoc {
        id: child.id.clone(),
        kind: child.kind.clone().or_else(|| parent.kind.clone()),
        inherits_from: None,
        main_class: child
            .main_class
            .clone()
            .or_else(|| parent.main_class.clone()),
        arguments,
        minecraft_arguments: child
            .minecraft_arguments
            .clone()
            .or_else(|| parent.minecraft_arguments.clone()),
        libraries,
        asset_index: child
            .asset_index
            .clone()
            .or_else(|| parent.asset_index.clone()),
        assets: child.assets.clone().or_else(|| parent.assets.clone()),
        downloads: if child.downloads.is_empty() {
            parent.downloads.clone()
        } else {
            child.downloads.clone()
        },
        logging: child.logging.clone().or_else(|| parent.logging.clone()),
        java_version: child
            .java_version
            .clone()
            .or_else(|| parent.java_version.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Argument, Library};

    fn lib(name: &str) -> Library {
        Library {
            name: name.to_string(),
            downloads: None,
            url: None,
            sha1: None,
            size: None,
            natives: Default::default(),
            rules: vec![],
            extract: None,
        }
    }

    #[test]
    fn child_overrides_colliding_parent_library() {
        let parent = VersionDoc {
            id: "1.21".to_string(),
            main_class: Some("net.minecraft.client.main.Main".to_string()),
            libraries: vec![lib("org.ow2.asm:asm:9.6"), lib("com.mojang:brigadier:1.2")],
            arguments: Some(Arguments {
                game: vec![Argument::Plain("--version".into())],
                jvm: vec![],
            }),
            ..VersionDoc::default()
        };
        let child = VersionDoc {
            id: "fabric-1.21".to_string(),
            inherits_from: Some("1.21".to_string()),
            main_class: Some("net.fabricmc.loader.impl.launch.knot.KnotClient".to_string()),
            libraries: vec![
                lib("org.ow2.asm:asm:9.7"),
                lib("net.fabricmc:fabric-loader:0.16"),
            ],
            arguments: Some(Arguments {
                game: vec![],
                jvm: vec![Argument::Plain("-DFabricMcEmu=true".into())],
            }),
            ..VersionDoc::default()
        };
        let merged = merge_inherited(&parent, &child);
        assert_eq!(merged.id, "fabric-1.21");
        assert_eq!(
            merged.main_class.as_deref(),
            Some("net.fabricmc.loader.impl.launch.knot.KnotClient")
        );
        let names: Vec<&str> = merged.libraries.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "org.ow2.asm:asm:9.7",
                "net.fabricmc:fabric-loader:0.16",
                "com.mojang:brigadier:1.2"
            ],
            "asm 9.6 must be shadowed by the child's 9.7"
        );
        assert!(merged.inherits_from.is_none());
        let args = merged.arguments.unwrap();
        assert_eq!(args.game.len(), 1);
        assert_eq!(args.jvm.len(), 1);
    }
}
