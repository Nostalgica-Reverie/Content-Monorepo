//! The wire protocol and its framing.
//!
//! One closed enum. This is the entire attack surface a guest is given, and
//! there is deliberately **no** generic "invoke a Tauri command" message: any
//! capability a guest should have must be spelled out here as its own variant,
//! which makes adding one a decision somebody has to make on purpose.
//!
//! In particular the `accounts_*`, `shell_*`, `settings_*`, `instances_*` and
//! `packeater_*` command families have no representation here at all. The
//! first of those is the important one — it is what touches the OS keychain.

use serde::{Deserialize, Serialize};

/// A stable per-session identifier for one participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ParticipantId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Participant {
	pub id: ParticipantId,
	pub display_name: String,
	/// From `git config user.name`, for the `Co-authored-by:` trailer.
	#[serde(default)]
	pub git_name: String,
	/// From `git config user.email`, likewise.
	#[serde(default)]
	pub git_email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
	/// The one pack this session is scoped to. A guest can reach nothing else.
	pub pack_id: String,
	pub pack_name: String,
	/// Whether guests may stage, commit, pull or push on the host.
	pub allow_git_write: bool,
}

/// An error crossing the wire.
///
/// Mirrors the app's own `SerializableError { kind, message }` so the existing
/// `helpers/errors.ts:normalizeBridgeError` handles a proxied failure exactly
/// as it handles a local one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteError {
	pub kind: String,
	pub message: String,
}

/// A filesystem call, named exactly as the Tauri command it proxies.
///
/// Carrying the method as a string keeps this in lockstep with
/// `helpers/invoke/editor.ts` by construction, and the host matches it against
/// a closed list before dispatching — an unknown method is a protocol error,
/// never a pass-through.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsRequest {
	pub method: String,
	pub parameters: serde_json::Value,
}

/// Every filesystem method a guest may invoke.
///
/// Exhaustive and checked on the host. Adding to this list is the only way to
/// widen what a guest can do to the host's disk.
pub const PROXYABLE_FS_METHODS: [&str; 10] = [
	"editor_document_read",
	"editor_document_write",
	"editor_fs_stat",
	"editor_fs_read_dir",
	"editor_fs_read_file",
	"editor_fs_write_file",
	"editor_fs_create_dir",
	"editor_fs_delete",
	"editor_fs_rename",
	"editor_search",
];

