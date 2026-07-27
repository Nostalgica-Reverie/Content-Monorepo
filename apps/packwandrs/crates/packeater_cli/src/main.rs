//! Marker-driven command-line frontend for Packeater.

#![forbid(unsafe_code)]

mod discovery;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use packeater_cli::{MARKER_NAME, run_marker};
use packsquash::{PackSquasher, config::SquashOptions, vfs::os_fs::OsFilesystem};

#[derive(Debug)]
struct Arguments {
	mode: Mode,
	output: Option<PathBuf>,
	dry_run: bool
}

#[derive(Debug)]
enum Mode {
	Discover(PathBuf),
	Config(PathBuf),
	LegacyToml(PathBuf)
}

fn main() -> ExitCode {
	match run() {
		Ok(()) => ExitCode::SUCCESS,
		Err(error) => {
			eprintln!("packeater: {error}");
			ExitCode::FAILURE
		}
	}
}

fn run() -> Result<(), String> {
	let Some(arguments) = parse_arguments(env::args().skip(1).collect())? else {
		return Ok(());
	};
	match arguments.mode {
		Mode::Discover(root) => {
			if arguments.output.is_some() {
				return Err("--output can only be used with one packeater.json file".into());
			}
			let markers = discovery::discover(&root)?;
			if markers.is_empty() {
				println!("No {MARKER_NAME} folders found under {}", root.display());
				return Ok(());
			}
			for marker in markers {
				if arguments.dry_run {
					println!("Would eat {}", marker.display());
				} else {
					run_marker(&marker, None)?;
				}
			}
			Ok(())
		}
		Mode::Config(marker) => {
			if arguments.dry_run {
				println!("Would eat {}", marker.display());
				Ok(())
			} else {
				run_marker(&marker, arguments.output.as_deref()).map(|_| ())
			}
		}
		Mode::LegacyToml(path) => {
			if arguments.dry_run {
				println!(
					"Would optimize legacy PackSquash options from {}",
					path.display()
				);
				return Ok(());
			}
			let source = fs::read_to_string(&path)
				.map_err(|error| format!("could not read {}: {error}", path.display()))?;
			let mut options: SquashOptions = toml::from_str(&source)
				.map_err(|error| format!("could not parse {}: {error}", path.display()))?;
			if let Some(output) = arguments.output {
				options.global_options.output_file_path = output;
			}
			optimize(options)
		}
	}
}

fn optimize(options: SquashOptions) -> Result<(), String> {
	PackSquasher::new()
		.run(OsFilesystem, options, None)
		.map_err(|error| error.to_string())
}

fn parse_arguments(values: Vec<String>) -> Result<Option<Arguments>, String> {
	let mut path = None;
	let mut output = None;
	let mut discover = false;
	let mut dry_run = false;
	let mut index = 0;
	while index < values.len() {
		match values[index].as_str() {
			"-h" | "--help" => {
				print_help();
				return Ok(None);
			}
			"-V" | "--version" => {
				println!(
					"packeater {} (forked from PackSquash 0.4.1)",
					env!("CARGO_PKG_VERSION")
				);
				return Ok(None);
			}
			"--discover" => discover = true,
			"--dry-run" => dry_run = true,
			"-o" | "--output" => {
				index += 1;
				output = Some(PathBuf::from(
					values.get(index).ok_or("--output requires a path")?
				));
			}
			value if value.starts_with('-') => return Err(format!("unknown option {value}")),
			value => {
				if path.replace(PathBuf::from(value)).is_some() {
					return Err("only one input path may be specified".into());
				}
			}
		}
		index += 1;
	}
	let path = path.unwrap_or_else(|| PathBuf::from("."));
	let mode = if discover || path.is_dir() {
		Mode::Discover(path)
	} else if path.extension().and_then(|extension| extension.to_str()) == Some("toml") {
		Mode::LegacyToml(path)
	} else {
		Mode::Config(path)
	};
	Ok(Some(Arguments {
		mode,
		output,
		dry_run
	}))
}

fn print_help() {
	println!(
		"Packeater - aggressive Minecraft pack optimization\n\n\
Usage:\n  packeater [OPTIONS] [packeater.json]\n  packeater --discover [ROOT]\n  packeater [OPTIONS] packsquash.toml\n\n\
Options:\n  --discover       Recursively process folders containing packeater.json\n  -o, --output     Override output path for a single marker or legacy TOML\n  --dry-run        Print selected folders and outputs without processing\n  -h, --help       Print help\n  -V, --version    Print version\n\n\
With no arguments, Packeater discovers marker files below the current directory."
	);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn directories_imply_discovery() {
		let arguments = parse_arguments(vec![".".into(), "--dry-run".into()])
			.unwrap()
			.unwrap();
		assert!(matches!(arguments.mode, Mode::Discover(_)));
		assert!(arguments.dry_run);
	}

	#[test]
	fn json_file_selects_one_config() {
		let arguments = parse_arguments(vec!["packs/example/packeater.json".into()])
			.unwrap()
			.unwrap();
		assert!(matches!(arguments.mode, Mode::Config(_)));
	}
}
