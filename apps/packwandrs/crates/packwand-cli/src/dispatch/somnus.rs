use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::ArgMatches;
use packwand_identity_client::IdentityClient;

use super::Result;

pub(crate) fn somnus(args: &ArgMatches) -> Result {
	let root = repository_root()?;
	let binary = find_binary(&root)?;
	let changed = packwand_vcs::changed_paths(&root)
		.unwrap_or_default()
		.join(",");
	let (name, sub) = args
		.subcommand()
		.ok_or("somnus requires run, list, or status")?;
	let mut command = Command::new(binary);
	command
		.arg(name)
		.args(["--root", root.to_string_lossy().as_ref()]);
	if matches!(name, "run" | "list") {
		command.args(["--changed-paths", &changed]);
	}
	if sub
		.try_get_one::<bool>("json")
		.ok()
		.flatten()
		.copied()
		.unwrap_or(false)
	{
		command.arg("--json");
	}
	if let Some(workflow) = sub.try_get_one::<String>("workflow").ok().flatten() {
		command.arg(workflow);
	}
	let status = command
		.stdin(Stdio::inherit())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.status()?;
	if !status.success() {
		return Err(format!("somnus failed with {status}").into());
	}
	if name == "run" {
		report_status(&root);
	}
	Ok(())
}

fn report_status(root: &Path) {
	let Ok(client) = IdentityClient::new() else {
		return;
	};
	if !matches!(client.whoami(), Ok(Some(_))) {
		return;
	}
	let Ok(bytes) = fs::read(root.join(".somnus/status.json")) else {
		return;
	};
	let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
		return;
	};
	let Some(records) = value.get("records").and_then(serde_json::Value::as_array) else {
		return;
	};
	for record in records {
		if let Err(error) = client.create_record("sh.tangled.pipeline.status", None, record.clone())
		{
			eprintln!("warning: could not publish Somnus status: {error}");
		}
	}
}

fn repository_root() -> Result<PathBuf> {
	let output = Command::new("git")
		.args(["rev-parse", "--show-toplevel"])
		.output()?;
	if !output.status.success() {
		return Err("Somnus must run inside a Git repository".into());
	}
	Ok(PathBuf::from(String::from_utf8(output.stdout)?.trim()))
}

fn find_binary(root: &Path) -> Result<PathBuf> {
	let executable = if cfg!(windows) {
		"somnus.exe"
	} else {
		"somnus"
	};
	let candidates = std::env::var_os("PACKWAND_SOMNUS_BIN")
		.map(PathBuf::from)
		.into_iter()
		.chain([
			root.join(executable),
			root.join("apps/packwandrs/target/release").join(executable),
			root.join("apps/packwandrs/somnus").join(executable),
		])
		.collect::<Vec<_>>();
	for candidate in candidates {
		if candidate.is_file() {
			return Ok(candidate.canonicalize()?);
		}
	}
	Err("somnus was not found; set PACKWAND_SOMNUS_BIN".into())
}
