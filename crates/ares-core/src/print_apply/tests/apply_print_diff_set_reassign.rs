use super::super::apply_print_diff_set_reassign_state::staged_apply_print_diff_set_reassign;

#[test]
fn apply_print_diff_set_reassign_skips_when_sizes_match() {
    let reassign = staged_apply_print_diff_set_reassign(
        &["filament_map", "support_material"],
        &["filament_map", "support_material"],
    );

    assert_eq!(reassign.original_print_diff_len, 2);
    assert_eq!(reassign.print_diff_set_len, 2);
    assert!(!reassign.reassigned);
    assert_eq!(
        reassign.resulting_print_diff,
        ["filament_map", "support_material"]
    );
}

#[test]
fn apply_print_diff_set_reassign_assigns_when_key_erased() {
    let reassign = staged_apply_print_diff_set_reassign(
        &["filament_map", "support_material"],
        &["support_material"],
    );

    assert_eq!(reassign.original_print_diff_len, 2);
    assert_eq!(reassign.print_diff_set_len, 1);
    assert!(reassign.reassigned);
    assert_eq!(reassign.resulting_print_diff, ["support_material"]);
}

#[test]
fn apply_print_diff_set_reassign_assigns_when_original_had_duplicates() {
    let reassign = staged_apply_print_diff_set_reassign(
        &["filament_map", "filament_map", "support_material"],
        &["filament_map", "support_material", "support_material"],
    );

    assert_eq!(reassign.original_print_diff_len, 3);
    assert_eq!(reassign.print_diff_set_len, 2);
    assert!(reassign.reassigned);
    assert_eq!(
        reassign.resulting_print_diff,
        ["filament_map", "support_material"]
    );
}

#[test]
fn apply_print_diff_set_reassign_ignores_equal_size_membership_difference() {
    let reassign = staged_apply_print_diff_set_reassign(
        &["filament_map", "support_material"],
        &["brim", "support_material"],
    );

    assert_eq!(reassign.original_print_diff_len, 2);
    assert_eq!(reassign.print_diff_set_len, 2);
    assert!(!reassign.reassigned);
    assert_eq!(
        reassign.resulting_print_diff,
        ["filament_map", "support_material"]
    );
}
