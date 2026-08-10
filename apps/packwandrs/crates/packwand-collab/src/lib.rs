//! Host-authoritative live collaboration for the Packwand IDE.
//!
//! One person hosts, another joins with an invite code, and the guest works
//! inside the host's workspace — the host's files, the host's git, the host's
//! checkout. There is exactly one copy of the work, so there is nothing to
//! merge. See `markdowns/multiplayer.md` for the full design.
//!
//! This crate is deliberately free of Tauri, tokio and async. The socket is a
//! blocking `TcpListener` on its own thread, following
//! `packwand-cli`'s `serve::serve_listener`, and surfaces to the app over IPC.

#![forbid(unsafe_code)]

pub mod invite;
pub mod ot;
pub mod protocol;
pub mod transport;

pub use invite::{Invite, InviteError};
pub use ot::{TextOp, apply, transform, transform_all};
pub use protocol::{Frame, FrameError, Message, ParticipantId};
pub use transport::{Role, Session, SessionHandle, TransportError, TransportEvent};
