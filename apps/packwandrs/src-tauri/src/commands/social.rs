use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use packwand_identity_client::{
	Friend, IdentityClient, ManifestSummary, PackShare, PendingInvite, StrongRef, TangledRepo,
};
use packwand_workspace::Manifest;
use tauri::State;

use crate::commands::{off_thread, packs};
use crate::error::{CommandResult, SerializableError};
use crate::state::AppState;

/// Lists mutual follows and explicit Packwand contacts.
#[tauri::command]
pub async fn social_friends() -> CommandResult<Vec<Friend>> {
	off_thread(|| social_client()?.list_friends().map_err(social_error)).await
}

/// Lists addressed, unexpired collaboration invites from friends' repositories.
#[tauri::command]
pub async fn social_pending_invites() -> CommandResult<Vec<PendingInvite>> {
	off_thread(|| {
		social_client()?
			.list_pending_invites()
			.map_err(social_error)
	})
	.await
}

/// Lists Tangled repositories linked to the signed-in DID.
#[tauri::command]
pub async fn social_linked_tangled_repos() -> CommandResult<Vec<TangledRepo>> {
	off_thread(|| {
		social_client()?
			.linked_tangled_repos()
			.map_err(social_error)
	})
	.await
}

/// Publishes a live collaboration invite addressed to a friend's DID.
#[tauri::command]
pub async fn social_send_invite(
	to: String,
	invite: String,
	expires_in_minutes: Option<u64>,
) -> CommandResult<StrongRef> {
	let minutes = expires_in_minutes.unwrap_or(60).clamp(1, 10_080);
	off_thread(move || {
		social_client()?
			.send_invite(&to, &invite, Duration::from_secs(minutes * 60))
			.map_err(social_error)
	})
	.await
}

/// Publishes the selected project's manifest summary and repository link.
#[tauri::command]
pub async fn social_share_pack(
	pack_id: String,
	tangled_repo: Option<String>,
	git_remote: Option<String>,
	state: State<'_, AppState>,
) -> CommandResult<StrongRef> {
	let workspace = state.workspace()?;
	let pack_root = packs::pack_root(&workspace, &pack_id)?;
	off_thread(move || share_pack(&workspace, &pack_root, tangled_repo, git_remote)).await
}

/// Publishes a text snippet.
#[tauri::command]
pub async fn social_share_snippet(
	text: String,
	language: Option<String>,
) -> CommandResult<StrongRef> {
	if text.len() > 50_000 {
		return Err(SerializableError::new(
			"atproto_social",
			"snippet exceeds the 50,000-byte record limit",
		));
	}
	off_thread(move || {
		social_client()?
			.share_snippet(&text, language.as_deref())
			.map_err(social_error)
	})
	.await
}

/// Uploads and publishes an image selected by the user.
#[tauri::command]
pub async fn social_share_image(
	path: String,
	caption: Option<String>,
	mime_type: Option<String>,
) -> CommandResult<StrongRef> {
	off_thread(move || {
		let path = PathBuf::from(path);
		let mime_type = mime_type
			.as_deref()
			.map(str::to_owned)
			.map_or_else(|| infer_image_mime(&path).map(str::to_owned), Ok)?;
		let client = social_client()?;
		let blob = client
			.upload_blob(&mime_type, &fs::read(path)?)
			.map_err(social_error)?;
		client
			.share_image(blob, caption.as_deref())
			.map_err(social_error)
	})
	.await
}

fn share_pack(
	workspace: &Path,
	pack_root: &Path,
	tangled_uri: Option<String>,
	git_remote: Option<String>,
) -> CommandResult<StrongRef> {
	let manifest_path = manifest_path(workspace, pack_root).ok_or_else(|| {
		SerializableError::new(
			"manifest_not_found",
			"the selected pack is not under a project manifest.json",
		)
	})?;
	let project_root = manifest_path.parent().ok_or_else(|| {
		SerializableError::new("manifest_not_found", "manifest.json has no parent")
	})?;
	let manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
	let client = social_client()?;
	let tangled_repo = tangled_uri
		.as_deref()
		.map(|uri| resolve_tangled(&client, uri))
		.transpose()?;
	let git_remote = git_remote.or_else(|| origin_remote(project_root));
	if tangled_repo.is_none() && git_remote.is_none() {
		return Err(SerializableError::new(
			"atproto_social",
			"pack sharing requires a Tangled repository or Git origin remote",
		));
	}
	client
		.share_pack(&PackShare {
			name: manifest.effective_name().to_owned(),
			description: manifest.description,
			manifest: ManifestSummary {
				id: manifest.id,
				project_type: manifest.project_type,
				version: manifest.version,
				minecraft_version: manifest.mc_version,
				loader: manifest.loader,
				environment: manifest.environment,
				variants: manifest
					.variants
					.iter()
					.filter_map(|variant| variant.key().map(str::to_owned))
					.collect(),
			},
			tangled_repo,
			git_remote,
		})
		.map_err(social_error)
}

fn manifest_path(workspace: &Path, pack_root: &Path) -> Option<PathBuf> {
	pack_root
		.ancestors()
		.take_while(|path| path.starts_with(workspace))
		.map(|path| path.join("manifest.json"))
		.find(|path| path.is_file())
}

fn resolve_tangled(client: &IdentityClient, uri: &str) -> CommandResult<StrongRef> {
	let repositories = client.linked_tangled_repos().map_err(social_error)?;
	let repository = repositories
		.iter()
		.find(|repository| repository.uri == uri)
		.ok_or_else(|| {
			SerializableError::new(
				"atproto_social",
				format!("Tangled repository {uri:?} is not linked to the signed-in DID"),
			)
		})?;
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

fn infer_image_mime(path: &Path) -> CommandResult<&'static str> {
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
		_ => Err(SerializableError::new(
			"atproto_social",
			"cannot infer image MIME type",
		)),
	}
}

fn social_client() -> CommandResult<IdentityClient> {
	IdentityClient::new().map_err(social_error)
}

fn social_error(error: packwand_identity_client::Error) -> SerializableError {
	SerializableError::new("atproto_social", error.to_string())
}

#[cfg(test)]
mod tests {
	use super::{infer_image_mime, manifest_path};
	use std::fs;

	#[test]
	fn finds_nearest_project_manifest() {
		let workspace = tempfile::tempdir().unwrap();
		let project = workspace.path().join("modpacks/example");
		let pack = project.join("1.21-mr");
		fs::create_dir_all(&pack).unwrap();
		fs::write(project.join("manifest.json"), "{}").unwrap();
		assert_eq!(
			manifest_path(workspace.path(), &pack),
			Some(project.join("manifest.json"))
		);
	}

	#[test]
	fn accepts_supported_image_extensions() {
		assert_eq!(
			infer_image_mime(Path::new("cover.webp")).unwrap(),
			"image/webp"
		);
		assert!(infer_image_mime(Path::new("cover.bmp")).is_err());
	}

	use std::path::Path;
}
