use std::error::Error;

use clap::ArgMatches;
use packwand_workspace::{ScriptPreset, ScriptRequest};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScriptReport {
	path: String,
	preset: String,
	project: Option<String>,
}

pub fn run(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
	let preset_name = args
		.get_one::<String>("preset")
		.map(String::as_str)
		.unwrap_or("build");
	let preset = ScriptPreset::parse(preset_name)
		.ok_or_else(|| format!("unsupported script preset {preset_name:?}"))?;
	let request = ScriptRequest {
		name: args
			.get_one::<String>("name")
			.cloned()
			.unwrap_or_else(|| "workspace".to_owned()),
		preset,
		project: args.get_one::<String>("project").cloned(),
		category: args
			.get_one::<String>("kind")
			.cloned()
			.unwrap_or_else(|| "modpack".to_owned()),
		loader: args.get_one::<String>("loader").cloned(),
		force: args.get_flag("force"),
	};
	let generated = packwand_workspace::generate_script(std::env::current_dir()?, &request)?;
	let report = ScriptReport {
		path: generated.path.display().to_string(),
		preset: preset_name.to_owned(),
		project: generated.project,
	};
	if args.get_flag("json") {
		println!("{}", serde_json::to_string_pretty(&report)?);
	} else {
		println!("generated {}", report.path);
	}
	Ok(())
}
