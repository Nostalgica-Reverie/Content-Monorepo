//! Choosing a version for a parent, and saying why.
//!
//! Every loader publishes a list of builds for a given Minecraft version, and
//! every one of them marks some subset as the one to use — Fabric and Quilt
//! with a `stable` flag, NeoForge and Forge by ordering and naming
//! convention. "Recommended, else the newest available" is therefore the same
//! question four times, and it is worth asking in one place.
//!
//! The part that is not merely tidiness is the [`Recommendation`] the answer
//! carries. A launcher that silently falls back to a bleeding-edge build when
//! no stable one exists produces bug reports nobody can explain; one that can
//! say "no stable Fabric supports 1.21.6 yet, using 0.17.2" turns the same
//! situation into a sentence the user can act on.

use serde::Serialize;

/// Why a particular version was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Recommendation {
	/// The publisher marks this build as recommended or stable.
	Recommended,
	/// Nothing is marked recommended for this parent, so the newest entry
	/// was taken. Worth surfacing rather than hiding.
	LatestAvailable,
	/// The caller asked for this exact version, so nothing was chosen.
	Pinned,
}

impl Recommendation {
	/// Whether this choice came with the publisher's endorsement.
	pub fn is_endorsed(self) -> bool {
		matches!(self, Self::Recommended | Self::Pinned)
	}
}

/// A chosen version and the reason it was chosen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Choice {
	pub version: String,
	pub how: Recommendation,
}

impl Choice {
	/// A version the caller named explicitly.
	pub fn pinned(version: impl Into<String>) -> Self {
		Self {
			version: version.into(),
			how: Recommendation::Pinned,
		}
	}

	/// A human-readable note when the choice is worth mentioning.
	///
	/// `None` for the ordinary cases, so a caller can log unconditionally
	/// without adding noise to every launch.
	pub fn note(&self, what: &str, parent: &str) -> Option<String> {
		matches!(self.how, Recommendation::LatestAvailable).then(|| {
			format!(
				"no recommended {what} for {parent} yet; using the newest available ({})",
				self.version
			)
		})
	}
}

/// Picks the recommended candidate for a parent, else the first one.
///
/// Order is the publisher's: every one of these APIs returns newest first, so
/// "the first entry" is "the latest". Taking the caller's ordering rather than
/// sorting is deliberate — loader version strings are not comparable across
/// loaders, and inventing an ordering for them would be a source of wrong
/// answers rather than a convenience.
pub fn recommended_for_parent<T>(
	candidates: &[T],
	is_recommended: impl Fn(&T) -> bool,
	version_of: impl Fn(&T) -> String,
) -> Option<Choice> {
	if let Some(found) = candidates.iter().find(|c| is_recommended(c)) {
		return Some(Choice {
			version: version_of(found),
			how: Recommendation::Recommended,
		});
	}
	candidates.first().map(|first| Choice {
		version: version_of(first),
		how: Recommendation::LatestAvailable,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	struct Build {
		version: &'static str,
		stable: bool,
	}

	fn pick(builds: &[Build]) -> Option<Choice> {
		recommended_for_parent(builds, |b| b.stable, |b| b.version.to_string())
	}

	#[test]
	fn a_recommended_build_wins_over_a_newer_unstable_one() {
		// The list is newest-first, so this also pins that "recommended"
		// beats "latest" rather than the other way round.
		let choice = pick(&[
			Build {
				version: "0.17.2",
				stable: false,
			},
			Build {
				version: "0.16.14",
				stable: true,
			},
		])
		.unwrap();
		assert_eq!(choice.version, "0.16.14");
		assert_eq!(choice.how, Recommendation::Recommended);
		assert!(choice.how.is_endorsed());
		assert!(choice.note("Fabric", "1.21.1").is_none());
	}

	#[test]
	fn with_nothing_recommended_the_newest_is_used_and_said_so() {
		// The case that produces unexplainable bug reports when it is silent:
		// a brand-new Minecraft version that no stable loader supports yet.
		let choice = pick(&[
			Build {
				version: "0.17.2",
				stable: false,
			},
			Build {
				version: "0.17.1",
				stable: false,
			},
		])
		.unwrap();
		assert_eq!(choice.version, "0.17.2");
		assert_eq!(choice.how, Recommendation::LatestAvailable);
		assert!(!choice.how.is_endorsed());
		let note = choice.note("Fabric", "1.21.6").unwrap();
		assert!(note.contains("1.21.6"), "{note}");
		assert!(note.contains("0.17.2"), "{note}");
	}

	#[test]
	fn an_empty_list_chooses_nothing() {
		assert!(pick(&[]).is_none());
	}

	#[test]
	fn a_pinned_version_is_endorsed_and_unremarked() {
		let choice = Choice::pinned("0.16.5");
		assert_eq!(choice.how, Recommendation::Pinned);
		assert!(choice.how.is_endorsed());
		assert!(choice.note("Fabric", "1.21.1").is_none());
	}
}
