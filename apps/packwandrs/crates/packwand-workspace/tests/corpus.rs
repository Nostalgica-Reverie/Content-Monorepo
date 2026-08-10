use std::path::PathBuf;

#[test]
fn parses_every_repository_manifest_project() {
	let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.ancestors()
		.nth(4)
		.expect("crate lives below the repository root")
		.to_path_buf();
	let projects = packwand_workspace::discover(&root).unwrap();
	// Count drops whenever a pack is retired — `chore(sm): remove sm` took this
	// from 40 to 39 — so update it alongside the removal.
	assert_eq!(projects.len(), 39);
	assert!(
		projects
			.iter()
			.any(|project| project.manifest.id == "re-console-main")
	);
	assert!(
		projects
			.iter()
			.all(|project| project.root.join("manifest.json").is_file())
	);
}
