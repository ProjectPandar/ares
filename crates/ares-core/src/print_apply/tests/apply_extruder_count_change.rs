use super::super::apply_extruder_count_change_state::staged_apply_extruder_count_change;

#[test]
fn apply_extruder_count_change_skips_assignment_when_counts_match() {
    let staged = staged_apply_extruder_count_change(3, 3);

    assert!(!staged.branch_taken);
    assert_eq!(staged.assigned_num_extruders, None);
    assert!(!staged.num_extruders_changed);
}

#[test]
fn apply_extruder_count_change_assigns_current_count_when_counts_differ() {
    let staged = staged_apply_extruder_count_change(2, 4);

    assert!(staged.branch_taken);
    assert_eq!(staged.assigned_num_extruders, Some(4));
    assert!(staged.num_extruders_changed);
}

#[test]
fn apply_extruder_count_change_handles_zero_counts_without_special_case() {
    let unchanged = staged_apply_extruder_count_change(0, 0);
    let changed = staged_apply_extruder_count_change(0, 1);

    assert!(!unchanged.branch_taken);
    assert_eq!(unchanged.assigned_num_extruders, None);
    assert!(!unchanged.num_extruders_changed);
    assert!(changed.branch_taken);
    assert_eq!(changed.assigned_num_extruders, Some(1));
    assert!(changed.num_extruders_changed);
}
