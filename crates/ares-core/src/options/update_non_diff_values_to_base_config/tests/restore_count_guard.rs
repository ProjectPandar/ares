use super::super::non_diff_restore_skips_when_child_has_more_variants;

#[test]
fn restore_count_guard_skips_when_current_has_more_variants_than_target() {
    assert!(non_diff_restore_skips_when_child_has_more_variants(3, 2));
    assert!(non_diff_restore_skips_when_child_has_more_variants(1, 0));
}

#[test]
fn restore_count_guard_does_not_skip_equal_variant_counts() {
    assert!(!non_diff_restore_skips_when_child_has_more_variants(0, 0));
    assert!(!non_diff_restore_skips_when_child_has_more_variants(2, 2));
}

#[test]
fn restore_count_guard_does_not_skip_when_current_has_fewer_variants() {
    assert!(!non_diff_restore_skips_when_child_has_more_variants(0, 1));
    assert!(!non_diff_restore_skips_when_child_has_more_variants(2, 3));
}
