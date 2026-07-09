//! PKCE (RFC 7636) helper: a cryptographically random `code_verifier` and
//! its S256 `code_challenge`, plus a random `state` value for CSRF
//! protection. Used because this is a public OAuth client (no client
//! secret) — PKCE is what stops an intercepted authorization code from
//! being redeemed by anyone but the process that started this flow.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

fn random_url_safe(byte_len: usize) -> String {
    let mut bytes = vec![0u8; byte_len];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

impl Pkce {
    pub fn generate() -> Self {
        let verifier = random_url_safe(32);
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        Self {
            verifier,
            challenge,
        }
    }
}

/// A random value to detect CSRF / mismatched redirects: the value returned
/// in the browser redirect must match what we sent in the authorize URL.
pub fn random_state() -> String {
    random_url_safe(16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_distinct_verifier_and_challenge_each_time() {
        let a = Pkce::generate();
        let b = Pkce::generate();
        assert_ne!(a.verifier, b.verifier);
        assert_ne!(a.challenge, b.challenge);
        assert_ne!(a.verifier, a.challenge);
    }

    #[test]
    fn challenge_matches_rfc7636_appendix_b_test_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn state_values_are_reasonably_unique() {
        let a = random_state();
        let b = random_state();
        assert_ne!(a, b);
        assert!(!a.is_empty());
    }
}
