use std::path::{Component, Path, PathBuf};

use packwand_pack::{HashFormat, Index, Mod, Pack, hash_bytes};
use packwand_providers::{HttpRequest, Transport};
use url::Url;

use crate::InstallerError;

/// How one of the pack's own (non-metafile) entries reaches the instance.
/// A remote pack serves them over HTTP; a pack already on disk does not need
/// the network at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSource {
	Url(String),
	Path(PathBuf),
}

/// A pack's metadata and a way to read the files it references, whichever
/// side of the network it lives on.
pub trait PackFiles {
	/// The parsed index.
	fn index(&self) -> &Index;
	/// Verified bytes of one index entry, used to read mod metafiles while
	/// planning.
	fn entry(&self, path: &str, format: &str, hash: &str) -> Result<Vec<u8>, InstallerError>;
	/// Where a non-metafile entry is read from when the plan is applied.
	fn file_source(&self, path: &str) -> Result<FileSource, InstallerError>;
}

/// Parsed pack metadata and its canonical remote base URL.
pub struct RemotePack<'a> {
	pub pack: Pack,
	pub index: Index,
	pub base: Url,
	transport: &'a dyn Transport,
}

impl<'a> RemotePack<'a> {
	/// Fetches and verifies a pack file and its referenced index.
	pub fn load(url: &str, transport: &'a dyn Transport) -> Result<Self, InstallerError> {
		let pack_url =
			Url::parse(url).map_err(|error| InstallerError::InvalidUrl(error.to_string()))?;
		if !matches!(pack_url.scheme(), "http" | "https") {
			return Err(InstallerError::InvalidUrl(
				"only HTTP(S) pack URLs are supported".into(),
			));
		}
		let bytes = fetch(transport, pack_url.clone())?;
		let pack: Pack = toml::from_str(
			std::str::from_utf8(&bytes)
				.map_err(|error| InstallerError::Decode(error.to_string()))?,
		)
		.map_err(|error| InstallerError::Decode(error.to_string()))?;
		let index_url = join(&pack_url, &pack.index.file)?;
		let index_bytes = fetch(transport, index_url)?;
		verify(
			&pack.index.file,
			&pack.index.hash_format,
			&pack.index.hash,
			&index_bytes,
		)?;
		let index = decode_index(&index_bytes)?;
		Ok(Self {
			pack,
			index,
			base: pack_url,
			transport,
		})
	}
}

impl PackFiles for RemotePack<'_> {
	fn index(&self) -> &Index {
		&self.index
	}

	fn entry(&self, path: &str, format: &str, hash: &str) -> Result<Vec<u8>, InstallerError> {
		let url = join(&self.base, path)?;
		let bytes = fetch(self.transport, url)?;
		verify(path, format, hash, &bytes)?;
		Ok(bytes)
	}

	fn file_source(&self, path: &str) -> Result<FileSource, InstallerError> {
		Ok(FileSource::Url(join(&self.base, path)?.to_string()))
	}
}

/// A pack read straight from a directory on this machine.
///
/// The index is a generated artifact under `packwand:27`, so whatever
/// produces a `LocalPack` is responsible for refreshing it first —
/// see [`crate::install_local`].
pub struct LocalPack {
	pub pack: Pack,
	pub index: Index,
	pub root: PathBuf,
}

impl LocalPack {
	/// Reads and verifies `pack.toml` and its referenced index from `root`.
	pub fn load(root: &Path) -> Result<Self, InstallerError> {
		let pack_path = root.join("pack.toml");
		let bytes = std::fs::read(&pack_path)?;
		let pack: Pack = toml::from_str(
			std::str::from_utf8(&bytes)
				.map_err(|error| InstallerError::Decode(error.to_string()))?,
		)
		.map_err(|error| InstallerError::Decode(error.to_string()))?;
		let index_bytes = std::fs::read(root.join(safe_relative(&pack.index.file)?))?;
		verify(
			&pack.index.file,
			&pack.index.hash_format,
			&pack.index.hash,
			&index_bytes,
		)?;
		let index = decode_index(&index_bytes)?;
		Ok(Self {
			pack,
			index,
			root: root.to_path_buf(),
		})
	}
}

impl PackFiles for LocalPack {
	fn index(&self) -> &Index {
		&self.index
	}

	fn entry(&self, path: &str, format: &str, hash: &str) -> Result<Vec<u8>, InstallerError> {
		let bytes = std::fs::read(self.root.join(safe_relative(path)?))?;
		verify(path, format, hash, &bytes)?;
		Ok(bytes)
	}

	fn file_source(&self, path: &str) -> Result<FileSource, InstallerError> {
		Ok(FileSource::Path(self.root.join(safe_relative(path)?)))
	}
}

pub fn decode_mod(bytes: &[u8]) -> Result<Mod, InstallerError> {
	if bytes
		.iter()
		.copied()
		.find(|byte| !byte.is_ascii_whitespace())
		== Some(b'{')
	{
		serde_json::from_slice(bytes).map_err(|error| InstallerError::Decode(error.to_string()))
	} else {
		toml::from_str(
			std::str::from_utf8(bytes)
				.map_err(|error| InstallerError::Decode(error.to_string()))?,
		)
		.map_err(|error| InstallerError::Decode(error.to_string()))
	}
}

pub fn safe_relative(path: &str) -> Result<&Path, InstallerError> {
	let path = Path::new(path);
	if path.as_os_str().is_empty()
		|| path
			.components()
			.any(|component| !matches!(component, Component::Normal(_)))
	{
		return Err(InstallerError::InvalidPath(path.display().to_string()));
	}
	Ok(path)
}

fn decode_index(bytes: &[u8]) -> Result<Index, InstallerError> {
	if bytes
		.iter()
		.copied()
		.find(|byte| !byte.is_ascii_whitespace())
		== Some(b'{')
	{
		serde_json::from_slice(bytes).map_err(|error| InstallerError::Decode(error.to_string()))
	} else {
		toml::from_str(
			std::str::from_utf8(bytes)
				.map_err(|error| InstallerError::Decode(error.to_string()))?,
		)
		.map_err(|error| InstallerError::Decode(error.to_string()))
	}
}

fn fetch(transport: &dyn Transport, url: Url) -> Result<Vec<u8>, InstallerError> {
	transport
		.get_large(HttpRequest::get(url.to_string()))
		.map_err(|error| InstallerError::Transport(error.to_string()))
}

fn join(base: &Url, path: &str) -> Result<Url, InstallerError> {
	safe_relative(path)?;
	base.join(&path.replace('\\', "/"))
		.map_err(|error| InstallerError::InvalidUrl(error.to_string()))
}

pub fn verify(
	path: &str,
	format: &str,
	expected: &str,
	bytes: &[u8],
) -> Result<(), InstallerError> {
	if expected.is_empty() {
		return Ok(());
	}
	let format = format
		.parse::<HashFormat>()
		.map_err(|error| InstallerError::Decode(error.to_string()))?;
	let actual = hash_bytes(format, bytes);
	if actual.eq_ignore_ascii_case(expected) {
		Ok(())
	} else {
		Err(InstallerError::HashMismatch {
			path: path.into(),
			expected: expected.into(),
			actual,
		})
	}
}
