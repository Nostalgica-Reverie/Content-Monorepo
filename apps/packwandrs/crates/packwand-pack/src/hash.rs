use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

pub const DEFAULT_HASH_FORMAT: HashFormat = HashFormat::Sha512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashFormat {
	Sha1,
	Sha256,
	Sha512,
	Md5,
	Murmur2,
	LengthBytes,
}

impl HashFormat {
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::Sha1 => "sha1",
			Self::Sha256 => "sha256",
			Self::Sha512 => "sha512",
			Self::Md5 => "md5",
			Self::Murmur2 => "murmur2",
			Self::LengthBytes => "length-bytes",
		}
	}

	pub const fn is_internal(self) -> bool {
		matches!(self, Self::LengthBytes)
	}
}

impl FromStr for HashFormat {
	type Err = HashError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value.to_ascii_lowercase().as_str() {
			"sha1" => Ok(Self::Sha1),
			"sha256" => Ok(Self::Sha256),
			"sha512" => Ok(Self::Sha512),
			"md5" => Ok(Self::Md5),
			"murmur2" => Ok(Self::Murmur2),
			"length-bytes" => Ok(Self::LengthBytes),
			_ => Err(HashError::UnknownFormat(value.to_string())),
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum HashError {
	#[error("hash implementation {0} not found")]
	UnknownFormat(String),
	#[error("failed to read {path}: {source}")]
	Io { path: PathBuf, source: io::Error },
}

enum HasherState {
	Sha1(Sha1),
	Sha256(Sha256),
	Sha512(Sha512),
	Md5(Md5),
	Murmur2(Vec<u8>),
	Length(u64),
}

/// Incremental hasher with the same accepted format names and output strings
/// as the Go oracle.
pub struct Hasher {
	state: HasherState,
}

impl Hasher {
	pub fn new(format: HashFormat) -> Self {
		let state = match format {
			HashFormat::Sha1 => HasherState::Sha1(Sha1::new()),
			HashFormat::Sha256 => HasherState::Sha256(Sha256::new()),
			HashFormat::Sha512 => HasherState::Sha512(Sha512::new()),
			HashFormat::Md5 => HasherState::Md5(Md5::new()),
			HashFormat::Murmur2 => HasherState::Murmur2(Vec::new()),
			HashFormat::LengthBytes => HasherState::Length(0),
		};
		Self { state }
	}

	pub fn update(&mut self, bytes: &[u8]) {
		match &mut self.state {
			HasherState::Sha1(hasher) => Digest::update(hasher, bytes),
			HasherState::Sha256(hasher) => Digest::update(hasher, bytes),
			HasherState::Sha512(hasher) => Digest::update(hasher, bytes),
			HasherState::Md5(hasher) => Digest::update(hasher, bytes),
			HasherState::Murmur2(buffer) => buffer.extend(
				bytes
					.iter()
					.copied()
					.filter(|byte| !matches!(byte, 9 | 10 | 13 | 32)),
			),
			HasherState::Length(length) => *length += bytes.len() as u64,
		}
	}

	pub fn finish(self) -> String {
		match self.state {
			HasherState::Sha1(hasher) => hex::encode(hasher.finalize()),
			HasherState::Sha256(hasher) => hex::encode(hasher.finalize()),
			HasherState::Sha512(hasher) => hex::encode(hasher.finalize()),
			HasherState::Md5(hasher) => hex::encode(hasher.finalize()),
			HasherState::Murmur2(buffer) => murmur_hash2(&buffer, 1).to_string(),
			HasherState::Length(length) => length.to_string(),
		}
	}
}

pub fn hash_bytes(format: HashFormat, bytes: &[u8]) -> String {
	let mut hasher = Hasher::new(format);
	hasher.update(bytes);
	hasher.finish()
}

pub fn hash_file(format: HashFormat, path: &Path) -> Result<String, HashError> {
	let mut file = File::open(path).map_err(|source| HashError::Io {
		path: path.to_path_buf(),
		source,
	})?;
	let mut hasher = Hasher::new(format);
	let mut buffer = [0u8; 64 * 1024];
	loop {
		let read = file.read(&mut buffer).map_err(|source| HashError::Io {
			path: path.to_path_buf(),
			source,
		})?;
		if read == 0 {
			break;
		}
		hasher.update(&buffer[..read]);
	}
	Ok(hasher.finish())
}

fn murmur_hash2(mut data: &[u8], seed: u32) -> u32 {
	const M: u32 = 0x5bd1_e995;
	let mut hash = seed ^ data.len() as u32;
	while data.len() >= 4 {
		let mut value = u32::from_le_bytes(data[..4].try_into().expect("four bytes"));
		value = value.wrapping_mul(M);
		value ^= value >> 24;
		value = value.wrapping_mul(M);
		hash = hash.wrapping_mul(M) ^ value;
		data = &data[4..];
	}
	match data {
		[a, b, c] => {
			hash ^= u32::from(*c) << 16;
			hash ^= u32::from(*b) << 8;
			hash ^= u32::from(*a);
			hash = hash.wrapping_mul(M);
		}
		[a, b] => {
			hash ^= u32::from(*b) << 8;
			hash ^= u32::from(*a);
			hash = hash.wrapping_mul(M);
		}
		[a] => {
			hash ^= u32::from(*a);
			hash = hash.wrapping_mul(M);
		}
		[] => {}
		_ => unreachable!(),
	}
	hash ^= hash >> 13;
	hash = hash.wrapping_mul(M);
	hash ^ (hash >> 15)
}
