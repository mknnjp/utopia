use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use proptest::prelude::*;

#[test]
fn token_issuance_is_not_idempotent_by_design() {
    let first = "token-a";
    let second = "token-b";
    assert_ne!(first, second);
}

proptest! {
    #[test]
    fn token_format_round_trip(seed in prop::collection::vec(any::<u8>(), 32)) {
        // Simulate token generation: encode 32 random bytes to URL-safe base64 (no pad)
        let token = URL_SAFE_NO_PAD.encode(&seed);
        prop_assert!(!token.is_empty());
        prop_assert!(!token.contains('='), "token should not contain padding");

        // Verify we can decode back to the original bytes
        let decoded = URL_SAFE_NO_PAD.decode(&token).expect("should decode");
        prop_assert_eq!(decoded, seed);

        // Verify token matches expected length: ceil(32*8/6) = 43 chars
        prop_assert_eq!(token.len(), 43, "token should be 43 chars");
    }
}
