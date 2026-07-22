//! Pack metadata and hashing shared by the future Packwand CLI and desktop app.

#![forbid(unsafe_code)]

mod hash;
mod model;

pub use hash::{DEFAULT_HASH_FORMAT, HashError, HashFormat, Hasher, hash_bytes, hash_file};
pub use model::{
    CURRENT_PACK_FORMAT, Download, Index, IndexFile, Mod, ModOption, Pack, PackFormat,
    PackFormatError, PackIndex,
};
