use std::cmp::Ordering;

use super::super::print_object_status_state::{
    StagedPrintObjectApplyStatus, StagedPrintObjectStatus, StagedPrintObjectStatusDb,
};

#[test]
fn print_object_status_defaults_to_unknown() {
    let status = StagedPrintObjectStatus::new(42);

    assert_eq!(status.id, 42);
    assert_eq!(status.status, StagedPrintObjectApplyStatus::Unknown);
}

#[test]
fn print_object_status_stores_each_status_variant() {
    let variants = [
        StagedPrintObjectApplyStatus::Unknown,
        StagedPrintObjectApplyStatus::Deleted,
        StagedPrintObjectApplyStatus::Reused,
        StagedPrintObjectApplyStatus::New,
    ];

    for variant in variants {
        assert_eq!(
            StagedPrintObjectStatus::with_status(7, variant).status,
            variant
        );
    }
}

#[test]
fn print_object_status_ordering_uses_id_only() {
    let deleted = StagedPrintObjectStatus::with_status(9, StagedPrintObjectApplyStatus::Deleted);
    let reused = StagedPrintObjectStatus::with_status(9, StagedPrintObjectApplyStatus::Reused);
    let newer = StagedPrintObjectStatus::with_status(10, StagedPrintObjectApplyStatus::New);

    assert_eq!(deleted, reused);
    assert_eq!(deleted.cmp(&reused), Ordering::Equal);
    assert!(deleted < newer);
}

#[test]
fn print_object_status_db_from_ids_creates_unknown_records() {
    let db = StagedPrintObjectStatusDb::from_ids([20, 10]);

    let pairs: Vec<(u64, StagedPrintObjectApplyStatus)> = db
        .records()
        .map(|record| (record.id, record.status))
        .collect();

    assert_eq!(
        pairs,
        vec![
            (10, StagedPrintObjectApplyStatus::Unknown),
            (20, StagedPrintObjectApplyStatus::Unknown),
        ]
    );
}

#[test]
fn print_object_status_db_preserves_duplicate_ids() {
    let db = StagedPrintObjectStatusDb::from_ids([20, 10, 20]);

    let ids: Vec<u64> = db.records().map(|record| record.id).collect();

    assert_eq!(ids, vec![10, 20, 20]);
}

#[test]
fn print_object_status_db_get_range_returns_only_matching_records() {
    let db = StagedPrintObjectStatusDb::from_ids([20, 10, 20, 30]);

    let records: Vec<(u64, StagedPrintObjectApplyStatus)> = db
        .get_range(20)
        .map(|record| (record.id, record.status))
        .collect();

    assert_eq!(
        records,
        vec![
            (20, StagedPrintObjectApplyStatus::Unknown),
            (20, StagedPrintObjectApplyStatus::Unknown),
        ]
    );
    assert!(db.get_range(40).next().is_none());
}

#[test]
fn print_object_status_db_count_returns_duplicate_count() {
    let db = StagedPrintObjectStatusDb::from_ids([20, 10, 20]);

    assert_eq!(db.count(20), 2);
    assert_eq!(db.count(10), 1);
    assert_eq!(db.count(40), 0);
}

#[test]
fn print_object_status_db_clear_removes_all_records() {
    let mut db = StagedPrintObjectStatusDb::from_ids([20, 10]);

    db.clear();

    assert_eq!(db.count(20), 0);
    assert!(db.records().next().is_none());
}
