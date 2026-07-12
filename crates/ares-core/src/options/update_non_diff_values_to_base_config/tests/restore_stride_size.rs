use super::super::non_diff_restore_stride_and_expected_size;

#[test]
fn restore_stride_size_uses_stride_one_when_key_is_absent_from_key_set2() {
    assert_eq!(
        non_diff_restore_stride_and_expected_size(
            "printer_extruder_variant",
            &["machine_max_speed_x"],
            3,
        ),
        (1, 3)
    );
}

#[test]
fn restore_stride_size_uses_stride_one_when_key_set2_is_empty() {
    assert_eq!(
        non_diff_restore_stride_and_expected_size("printer_extruder_variant", &[], 4),
        (1, 4)
    );
}

#[test]
fn restore_stride_size_uses_stride_two_when_key_is_in_key_set2() {
    assert_eq!(
        non_diff_restore_stride_and_expected_size(
            "machine_max_speed_x",
            &["machine_max_speed_x"],
            3
        ),
        (2, 6)
    );
}

#[test]
fn restore_stride_size_duplicate_key_set2_entries_still_use_stride_two() {
    assert_eq!(
        non_diff_restore_stride_and_expected_size(
            "machine_max_speed_x",
            &["machine_max_speed_x", "machine_max_speed_x"],
            2,
        ),
        (2, 4)
    );
}

#[test]
fn restore_stride_size_zero_restore_count_has_zero_expected_size() {
    assert_eq!(
        non_diff_restore_stride_and_expected_size("printer_extruder_variant", &[], 0),
        (1, 0)
    );
    assert_eq!(
        non_diff_restore_stride_and_expected_size(
            "machine_max_speed_x",
            &["machine_max_speed_x"],
            0
        ),
        (2, 0)
    );
}
