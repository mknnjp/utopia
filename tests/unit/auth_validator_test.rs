use std::sync::Arc;

use utopia::core::auth::cache::TokenCache;
use utopia::core::auth::error::AuthError;
use utopia::core::auth::metrics::PrometheusMetrics;

#[tokio::test]
async fn token_cache_stores_positive_and_negative_entries() {
    let cache = TokenCache::new(60, 60, 100);
    let metrics = Arc::new(PrometheusMetrics::new());

    cache
        .insert_invalid("abc".to_string(), AuthError::TokenNotFound)
        .await;

    let found = cache.get("abc").await;
    assert!(found.is_some());

    metrics.auth_cache_miss_total.inc();
}
