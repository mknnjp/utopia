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

async fn seed_account(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    account_type: &str,
    name: &str,
    current_balance: Decimal,
    currency_code: &str,
) -> uuid::Uuid {
    sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO accounts \
         (user_id, account_type, name, current_balance, currency_code, active, initial_balance, virtual_balance, include_net_worth) \
         VALUES ($1, $2, $3, $4, $5, true, $6, 0, true) \
         RETURNING id",
    )
    .bind(user_id)
    .bind(account_type)
    .bind(name)
    .bind(current_balance)
    .bind(currency_code)
    .bind(current_balance)
    .fetch_one(pool)
    .await
    .expect("insert account")
}

#[allow(clippy::too_many_arguments)]
async fn create_transaction_journal(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    transaction_type: &str,
    description: &str,
    amount: Decimal,
    currency_code: &str,
    source_id: Option<uuid::Uuid>,
    destination_id: Option<uuid::Uuid>,
) -> uuid::Uuid {
    sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO transaction_journals \
         (user_id, group_id, transaction_type, description, amount, currency_code, source_id, destination_id) \
         VALUES ($1, gen_random_uuid(), $2, $3, $4, $5, $6, $7) \
         RETURNING id",
    )
    .bind(user_id)
    .bind(transaction_type)
    .bind(description)
    .bind(amount)
    .bind(currency_code)
    .bind(source_id)
    .bind(destination_id)
    .fetch_one(pool)
    .await
    .expect("insert transaction journal")
}

