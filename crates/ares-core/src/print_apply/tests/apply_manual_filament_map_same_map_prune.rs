use super::super::apply_manual_filament_map_same_map_prune_state::staged_apply_manual_filament_map_same_map_prune;

#[test]
fn apply_manual_filament_map_same_map_prune_skips_when_lengths_differ() {
    let prune =
        staged_apply_manual_filament_map_same_map_prune(&["filament_map"], &[1, 2], &[1], &[0, 1]);

    assert!(!prune.size_matched);
    assert_eq!(prune.same_map, None);
    assert!(prune.visited_indices.is_empty());
    assert!(!prune.erased_filament_map);
    assert_eq!(prune.resulting_print_diff_set, ["filament_map"]);
}

#[test]
fn apply_manual_filament_map_same_map_prune_erases_when_maps_equal() {
    let prune = staged_apply_manual_filament_map_same_map_prune(
        &["filament_map", "support_material"],
        &[1, 2],
        &[1, 2],
        &[0, 1],
    );

    assert!(prune.size_matched);
    assert_eq!(prune.same_map, Some(true));
    assert_eq!(prune.visited_indices, [0, 1]);
    assert!(prune.erased_filament_map);
    assert_eq!(prune.resulting_print_diff_set, ["support_material"]);
}

#[test]
fn apply_manual_filament_map_same_map_prune_erases_when_differences_are_unused() {
    let prune = staged_apply_manual_filament_map_same_map_prune(
        &["filament_map"],
        &[1, 2, 3],
        &[1, 9, 3],
        &[0, 2],
    );

    assert_eq!(prune.same_map, Some(true));
    assert_eq!(prune.visited_indices, [0, 1, 2]);
    assert!(prune.erased_filament_map);
    assert!(prune.resulting_print_diff_set.is_empty());
}

#[test]
fn apply_manual_filament_map_same_map_prune_keeps_key_for_used_difference() {
    let prune = staged_apply_manual_filament_map_same_map_prune(
        &["filament_map"],
        &[1, 2],
        &[1, 3],
        &[0, 1],
    );

    assert_eq!(prune.same_map, Some(false));
    assert_eq!(prune.first_used_difference_index, Some(1));
    assert!(!prune.erased_filament_map);
    assert_eq!(prune.resulting_print_diff_set, ["filament_map"]);
}

#[test]
fn apply_manual_filament_map_same_map_prune_stops_at_first_used_difference() {
    let prune = staged_apply_manual_filament_map_same_map_prune(
        &["filament_map"],
        &[1, 2, 3],
        &[9, 8, 7],
        &[0, 1, 2],
    );

    assert_eq!(prune.first_used_difference_index, Some(0));
    assert_eq!(prune.visited_indices, [0]);
}

#[test]
fn apply_manual_filament_map_same_map_prune_suppresses_duplicate_diff_keys() {
    let prune = staged_apply_manual_filament_map_same_map_prune(
        &["filament_map", "support_material", "support_material"],
        &[1],
        &[2],
        &[0],
    );

    assert_eq!(prune.same_map, Some(false));
    assert_eq!(
        prune.resulting_print_diff_set,
        ["filament_map", "support_material"]
    );
}
