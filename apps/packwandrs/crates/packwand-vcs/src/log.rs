use std::collections::BTreeSet;
use std::path::Path;

use crate::{StackEntry, VcsError, with_repo};

const LOG_TEMPLATE: &str = r#"change_id ++ "\t" ++ commit_id ++ "\t" ++ description.first_line() ++ "\t" ++ if(current_working_copy, "true", "false") ++ "\t" ++ if(divergent, "true", "false") ++ "\t" ++ parents.map(|commit| commit.change_id()).join(",") ++ "\n""#;

/// Returns a flat newest-first stack suitable for CLI or client-side nesting.
pub fn stack_log(workspace_root: &Path) -> Result<Vec<StackEntry>, VcsError> {
	with_repo(workspace_root, |repository| {
		let output = repository.command(&[
			"log",
			"--no-graph",
			"-r",
			"ancestors(@, 100) & ~root()",
			"-T",
			LOG_TEMPLATE,
		])?;
		String::from_utf8_lossy(&output.stdout)
			.lines()
			.filter(|line| !line.trim().is_empty())
			.map(parse_entry)
			.collect()
	})
}

/// Returns files touched by the working-copy change, with a Git fallback for
/// repositories that have not enabled Jujutsu yet.
pub fn changed_paths(workspace_root: &Path) -> Result<Vec<String>, VcsError> {
	if workspace_root.join(".jj").is_dir() {
		let stack = stack_log(workspace_root)?;
		return with_repo(workspace_root, |repository| {
			let mut changed = BTreeSet::new();
			for entry in &stack {
				let output =
					repository.command(&["diff", "--name-only", "-r", &entry.commit_id])?;
				changed.extend(lines(&output.stdout));
			}
			Ok(changed.into_iter().collect())
		});
	}
	let output = std::process::Command::new("git")
		.args(["diff", "--name-only", "HEAD"])
		.current_dir(workspace_root)
		.output()?;
	if !output.status.success() {
		return Err(VcsError::Command(
			String::from_utf8_lossy(&output.stderr).trim().to_owned(),
		));
	}
	Ok(lines(&output.stdout))
}

fn lines(bytes: &[u8]) -> Vec<String> {
	String::from_utf8_lossy(bytes)
		.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty())
		.map(|line| line.replace('\\', "/"))
		.collect()
}

fn parse_entry(line: &str) -> Result<StackEntry, VcsError> {
	let fields = line.splitn(6, '\t').collect::<Vec<_>>();
	if fields.len() != 6 {
		return Err(VcsError::InvalidOutput(line.to_owned()));
	}
	Ok(StackEntry {
		change_id: fields[0].to_owned(),
		commit_id: fields[1].to_owned(),
		description: fields[2].to_owned(),
		is_working_copy: fields[3] == "true",
		divergent: fields[4] == "true",
		parent_change_id: fields[5]
			.split(',')
			.next()
			.filter(|value| !value.is_empty())
			.map(str::to_owned),
	})
}

#[cfg(test)]
mod tests {
	use super::parse_entry;

	#[test]
	fn parses_machine_readable_log_entry() {
		let entry = parse_entry("change\tcommit\tmessage\ttrue\tfalse\tparent").unwrap();
		assert_eq!(entry.change_id, "change");
		assert_eq!(entry.parent_change_id.as_deref(), Some("parent"));
		assert!(entry.is_working_copy);
	}
}
