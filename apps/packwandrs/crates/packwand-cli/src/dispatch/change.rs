use std::env;

use clap::ArgMatches;

use super::{Result, required};

pub(crate) fn change(args: &ArgMatches) -> Result {
	let root = env::current_dir()?;
	match args.subcommand() {
		Some(("enable", _)) => {
			packwand_vcs::enable_colocated(&root)?;
			println!("enabled colocated Jujutsu repository");
			Ok(())
		}
		Some(("new", sub)) => {
			let entry = packwand_vcs::new_change(
				&root,
				sub.get_one::<String>("parent").map(String::as_str),
			)?;
			if sub.get_flag("json") {
				println!("{}", serde_json::to_string_pretty(&entry)?);
			} else {
				println!(
					"{} {}",
					entry.change_id,
					display_description(&entry.description)
				);
			}
			Ok(())
		}
		Some(("describe", sub)) => {
			packwand_vcs::describe(
				&root,
				required(sub, "change-id")?,
				required(sub, "message")?,
			)?;
			Ok(())
		}
		Some(("squash", sub)) => {
			packwand_vcs::squash(
				&root,
				required(sub, "change-id")?,
				sub.get_flag("into-parent"),
			)?;
			Ok(())
		}
		Some(("log", sub)) => {
			let entries = packwand_vcs::stack_log(&root)?;
			if sub.get_flag("json") {
				println!("{}", serde_json::to_string_pretty(&entries)?);
			} else {
				for entry in entries {
					let marker = if entry.is_working_copy { "@" } else { "o" };
					let divergent = if entry.divergent { " divergent" } else { "" };
					println!(
						"{marker} {} {}{divergent}",
						entry.change_id,
						display_description(&entry.description)
					);
				}
			}
			Ok(())
		}
		_ => Err("change requires enable, new, describe, squash, or log".into()),
	}
}

fn display_description(description: &str) -> &str {
	if description.is_empty() {
		"(no description set)"
	} else {
		description
	}
}
