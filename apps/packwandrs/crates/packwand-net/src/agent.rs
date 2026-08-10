use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

const API_READ_TIMEOUT: Duration = Duration::from_secs(60);
/// Bounds metadata/API responses: large enough for any provider payload,
/// small enough that a runaway response cannot exhaust memory.
const API_MAX_BODY_BYTES: u64 = 16 * 1024 * 1024;

/// Generous enough that a real download over a slow link is never cut off,
/// while a hung connection still fails eventually.
const DOWNLOAD_READ_TIMEOUT: Duration = Duration::from_secs(600);
/// Sized for jars rather than JSON.
const DOWNLOAD_MAX_BODY_BYTES: u64 = 1024 * 1024 * 1024;

/// Which kind of request an agent is tuned for.
///
/// The split is not cosmetic: one agent means one read timeout, and applying
/// a transfer-scale timeout to metadata calls hides a dead API behind a
/// ten-minute wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
	/// Metadata and API calls.
	Api,
	/// File transfers.
	Download,
}

impl Profile {
	/// The response ceiling for this profile, in bytes.
	pub const fn max_body_bytes(self) -> u64 {
		match self {
			Self::Api => API_MAX_BODY_BYTES,
			Self::Download => DOWNLOAD_MAX_BODY_BYTES,
		}
	}
}

fn build(read_timeout: Duration) -> ureq::Agent {
	ureq::AgentBuilder::new()
		.timeout_connect(CONNECT_TIMEOUT)
		.timeout_read(read_timeout)
		.user_agent(concat!("packwand/", env!("CARGO_PKG_VERSION")))
		.build()
}

/// The process-wide API agent.
///
/// Callers build a fresh client per operation, and an agent owns the
/// connection pool — so constructing one per client threw away every
/// keep-alive connection and made each operation pay a fresh TCP and TLS
/// handshake against the same host. `ureq::Agent` is a cheap `Arc` handle, so
/// cloning one shared agent gives every client the same pool.
static API_AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| build(API_READ_TIMEOUT));

static DOWNLOAD_AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| build(DOWNLOAD_READ_TIMEOUT));

/// The shared agent for `profile`.
pub fn agent(profile: Profile) -> ureq::Agent {
	match profile {
		Profile::Api => API_AGENT.clone(),
		Profile::Download => DOWNLOAD_AGENT.clone(),
	}
}

/// Minimum spacing between attempts against a host.
///
/// Only providers with a documented request budget get an entry, so the
/// client stays under the limit instead of discovering it through 429s.
/// Modrinth allows 300 req/min; 220ms (~272 req/min) leaves headroom for
/// retries and other processes.
///
/// **CDN hosts must stay absent.** `cdn.modrinth.com` and the CurseForge edge
/// serve the actual files, and pacing those would serialize every download
/// worker — which is the entire cost this crate exists to remove.
fn host_rate_interval(host: &str) -> Option<Duration> {
	match host {
		"api.modrinth.com" | "staging-api.modrinth.com" => Some(Duration::from_millis(220)),
		_ => None,
	}
}

/// Next free slot per host, on a monotonic timeline. Global rather than
/// per-client because callers construct a fresh client per operation —
/// per-instance state would reset the pacing on every call and defeat the
/// budget.
static HOST_GATES: LazyLock<Mutex<HashMap<String, Instant>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));

/// The host part of a URL, without pulling in a URL parser for what is a
/// prefix scan.
fn host_of(url: &str) -> Option<&str> {
	let rest = url.split_once("://")?.1;
	let authority = rest.split(['/', '?', '#']).next()?;
	let authority = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
	Some(match authority.rsplit_once(':') {
		// Not a port if it is IPv6 (`[::1]`) or non-numeric.
		Some((head, port)) if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) => head,
		_ => authority,
	})
}

/// Blocks until the request's host allows another attempt, by handing each
/// caller the next free slot. Contended threads queue in slot order rather
/// than thundering at once. Hosts without a configured budget pass through
/// untouched.
pub fn wait_host_rate_limit(url: &str) {
	let Some(interval) = host_of(url).and_then(host_rate_interval) else {
		return;
	};
	let host = host_of(url).expect("checked above").to_owned();
	let now = Instant::now();
	let at = {
		let mut gates = HOST_GATES
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner);
		let slot = gates.entry(host).or_insert(now);
		let at = if *slot < now { now } else { *slot };
		*slot = at + interval;
		at
	};
	let delay = at.saturating_duration_since(now);
	if !delay.is_zero() {
		std::thread::sleep(delay);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn hosts_parse_out_of_realistic_urls() {
		assert_eq!(
			host_of("https://api.modrinth.com/v2/x"),
			Some("api.modrinth.com")
		);
		assert_eq!(
			host_of("https://cdn.modrinth.com/data/a.jar"),
			Some("cdn.modrinth.com")
		);
		assert_eq!(
			host_of("http://127.0.0.1:8080/pack.toml"),
			Some("127.0.0.1")
		);
		assert_eq!(host_of("https://user@example.com/x"), Some("example.com"));
		assert_eq!(host_of("not a url"), None);
	}

	#[test]
	fn only_documented_api_hosts_are_paced() {
		assert!(host_rate_interval("api.modrinth.com").is_some());
		// The regression that would matter: pacing the CDN serializes every
		// download worker.
		assert!(host_rate_interval("cdn.modrinth.com").is_none());
		assert!(host_rate_interval("edge.forgecdn.net").is_none());
		assert!(host_rate_interval("piston-meta.mojang.com").is_none());
	}
}
