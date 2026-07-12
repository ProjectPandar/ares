use super::super::apply_manual_filament_map_setup_state::staged_apply_manual_filament_map_setup;

#[test]
fn apply_manual_filament_map_setup_skips_when_manual_branch_not_entered() {
    let setup = staged_apply_manual_filament_map_setup(
        false,
        &["extruder_ams_count", "filament_map"],
        &[1],
        &[2],
    );

    assert!(!setup.entered);
    assert_eq!(
        setup.resulting_print_diff_set,
        ["extruder_ams_count", "filament_map"]
    );
}

#[test]
fn apply_manual_filament_map_setup_erases_extruder_ams_count_when_entered() {
    let setup = staged_apply_manual_filament_map_setup(
        true,
        &["extruder_ams_count", "filament_map"],
        &[],
        &[],
    );

    assert!(setup.entered);
    assert_eq!(setup.resulting_print_diff_set, ["filament_map"]);
}

#[test]
fn apply_manual_filament_map_setup_suppresses_duplicate_diff_keys() {
    let setup = staged_apply_manual_filament_map_setup(
        true,
        &["extruder_ams_count", "filament_map", "filament_map"],
        &[],
        &[],
    );

    assert_eq!(setup.resulting_print_diff_set, ["filament_map"]);
}

#[test]
fn apply_manual_filament_map_setup_preserves_map_value_order_duplicates_and_negative_values() {
    let setup = staged_apply_manual_filament_map_setup(true, &[], &[2, -1, 2], &[3, 1, 3]);

    assert_eq!(setup.old_filament_map, [2, -1, 2]);
    assert_eq!(setup.new_filament_map, [3, 1, 3]);
}
