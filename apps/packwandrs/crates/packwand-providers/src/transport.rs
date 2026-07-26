use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
}

impl HttpRequest {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: Vec::new(),
        }
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("GET {url} failed: {message}")]
pub struct TransportError {
    pub url: String,
    pub message: String,
    /// HTTP status code, when the failure was a non-2xx response rather than
    /// a connection/transport-level failure.
    pub status: Option<u16>,
    /// First ~500 characters of the response body, when available. Lets
    /// callers distinguish a real API error payload from a generic
    /// CDN/WAF block page (e.g. CloudFront's static "Request blocked" HTML),
    /// which returns the same status code but never reaches the actual API.
    pub body_snippet: Option<String>,
}

const API_READ_TIMEOUT: Duration = Duration::from_secs(60);
/// Bounds metadata/API responses: large enough for any provider payload,
/// small enough that a runaway response cannot exhaust memory.
const API_MAX_BODY_BYTES: u64 = 16 * 1024 * 1024;

/// Transfers are generous enough that a real download over a slow link is
/// never cut off, but a hung connection still fails eventually.
const DOWNLOAD_READ_TIMEOUT: Duration = Duration::from_secs(600);
/// Downloaded artifacts are buffered in memory to be hashed, so they still
/// need a ceiling — just one sized for jars rather than JSON.
const DOWNLOAD_MAX_BODY_BYTES: u64 = 1024 * 1024 * 1024;

const RETRY_MAX_ATTEMPTS: u32 = 3;
const RETRY_INITIAL_BACKOFF: Duration = Duration::from_millis(500);
/// Bounds how long a server-provided Retry-After header can stall a worker; a
/// hostile or misconfigured server must not be able to park a slot for
/// minutes.
const RETRY_AFTER_CAP: Duration = Duration::from_secs(60);

/// Minimum spacing between attempts against a host. Providers with a
/// documented request budget get an entry here so the client stays under the
/// limit instead of discovering it through 429s. Modrinth allows 300 req/min;
/// 220ms spacing (~272 req/min) leaves headroom for retries and other
/// processes.
fn host_rate_interval(host: &str) -> Option<Duration> {
    match host {
        "api.modrinth.com" | "staging-api.modrinth.com" => Some(Duration::from_millis(220)),
        _ => None,
    }
}

/// Next free slot per host, on a monotonic timeline. Global rather than
/// per-transport because callers construct a fresh `UreqTransport` per
/// operation — per-instance state would reset the pacing on every call and
/// defeat the budget.
static HOST_GATES: LazyLock<Mutex<HashMap<String, Instant>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Blocks until the request's host allows another attempt by handing each
/// caller the next free slot. Contended threads queue in slot order rather
/// than thundering at once. Hosts without a configured budget pass through
/// untouched.
fn wait_host_rate_limit(url: &str) {
    let Some(host) = url::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_owned))
    else {
        return;
    };
    let Some(interval) = host_rate_interval(&host) else {
        return;
    };
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

/// Jitter source for backoff. Not cryptographic — it only needs to stop
/// workers that were rate-limited together from retrying in lockstep.
static JITTER_STATE: AtomicU64 = AtomicU64::new(0);

fn next_random() -> u64 {
    let mut state = JITTER_STATE.load(Ordering::Relaxed);
    if state == 0 {
        state = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15)
            | 1;
    }
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    JITTER_STATE.store(state, Ordering::Relaxed);
    state
}

/// Spreads `backoff` over [0.5x, 1.5x).
fn jittered(backoff: Duration) -> Duration {
    let base = backoff.as_millis().max(1) as u64;
    Duration::from_millis(base / 2 + next_random() % base)
}

/// Parses a Retry-After header value (either delta-seconds or an HTTP-date
/// per RFC 9110 section 10.2.3). Returns `None` if absent or unparseable.
fn retry_after_delay(value: &str) -> Option<Duration> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(seconds) = value.parse::<i64>() {
        return (seconds > 0).then(|| Duration::from_secs(seconds as u64));
    }
    let target = parse_http_date(value)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
    let remaining = target - now;
    (remaining > 0).then(|| Duration::from_secs(remaining as u64))
}

