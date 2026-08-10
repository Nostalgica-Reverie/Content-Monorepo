use std::path::{Component, Path};

use packwand_pack::{HashFormat, Index, Mod, Pack, hash_bytes};
use packwand_providers::{HttpRequest, Transport};
use url::Url;

use crate::InstallerError;

/// Parsed pack metadata and its canonical remote base URL.
pub struct RemotePack {
	pub pack: Pack,
	pub index: Index,
	pub base: Url,
}

impl RemotePack {
	/// Fetches and verifies a pack file and its referenced index.
	pub fn load(url: &str, transport: &dyn Transport) -> Result<Self, InstallerError> {
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
		})
	}

	/// Fetches and verifies one index entry.
	pub fn entry(
		&self,
		path: &str,
		format: &str,
		hash: &str,
		transport: &dyn Transport,
	) -> Result<Vec<u8>, InstallerError> {
		let url = join(&self.base, path)?;
		let bytes = fetch(transport, url)?;
		verify(path, format, hash, &bytes)?;
		Ok(bytes)
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
