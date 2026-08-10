use packwand_instance::FsUserInstanceRepository;
use serde::Deserialize;

use crate::error::Result;
use crate::paths::{backing_pack, safe_content_path};

/// Which picture an instance card wants.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageKind {
	Icon,
	Background,
}

impl ImageKind {
	const fn file_name(self) -> &'static str {
		match self {
			Self::Icon => "icon.png",
			Self::Background => "bg.png",
		}
	}
}

/// Reads an instance's icon or background.
///
/// Falls back to the backing pack's art, so a linked instance looks like the
/// pack it came from without anyone copying files around. An instance-local
/// image always wins, which is what makes overriding it possible.
pub fn read(repo: &FsUserInstanceRepository, id: &str, kind: ImageKind) -> Result<Option<Vec<u8>>> {
	let instance = repo.get(id)?;
	let instance_root = repo.instance_dir(id)?;
	let pack_root = backing_pack(repo, &instance)?;
	let mut candidates = Vec::new();
	if matches!(kind, ImageKind::Icon)
		&& let Some(icon) = &instance.icon
	{
		candidates.push(safe_content_path(&instance_root, icon)?);
	}
	candidates.push(instance_root.join(kind.file_name()));
	candidates.push(pack_root.join(kind.file_name()));
	for path in candidates {
		match std::fs::read(&path) {
			Ok(bytes) => return Ok(Some(bytes)),
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
			Err(error) => return Err(error.into()),
		}
	}
	Ok(None)
}

#[cfg(test)]
mod tests {
	use super::*;
	use packwand_instance::{Instance, InstanceSource};

	#[test]
	fn pack_art_is_the_fallback_and_instance_art_wins() {
		let root = tempfile::tempdir().unwrap();
		let pack = tempfile::tempdir().unwrap();
		std::fs::write(pack.path().join("icon.png"), b"pack icon").unwrap();
		std::fs::write(pack.path().join("bg.png"), b"pack background").unwrap();
		let repo = FsUserInstanceRepository::new(root.path().to_path_buf());
		let instance = Instance::new(
			"visual".into(),
			"Visual".into(),
			InstanceSource::Linked {
				pack_dir: pack.path().to_path_buf(),
			},
			"1.21.1".into(),
			"fabric".into(),
			None,
			0,
		);
		repo.create(&instance).unwrap();

		assert_eq!(
			read(&repo, "visual", ImageKind::Icon).unwrap(),
			Some(b"pack icon".to_vec())
		);
		assert_eq!(
			read(&repo, "visual", ImageKind::Background).unwrap(),
			Some(b"pack background".to_vec())
		);

		let local = repo.instance_dir("visual").unwrap();
		std::fs::create_dir_all(&local).unwrap();
		std::fs::write(local.join("icon.png"), b"local icon").unwrap();
		assert_eq!(
			read(&repo, "visual", ImageKind::Icon).unwrap(),
			Some(b"local icon".to_vec())
		);
	}
}