/// Parses an IMF-fixdate ("Sun, 06 Nov 1994 08:49:37 GMT") into epoch
/// seconds. The other two RFC 9110 formats are obsolete and not accepted.
fn parse_http_date(value: &str) -> Option<i64> {
    let rest = value.split_once(", ")?.1;
    let mut fields = rest.split(' ');
    let day: i64 = fields.next()?.parse().ok()?;
    let month = match fields.next()? {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };
    let year: i64 = fields.next()?.parse().ok()?;
    let mut clock = fields.next()?.split(':');
    let hour: i64 = clock.next()?.parse().ok()?;
    let minute: i64 = clock.next()?.parse().ok()?;
    let second: i64 = clock.next()?.parse().ok()?;
    if !(1..=31).contains(&day) || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// Days since 1970-01-01 (Howard Hinnant's civil-from-days inverse).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Whether a failed attempt should be retried: connection-level failures,
/// 429, and 5xx are transient.
fn is_transient(error: &ureq::Error) -> bool {
    match error {
        ureq::Error::Status(code, _) => *code == 429 || *code >= 500,
        ureq::Error::Transport(_) => true,
    }
}

fn to_transport_error(url: &str, error: ureq::Error) -> TransportError {
    match error {
        ureq::Error::Status(code, response) => {
            let body_snippet = response
                .into_string()
                .ok()
                .map(|body| body.chars().take(500).collect());
            TransportError {
                url: url.to_owned(),
                message: format!("http status {code}"),
                status: Some(code),
                body_snippet,
            }
        }
        ureq::Error::Transport(inner) => TransportError {
            url: url.to_owned(),
            message: inner.to_string(),
            status: None,
            body_snippet: None,
        },
    }
}

pub trait Transport: Send + Sync {
    fn get(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError>;

    /// Fetches a file rather than an API response.
    ///
    /// Release assets are downloaded in full to hash them, and a mod jar
    /// routinely exceeds the ceiling that is right for JSON. Defaults to
    /// [`Transport::get`] so test doubles need not care.
    fn get_large(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError> {
        self.get(request)
    }

    fn post_json(&self, request: HttpRequest, _body: &[u8]) -> Result<Vec<u8>, TransportError> {
        Err(TransportError {
            url: request.url,
            message: "transport does not support JSON POST requests".into(),
            status: None,
            body_snippet: None,
        })
    }
}

pub struct UreqTransport {
    agent: ureq::Agent,
    max_body_bytes: u64,
    /// Built lazily on first use so an API-only transport pays nothing for it.
    download: OnceLock<(ureq::Agent, u64)>,
}

impl UreqTransport {
    /// The client for metadata/API requests: 30s-scale timeouts and a body
    /// ceiling sized for provider JSON.
    pub fn new() -> Self {
        Self {
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(15))
                .timeout_read(API_READ_TIMEOUT)
                .user_agent(concat!("packwand/", env!("CARGO_PKG_VERSION")))
                .build(),
            max_body_bytes: API_MAX_BODY_BYTES,
            download: OnceLock::new(),
        }
    }

    /// The client for file transfers (mod jars): transfer-scale timeout and a
    /// body ceiling sized for real artifacts. Callers that download a jar to
    /// hash it must use this — the API ceiling silently fails any mod larger
    /// than 16 MiB, which is an ordinary size for a modpack jar.
    pub fn for_downloads() -> Self {
        Self {
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(15))
                .timeout_read(DOWNLOAD_READ_TIMEOUT)
                .user_agent(concat!("packwand/", env!("CARGO_PKG_VERSION")))
                .build(),
            max_body_bytes: DOWNLOAD_MAX_BODY_BYTES,
            download: OnceLock::new(),
        }
    }

    /// Runs `attempt` under the per-host rate limit, retrying transient
    /// failures with doubling, jittered backoff. Every attempt (including
    /// retries) pays the pacing cost, so a retry storm cannot blow the
    /// provider's request budget either.
    ///
    /// Only used for idempotent requests: GETs, and the CurseForge
    /// fingerprint-match query. Publish uploads deliberately run their own
    /// bounded retry elsewhere, because a retried upload after an ambiguous
    /// failure can publish twice.
    fn with_retry(
        &self,
        url: &str,
        mut attempt: impl FnMut() -> Result<ureq::Response, ureq::Error>,
    ) -> Result<Vec<u8>, TransportError> {
        let mut backoff = RETRY_INITIAL_BACKOFF;
        for attempt_number in 1..=RETRY_MAX_ATTEMPTS {
            wait_host_rate_limit(url);
            let error = match attempt() {
                Ok(response) => return self.read_body(url, response),
                Err(error) => error,
            };
            if attempt_number == RETRY_MAX_ATTEMPTS || !is_transient(&error) {
                return Err(to_transport_error(url, error));
            }
            // When the server says how long to wait, believe it (capped).
            let wait = match &error {
                ureq::Error::Status(_, response) => response
                    .header("Retry-After")
                    .and_then(retry_after_delay)
                    .map(|delay| delay.min(RETRY_AFTER_CAP)),
                ureq::Error::Transport(_) => None,
            }
            .unwrap_or_else(|| jittered(backoff));
            std::thread::sleep(wait);
            backoff *= 2;
        }
        unreachable!("retry loop returns on the final attempt")
    }

    fn read_body(&self, url: &str, response: ureq::Response) -> Result<Vec<u8>, TransportError> {
        self.read_body_limited(url, response, self.max_body_bytes)
    }

    fn read_body_limited(
        &self,
        url: &str,
        response: ureq::Response,
        limit: u64,
    ) -> Result<Vec<u8>, TransportError> {
        let mut reader = response.into_reader().take(limit + 1);
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .map_err(|error| TransportError {
                url: url.to_owned(),
                message: error.to_string(),
                status: None,
                body_snippet: None,
            })?;
        if bytes.len() as u64 > limit {
            return Err(TransportError {
                url: url.to_owned(),
                message: format!("response exceeded the {limit} byte limit"),
                status: None,
                body_snippet: None,
            });
        }
        Ok(bytes)
    }
}

impl Default for UreqTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl UreqTransport {
    /// The transfer-scale agent, built on first use.
    fn download_agent(&self) -> &(ureq::Agent, u64) {
        self.download.get_or_init(|| {
            (
                ureq::AgentBuilder::new()
                    .timeout_connect(Duration::from_secs(15))
                    .timeout_read(DOWNLOAD_READ_TIMEOUT)
                    .user_agent(concat!("packwand/", env!("CARGO_PKG_VERSION")))
                    .build(),
                DOWNLOAD_MAX_BODY_BYTES,
            )
        })
    }
}

impl Transport for UreqTransport {
    // ureq::Error embeds the whole Response, so any Result carrying it trips
    // clippy's large-error lint. The size is ureq's, not ours, and boxing it
    // here would only move the allocation.
    #[allow(clippy::result_large_err)]
    fn get(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError> {
        self.with_retry(&request.url, || {
            let mut call = self.agent.get(&request.url);
            for (name, value) in &request.headers {
                call = call.set(name, value);
            }
            call.call()
        })
    }

    fn get_large(&self, request: HttpRequest) -> Result<Vec<u8>, TransportError> {
        let (agent, limit) = self.download_agent();
        let mut backoff = RETRY_INITIAL_BACKOFF;
        for attempt_number in 1..=RETRY_MAX_ATTEMPTS {
            wait_host_rate_limit(&request.url);
            let mut call = agent.get(&request.url);
            for (name, value) in &request.headers {
                call = call.set(name, value);
            }
            let error = match call.call() {
                Ok(response) => return self.read_body_limited(&request.url, response, *limit),
                Err(error) => error,
            };
            if attempt_number == RETRY_MAX_ATTEMPTS || !is_transient(&error) {
                return Err(to_transport_error(&request.url, error));
            }
            let wait = match &error {
                ureq::Error::Status(_, response) => response
                    .header("Retry-After")
                    .and_then(retry_after_delay)
                    .map(|delay| delay.min(RETRY_AFTER_CAP)),
                ureq::Error::Transport(_) => None,
            }
            .unwrap_or_else(|| jittered(backoff));
            std::thread::sleep(wait);
            backoff *= 2;
        }
        unreachable!("retry loop returns on the final attempt")
    }

    // ureq::Error embeds the whole Response, so any Result carrying it trips
    // clippy's large-error lint. The size is ureq's, not ours, and boxing it
    // here would only move the allocation.
    #[allow(clippy::result_large_err)]
    fn post_json(&self, request: HttpRequest, body: &[u8]) -> Result<Vec<u8>, TransportError> {
        self.with_retry(&request.url, || {
            let mut call = self
                .agent
                .post(&request.url)
                .set("Content-Type", "application/json");
            for (name, value) in &request.headers {
                call = call.set(name, value);
            }
            call.send_bytes(body)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_after_accepts_delta_seconds() {
        assert_eq!(retry_after_delay("30"), Some(Duration::from_secs(30)));
        assert_eq!(retry_after_delay(" 5 "), Some(Duration::from_secs(5)));
    }

    #[test]
    fn retry_after_rejects_absent_and_non_positive_values() {
        assert_eq!(retry_after_delay(""), None);
        assert_eq!(retry_after_delay("0"), None);
        assert_eq!(retry_after_delay("-3"), None);
        assert_eq!(retry_after_delay("soon"), None);
    }

    #[test]
    fn http_date_epoch_matches_known_values() {
        // The RFC 9110 example date.
        assert_eq!(
            parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT"),
            Some(784_111_777)
        );
        assert_eq!(parse_http_date("Thu, 01 Jan 1970 00:00:00 GMT"), Some(0));
        assert_eq!(
            parse_http_date("Wed, 21 Oct 2015 07:28:00 GMT"),
            Some(1_445_412_480)
        );
    }

    #[test]
    fn http_date_rejects_malformed_values() {
        assert_eq!(parse_http_date("not a date"), None);
        assert_eq!(parse_http_date("Sun, 06 Xxx 1994 08:49:37 GMT"), None);
        assert_eq!(parse_http_date("Sun, 06 Nov 1994 99:49:37 GMT"), None);
    }

    #[test]
    fn retry_after_in_the_past_is_ignored() {
        assert_eq!(retry_after_delay("Sun, 06 Nov 1994 08:49:37 GMT"), None);
    }

    #[test]
    fn only_rate_limited_hosts_are_paced() {
        assert_eq!(
            host_rate_interval("api.modrinth.com"),
            Some(Duration::from_millis(220))
        );
        assert_eq!(host_rate_interval("api.curseforge.com"), None);
    }

    #[test]
    fn rate_limit_paces_repeated_requests_to_the_same_host() {
        let url = "https://api.modrinth.com/v2/project/test";
        wait_host_rate_limit(url);
        let start = Instant::now();
        wait_host_rate_limit(url);
        wait_host_rate_limit(url);
        // Two further slots must cost at least two intervals.
        assert!(start.elapsed() >= Duration::from_millis(400));
    }

    #[test]
    fn unpaced_hosts_do_not_block() {
        let start = Instant::now();
        for _ in 0..50 {
            wait_host_rate_limit("https://example.invalid/thing");
        }
        assert!(start.elapsed() < Duration::from_millis(50));
    }

    #[test]
    fn transient_classification_matches_retry_policy() {
        assert!(is_transient(&ureq::Error::Status(429, dummy_response())));
        assert!(is_transient(&ureq::Error::Status(503, dummy_response())));
        assert!(!is_transient(&ureq::Error::Status(404, dummy_response())));
        assert!(!is_transient(&ureq::Error::Status(401, dummy_response())));
    }

    #[test]
    fn jitter_stays_within_half_and_one_and_a_half_times() {
        let backoff = Duration::from_millis(500);
        for _ in 0..200 {
            let value = jittered(backoff);
            assert!(value >= Duration::from_millis(250), "{value:?} below range");
            assert!(value < Duration::from_millis(750), "{value:?} above range");
        }
    }

    fn dummy_response() -> ureq::Response {
        ureq::Response::new(429, "Too Many Requests", "").expect("valid synthetic response")
    }
}
