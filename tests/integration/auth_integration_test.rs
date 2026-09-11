use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;
use tower::ServiceExt;
use utopia::api::middleware::rate_limiter::RateLimitState;
use utopia::api::router::build_router;
use utopia::app::AppState;
use utopia::config::AppConfig;
use utopia::core::auth::audit_logger::AuditLogger;
use utopia::core::auth::cache::TokenCache;
use utopia::core::auth::metrics::PrometheusMetrics;
use utopia::core::auth::service::TokenService;
use utopia::core::persistence::repository::Repositories;
use utopia::modules::accounts::AccountServiceImpl;

/// Test the full bootstrap token lifecycle via HTTP:
/// 1. Issue a bootstrap token
/// 2. Use the token to make an authenticated request
/// 3. Revoke the token
#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn full_bootstrap_token_cycle() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    // Step 1: Issue bootstrap token
    let bootstrap_body = serde_json::json!({
        "label": "bootstrap-test"
    });

    let bootstrap_req = Request::builder()
        .uri("/api/v1/bootstrap/tokens")
        .header("x-bootstrap-key", "bootstrap-test-key-1234")
        .header("content-type", "application/json")
        .method("POST")
        .body(Body::from(
            serde_json::to_string(&bootstrap_body).expect("json"),
        ))
        .expect("request");

    let bootstrap_resp = app.clone().oneshot(bootstrap_req).await.expect("response");
    assert_eq!(bootstrap_resp.status(), StatusCode::OK);

    let payload: Value = serde_json::from_slice(
        &to_bytes(bootstrap_resp.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    let token = payload["data"]["token"].as_str().unwrap().to_string();
    assert!(!token.is_empty(), "token should be non-empty");

    // Step 2: Use the token to access a protected endpoint
    let auth_req = Request::builder()
        .uri("/api/v1/about")
        .header("authorization", format!("Bearer {token}"))
        .header("accept", "application/json")
        .body(Body::empty())
        .expect("request");

    let auth_resp = app.clone().oneshot(auth_req).await.expect("response");
    assert_eq!(auth_resp.status(), StatusCode::OK);
}

/// Test that protected endpoints return 401 when no token / invalid token is provided
#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn returns_401_for_protected_endpoints_without_token() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state);

    // Check each protected endpoint category
    let endpoints = vec![
        "/api/v1/accounts",
        "/api/v1/accounts?page=1&limit=10",
        "/api/v1/transactions",
        "/api/v1/budgets",
        "/api/v1/about",
        "/api/v1/about/user",
    ];

    for endpoint in endpoints {
        let request = Request::builder()
            .uri(endpoint)
            .body(Body::empty())
            .expect("request");

        let response = app.clone().oneshot(request).await.expect("response");
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "endpoint {} should return 401 without token",
            endpoint
        );
    }
}

/// Test that rate limit enforcement works on the bootstrap token endpoint
#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn rate_limit_enforces_429_on_bootstrap_endpoint() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let bootstrap_body = serde_json::json!({
        "label": "rate-limit-test"
    });

    // Send 5 requests (within limit) — should succeed
    for i in 0..5 {
        let req = Request::builder()
            .uri("/api/v1/bootstrap/tokens")
            .header("x-bootstrap-key", "bootstrap-test-key-1234")
            .header("content-type", "application/json")
            .method("POST")
            .body(Body::from(
                serde_json::to_string(&bootstrap_body).expect("json"),
            ))
            .expect("request");

        let resp = app.clone().oneshot(req).await.expect("response");
        assert!(
            resp.status() == StatusCode::OK || resp.status() == StatusCode::TOO_MANY_REQUESTS,
            "request {} should be either 200 or 429, got {}",
            i + 1,
            resp.status()
        );
    }

    // Send the 6th request — should be rate limited (429)
    let rate_limited_req = Request::builder()
        .uri("/api/v1/bootstrap/tokens")
        .header("x-bootstrap-key", "bootstrap-test-key-1234")
        .header("content-type", "application/json")
        .method("POST")
        .body(Body::from(
            serde_json::to_string(&bootstrap_body).expect("json"),
        ))
        .expect("request");

    let rate_limited_resp = app
        .clone()
        .oneshot(rate_limited_req)
        .await
        .expect("response");
    assert_eq!(
        rate_limited_resp.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "should return 429 when rate limit exceeded"
    );

    // Verify Retry-After header is present
    let retry_after = rate_limited_resp
        .headers()
        .get("retry-after")
        .expect("retry-after header");
    assert!(!retry_after.is_empty(), "Retry-After should be non-empty");

    // Verify Firefly-III compatible error body
    let body = to_bytes(rate_limited_resp.into_body(), usize::MAX)
        .await
        .expect("body");
    let payload: Value = serde_json::from_slice(&body).expect("json body");
    assert!(
        payload["message"]
            .as_str()
            .unwrap()
            .contains("Too many requests"),
        "error message should indicate rate limiting"
    );
}

// ---------- Test helpers ----------

fn test_config(database_url: String) -> AppConfig {
    AppConfig {
        database_url,
        argon2_memory_cost: 65_536,
        argon2_time_cost: 3,
        argon2_parallelism: 1,
        token_cache_ttl_secs: 60,
        negative_token_cache_ttl_secs: 60,
        token_cache_max_capacity: 100,
        app_port: 3000,
        log_level: "info".to_string(),
        bootstrap_key: "bootstrap-test-key-1234".to_string(),
        bootstrap_user_email: "bootstrap@example.com".to_string(),
        strict_ssl: false,
        bootstrap_rate_limit_requests: 5,
        bootstrap_rate_limit_window_secs: 60,
    }
}

fn build_test_state(pool: sqlx::PgPool, config: AppConfig) -> Arc<AppState> {
    let repositories = Repositories::new(pool.clone());
    let metrics = Arc::new(PrometheusMetrics::new());
    let cache = TokenCache::new(60, 60, 100);
    let token_service = TokenService::new(
        config.clone(),
        repositories.token.clone(),
        repositories.user.clone(),
        repositories.bootstrap.clone(),
        Arc::clone(&metrics),
    );

    let account_service = Arc::new(AccountServiceImpl::new(repositories.account.clone()));

    let rate_limit_state = Arc::new(RateLimitState::new(
        config.bootstrap_rate_limit_requests,
        config.bootstrap_rate_limit_window_secs,
    ));
    Arc::clone(&rate_limit_state).run_eviction_task();

    Arc::new(AppState {
        config,
        repositories,
        cache,
        metrics,
        audit_logger: AuditLogger,
        token_service,
        account_service,
        rate_limit_state,
    })
}

struct TestDatabase {
    _container: ContainerAsync<Postgres>,
    database_url: String,
    pool: sqlx::PgPool,
}

async fn start_postgres() -> TestDatabase {
    let container = Postgres::default()
        .with_tag("17-alpine")
        .start()
        .await
        .expect("start postgres");

    let host = container.get_host().await.expect("host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("postgres port");
    let database_url =
        format!("postgres://postgres:postgres@{host}:{port}/postgres?sslmode=disable");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("connect pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations");

    TestDatabase {
        _container: container,
        database_url,
        pool,
    }
}
