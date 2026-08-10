use std::error::Error;

use clap::ArgMatches;
use packwand_identity_client::{Identity, IdentityClient};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub fn account(args: &ArgMatches) -> Result {
	let client = IdentityClient::new()?;
	match args.subcommand() {
		Some(("login", _)) => print_identity(&client.login(None)?, false),
		Some(("whoami", sub)) => match client.whoami()? {
			Some(identity) => print_identity(&identity, sub.get_flag("json")),
			None if sub.get_flag("json") => {
				println!("null");
				Ok(())
			}
			None => {
				println!("not signed in");
				Ok(())
			}
		},
		Some(("logout", _)) => {
			client.logout()?;
			println!("signed out");
			Ok(())
		}
		Some((name, _)) => Err(format!("unknown account command {name:?}").into()),
		None => Err("account requires login, whoami, or logout".into()),
	}
}

fn print_identity(identity: &Identity, json: bool) -> Result {
	if json {
		serde_json::to_writer_pretty(std::io::stdout(), identity)?;
		println!();
	} else {
		println!("{} ({})", identity.handle, identity.did);
		println!("PDS: {}", identity.pds);
	}
	Ok(())
}
