use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use clap::ArgMatches;
use packwand_identity_client::{IdentityClient, ManifestSummary, PackShare, StrongRef};
use packwand_workspace::Manifest;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

/// Publishes packs, snippets, and images to the signed-in repository.
pub fn share(args: &ArgMatches) -> Result {
	let client = IdentityClient::new()?;
	let (reference, json) = match args.subcommand() {
		Some(("pack", sub)) => (share_pack(&client, sub)?, sub.get_flag("json")),
		Some(("snippet", sub)) => {
			let path = required(sub, "file")?;
			let text = fs::read_to_string(path)?;
			if text.len() > 50_000 {
				return Err("snippet exceeds the 50,000-byte record limit".into());
			}
			(
				client
					.share_snippet(&text, sub.get_one::<String>("language").map(String::as_str))?,
				sub.get_flag("json"),
			)
		}
		Some(("image", sub)) => {
			let path = Path::new(required(sub, "file")?);
			let mime_type = sub
				.get_one::<String>("mime")
				.map(String::as_str)
				.map_or_else(|| image_mime_type(path), Ok)?;
			let blob = client.upload_blob(mime_type, &fs::read(path)?)?;
			(
				client.share_image(blob, sub.get_one::<String>("caption").map(String::as_str))?,
				sub.get_flag("json"),
			)
		}
		Some((name, _)) => return Err(format!("unknown share command {name:?}").into()),
		None => return Err("share requires pack, snippet, or image".into()),
	};
	print_reference(&reference, json)
}

/// Lists social contacts or publishes an addressed collaboration invite.
pub fn friends(args: &ArgMatches) -> Result {
	let client = IdentityClient::new()?;
	match args.subcommand() {
		Some(("list", sub)) => {
			let friends = client.list_friends()?;
			if sub.get_flag("json") {
				serde_json::to_writer_pretty(std::io::stdout(), &friends)?;
				println!();
			} else if friends.is_empty() {
				println!("no mutual follows or Packwand contacts");
			} else {
				for friend in friends {
					let label = if friend.handle.is_empty() {
						friend.did.clone()
					} else {
						format!("{} ({})", friend.handle, friend.did)
					};
					println!("{label} [{}]", friend.sources.join(", "));
				}
			}
			Ok(())
		}
		Some(("invite", sub)) => {
			let minutes = sub.get_one::<u64>("expires-in").copied().unwrap_or(60);
			let reference = client.send_invite(
				required(sub, "did")?,
				required(sub, "invite")?,
				Duration::from_secs(minutes.saturating_mul(60)),
			)?;
			print_reference(&reference, sub.get_flag("json"))
		}
		Some((name, _)) => Err(format!("unknown friends command {name:?}").into()),
		None => Err("friends requires list or invite".into()),
	}
}

fn share_pack(client: &IdentityClient, args: &ArgMatches) -> Result<StrongRef> {
	let root = Path::new(required(args, "dir")?);
	let manifest: Manifest = serde_json::from_slice(&fs::read(root.join("manifest.json"))?)?;
	let tangled_repo = args
		.get_one::<String>("tangled-repo")
		.map(|uri| find_tangled_reference(client, uri))
		.transpose()?;
	let supplied_remote = args.get_one::<String>("git-remote").cloned();
	let git_remote = supplied_remote.or_else(|| origin_remote(root));
	if tangled_repo.is_none() && git_remote.is_none() {
		return Err("pack sharing requires a Tangled repository or Git origin remote".into());
	}
	let summary = ManifestSummary {
		id: manifest.id.clone(),
		project_type: manifest.project_type.clone(),
		version: manifest.version.clone(),
		minecraft_version: manifest.mc_version.clone(),
		loader: manifest.loader.clone(),
		environment: manifest.environment.clone(),
		variants: manifest
			.variants
			.iter()
			.filter_map(|variant| variant.key().map(str::to_owned))
			.collect(),
	};
	Ok(client.share_pack(&PackShare {
		name: manifest.effective_name().to_owned(),
		description: manifest.description,
		manifest: summary,
		tangled_repo,
		git_remote,
	})?)
}

fn find_tangled_reference(client: &IdentityClient, uri: &str) -> Result<StrongRef> {
	let repositories = client.linked_tangled_repos()?;
	let repository = repositories
		.iter()
		.find(|repository| repository.uri == uri)
		.ok_or_else(|| format!("Tangled repository {uri:?} is not linked to the signed-in DID"))?;
	Ok(StrongRef {
		uri: repository.uri.clone(),
		cid: repository.cid.clone(),
	})
}

fn origin_remote(root: &Path) -> Option<String> {
	let output = Command::new("git")
		.args(["remote", "get-url", "origin"])
		.current_dir(root)
		.output()
		.ok()?;
	output
		.status
		.success()
		.then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
		.filter(|value| !value.is_empty())
}

fn image_mime_type(path: &Path) -> Result<&'static str> {
	match path
		.extension()
		.and_then(|extension| extension.to_str())
		.map(str::to_ascii_lowercase)
		.as_deref()
	{
		Some("png") => Ok("image/png"),
		Some("jpg" | "jpeg") => Ok("image/jpeg"),
		Some("webp") => Ok("image/webp"),
		Some("gif") => Ok("image/gif"),
		_ => Err("cannot infer image MIME type; pass --mime".into()),
	}
}

fn print_reference(reference: &StrongRef, json: bool) -> Result {
	if json {
		serde_json::to_writer_pretty(std::io::stdout(), reference)?;
		println!();
	} else {
		println!("{}", reference.uri);
		println!("CID: {}", reference.cid);
	}
	Ok(())
}

fn required<'a>(args: &'a ArgMatches, name: &str) -> Result<&'a str> {
	args.get_one::<String>(name)
		.map(String::as_str)
		.ok_or_else(|| format!("missing {name}").into())
}

#[cfg(test)]
mod tests {
	use super::image_mime_type;
	use std::path::Path;

	#[test]
	fn infers_supported_image_types() {
		assert_eq!(
			image_mime_type(Path::new("cover.PNG")).unwrap(),
			"image/png"
		);
		assert!(image_mime_type(Path::new("cover.bmp")).is_err());
	}
}