/// Every git method a guest may invoke, and whether it writes.
///
/// `git_clone` and `git_init` are absent because they create repositories and
/// are meaningless against a session already scoped to one. `git_checkout`
/// would move the host's working tree under them. `git_set_identity` and
/// `git_remote_add` write the host's config. None of those is proxyable at any
/// permission level.
pub const PROXYABLE_GIT_METHODS: [(&str, bool); 9] = [
	("git_status", false),
	("git_diff", false),
	("git_diff_document", false),
	("git_log", false),
	("git_branches", false),
	("git_repository", false),
	("git_stage", true),
	("git_unstage", true),
	("git_commit", true),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
	pub start: usize,
	pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
	Created,
	Modified,
	Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Message {
	Hello {
		participant: Participant,
		protocol: u32,
	},
	Welcome {
		session: SessionInfo,
		participants: Vec<Participant>,
	},
	ParticipantJoined(Participant),
	ParticipantLeft(ParticipantId),

	FsRequest {
		id: u64,
		request: FsRequest,
	},
	FsResponse {
		id: u64,
		result: Result<serde_json::Value, RemoteError>,
	},
	FsChanged {
		path: String,
		kind: ChangeKind,
	},

	DocOpen {
		path: String,
	},
	DocClose {
		path: String,
	},
	DocSnapshot {
		path: String,
		revision: u64,
		text: String,
	},
	DocEdit {
		path: String,
		base_revision: u64,
		ops: Vec<crate::ot::TextOp>,
	},
	DocApplied {
		path: String,
		revision: u64,
		ops: Vec<crate::ot::TextOp>,
		origin: ParticipantId,
	},
	DocSave {
		path: String,
	},

	Presence {
		path: Option<String>,
		selections: Vec<Selection>,
	},
	FollowRequest {
		target: ParticipantId,
	},

	GitRequest {
		id: u64,
		method: String,
		parameters: serde_json::Value,
	},
	GitResponse {
		id: u64,
		result: Result<serde_json::Value, RemoteError>,
	},
	Output {
		channel: String,
		line: String,
	},
	Problems {
		snapshot: serde_json::Value,
	},
	JobEvent {
		event: String,
		payload: serde_json::Value,
	},
}

/// The protocol version carried in [`Message::Hello`].
///
/// A mismatch is refused rather than negotiated: two builds of a desktop app
/// that disagree about the wire format should say so plainly instead of
/// half-working.
pub const PROTOCOL_VERSION: u32 = 1;

/// Hard cap on a single frame.
///
/// Without this, a hostile or simply buggy peer can announce a 4 GiB length
/// and force the host to allocate it before any validation happens. 8 MiB is
/// far above any real message — the largest is a document snapshot — and far
/// below anything that threatens the process.
pub const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
	#[error("frame of {0} bytes exceeds the {MAX_FRAME_BYTES} byte limit")]
	TooLarge(usize),
	#[error("malformed frame: {0}")]
	Malformed(String),
	#[error(transparent)]
	Io(#[from] std::io::Error),
}

/// Length-prefixed JSON framing: `u32` little-endian length, then the body.
pub struct Frame;

impl Frame {
	/// Serializes a message into a length-prefixed frame.
	pub fn encode(message: &Message) -> Result<Vec<u8>, FrameError> {
		let body = serde_json::to_vec(message)
			.map_err(|error| FrameError::Malformed(error.to_string()))?;
		if body.len() > MAX_FRAME_BYTES {
			return Err(FrameError::TooLarge(body.len()));
		}
		let mut frame = Vec::with_capacity(body.len() + 4);
		frame.extend_from_slice(&(body.len() as u32).to_le_bytes());
		frame.extend_from_slice(&body);
		Ok(frame)
	}

	/// Reads one frame from a blocking reader.
	pub fn decode(reader: &mut impl std::io::Read) -> Result<Message, FrameError> {
		let mut length = [0u8; 4];
		reader.read_exact(&mut length)?;
		let length = u32::from_le_bytes(length) as usize;
		// Checked *before* allocating, which is the entire point.
		if length > MAX_FRAME_BYTES {
			return Err(FrameError::TooLarge(length));
		}
		let mut body = vec![0u8; length];
		reader.read_exact(&mut body)?;
		serde_json::from_slice(&body).map_err(|error| FrameError::Malformed(error.to_string()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_frame_round_trips() {
		let message = Message::Hello {
			participant: Participant {
				id: ParticipantId(7),
				display_name: "omo".into(),
				git_name: "omo50".into(),
				git_email: "omo@example.com".into(),
			},
			protocol: PROTOCOL_VERSION,
		};
		let encoded = Frame::encode(&message).unwrap();
		let decoded = Frame::decode(&mut encoded.as_slice()).unwrap();
		assert_eq!(decoded, message);
	}

	/// The allocation guard. An announced length over the cap must be refused
	/// before a buffer that size is created.
	#[test]
	fn an_oversized_length_prefix_is_refused_without_allocating() {
		let mut frame = Vec::new();
		frame.extend_from_slice(&u32::MAX.to_le_bytes());
		let error = Frame::decode(&mut frame.as_slice()).unwrap_err();
		assert!(matches!(error, FrameError::TooLarge(_)));
	}

	#[test]
	fn a_truncated_frame_is_an_error_not_a_hang() {
		let mut frame = Vec::new();
		frame.extend_from_slice(&64u32.to_le_bytes());
		frame.extend_from_slice(b"only a few bytes");
		assert!(Frame::decode(&mut frame.as_slice()).is_err());
	}

	#[test]
	fn a_body_that_is_not_a_known_message_is_rejected() {
		let body = br#"{"type":"somethingElse","payload":1}"#;
		let mut frame = (body.len() as u32).to_le_bytes().to_vec();
		frame.extend_from_slice(body);
		assert!(matches!(
			Frame::decode(&mut frame.as_slice()),
			Err(FrameError::Malformed(_))
		));
	}

	/// The closed-surface guarantee. If these ever contain a command that can
	/// reach the keychain, a shell, or the settings store, the session has
	/// stopped being a file proxy.
	#[test]
	fn the_proxyable_surface_excludes_every_dangerous_family() {
		let all: Vec<&str> = PROXYABLE_FS_METHODS
			.iter()
			.copied()
			.chain(PROXYABLE_GIT_METHODS.iter().map(|(name, _)| *name))
			.collect();
		for method in &all {
			for forbidden in [
				"accounts_",
				"shell_",
				"settings_",
				"instances_",
				"packeater_",
				"themes_",
				"workspace_",
			] {
				assert!(
					!method.starts_with(forbidden),
					"{method} must not be proxyable"
				);
			}
		}
		// The two repository-creating commands and the three that write host
		// config or move the host's tree are absent by name.
		for absent in [
			"git_clone",
			"git_init",
			"git_checkout",
			"git_set_identity",
			"git_remote_add",
		] {
			assert!(!all.contains(&absent), "{absent} must not be proxyable");
		}
	}

	#[test]
	fn only_the_three_write_git_methods_are_marked_as_writes() {
		let writes: Vec<&str> = PROXYABLE_GIT_METHODS
			.iter()
			.filter(|(_, writes)| *writes)
			.map(|(name, _)| *name)
			.collect();
		assert_eq!(writes, ["git_stage", "git_unstage", "git_commit"]);
	}
}
