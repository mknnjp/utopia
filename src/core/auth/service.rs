use std::sync::Arc;

use argon2::{Argon2, Params, PasswordHasher, Version};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::core::auth::cache::TokenCache;
use crate::core::auth::error::AuthError;
use crate::core::auth::metrics::PrometheusMetrics;
use crate::core::auth::models::{
    Principal, TokenIssuancePayload, TokenIssuanceResponse, TokenRecord,
};
use crate::core::persistence::repository::{
    BootstrapKeyRepository, PgBootstrapKeyRepository, PgTokenRepository, PgUserRepository,
    TokenReadRepository, TokenWriteRepository, UserReadRepository, UserWriteRepository,
};

#[derive(Debug, Clone)]
pub struct TokenService {
    pub config: AppConfig,
    pub token_repo: PgTokenRepository,
    pub user_repo: PgUserRepository,
    pub bootstrap_repo: PgBootstrapKeyRepository,
    pub metrics: Arc<PrometheusMetrics>,
}

impl TokenService {
    pub fn new(
        config: AppConfig,
        token_repo: PgTokenRepository,
        user_repo: PgUserRepository,
        bootstrap_repo: PgBootstrapKeyRepository,
        metrics: Arc<PrometheusMetrics>,
    ) -> Self {
        Self {
            config,
            token_repo,
            user_repo,
            bootstrap_repo,
            metrics,
        }
    }

    pub async fn issue_token(
        &self,
        label: String,
        principal: &Principal,
        pool: &sqlx::PgPool,
    ) -> Result<TokenIssuanceResponse, AuthError> {
        let raw_token = generate_token();
        let token_sha256 = sha256_hex(&raw_token);
        let token_hash = self.hash_token(&raw_token)?;
        let now = Utc::now();

        let token_record = TokenRecord {
            id: Uuid::new_v4(),
            user_id: principal.user_id,
            label,
            token_sha256,
            token_hash,
            status: "Active".to_string(),
            last_used_at: None,
            created_at: now,
        };

        let mut tx = pool
            .begin()
            .await
            .map_err(|_| AuthError::DependencyFailure)?;
        self.token_repo
            .create_token(&mut tx, &token_record)
            .await
            .map_err(|_| AuthError::DependencyFailure)?;
        tx.commit()
            .await
            .map_err(|_| AuthError::DependencyFailure)?;

        self.metrics.token_issue_total.inc();

        Ok(TokenIssuanceResponse {
            data: TokenIssuancePayload {
                id: token_record.id,
                label: token_record.label,
                token: raw_token,
                status: token_record.status,
                created_at: token_record.created_at,
            },
        })
    }

    pub async fn issue_bootstrap_token(
        &self,
        label: String,
        bootstrap_key_header: Option<&str>,
        pool: &sqlx::PgPool,
    ) -> Result<TokenIssuanceResponse, AuthError> {
        let provided = bootstrap_key_header.ok_or(AuthError::BootstrapKeyMissing)?;
        if !constant_time_eq(provided.as_bytes(), self.config.bootstrap_key.as_bytes()) {
            return Err(AuthError::BootstrapKeyInvalid);
        }

        let bootstrap_hash = sha256_hex(&self.config.bootstrap_key);
        let mut tx = pool
            .begin()
            .await
            .map_err(|_| AuthError::DependencyFailure)?;

        let mut principal_user = self
            .user_repo
            .find_by_email(tx.as_mut(), &self.config.bootstrap_user_email)
            .await
            .map_err(|_| AuthError::DependencyFailure)?;

        if principal_user.is_none() {
            principal_user = Some(
                self.user_repo
                    .create_user(&mut tx, &self.config.bootstrap_user_email)
                    .await
                    .map_err(|_| AuthError::DependencyFailure)?,
            );
        }

        let user = principal_user.ok_or(AuthError::DependencyFailure)?;
        let claimed = self
            .bootstrap_repo
            .claim_bootstrap_key(&mut tx, &bootstrap_hash, Some(user.id))
            .await
            .map_err(|_| AuthError::DependencyFailure)?;

        if !claimed {
            return Err(AuthError::BootstrapAlreadyUsed);
        }

        let raw_token = generate_token();
        let token_sha256 = sha256_hex(&raw_token);
        let token_hash = self.hash_token(&raw_token)?;
        let now = Utc::now();

        let token_record = TokenRecord {
            id: Uuid::new_v4(),
            user_id: user.id,
            label,
            token_sha256,
            token_hash,
            status: "Active".to_string(),
            last_used_at: None,
            created_at: now,
        };

        self.token_repo
            .create_token(&mut tx, &token_record)
            .await
            .map_err(|_| AuthError::DependencyFailure)?;

        tx.commit()
            .await
            .map_err(|_| AuthError::DependencyFailure)?;

        self.metrics.token_issue_total.inc();

        Ok(TokenIssuanceResponse {
            data: TokenIssuancePayload {
                id: token_record.id,
                label: token_record.label,
                token: raw_token,
                status: token_record.status,
                created_at: token_record.created_at,
            },
        })
    }

    pub async fn revoke_token(
        &self,
        token_id: Uuid,
        principal: &Principal,
        pool: &sqlx::PgPool,
        cache: &TokenCache,
    ) -> Result<(), AuthError> {
        let mut tx = pool
            .begin()
            .await
            .map_err(|_| AuthError::DependencyFailure)?;
        let token = self
            .token_repo
            .find_by_id(tx.as_mut(), token_id)
            .await
            .map_err(|_| AuthError::DependencyFailure)?
            .ok_or(AuthError::TokenNotFound)?;

        if token.user_id != principal.user_id {
            return Err(AuthError::TokenNotFound);
        }

        self.token_repo
            .revoke_token(&mut tx, token.id)
            .await
            .map_err(|_| AuthError::DependencyFailure)?;

        tx.commit()
            .await
            .map_err(|_| AuthError::DependencyFailure)?;
        cache.invalidate(&token.token_sha256).await;
        self.metrics.token_revoke_total.inc();

        Ok(())
    }

    fn hash_token(&self, raw_token: &str) -> Result<String, AuthError> {
        let params = Params::new(
            self.config.argon2_memory_cost,
            self.config.argon2_time_cost,
            self.config.argon2_parallelism,
            None,
        )
        .map_err(|_| AuthError::DependencyFailure)?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);
        let hash = argon2
            .hash_password(raw_token.as_bytes())
            .map_err(|_| AuthError::DependencyFailure)?;

        Ok(hash.to_string())
    }
}

fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.ct_eq(right).into()
}
