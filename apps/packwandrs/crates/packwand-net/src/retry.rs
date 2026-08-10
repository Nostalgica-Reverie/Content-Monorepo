use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// How many times an idempotent request is attempted in total.
pub const MAX_ATTEMPTS: u32 = 3;

const INITIAL_BACKOFF: Duration = Duration::from_millis(500);

/// Bounds how long a server-provided `Retry-After` can stall a worker; a
/// hostile or misconfigured server must not be able to park a slot for
/// minutes.
const RETRY_AFTER_CAP: Duration = Duration::from_secs(60);

/// Doubling backoff between attempts.
pub(crate) fn backoff_for(attempt: u32) -> Duration {
	INITIAL_BACKOFF * 2u32.saturating_pow(attempt.saturating_sub(1))
}

/// Jitter source. Not cryptographic — it only needs to stop workers that were
/// rate-limited together from retrying in lockstep.
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
pub(crate) fn jittered(backoff: Duration) -> Duration {
	let base = backoff.as_millis().max(1) as u64;
	Duration::from_millis(base / 2 + next_random() % base)
}

/// Whether a failed attempt is worth repeating.
///
/// 404 is deliberately absent: a missing resource does not become present by
/// asking again, and retrying it only delays the real error.
pub(crate) fn is_transient(error: &ureq::Error) -> bool {
	match error {
		ureq::Error::Status(code, _) => *code == 429 || *code >= 500,
		ureq::Error::Transport(_) => true,
	}
}

/// How long to wait before the next attempt: what the server asked for when
/// it said, capped; otherwise jittered exponential backoff.
pub(crate) fn wait_for(error: &ureq::Error, attempt: u32) -> Duration {
	match error {
		ureq::Error::Status(_, response) => response
			.header("Retry-After")
			.and_then(parse_retry_after)
			.map(|delay| delay.min(RETRY_AFTER_CAP)),
		ureq::Error::Transport(_) => None,
	}
	.unwrap_or_else(|| jittered(backoff_for(attempt)))
}

/// Parses a `Retry-After` value: either delta-seconds or an HTTP-date, per
/// RFC 9110 section 10.2.3. `None` when absent or unparseable.
pub(crate) fn parse_retry_after(value: &str) -> Option<Duration> {
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
pub(crate) fn parse_http_date(value: &str) -> Option<i64> {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn retry_after_accepts_both_wire_forms() {
		assert_eq!(parse_retry_after("30"), Some(Duration::from_secs(30)));
		assert_eq!(parse_retry_after("0"), None);
		assert_eq!(parse_retry_after(""), None);
		assert_eq!(parse_retry_after("soon"), None);
		// An HTTP-date in the past is not a wait.
		assert_eq!(parse_retry_after("Sun, 06 Nov 1994 08:49:37 GMT"), None);
	}

	#[test]
	fn http_dates_round_trip_against_known_epochs() {
		assert_eq!(
			parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT"),
			Some(784_111_777)
		);
		assert_eq!(parse_http_date("Thu, 01 Jan 1970 00:00:00 GMT"), Some(0));
		assert_eq!(parse_http_date("garbage"), None);
	}

	#[test]
	fn backoff_doubles_and_jitter_stays_in_band() {
		assert_eq!(backoff_for(1), Duration::from_millis(500));
		assert_eq!(backoff_for(2), Duration::from_millis(1000));
		assert_eq!(backoff_for(3), Duration::from_millis(2000));
		for _ in 0..200 {
			let value = jittered(Duration::from_millis(1000));
			assert!((500..1500).contains(&value.as_millis()), "{value:?}");
		}
	}
}
