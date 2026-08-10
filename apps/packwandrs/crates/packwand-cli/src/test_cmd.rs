use std::error::Error;
use std::path::{Path, PathBuf};

use clap::ArgMatches;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub fn run(args: &ArgMatches) -> Result {
	let pack = absolute(
		args.get_one::<String>("pack-subdir")
			.ok_or("missing pack subdir")?,
	)?;
	let instance = std::env::var_os("PACKWAND_TEST_INSTANCE")
		.map(PathBuf::from)
		.unwrap_or(std::env::current_dir()?.join(".packwand-test-instance"));
	let report =
		packwand_build::test_with_installer(&pack, &instance).map_err(|error| error.to_string())?;
	println!(
		"validated {} ({} actions)",
		report.pack.display(),
		report.actions
	);
	for pending in &report.manual {
		println!("manual download required: {}", pending.name);
	}
	println!("test instance ready at {}", report.instance.display());
	Ok(())
}

fn absolute(path: impl AsRef<Path>) -> Result<PathBuf> {
	let path = path.as_ref();
	Ok(if path.is_absolute() {
		path.to_path_buf()
	} else {
		std::env::current_dir()?.join(path)
	})
}
