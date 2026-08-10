//! Java version strings, parsed and ordered.
//!
//! Two schemes have to compare against each other: the legacy `1.8.0_392`
//! form and the modern `21.0.5+11` form, plus prerelease suffixes like
//! `25-ea`. Parsing both into one struct is what lets "is this new enough"
//! and "which of these two is newer" be the same comparison.

use std::cmp::Ordering;

use regex_lite::Regex;
use serde::Serialize;

use crate::error::RuntimeError;

/// A parsed Java version.
///
/// `Ord` is the point of the type: it sorts by feature release, then minor,
/// then security, and places a prerelease *below* the release it leads up to,
/// so `25-ea` never outranks `25`. Picking the newest JVM by `max()` would
/// otherwise select an early-access build over a general-availability one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JavaVersion {
	/// Feature release: 8 for `1.8.0_392`, 21 for `21.0.5`.
	pub major: u32,
	pub minor: u32,
	pub security: u32,
	/// Build number (`+11`) or legacy update (`_392`), when present.
	pub build: Option<u32>,
	/// Prerelease tag such as `ea` or `beta`, when present.
	pub prerelease: Option<String>,
	/// The string this was parsed from, preserved for display.
	pub original: String,
}

/// The modern scheme, which also covers the legacy `1.8.0_392` shape: the
/// update separator is accepted alongside the build separator so one pattern
/// reads both, and the `1.x` case is normalised after matching.
fn modern() -> &'static Regex {
	static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
	RE.get_or_init(|| {
		Regex::new(r"^(\d+)(?:\.(\d+))?(?:\.(\d+))?(?:[+_](\d+))?(?:-(.+))?$")
			.expect("version pattern is valid")
	})
}

impl JavaVersion {
	/// Parses a `JAVA_VERSION`-style string.
	pub fn parse(version: &str) -> Result<Self, RuntimeError> {
		let unparseable = || RuntimeError::UnparseableVersion(version.to_string());
		let caps = modern().captures(version.trim()).ok_or_else(unparseable)?;
		let number = |i: usize| caps.get(i).and_then(|m| m.as_str().parse::<u32>().ok());
		let mut major = number(1).ok_or_else(unparseable)?;
		let mut minor = number(2).unwrap_or(0);
		let mut security = number(3).unwrap_or(0);
		// Legacy `1.N.x`: the feature release is the second component, not
		// the first. `1` alone is left as-is — there is no second component
		// to promote, so it is version 1 rather than a malformed 1.x.
		if major == 1 && caps.get(2).is_some() {
			major = minor;
			minor = security;
			security = 0;
		}
		Ok(Self {
			major,
			minor,
			security,
			build: number(4),
			prerelease: caps.get(5).map(|m| m.as_str().to_string()),
			original: version.trim().to_string(),
		})
	}

	/// Whether this is a prerelease (early access, beta, release candidate).
	pub fn is_prerelease(&self) -> bool {
		self.prerelease.is_some()
	}
}

impl Ord for JavaVersion {
	fn cmp(&self, other: &Self) -> Ordering {
		self.major
			.cmp(&other.major)
			.then(self.minor.cmp(&other.minor))
			.then(self.security.cmp(&other.security))
			// A release outranks any prerelease of the same numbers, so
			// `None` must sort above `Some`, which is the reverse of the
			// derived order on `Option`.
			.then(match (&self.prerelease, &other.prerelease) {
				(None, None) => Ordering::Equal,
				(None, Some(_)) => Ordering::Greater,
				(Some(_), None) => Ordering::Less,
				(Some(a), Some(b)) => a.cmp(b),
			})
			.then(self.build.cmp(&other.build))
	}
}

impl PartialOrd for JavaVersion {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl std::fmt::Display for JavaVersion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.original)
	}
}

/// Parses a `JAVA_VERSION` string into its feature-release number:
/// `1.8.0_392` is 8, `17.0.2` is 17, `9` is 9.
pub fn parse_major_version(version: &str) -> Result<u32, RuntimeError> {
	JavaVersion::parse(version).map(|v| v.major)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn major_version_parsing() {
		assert_eq!(parse_major_version("1.8.0_392").unwrap(), 8);
		assert_eq!(parse_major_version("17.0.2").unwrap(), 17);
		assert_eq!(parse_major_version("25.0.1").unwrap(), 25);
		assert_eq!(parse_major_version("9").unwrap(), 9);
		assert!(parse_major_version("").is_err());
		assert!(parse_major_version("banana").is_err());
		assert!(parse_major_version("1.x").is_err());
	}

	#[test]
	fn components_and_build_numbers_are_kept() {
		let v = JavaVersion::parse("21.0.5+11").unwrap();
		assert_eq!((v.major, v.minor, v.security), (21, 0, 5));
		assert_eq!(v.build, Some(11));
		assert!(!v.is_prerelease());

		let legacy = JavaVersion::parse("1.8.0_392").unwrap();
		assert_eq!((legacy.major, legacy.minor, legacy.security), (8, 0, 0));
		assert_eq!(legacy.build, Some(392));

		// A bare `1` has no second component to promote.
		assert_eq!(JavaVersion::parse("1").unwrap().major, 1);
	}

	#[test]
	fn a_prerelease_sorts_below_its_release() {
		// The ordering that matters when picking "the newest available": an
		// early-access build must never win over the real release.
		let ea = JavaVersion::parse("25-ea").unwrap();
		let ga = JavaVersion::parse("25").unwrap();
		assert!(ea < ga, "{ea} should sort below {ga}");
		assert!(JavaVersion::parse("21.0.5").unwrap() < ga);
		assert!(JavaVersion::parse("17.0.9").unwrap() < JavaVersion::parse("21.0.1").unwrap());
		assert!(JavaVersion::parse("1.8.0_392").unwrap() < JavaVersion::parse("11").unwrap());

		let mut all = [
			JavaVersion::parse("25-ea").unwrap(),
			JavaVersion::parse("21.0.5+11").unwrap(),
			JavaVersion::parse("25").unwrap(),
			JavaVersion::parse("1.8.0_392").unwrap(),
		];
		all.sort();
		let names: Vec<&str> = all.iter().map(|v| v.original.as_str()).collect();
		assert_eq!(names, ["1.8.0_392", "21.0.5+11", "25-ea", "25"]);
	}

	#[test]
	fn build_numbers_break_ties_but_never_outrank_a_component() {
		assert!(JavaVersion::parse("21.0.5+7").unwrap() < JavaVersion::parse("21.0.5+11").unwrap());
		assert!(JavaVersion::parse("21.0.5+99").unwrap() < JavaVersion::parse("21.0.6").unwrap());
	}
}
