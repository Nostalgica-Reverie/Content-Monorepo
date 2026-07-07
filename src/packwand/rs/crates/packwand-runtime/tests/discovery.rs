//! Discovery and selection tests against synthetic JDK layouts.

use std::path::{Path, PathBuf};

use packwand_runtime::{
    discover, inspect_java_home, select_compatible, DiscoveryConfig, DiscoverySource,
};

/// Creates a fake JDK home with a `bin/java` and a `release` file.
fn fake_jdk(root: &Path, name: &str, version: &str, vendor: &str) -> PathBuf {
    let home = root.join(name);
    let bin = home.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let exe = if cfg!(windows) { "java.exe" } else { "java" };
    std::fs::write(bin.join(exe), b"not a real jvm").unwrap();
    std::fs::write(
        home.join("release"),
        format!("JAVA_VERSION=\"{version}\"\nOS_ARCH=\"x86_64\"\nIMPLEMENTOR=\"{vendor}\"\n"),
    )
    .unwrap();
    home
}

#[test]
fn inspect_reads_release_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let home = fake_jdk(dir.path(), "jdk-21", "21.0.5", "Fixture Vendor");
    let install = inspect_java_home(&home, DiscoverySource::Explicit).unwrap();
    assert_eq!(install.major_version, 21);
    assert_eq!(install.version, "21.0.5");
    assert_eq!(install.vendor.as_deref(), Some("Fixture Vendor"));
    assert_eq!(install.architecture.as_deref(), Some("x86_64"));
    assert!(install.executable.is_file());
}

#[test]
fn inspect_rejects_non_jdk_directories() {
    let dir = tempfile::tempdir().unwrap();
    // Empty directory: no bin/java.
    assert!(inspect_java_home(dir.path(), DiscoverySource::Explicit).is_err());
    // bin/java but no release file.
    let home = fake_jdk(dir.path(), "broken", "17", "V");
    std::fs::remove_file(home.join("release")).unwrap();
    assert!(inspect_java_home(&home, DiscoverySource::Explicit).is_err());
}

#[test]
fn discover_finds_dedupes_and_orders_installations() {
    let dir = tempfile::tempdir().unwrap();
    let vendor_dir = dir.path().join("vendors");
    std::fs::create_dir_all(&vendor_dir).unwrap();
    let jdk8 = fake_jdk(&vendor_dir, "jdk-8", "1.8.0_392", "Legacy");
    let jdk21 = fake_jdk(&vendor_dir, "jdk-21", "21.0.5", "Modern");
    fake_jdk(&vendor_dir, "not-java-missing-release", "17", "X");
    std::fs::remove_file(vendor_dir.join("not-java-missing-release/release")).unwrap();

    let config = DiscoveryConfig {
        // JAVA_HOME points at one of the vendor JDKs: it must not appear twice.
        java_home: Some(jdk21.clone()),
        path_entries: vec![jdk8.join("bin")],
        vendor_dirs: vec![vendor_dir.clone(), dir.path().join("does-not-exist")],
    };
    let found = discover(&config);
    assert_eq!(found.len(), 2, "unexpected set: {found:#?}");
    // JAVA_HOME source is preferred, so the jdk-21 entry leads.
    assert_eq!(found[0].major_version, 21);
    assert_eq!(found[0].source, DiscoverySource::JavaHome);
    assert_eq!(found[1].major_version, 8);
    assert_eq!(found[1].source, DiscoverySource::PathEnv);
}

#[test]
fn selection_prefers_exact_major_then_next_above() {
    let dir = tempfile::tempdir().unwrap();
    let vendor_dir = dir.path().join("vendors");
    std::fs::create_dir_all(&vendor_dir).unwrap();
    for (name, version) in [
        ("jdk-8", "1.8.0_392"),
        ("jdk-17", "17.0.2"),
        ("jdk-25", "25"),
    ] {
        fake_jdk(&vendor_dir, name, version, "V");
    }
    let found = discover(&DiscoveryConfig {
        java_home: None,
        path_entries: vec![],
        vendor_dirs: vec![vendor_dir],
    });
    assert_eq!(found.len(), 3);

    assert_eq!(select_compatible(&found, 17).unwrap().major_version, 17);
    // No jdk-21: the smallest major above the requirement wins.
    assert_eq!(select_compatible(&found, 21).unwrap().major_version, 25);
    // Nothing at or above 26: selection fails and names what was found.
    let err = select_compatible(&found, 26).unwrap_err().to_string();
    assert!(err.contains("major version 26"), "{err}");
    assert!(err.contains("17.0.2"), "{err}");
}
