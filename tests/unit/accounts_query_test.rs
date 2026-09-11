use proptest::prelude::*;

use utopia::core::compatibility::pagination::{DEFAULT_LIMIT, DEFAULT_PAGE, MAX_LIMIT};
use utopia::core::error_mapping::mapper::DomainError;
use utopia::modules::accounts::AccountListQuery;

const SUPPORTED_ACCOUNT_TYPES: &[&str] = &[
    "asset",
    "cash",
    "expense",
    "revenue",
    "special",
    "hidden",
    "liability",
    "liabilities",
    "credit card",
    "default account",
    "cash account",
    "asset account",
    "expense account",
    "revenue account",
    "initial balance account",
    "beneficiary account",
    "import account",
    "reconciliation account",
    "loan",
    "debt",
    "mortgage",
];

#[test]
fn applies_default_pagination_when_query_is_empty() {
    let query = AccountListQuery::from_params(None, None, None).expect("default query");

    assert_eq!(query.page, DEFAULT_PAGE);
    assert_eq!(query.limit, DEFAULT_LIMIT);
    assert_eq!(query.account_type, None);
}

#[test]
fn treats_all_type_filter_as_unfiltered_query() {
    let query = AccountListQuery::from_params(None, None, Some("all")).expect("all filter");

    assert_eq!(query.account_type, None);
}

#[test]
fn rejects_unknown_account_type_filters() {
    let err = AccountListQuery::from_params(None, None, Some("brokerage"))
        .expect_err("invalid type must fail");

    match err {
        DomainError::Validation(fields) => {
            assert_eq!(
                fields.get("type"),
                Some(&vec!["The selected type is invalid.".to_string()])
            );
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[test]
fn rejects_limit_greater_than_maximum() {
    let err = AccountListQuery::from_params(None, Some("101"), None)
        .expect_err("oversized limit must fail");

    match err {
        DomainError::Validation(fields) => {
            assert_eq!(
                fields.get("limit"),
                Some(&vec![
                    "The limit field may not be greater than 100.".to_string()
                ])
            );
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

proptest! {
    #[test]
    fn preserves_valid_pagination_inputs(page in 1u32..10_000, limit in 1u32..=MAX_LIMIT) {
        let query = AccountListQuery::from_params(
            Some(&page.to_string()),
            Some(&limit.to_string()),
            None,
        )
        .expect("valid pagination query");

        prop_assert_eq!(query.page, page);
        prop_assert_eq!(query.limit, limit);
        prop_assert_eq!(query.account_type, None);
    }

    #[test]
    fn normalizes_supported_type_filters(type_index in 0usize..SUPPORTED_ACCOUNT_TYPES.len()) {
        let expected = SUPPORTED_ACCOUNT_TYPES[type_index];
        let expected_upper = expected.to_ascii_uppercase();
        let raw = format!("  {expected_upper}  ");

        let query = AccountListQuery::from_params(None, None, Some(&raw))
            .expect("supported type filter");

        prop_assert_eq!(query.account_type, Some(expected.to_string()));
    }
}
