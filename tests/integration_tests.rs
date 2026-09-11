// Test entry point for integration tests
// This file wires the API and DB integration tests under tests/integration/
// into Cargo's integration test runner
#[path = "integration/accounts_api_test.rs"]
mod accounts_api_test;
#[path = "integration/auth_integration_test.rs"]
mod auth_integration_test;
#[path = "integration/db_integration_test.rs"]
mod db_integration_test;
#[path = "integration/transactions_api_test.rs"]
mod transactions_api_test;
