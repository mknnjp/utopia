use utopia::core::compatibility::pagination::{compute_pagination, Paginated};

#[test]
fn computes_pagination_meta() {
    let paginated = Paginated {
        total_records: 120,
        records: vec![1, 2, 3],
        current_page: 2,
        per_page: 50,
    };

    let meta = compute_pagination(&paginated);
    assert_eq!(meta.total, 120);
    assert_eq!(meta.count, 3);
    assert_eq!(meta.total_pages, 3);
}
