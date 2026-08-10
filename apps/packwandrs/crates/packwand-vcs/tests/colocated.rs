use std::process::Command;

#[test]
fn colocated_initialization_runs_in_process_without_jj_cli() {
	let repository = tempfile::tempdir().expect("repository directory");
	let initialized = Command::new("git")
		.args(["init", "--initial-branch", "main"])
		.current_dir(repository.path())
		.status()
		.expect("run git init");
	assert!(initialized.success());
	packwand_vcs::enable_colocated(repository.path()).expect("enable colocated repository");
	assert!(repository.path().join(".jj").is_dir());
}

#[test]
#[ignore = "downloads the pinned official Jujutsu release"]
fn managed_jj_runs_colocated_change_lifecycle() {
	let tools = tempfile::tempdir().expect("tool directory");
	let request = packwand_devboot::jj_toolchain::JjToolchainRequest::pinned(tools.path().into());
	let binary = packwand_devboot::jj_toolchain::ensure_jj(&request, |_| {})
		.expect("download managed Jujutsu");
	packwand_vcs::configure_jj_binary(binary).expect("configure managed Jujutsu");

	let repository = tempfile::tempdir().expect("repository directory");
	let initialized = Command::new("git")
		.args(["init", "--initial-branch", "main"])
		.current_dir(repository.path())
		.status()
		.expect("run git init");
	assert!(initialized.success());
	packwand_vcs::enable_colocated(repository.path()).expect("enable colocated repository");
	let created = packwand_vcs::new_change(repository.path(), None).expect("create change");
	packwand_vcs::describe(repository.path(), &created.change_id, "integration change")
		.expect("describe change");
	std::fs::write(repository.path().join("changed.txt"), "stacked change\n")
		.expect("write changed file");
	let stack = packwand_vcs::stack_log(repository.path()).expect("read stack");
	assert!(
		stack
			.iter()
			.any(|entry| entry.description == "integration change")
	);
	assert_eq!(
		packwand_vcs::changed_paths(repository.path()).expect("read changed paths"),
		vec!["changed.txt"]
	);
}
