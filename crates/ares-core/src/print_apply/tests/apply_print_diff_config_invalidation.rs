use super::super::apply_print_diff_config_invalidation_state::{
    StagedApplyStatus, StagedPrintDiffConfigInvalidationEvent,
    staged_apply_print_diff_config_invalidation,
};

#[test]
fn apply_print_diff_config_invalidation_empty_diff_only_records_lock() {
    let update =
        staged_apply_print_diff_config_invalidation(StagedApplyStatus::Unchanged, &[], true);

    assert_eq!(update.status, StagedApplyStatus::Unchanged);
    assert_eq!(
        update.events,
        [StagedPrintDiffConfigInvalidationEvent::LockStateMutex]
    );
}

#[test]
fn apply_print_diff_config_invalidation_empty_diff_preserves_invalidated_status() {
    let update =
        staged_apply_print_diff_config_invalidation(StagedApplyStatus::Invalidated, &[], false);

    assert_eq!(update.status, StagedApplyStatus::Invalidated);
    assert_eq!(
        update.events,
        [StagedPrintDiffConfigInvalidationEvent::LockStateMutex]
    );
}

#[test]
fn apply_print_diff_config_invalidation_records_call_for_non_empty_diff() {
    let update = staged_apply_print_diff_config_invalidation(
        StagedApplyStatus::Unchanged,
        &["support_material", "filament_map"],
        false,
    );

    assert_eq!(
        update.events,
        [
            StagedPrintDiffConfigInvalidationEvent::LockStateMutex,
            StagedPrintDiffConfigInvalidationEvent::InvalidateStateByConfigOptions {
                print_diff: vec!["support_material", "filament_map"],
                invalidated: false,
            },
        ]
    );
}

#[test]
fn apply_print_diff_config_invalidation_false_result_changes_unchanged_status() {
    let update = staged_apply_print_diff_config_invalidation(
        StagedApplyStatus::Unchanged,
        &["support_material"],
        false,
    );

    assert_eq!(update.status, StagedApplyStatus::Changed);
}

#[test]
fn apply_print_diff_config_invalidation_true_result_invalidates_status() {
    let update = staged_apply_print_diff_config_invalidation(
        StagedApplyStatus::Changed,
        &["support_material"],
        true,
    );

    assert_eq!(update.status, StagedApplyStatus::Invalidated);
}

#[test]
fn apply_print_diff_config_invalidation_false_result_does_not_downgrade_invalidated() {
    let update = staged_apply_print_diff_config_invalidation(
        StagedApplyStatus::Invalidated,
        &["support_material"],
        false,
    );

    assert_eq!(update.status, StagedApplyStatus::Invalidated);
}

#[test]
fn apply_print_diff_config_invalidation_records_lock_before_call() {
    let update = staged_apply_print_diff_config_invalidation(
        StagedApplyStatus::Unchanged,
        &["support_material"],
        false,
    );

    assert_eq!(
        update.events[0],
        StagedPrintDiffConfigInvalidationEvent::LockStateMutex
    );
    assert!(matches!(
        update.events[1],
        StagedPrintDiffConfigInvalidationEvent::InvalidateStateByConfigOptions { .. }
    ));
}
