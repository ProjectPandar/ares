use std::{cmp::Ordering, panic};

use super::super::model_object_status_state::{
    StagedModelObjectApplyStatus, StagedModelObjectStatus, StagedModelObjectStatusDb,
    StagedPrintObjectRegionsStatus,
};

#[test]
fn model_object_status_defaults_to_unknown_and_invalid_regions() {
    let status = StagedModelObjectStatus::new(42);

    assert_eq!(status.id, 42);
    assert_eq!(status.status, StagedModelObjectApplyStatus::Unknown);
    assert_eq!(
        status.print_object_regions_status,
        StagedPrintObjectRegionsStatus::Invalid
    );
}

#[test]
fn model_object_status_stores_each_apply_status_variant() {
    let variants = [
        StagedModelObjectApplyStatus::Unknown,
        StagedModelObjectApplyStatus::Old,
        StagedModelObjectApplyStatus::New,
        StagedModelObjectApplyStatus::Moved,
        StagedModelObjectApplyStatus::Deleted,
    ];

    for variant in variants {
        assert_eq!(
            StagedModelObjectStatus::with_status(7, variant).status,
            variant
        );
    }
}

#[test]
fn model_object_status_stores_each_regions_status_variant() {
    for variant in [
        StagedPrintObjectRegionsStatus::Invalid,
        StagedPrintObjectRegionsStatus::Valid,
        StagedPrintObjectRegionsStatus::PartiallyValid,
    ] {
        let mut status = StagedModelObjectStatus::new(7);
        status.print_object_regions_status = variant;

        assert_eq!(status.print_object_regions_status, variant);
    }
}

#[test]
fn model_object_status_ordering_uses_id_only() {
    let old = StagedModelObjectStatus::with_status(9, StagedModelObjectApplyStatus::Old);
    let deleted = StagedModelObjectStatus::with_status(9, StagedModelObjectApplyStatus::Deleted);
    let newer = StagedModelObjectStatus::with_status(10, StagedModelObjectApplyStatus::New);

    assert_eq!(old, deleted);
    assert_eq!(old.cmp(&deleted), Ordering::Equal);
    assert!(old < newer);
}

#[test]
fn model_object_status_db_add_inserts_records_retrievable_by_id() {
    let mut db = StagedModelObjectStatusDb::default();

    db.add(12, StagedModelObjectApplyStatus::Old);

    assert_eq!(db.get(12).id, 12);
    assert_eq!(db.get(12).status, StagedModelObjectApplyStatus::Old);
}

#[test]
fn model_object_status_db_add_panics_on_duplicate_id_without_replacing_existing_record() {
    let mut db = StagedModelObjectStatusDb::default();

    db.add(12, StagedModelObjectApplyStatus::Old);

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        db.add(12, StagedModelObjectApplyStatus::New);
    }));

    assert!(result.is_err());
    assert_eq!(db.get(12).status, StagedModelObjectApplyStatus::Old);
}

#[test]
fn model_object_status_db_add_if_new_inserts_absent_id_and_returns_true() {
    let mut db = StagedModelObjectStatusDb::default();

    assert!(db.add_if_new(12, StagedModelObjectApplyStatus::Old));

    assert_eq!(db.get(12).status, StagedModelObjectApplyStatus::Old);
}

#[test]
fn model_object_status_db_add_if_new_returns_false_and_preserves_existing_duplicate() {
    let mut db = StagedModelObjectStatusDb::default();

    db.add(12, StagedModelObjectApplyStatus::Old);

    assert!(!db.add_if_new(12, StagedModelObjectApplyStatus::New));

    assert_eq!(db.get(12).status, StagedModelObjectApplyStatus::Old);
}

#[test]
#[should_panic]
fn model_object_status_db_get_panics_on_missing_id() {
    StagedModelObjectStatusDb::default().get(12);
}

#[test]
fn model_object_status_db_reuse_returns_non_deleted_record() {
    let mut db = StagedModelObjectStatusDb::default();

    db.add(12, StagedModelObjectApplyStatus::Moved);

    assert_eq!(db.reuse(12).status, StagedModelObjectApplyStatus::Moved);
}

#[test]
#[should_panic]
fn model_object_status_db_reuse_panics_for_deleted_records() {
    let mut db = StagedModelObjectStatusDb::default();

    db.add(12, StagedModelObjectApplyStatus::Deleted);

    db.reuse(12);
}

#[test]
fn model_object_status_db_records_are_ordered_by_id() {
    let mut db = StagedModelObjectStatusDb::default();

    db.add(30, StagedModelObjectApplyStatus::Unknown);
    db.add(10, StagedModelObjectApplyStatus::Unknown);
    db.add(20, StagedModelObjectApplyStatus::Unknown);

    let ids: Vec<u64> = db.records().map(|record| record.id).collect();

    assert_eq!(ids, vec![10, 20, 30]);
}
