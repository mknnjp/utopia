use std::sync::Arc;
use std::time::Instant;

use argon2::{Argon2, PasswordVerifier};
use argon2::password_hash::phc::PasswordHash;
use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::core::auth::cache::{CachedAuthResult, TokenCache, TokenCacheEntry};
use crate::core::auth::error::AuthError;
use crate::core::auth::metrics::PrometheusMetrics;
use crate::core::auth::models::Principal;
use crate::core::persistence::repository::{
    PgTokenRepository, PgUserRepository, TokenReadRepository, TokenUpdateRepository,
    UserReadRepository,
};

pub async fn validate_bearer(
    authorization_header: Option<&str>,
    token_repo: &PgTokenRepository,
    user_repo: &PgUserRepository,
    pool: &PgPool,
    cache: &TokenCache,
    metrics: &Arc<PrometheusMetrics>,
) -> Result<Principal, AuthError> {
    let started = Instant::now();
    let raw_token = parse_bearer(authorization_header)?;
    let sha256_token = sha256_hex(raw_token);

    if let Some(cached) = cache.get(&sha256_token).await {
        match cached {
            CachedAuthResult::Valid(entry) => {
                metrics
                    .auth_cache_hit_total
                    .with_label_values(&["positive"])
                    .inc();
                metrics.authenticated_requests_total.inc();

                let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                metrics.auth_validation_latency_ms.observe(elapsed_ms);

                return Ok(Principal {
                    user_id: entry.user_id,
                    email: entry.email,
                });
            }
            CachedAuthResult::Invalid { reason } => {
                metrics
                    .auth_cache_hit_total
                    .with_label_values(&["negative"])
                    .inc();
                metrics
                    .auth_failures_total
                    .with_label_values(&[reason.reason_code()])
                    .inc();
                return Err(reason);
            }
        }
    }

    metrics.auth_cache_miss_total.inc();

    let token_record = token_repo
        .find_by_sha256(pool, &sha256_token)
        .await
        .map_err(|_| AuthError::DependencyFailure)?
        .ok_or(AuthError::TokenNotFound)?;

    if token_record.is_revoked() {
        cache
            .insert_invalid(sha256_token, AuthError::TokenRevoked)
            .await;
        metrics
            .auth_failures_total
            .with_label_values(&[AuthError::TokenRevoked.reason_code()])
            .inc();
        return Err(AuthError::TokenRevoked);
    }

    verify_argon2(raw_token, &token_record.token_hash).map_err(|_| AuthError::TokenNotFound)?;

    let user = user_repo
        .find_by_id(pool, token_record.user_id)
        .await
        .map_err(|_| AuthError::DependencyFailure)?
        .ok_or(AuthError::TokenNotFound)?;

    if user.blocked {
        cache
            .insert_invalid(sha256_token, AuthError::UserBlocked)
            .await;
        metrics
            .auth_failures_total
            .with_label_values(&[AuthError::UserBlocked.reason_code()])
            .inc();
        return Err(AuthError::UserBlocked);
    }

    let principal = Principal {
        user_id: user.id,
        email: user.email,
    };

    cache
        .insert_valid(
            token_record.token_sha256.clone(),
            TokenCacheEntry {
                user_id: principal.user_id,
                email: principal.email.clone(),
            },
        )
        .await;

    let token_sha256_for_update = token_record.token_sha256.clone();
    let metrics_for_task = Arc::clone(metrics);
    let pool_for_task = pool.clone();
    let repo_for_task = token_repo.clone();

    tokio::spawn(async move {
        if let Err(err) = repo_for_task
            .update_last_used_at(&pool_for_task, &token_sha256_for_update, Utc::now())
            .await
        {
            metrics_for_task.auth_dependency_failure_total.inc();
            tracing::error!(error = %err, token_sha256 = %token_sha256_for_update, "failed to update last_used_at");
        }
    });

    metrics.authenticated_requests_total.inc();
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    metrics.auth_validation_latency_ms.observe(elapsed_ms);

    Ok(principal)
}

fn parse_bearer(header: Option<&str>) -> Result<&str, AuthError> {
    let raw = header.ok_or(AuthError::MissingAuthorizationHeader)?;
    let parts: Vec<&str> = raw.splitn(2, ' ').collect();
    if parts.len() != 2 || parts[0] != "Bearer" || parts[1].is_empty() {
        return Err(AuthError::TokenMalformed);
    }
    Ok(parts[1])
}

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

fn verify_argon2(raw_token: &str, encoded_hash: &str) -> Result<(), AuthError> {
    let parsed = PasswordHash::new(encoded_hash).map_err(|_| AuthError::TokenMalformed)?;
    Argon2::default()
        .verify_password(raw_token.as_bytes(), &parsed)
        .map_err(|_| AuthError::TokenNotFound)
}
