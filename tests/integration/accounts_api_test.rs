use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use rust_decimal::Decimal;
use serde_json::{json, Value};
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
use utopia::core::auth::models::{Principal, UserRecord};
use utopia::core::auth::service::TokenService;
use utopia::core::persistence::repository::Repositories;
use utopia::modules::accounts::AccountServiceImpl;

#[tokio::test]
async fn returns_401_when_bearer_token_is_missing() {
    let database_url =
        "postgres://postgres:postgres@localhost/utopia_test?sslmode=disable".to_string();
    let pool = PgPoolOptions::new()
        .connect_lazy(&database_url)
        .expect("lazy pool");

    let app = build_test_router(pool, test_config(database_url));
    let request = Request::builder()
        .uri("/api/v1/accounts")
        .body(Body::empty())
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let payload: Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(payload["errors"], json!({}));
    assert!(payload["message"]
        .as_str()
        .expect("message")
        .contains("unauthenticated"));
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn lists_accounts_in_firefly_format_with_pagination_and_type_filter() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let user = create_user(&test_db.pool, "maya@example.com").await;
    set_user_primary_currency(&test_db.pool, user.id, "USD").await;
    let other_user = create_user(&test_db.pool, "other@example.com").await;
    let principal = Principal {
        user_id: user.id,
        email: user.email.clone(),
    };
    let token = state
        .token_service
        .issue_token("integration".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    seed_account(
        &test_db.pool,
        user.id,
        "asset",
        "Alpha Asset",
        Decimal::new(125050, 2),
        "JPY",
    )
    .await;
    seed_account(
        &test_db.pool,
        user.id,
        "asset",
        "Zulu Asset",
        Decimal::new(4510, 2),
        "JPY",
    )
    .await;
    seed_account(
        &test_db.pool,
        user.id,
        "expense",
        "Groceries",
        Decimal::new(0, 0),
        "JPY",
    )
    .await;
    seed_account(
        &test_db.pool,
        other_user.id,
        "asset",
        "Other User Account",
        Decimal::new(9999, 2),
        "JPY",
    )
    .await;

    let request = Request::builder()
        .uri("/api/v1/accounts?type=asset&page=1&limit=1")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let payload: Value = serde_json::from_slice(&body).expect("json body");

    // Verify pagination
    assert_eq!(payload["meta"]["pagination"]["total"], json!(2));
    assert_eq!(payload["meta"]["pagination"]["count"], json!(1));
    assert_eq!(payload["meta"]["pagination"]["per_page"], json!(1));
    assert_eq!(payload["meta"]["pagination"]["current_page"], json!(1));
    assert_eq!(payload["meta"]["pagination"]["total_pages"], json!(2));

    // Verify Firefly‑III envelope structure and extended attributes
    let account = &payload["data"][0];
    assert_eq!(account["type"], json!("accounts"));
    assert_eq!(account["attributes"]["type"], json!("asset"));
    assert_eq!(account["attributes"]["name"], json!("Alpha Asset"));
    assert_eq!(account["attributes"]["current_balance"], json!("1250.50"));
    assert_eq!(account["attributes"]["currency_code"], json!("JPY"));
    assert_eq!(account["attributes"]["active"], json!(true));
    assert_eq!(account["attributes"]["include_net_worth"], json!(true));
    assert_eq!(
        account["attributes"]["currency_name"],
        json!("Japanese Yen")
    );
    assert_eq!(account["attributes"]["currency_symbol"], json!("¥"));
    assert_eq!(account["attributes"]["currency_decimal_places"], json!(2));
    assert_eq!(account["attributes"]["primary_currency_code"], json!("USD"));
    assert_eq!(
        account["attributes"]["primary_currency_name"],
        json!("US Dollar")
    );
    assert_eq!(account["attributes"]["primary_currency_symbol"], json!("$"));
    assert_eq!(
        account["attributes"]["primary_currency_decimal_places"],
        json!(2)
    );
    assert!(!account["attributes"]["current_balance_date"]
        .as_str()
        .unwrap()
        .is_empty());
    assert!(account["attributes"]["order"].is_null());
    assert_eq!(account["links"][0]["rel"], json!("self"));
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn creates_account_and_returns_201_with_firefly_envelope() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let user = create_user(&test_db.pool, "alice@example.com").await;
    set_user_primary_currency(&test_db.pool, user.id, "USD").await;
    let principal = Principal {
        user_id: user.id,
        email: user.email.clone(),
    };
    let token = state
        .token_service
        .issue_token("create-test".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    let body = serde_json::json!({
        "name": "My Checking",
        "type": "asset",
        "account_role": "defaultAsset",
        "opening_balance": "50000.00",
        "opening_balance_date": "2026-01-01T00:00:00Z",
        "notes": "Main checking account"
    });

    let request = Request::builder()
        .uri("/api/v1/accounts")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .method("POST")
        .body(Body::from(serde_json::to_string(&body).expect("json")))
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let payload: Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json body");

    // Verify envelope
    assert_eq!(payload["data"]["type"], json!("accounts"));
    assert!(!payload["data"]["id"].as_str().unwrap().is_empty());

    // Verify attributes
    let attrs = &payload["data"]["attributes"];
    assert_eq!(attrs["name"], json!("My Checking"));
    assert_eq!(attrs["type"], json!("asset"));
    assert_eq!(attrs["currency_code"], json!("USD"));
    assert_eq!(attrs["active"], json!(true));
    assert_eq!(attrs["include_net_worth"], json!(true));
    assert_eq!(attrs["account_role"], json!("defaultAsset"));
    assert_eq!(attrs["notes"], json!("Main checking account"));
    assert!(!attrs["created_at"].as_str().unwrap().is_empty());
    assert!(!attrs["updated_at"].as_str().unwrap().is_empty());
    assert_eq!(attrs["primary_currency_code"], json!("USD"));
    assert_eq!(attrs["primary_currency_name"], json!("US Dollar"));
    assert_eq!(attrs["primary_currency_symbol"], json!("$"));
    assert_eq!(attrs["primary_currency_decimal_places"], json!(2));
    assert!(!attrs["current_balance_date"].as_str().unwrap().is_empty());
    assert!(attrs["order"].is_null());
    assert_eq!(payload["data"]["links"][0]["rel"], json!("self"));

    // Verify parsable ID for subsequent tests
    let account_id = payload["data"]["id"].as_str().unwrap().to_string();
    assert!(!account_id.is_empty());
    eprintln!("Created account ID: {account_id}");
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn gets_single_account_by_id() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let user = create_user(&test_db.pool, "bob@example.com").await;
    let principal = Principal {
        user_id: user.id,
        email: user.email.clone(),
    };
    let token = state
        .token_service
        .issue_token("get-test".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    // First create an account
    let create_body = serde_json::json!({
        "name": "Savings Account",
        "type": "asset",
        "account_role": "savingAsset",
        "currency_code": "JPY"
    });

    let create_req = Request::builder()
        .uri("/api/v1/accounts")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .method("POST")
        .body(Body::from(
            serde_json::to_string(&create_body).expect("json"),
        ))
        .expect("request");

    let create_resp = app.clone().oneshot(create_req).await.expect("response");
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let create_payload: Value = serde_json::from_slice(
        &to_bytes(create_resp.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    let account_id = create_payload["data"]["id"].as_str().unwrap().to_string();

    // Get by ID
    let get_req = Request::builder()
        .uri(format!("/api/v1/accounts/{account_id}"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");

    let get_resp = app.oneshot(get_req).await.expect("response");
    assert_eq!(get_resp.status(), StatusCode::OK);

    let payload: Value = serde_json::from_slice(
        &to_bytes(get_resp.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");

    assert_eq!(payload["data"]["id"], json!(account_id));
    assert_eq!(
        payload["data"]["attributes"]["name"],
        json!("Savings Account")
    );
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn returns_404_for_nonexistent_account() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let user = create_user(&test_db.pool, "carol@example.com").await;
    let principal = Principal {
        user_id: user.id,
        email: user.email.clone(),
    };
    let token = state
        .token_service
        .issue_token("404-test".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    let fake_id = uuid::Uuid::nil().to_string();
    let request = Request::builder()
        .uri(format!("/api/v1/accounts/{fake_id}"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn deletes_account_returns_204() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let user = create_user(&test_db.pool, "dave@example.com").await;
    let principal = Principal {
        user_id: user.id,
        email: user.email.clone(),
    };
    let token = state
        .token_service
        .issue_token("delete-test".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    // Create account
    let create_body = serde_json::json!({
        "name": "To Delete",
        "type": "expense",
        "currency_code": "JPY"
    });

    let create_req = Request::builder()
        .uri("/api/v1/accounts")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .method("POST")
        .body(Body::from(
            serde_json::to_string(&create_body).expect("json"),
        ))
        .expect("request");

    let create_resp = app.clone().oneshot(create_req).await.expect("response");
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let payload: Value = serde_json::from_slice(
        &to_bytes(create_resp.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    let account_id = payload["data"]["id"].as_str().unwrap().to_string();

    // Delete
    let delete_req = Request::builder()
        .uri(format!("/api/v1/accounts/{account_id}"))
        .header("authorization", format!("Bearer {token}"))
        .method("DELETE")
        .body(Body::empty())
        .expect("request");

    let delete_resp = app.clone().oneshot(delete_req).await.expect("response");
    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

    // Verify deleted account returns 404
    let get_req = Request::builder()
        .uri(format!("/api/v1/accounts/{account_id}"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");

    let get_resp = app.oneshot(get_req).await.expect("response");
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn validates_required_fields_on_create() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let user = create_user(&test_db.pool, "eve@example.com").await;
    let principal = Principal {
        user_id: user.id,
        email: user.email.clone(),
    };
    let token = state
        .token_service
        .issue_token("validate-test".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    // Missing account_role for asset account
    let invalid_body = serde_json::json!({
        "name": "No Role Asset",
        "type": "asset",
        "currency_code": "JPY"
    });

    let request = Request::builder()
        .uri("/api/v1/accounts")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .method("POST")
        .body(Body::from(
            serde_json::to_string(&invalid_body).expect("json"),
        ))
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

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

fn build_test_router(pool: sqlx::PgPool, config: AppConfig) -> axum::Router {
    let state = build_test_state(pool, config);
    build_router(state)
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

    let rate_limit_state = Arc::new(RateLimitState::new(5, 60));
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

async fn create_user(pool: &sqlx::PgPool, email: &str) -> UserRecord {
    sqlx::query_as::<_, UserRecord>(
        "INSERT INTO users (email) VALUES ($1) RETURNING id, email, blocked, primary_currency_code, created_at, updated_at",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("create user")
}

async fn set_user_primary_currency(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    primary_currency_code: &str,
) {
    sqlx::query("UPDATE users SET primary_currency_code = $1 WHERE id = $2")
        .bind(primary_currency_code)
        .bind(user_id)
        .execute(pool)
        .await
        .expect("update primary currency");
}

async fn seed_account(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    account_type: &str,
    name: &str,
    current_balance: Decimal,
    currency_code: &str,
) {
    sqlx::query(
        "INSERT INTO accounts \
         (user_id, account_type, name, current_balance, currency_code, active, initial_balance, virtual_balance, include_net_worth) \
         VALUES ($1, $2, $3, $4, $5, true, $6, 0, true)",
    )
    .bind(user_id)
    .bind(account_type)
    .bind(name)
    .bind(current_balance)
    .bind(currency_code)
    .bind(current_balance)
    .execute(pool)
    .await
    .expect("insert account");
}
