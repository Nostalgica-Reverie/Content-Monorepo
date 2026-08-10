//! What a failed sign-in means, and what to do about it.
//!
//! Authentication does not have two outcomes. Treating it as "worked" or
//! "did not work" forces one decision to cover several unrelated situations,
//! and the decision that gets made is always the destructive one: throw the
//! credential away and make the user sign in again.
//!
//! The split that matters is between *the service is unreachable or unhappy*
//! and *this credential is finished*. Xbox Live has outages. During one, a
//! two-state launcher signs everybody out; a five-state one lets them keep
//! playing offline and picks the session back up when the service returns.
//! Only [`AuthState::Gone`] justifies deleting anything.

use serde::Serialize;

use crate::chain::MsaError;

/// Why a session is not currently available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthState {
	/// The service could not be reached at all.
	///
	/// Says nothing about the credential, so the credential is kept and the
	/// launch continues offline.
	Offline,
	/// The user turned this account off. Not a failure; nothing was tried.
	Disabled,
	/// A transient failure — a 5xx, a timeout, a malformed response.
	/// Worth retrying, and never worth discarding a token over.
	SoftFail,
	/// Something is genuinely wrong and needs a person: no Xbox profile, a
	/// region restriction, an unverified minor account, an unwhitelisted
	/// app registration. The stored credential is still valid, so it stays.
	HardFail,
	/// The credential or the entitlement is finished: the refresh token was
	/// rejected, or the account does not own the game. The only state that
	/// discards anything.
	Gone,
}

impl AuthState {
	/// Whether the stored refresh token should survive this outcome.
	///
	/// The whole reason for the enum. Everything except [`Self::Gone`]
	/// answers yes, so an outage costs a user nothing but a session.
	pub fn keeps_credentials(self) -> bool {
		!matches!(self, Self::Gone)
	}

	/// Whether the launcher may continue into offline play.
	///
	/// A service problem is not the user's problem. A problem with the
	/// account is something they have to see and act on, so it stops here
	/// rather than silently starting the game under a different identity.
	pub fn allows_offline_fallback(self) -> bool {
		matches!(self, Self::Offline | Self::SoftFail | Self::Disabled)
	}

	/// Whether an interactive sign-in would plausibly fix this.
	pub fn needs_interactive_login(self) -> bool {
		matches!(self, Self::Gone)
	}
}

/// Classifies an authentication failure.
pub fn classify(error: &MsaError) -> AuthState {
	match error {
		// Nothing was learned about the credential.
		MsaError::Network(..) => AuthState::Offline,
		// The service answered, but not usefully.
		MsaError::UnexpectedResponse(..) | MsaError::Other(_) | MsaError::Store(_) => {
			AuthState::SoftFail
		}
		// The credential itself is finished; this is the only discard.
		MsaError::RefreshRejected(_) | MsaError::NoEntitlement => AuthState::Gone,
		// Real problems, but ones a person has to resolve — and none of them
		// mean the saved sign-in is bad.
		MsaError::Denied(_)
		| MsaError::NoXboxAccount
		| MsaError::XboxLiveUnavailableInRegion
		| MsaError::NeedsAdultVerification
		| MsaError::NeedsFamilyGroup
		| MsaError::NotWhitelisted => AuthState::HardFail,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn an_outage_never_costs_the_user_their_sign_in() {
		// The property the whole enum exists for: Xbox Live being down must
		// not log anyone out.
		for error in [
			MsaError::Network("xbl".into(), "connection refused".into()),
			MsaError::UnexpectedResponse("xsts".into(), "503".into()),
			MsaError::Other("something odd".into()),
		] {
			let state = classify(&error);
			assert!(state.keeps_credentials(), "{error} discarded the token");
			assert!(
				state.allows_offline_fallback(),
				"{error} blocked offline play"
			);
			assert!(!state.needs_interactive_login(), "{error}");
		}
	}

	#[test]
	fn only_a_finished_credential_is_discarded() {
		for error in [
			MsaError::RefreshRejected("expired".into()),
			MsaError::NoEntitlement,
		] {
			let state = classify(&error);
			assert_eq!(state, AuthState::Gone);
			assert!(!state.keeps_credentials(), "{error} kept a dead token");
			assert!(state.needs_interactive_login());
		}
	}

	#[test]
	fn an_account_problem_stops_rather_than_silently_playing_as_someone_else() {
		// These need a person. Falling back to offline would start the game
		// under a different name without saying so.
		for error in [
			MsaError::NoXboxAccount,
			MsaError::NeedsFamilyGroup,
			MsaError::NeedsAdultVerification,
			MsaError::XboxLiveUnavailableInRegion,
			MsaError::NotWhitelisted,
		] {
			let state = classify(&error);
			assert_eq!(state, AuthState::HardFail, "{error}");
			assert!(state.keeps_credentials(), "{error}");
			assert!(!state.allows_offline_fallback(), "{error}");
		}
	}

	#[test]
	fn a_disabled_account_is_not_a_failure() {
		assert!(AuthState::Disabled.keeps_credentials());
		assert!(AuthState::Disabled.allows_offline_fallback());
		assert!(!AuthState::Disabled.needs_interactive_login());
	}
}
