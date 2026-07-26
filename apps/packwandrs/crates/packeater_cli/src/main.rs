//! Marker-driven command-line frontend for Packeater.

#![forbid(unsafe_code)]

mod config;
mod discovery;

use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use config::{MARKER_NAME, PackeaterConfig};
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
				run_marker(&marker, None, arguments.dry_run)?;
			}
			Ok(())
		}
		Mode::Config(marker) => run_marker(&marker, arguments.output.as_deref(), arguments.dry_run),
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

fn run_marker(marker: &Path, output: Option<&Path>, dry_run: bool) -> Result<(), String> {
	if marker.file_name().and_then(|name| name.to_str()) != Some(MARKER_NAME) {
		return Err(format!(
			"JSON marker must be named {MARKER_NAME}: {}",
			marker.display()
		));
	}
	let pack_directory = marker
		.parent()
		.ok_or_else(|| format!("{} has no parent folder", marker.display()))?;
	let config = PackeaterConfig::read(marker)?;
	if !config.enabled {
		println!("Skipping disabled pack {}", pack_directory.display());
		return Ok(());
	}
	let output = output
		.map(Path::to_path_buf)
		.unwrap_or_else(|| config.output_path(pack_directory));
	let normalized_pack_directory = absolute_lexical(pack_directory)?;
	let output = absolute_lexical(&output)?;
	if output.starts_with(&normalized_pack_directory) {
		return Err(format!(
			"output {} must be outside the source pack folder {}",
			output.display(),
			pack_directory.display()
		));
	}
	if dry_run {
		println!(
			"Would eat {} -> {} (aggressive compression, lossy PNG={}, lossy audio={})",
			pack_directory.display(),
			output.display(),
			config.lossy.png,
			config.lossy.audio
		);
		return Ok(());
	}
	if let Some(parent) = output.parent() {
		fs::create_dir_all(parent)
			.map_err(|error| format!("could not create {}: {error}", parent.display()))?;
	}
	println!(
		"Eating {} -> {}",
		pack_directory.display(),
		output.display()
	);
	let options = config.squash_options(&normalized_pack_directory, &output)?;
	optimize(options)?;
	let bytes = fs::metadata(&output)
		.map_err(|error| format!("could not inspect {}: {error}", output.display()))?
		.len();
	println!("Packed {} bytes into {}", bytes, output.display());
	Ok(())
}

fn absolute_lexical(path: &Path) -> Result<PathBuf, String> {
	let path = if path.is_absolute() {
		path.to_path_buf()
	} else {
		env::current_dir()
			.map_err(|error| format!("could not resolve current directory: {error}"))?
			.join(path)
	};
	let mut normalized = PathBuf::new();
	for component in path.components() {
		match component {
			Component::CurDir => {}
			Component::ParentDir => {
				normalized.pop();
			}
			Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
				normalized.push(component.as_os_str());
			}
		}
	}
	Ok(normalized)
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

	#[test]
	fn parent_output_is_normalized_outside_the_pack() {
		let pack = absolute_lexical(Path::new("packs/example")).unwrap();
		let output = absolute_lexical(Path::new("packs/example/../dist/result.zip")).unwrap();
		assert!(!output.starts_with(pack));
	}
}
