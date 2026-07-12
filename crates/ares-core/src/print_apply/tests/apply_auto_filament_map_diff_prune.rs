use super::super::apply_auto_filament_map_diff_prune_state::staged_apply_auto_filament_map_diff_prune;

#[test]
fn apply_auto_filament_map_diff_prune_skips_when_filament_map_absent() {
    let state = staged_apply_auto_filament_map_diff_prune(&["layer_height", "infill_density"]);

    assert!(!state.entered);
    assert_eq!(
        state.resulting_print_diff_set,
        ["infill_density", "layer_height"]
    );
}

#[test]
fn apply_auto_filament_map_diff_prune_enters_and_erases_filament_map() {
    let state = staged_apply_auto_filament_map_diff_prune(&["filament_map", "layer_height"]);

    assert!(state.entered);
    assert_eq!(state.resulting_print_diff_set, ["layer_height"]);
}

#[test]
fn apply_auto_filament_map_diff_prune_suppresses_duplicate_membership() {
    let state = staged_apply_auto_filament_map_diff_prune(&[
        "filament_map",
        "layer_height",
        "filament_map",
        "layer_height",
    ]);

    assert!(state.entered);
    assert_eq!(state.resulting_print_diff_set, ["layer_height"]);
}
