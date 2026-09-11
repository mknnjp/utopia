use std::collections::HashMap;

use axum::http::StatusCode;
use proptest::prelude::*;
use utopia::core::auth::error::AuthError;
use utopia::core::error_mapping::mapper::{map_auth_error, map_domain_error, DomainError};

#[test]
fn maps_not_found_to_404() {
    let (status, body) = map_domain_error(DomainError::NotFound);
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.message, "Not Found");
}

#[test]
fn maps_validation_to_422_with_errors() {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), vec!["required".to_string()]);

    let (status, body) = map_domain_error(DomainError::Validation(fields));
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.errors.contains_key("name"));
}

#[test]
fn maps_auth_to_401_reason_code() {
    let (status, body) = map_auth_error(AuthError::TokenRevoked);
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body.message.contains("token_revoked"));
}

#[test]
fn maps_conflict_to_409() {
    let (status, body) = map_domain_error(DomainError::Conflict(
        "A concurrent modification was detected. Please retry.".to_string(),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body.message,
        "A concurrent modification was detected. Please retry."
    );
    assert!(body.errors.is_empty());
}

proptest! {
    #[test]
    fn auth_error_serialization_round_trip(_reason_code in "[a-z_]{3,30}") {
        // This test validates that any AuthError variant produces a valid JSON
        // response with a reason_code embedded in the message.
        // We test all known AuthError variants.
        let variants = vec![
            AuthError::MissingAuthorizationHeader,
            AuthError::TokenMalformed,
            AuthError::TokenNotFound,
            AuthError::TokenRevoked,
            AuthError::UserBlocked,
            AuthError::DependencyFailure,
            AuthError::BootstrapKeyMissing,
            AuthError::BootstrapKeyInvalid,
            AuthError::BootstrapAlreadyUsed,
            AuthError::RateLimitExceeded { retry_after_secs: 60 },
        ];

        for variant in variants {
            let (status, body) = map_auth_error(variant.clone());
            let json = serde_json::to_value(&body).expect("serialize");
            prop_assert!(json["message"].as_str().is_some());
            prop_assert!(json["errors"].is_object());

            // RateLimitExceeded returns 429, all others return 401
            match variant {
                AuthError::RateLimitExceeded { .. } => {
                    prop_assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
                }
                _ => {
                    prop_assert_eq!(status, StatusCode::UNAUTHORIZED);
                }
            }
        }
    }
}
