use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Arg, Command};
use packwand_installer::{InstallSide, ManualDownload};

fn main() -> ExitCode {
	let matches = Command::new("packwand-installer")
		.about("Install a remote Packwand or packwiz content pack")
		.arg(
			Arg::new("side")
				.long("side")
				.short('s')
				.default_value("client")
				.value_parser(["client", "server"]),
		)
		.arg(
			Arg::new("game-dir")
				.long("game-dir")
				.short('g')
				.value_name("PATH")
				.default_value("."),
		)
		.arg(Arg::new("pack-url").required(true))
		.get_matches();
	let side = matches
		.get_one::<String>("side")
		.expect("defaulted side")
		.parse::<InstallSide>()
		.expect("validated side");
	let game_dir = PathBuf::from(
		matches
			.get_one::<String>("game-dir")
			.expect("defaulted game directory"),
	);
	match packwand_installer::install(
		matches
			.get_one::<String>("pack-url")
			.expect("required pack URL"),
		&game_dir,
		side,
	) {
		Ok(plan) => {
			println!("applied {} installer actions", plan.actions.len());
			if let Err(error) = write_manual_pending(&game_dir, &plan.manual) {
				eprintln!("packwand-installer: failed to record pending manual downloads: {error}");
			}
			if !plan.manual.is_empty() {
				eprintln!("{} mod(s) need manual download:", plan.manual.len());
				for pending in &plan.manual {
					eprintln!("  - {} -> {}", pending.name, pending.target.display());
					if let Some(url) = &pending.page_url {
						eprintln!("    {url}");
					}
				}
			}
			ExitCode::SUCCESS
		}
		Err(error) => {
			eprintln!("packwand-installer: {error}");
			ExitCode::FAILURE
		}
	}
}

/// Records what still needs a human, so the GUI can surface it instead of
/// marking the instance simply "Ready" after a successful exit. Always
/// rewritten (even to an empty list) so a mod resolved by hand since the
/// last run drops off.
fn write_manual_pending(game_dir: &Path, manual: &[ManualDownload]) -> std::io::Result<()> {
	let dir = game_dir.join(".packwand-installer");
	fs::create_dir_all(&dir)?;
	let bytes = serde_json::to_vec_pretty(manual)
		.map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
	fs::write(dir.join("manual-pending.json"), bytes)
}
