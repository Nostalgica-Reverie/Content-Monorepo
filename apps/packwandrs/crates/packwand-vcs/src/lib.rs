//! Short-lived Jujutsu operations for Packwand's CLI and desktop shell.

#![forbid(unsafe_code)]

mod change;
pub mod jj_toolchain;
mod log;
mod repo;

pub use change::{describe, enable_colocated, new_change, squash};
pub use log::changed_paths;
pub use log::stack_log;
pub use repo::{Repository, VcsError, configure_jj_binary, with_repo};

use serde::{Deserialize, Serialize};

/// One commit in the visible stacked-change graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackEntry {
	pub change_id: String,
	pub commit_id: String,
	pub description: String,
	pub is_working_copy: bool,
	pub divergent: bool,
	pub parent_change_id: Option<String>,
}
