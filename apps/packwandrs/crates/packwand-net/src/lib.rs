//! One HTTP core for Packwand.
//!
//! Before this crate there were two independent stacks — one inside
//! `packwand-minecraft` for Mojang metadata and game files, another inside
//! `packwand-providers` for Modrinth and CurseForge — and only the second had
//! retries or per-host pacing, while neither revalidated anything. Every cold
//! boot refetched Mojang's version manifest in full, and downloads buffered
//! whole files in memory to hash them.
//!
//! What replaces that:
//!
//! * **Shared agents** ([`Profile`]), so the connection pool survives across
//!   operations, with separate API and transfer timeouts.
//! * **Per-host pacing** for providers that publish a request budget, and
//!   deliberately not for the CDNs that serve the actual files.
//! * **One retry policy**: `Retry-After` when the server sends it, jittered
//!   exponential backoff otherwise, and never on a 404.
//! * **Mirrors** — a [`Request`] can carry alternates, tried in order.
//! * **Verified streaming** ([`FileSink`]), so peak memory is one chunk and
//!   nothing unverified is ever visible at the target path.
//! * **A revalidating cache** ([`MetaCache`]) for documents that rarely change.
//!
//! Blocking on purpose. The launcher core is uniformly `ureq` plus threads
//! with a clean `spawn_blocking` boundary at the GUI, and the wins here come
//! from parallelising and caching rather than from a runtime.

#![forbid(unsafe_code)]

pub mod agent;
pub mod batch;
pub mod cache;
pub mod client;
pub mod error;
pub mod request;
pub mod retry;
pub mod sink;
#[cfg(feature = "testing")]
pub mod testing;

pub use agent::Profile;
pub use batch::{BatchProgress, BatchReport, Download, download_all};
pub use cache::MetaCache;
pub use client::{Client, Fetched, Freshness, ProgressFn, Source};
pub use error::NetError;
pub use request::{Checksum, Request};
pub use sink::{FileSink, staging_path};
