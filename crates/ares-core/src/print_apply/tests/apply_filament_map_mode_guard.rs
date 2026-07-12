use super::super::apply_filament_map_mode_guard_state::staged_apply_filament_map_mode_guard;

#[test]
fn apply_filament_map_mode_guard_enters_when_key_absent() {
    let guard = staged_apply_filament_map_mode_guard(&["filament_map"]);

    assert!(guard.enter_filament_map_processing);
}

#[test]
fn apply_filament_map_mode_guard_skips_when_key_present() {
    let guard = staged_apply_filament_map_mode_guard(&["filament_map_mode"]);

    assert!(!guard.enter_filament_map_processing);
}

#[test]
fn apply_filament_map_mode_guard_suppresses_duplicate_membership() {
    let guard = staged_apply_filament_map_mode_guard(&["filament_map", "filament_map"]);

    assert_eq!(guard.set_size, 1);
    assert!(guard.enter_filament_map_processing);
}

#[test]
fn apply_filament_map_mode_guard_ignores_non_guard_keys() {
    let guard = staged_apply_filament_map_mode_guard(&[
        "filament_map",
        "extruder_ams_count",
        "support_material",
    ]);

    assert_eq!(guard.set_size, 3);
    assert!(guard.enter_filament_map_processing);
}
