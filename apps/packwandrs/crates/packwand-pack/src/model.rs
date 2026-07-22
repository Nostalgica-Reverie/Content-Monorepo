use std::collections::BTreeMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub const CURRENT_PACK_FORMAT: &str = "packwand:26";
const PACKWAND_GENERATION: u32 = 26;

fn default_index_file() -> String {
    "index.toml".to_string()
}

fn default_hash_format() -> String {
    "sha512".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackFormat {
    Packwand(u32),
    Packwiz { major: u32, minor: u32, patch: u32 },
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PackFormatError {
    #[error("pack-format does not indicate a valid packwiz or packwand pack")]
    Unknown,
    #[error("pack-format generation is not a valid integer")]
    InvalidGeneration,
    #[error("pack-format field is not valid semver")]
    InvalidVersion,
    #[error("pack-format packwand:{found} predates the minimum supported generation ({minimum})")]
    OldGeneration { found: u32, minimum: u32 },
    #[error("the modpack is incompatible with this version of packwand")]
    IncompatiblePackwiz,
}

impl FromStr for PackFormat {
    type Err = PackFormatError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(generation) = value.strip_prefix("packwand:") {
            let generation = generation
                .parse::<u32>()
                .map_err(|_| PackFormatError::InvalidGeneration)?;
            if generation < PACKWAND_GENERATION {
                return Err(PackFormatError::OldGeneration {
                    found: generation,
                    minimum: PACKWAND_GENERATION,
                });
            }
            return Ok(Self::Packwand(generation));
        }
        if let Some(version) = value.strip_prefix("packwiz:") {
            let parts = version
                .split('.')
                .map(str::parse::<u32>)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| PackFormatError::InvalidVersion)?;
            let [major, minor, patch] = parts.as_slice() else {
                return Err(PackFormatError::InvalidVersion);
            };
            if *major != 1 {
                return Err(PackFormatError::IncompatiblePackwiz);
            }
            return Ok(Self::Packwiz {
                major: *major,
                minor: *minor,
                patch: *patch,
            });
        }
        Err(PackFormatError::Unknown)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackIndex {
    #[serde(default = "default_index_file")]
    pub file: String,
    #[serde(default = "default_hash_format")]
    pub hash_format: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hash: String,
}

impl Default for PackIndex {
    fn default() -> Self {
        Self {
            file: default_index_file(),
            hash_format: default_hash_format(),
            hash: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Pack {
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub author: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default)]
    pub pack_format: String,
    #[serde(default)]
    pub index: PackIndex,
    #[serde(default)]
    pub versions: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub export: BTreeMap<String, toml::Table>,
    #[serde(default, skip_serializing_if = "toml::Table::is_empty")]
    pub options: toml::Table,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub scripts: BTreeMap<String, String>,
}

impl Pack {
    pub fn format(&self) -> Result<PackFormat, PackFormatError> {
        let format = if self.pack_format.is_empty() {
            "packwiz:1.1.0"
        } else {
            &self.pack_format
        };
        PackFormat::from_str(format)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Index {
    #[serde(default = "default_hash_format")]
    pub hash_format: String,
    #[serde(default)]
    pub files: Vec<IndexFile>,
}

impl Default for Index {
    fn default() -> Self {
        Self {
            hash_format: default_hash_format(),
            files: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct IndexFile {
    pub file: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub metafile: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub preserve: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Mod {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub side: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub pin: bool,
    #[serde(default)]
    pub download: Download,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub update: BTreeMap<String, toml::Table>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option: Option<ModOption>,
}

impl Mod {
    /// Encodes with the explicit `[update]` parent table emitted by the Go
    /// BurntSushi encoder. TOML treats the implicit and explicit forms alike,
    /// but index hashes require the bytes to stay identical during the port.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        let mut encoded = toml::to_string(self)?;
        if !self.update.is_empty()
            && let Some(position) = encoded.find("\n[update.")
        {
            encoded.insert_str(position + 1, "[update]\n");
        }
        Ok(encoded)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Download {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    #[serde(default)]
    pub hash_format: String,
    #[serde(default)]
    pub hash: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra_hashes: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub size: u64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mode: String,
}

const fn is_zero(value: &u64) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModOption {
    #[serde(default)]
    pub optional: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub default: bool,
}
