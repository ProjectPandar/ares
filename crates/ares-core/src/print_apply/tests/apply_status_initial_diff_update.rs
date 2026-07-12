use super::super::apply_status_initial_diff_update_state::{
    StagedApplyStatus, staged_apply_status_initial_diff_update, staged_update_apply_status,
};

#[test]
fn apply_status_initial_diff_update_keeps_unchanged_for_empty_diffs() {
    let update = staged_apply_status_initial_diff_update(0, 0, 0);

    assert_eq!(update.status, StagedApplyStatus::Unchanged);
    assert_eq!(update.log, None);
}

#[test]
fn apply_status_initial_diff_update_changes_for_print_diff() {
    let update = staged_apply_status_initial_diff_update(2, 0, 0);

    assert_eq!(update.status, StagedApplyStatus::Changed);
    let log = update
        .log
        .expect("non-empty diff should stage log metadata");
    assert_eq!(log.print_diff_len, 2);
    assert_eq!(log.object_diff_len, 0);
    assert_eq!(log.region_diff_len, 0);
}

#[test]
fn apply_status_initial_diff_update_changes_for_object_diff() {
    let update = staged_apply_status_initial_diff_update(0, 3, 0);

    assert_eq!(update.status, StagedApplyStatus::Changed);
    let log = update
        .log
        .expect("non-empty diff should stage log metadata");
    assert_eq!(log.print_diff_len, 0);
    assert_eq!(log.object_diff_len, 3);
    assert_eq!(log.region_diff_len, 0);
}

#[test]
fn apply_status_initial_diff_update_changes_for_region_diff() {
    let update = staged_apply_status_initial_diff_update(0, 0, 4);

    assert_eq!(update.status, StagedApplyStatus::Changed);
    let log = update
        .log
        .expect("non-empty diff should stage log metadata");
    assert_eq!(log.print_diff_len, 0);
    assert_eq!(log.object_diff_len, 0);
    assert_eq!(log.region_diff_len, 4);
}

#[test]
fn apply_status_initial_diff_update_helper_invalidates() {
    let mut status = StagedApplyStatus::Unchanged;

    staged_update_apply_status(&mut status, true);

    assert_eq!(status, StagedApplyStatus::Invalidated);
}

#[test]
fn apply_status_initial_diff_update_helper_does_not_downgrade_invalidated() {
    let mut status = StagedApplyStatus::Invalidated;

    staged_update_apply_status(&mut status, false);

    assert_eq!(status, StagedApplyStatus::Invalidated);
}

#[test]
fn apply_status_initial_diff_update_discriminants_preserve_max_order() {
    assert_eq!(StagedApplyStatus::Unchanged as u8, 0);
    assert_eq!(StagedApplyStatus::Changed as u8, 1);
    assert_eq!(StagedApplyStatus::Invalidated as u8, 2);
}
