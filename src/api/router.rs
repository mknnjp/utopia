use std::sync::Arc;

use axum::http::HeaderValue;
use axum::middleware;
use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::api::handlers::accounts::{
    create_account_handler, delete_account_handler, get_account_handler, list_accounts_handler,
    update_account_handler,
};
use crate::api::handlers::metadata::{
    get_about_handler, get_about_user_handler, list_currencies_handler,
};
use crate::api::handlers::tokens::{
    bootstrap_issue_token_handler, issue_token_handler, revoke_token_handler,
};
use crate::api::handlers::transactions::{
    create_transaction_handler, delete_transaction_handler, get_transaction_handler,
    list_account_transactions_handler, list_transactions_handler, update_transaction_handler,
};
use crate::api::middleware::accept_header_middleware;
use crate::api::middleware::rate_limiter::rate_limit_middleware;
use crate::app::AppState;
use crate::core::auth::metrics::metrics_handler;
use crate::core::auth::middleware::auth_middleware;

pub fn build_router(state: Arc<AppState>) -> Router {
    let protected = Router::new()
        .route("/api/v1/accounts", get(list_accounts_handler))
        .route("/api/v1/accounts", post(create_account_handler))
        .route("/api/v1/accounts/{id}", get(get_account_handler))
        .route("/api/v1/accounts/{id}", put(update_account_handler))
        .route("/api/v1/accounts/{id}", delete(delete_account_handler))
        .route(
            "/api/v1/accounts/{id}/transactions",
            get(list_account_transactions_handler),
        )
        .route("/api/v1/transactions", get(list_transactions_handler))
        .route("/api/v1/transactions", post(create_transaction_handler))
        .route("/api/v1/transactions/{id}", get(get_transaction_handler))
        .route("/api/v1/transactions/{id}", put(update_transaction_handler))
        .route(
            "/api/v1/transactions/{id}",
            delete(delete_transaction_handler),
        )
        .route("/api/v1/tokens", post(issue_token_handler))
        .route("/api/v1/tokens/{id}", delete(revoke_token_handler))
        .route("/api/v1/currencies", get(list_currencies_handler))
        .route("/api/v1/about", get(get_about_handler))
        .route("/api/v1/about/user", get(get_about_user_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let rate_limited = Router::new()
        .route(
            "/api/v1/bootstrap/tokens",
            post(bootstrap_issue_token_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware
                as fn(
                    axum::extract::State<std::sync::Arc<crate::app::AppState>>,
                    axum::http::Request<axum::body::Body>,
                    axum::middleware::Next,
                ) -> _,
        ));

    Router::new()
        .merge(protected)
        .merge(rate_limited)
        .route("/metrics", axum::routing::get(metrics_handler))
        .with_state(state)
        .layer(middleware::from_fn(accept_header_middleware))
        .layer(SetRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
        .layer(PropagateRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
}