async fn get_account_balance(pool: &sqlx::PgPool, account_id: uuid::Uuid) -> Decimal {
    sqlx::query_scalar::<_, Decimal>("SELECT current_balance FROM accounts WHERE id = $1")
        .bind(account_id)
        .fetch_one(pool)
        .await
        .expect("fetch balance")
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn creates_transaction_and_updates_balance() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let user = create_user(&test_db.pool, "alice@example.com").await;
    let principal = Principal {
        user_id: user.id,
        email: user.email.clone(),
    };
    let token = state
        .token_service
        .issue_token("test".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    let src_id = seed_account(
        &test_db.pool,
        user.id,
        "asset",
        "Checking",
        Decimal::new(100000, 2),
        "USD",
    )
    .await;
    let dst_id = seed_account(
        &test_db.pool,
        user.id,
        "expense",
        "Groceries",
        Decimal::new(0, 2),
        "USD",
    )
    .await;

    let body = serde_json::json!({
        "transaction_type": "withdrawal",
        "description": "Grocery store",
        "amount": "25.50",
        "currency_code": "USD",
        "source_id": src_id.to_string(),
        "destination_id": dst_id.to_string()
    });

    let request = Request::builder()
        .uri("/api/v1/transactions")
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

    assert_eq!(payload["data"]["type"], json!("transactions"));
    assert!(!payload["data"]["id"].as_str().unwrap().is_empty());
    assert_eq!(
        payload["data"]["attributes"]["description"],
        json!("Grocery store")
    );
    assert_eq!(payload["data"]["attributes"]["amount"], json!("25.50"));

    // Verify balance was updated atomically
    let src_balance = get_account_balance(&test_db.pool, src_id).await;
    // 1000.00 - 25.50 = 974.50
    assert_eq!(src_balance, Decimal::new(97450, 2));

    let dst_balance = get_account_balance(&test_db.pool, dst_id).await;
    assert_eq!(dst_balance, Decimal::new(0, 2));
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn creates_and_gets_transaction() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let user = create_user(&test_db.pool, "bob@example.com").await;
    let _other = create_user(&test_db.pool, "other@example.com").await;
    let principal = Principal {
        user_id: user.id,
        email: user.email.clone(),
    };
    let token = state
        .token_service
        .issue_token("test".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    let src_id = seed_account(
        &test_db.pool,
        user.id,
        "asset",
        "Wallet",
        Decimal::new(50000, 2),
        "USD",
    )
    .await;

    // Create transaction via seed
    let txn_id = create_transaction_journal(
        &test_db.pool,
        user.id,
        "deposit",
        "Salary",
        Decimal::new(200000, 2),
        "USD",
        None,
        Some(src_id),
    )
    .await;

    // Get by ID
    let request = Request::builder()
        .uri(format!("/api/v1/transactions/{txn_id}"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let payload: Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json body");

    assert_eq!(payload["data"]["type"], json!("transactions"));
    assert_eq!(
        payload["data"]["attributes"]["description"],
        json!("Salary")
    );
    assert_eq!(payload["data"]["attributes"]["type"], json!("deposit"));
    assert_eq!(
        payload["data"]["attributes"]["destination_name"],
        json!("Wallet")
    );
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn cannot_get_other_users_transaction() {
    let test_db = start_postgres().await;
    let config = test_config(test_db.database_url.clone());
    let state = build_test_state(test_db.pool.clone(), config);
    let app = build_router(state.clone());

    let user = create_user(&test_db.pool, "alice@example.com").await;
    let other = create_user(&test_db.pool, "mallory@example.com").await;

    // Alice creates a transaction
    let src_id = seed_account(
        &test_db.pool,
        user.id,
        "asset",
        "Wallet",
        Decimal::new(10000, 2),
        "USD",
    )
    .await;
    let txn_id = create_transaction_journal(
        &test_db.pool,
        user.id,
        "deposit",
        "Payment",
        Decimal::new(5000, 2),
        "USD",
        None,
        Some(src_id),
    )
    .await;

    // Mallory tries to read it
    let principal = Principal {
        user_id: other.id,
        email: other.email.clone(),
    };
    let token = state
        .token_service
        .issue_token("test".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    let request = Request::builder()
        .uri(format!("/api/v1/transactions/{txn_id}"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn lists_transactions_with_pagination() {
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
        .issue_token("test".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    let dst_id = seed_account(
        &test_db.pool,
        user.id,
        "asset",
        "Savings",
        Decimal::new(0, 2),
        "USD",
    )
    .await;

    // Create 3 deposits
    for i in 0..3 {
        create_transaction_journal(
            &test_db.pool,
            user.id,
            "deposit",
            &format!("Deposit {}", i + 1),
            Decimal::new(10000, 2),
            "USD",
            None,
            Some(dst_id),
        )
        .await;
    }

    let request = Request::builder()
        .uri("/api/v1/transactions?page=1&limit=2")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let payload: Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json body");

    assert_eq!(payload["meta"]["pagination"]["total"], json!(3));
    assert_eq!(payload["meta"]["pagination"]["count"], json!(2));
    assert_eq!(payload["meta"]["pagination"]["per_page"], json!(2));
    assert_eq!(payload["meta"]["pagination"]["current_page"], json!(1));
    assert_eq!(payload["meta"]["pagination"]["total_pages"], json!(2));
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn creates_transaction_and_deletes_it_restoring_balance() {
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
        .issue_token("test".to_string(), &principal, &test_db.pool)
        .await
        .expect("issue token")
        .data
        .token;

    let src_id = seed_account(
        &test_db.pool,
        user.id,
        "asset",
        "Checking",
        Decimal::new(50000, 2),
        "USD",
    )
    .await;

    let body = serde_json::json!({
        "transaction_type": "withdrawal",
        "description": "Test delete",
        "amount": "30.00",
        "currency_code": "USD",
        "source_id": src_id.to_string()
    });

    // Create
    let create_req = Request::builder()
        .uri("/api/v1/transactions")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .method("POST")
        .body(Body::from(serde_json::to_string(&body).expect("json")))
        .expect("request");

    let create_resp = app.clone().oneshot(create_req).await.expect("response");
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let payload: Value = serde_json::from_slice(
        &to_bytes(create_resp.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json body");
    let txn_id = payload["data"]["id"].as_str().unwrap().to_string();

    // Verify balance decreased
    let balance_after_create = get_account_balance(&test_db.pool, src_id).await;
    assert_eq!(balance_after_create, Decimal::new(47000, 2)); // 500.00 - 30.00 = 470.00

    // Delete
    let delete_req = Request::builder()
        .uri(format!("/api/v1/transactions/{txn_id}"))
        .header("authorization", format!("Bearer {token}"))
        .method("DELETE")
        .body(Body::empty())
        .expect("request");

    let delete_resp = app.oneshot(delete_req).await.expect("response");
    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

    // Verify balance restored
    let balance_after_delete = get_account_balance(&test_db.pool, src_id).await;
    assert_eq!(balance_after_delete, Decimal::new(50000, 2));
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn returns_401_when_bearer_token_is_missing_for_transactions() {
    let database_url =
        "postgres://postgres:postgres@localhost/utopia_test?sslmode=disable".to_string();
    let pool = PgPoolOptions::new()
        .connect_lazy(&database_url)
        .expect("lazy pool");

    let app = build_router(build_test_state(pool, test_config(database_url)));

    let request = Request::builder()
        .uri("/api/v1/transactions")
        .body(Body::empty())
        .expect("request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
