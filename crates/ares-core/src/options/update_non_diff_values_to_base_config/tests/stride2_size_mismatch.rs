use super::super::non_diff_stride2_restore_sizes;

#[test]
fn stride2_size_mismatch_is_false_when_both_sizes_match_expected_size() {
    assert_eq!(
        non_diff_stride2_restore_sizes(&[10.0, 20.0], &[30.0, 40.0], 2),
        (2, 2, false)
    );
}

#[test]
fn stride2_size_mismatch_is_true_when_source_size_differs() {
    assert_eq!(
        non_diff_stride2_restore_sizes(&[10.0], &[30.0, 40.0], 2),
        (1, 2, true)
    );
}

#[test]
fn stride2_size_mismatch_is_true_when_target_size_differs() {
    assert_eq!(
        non_diff_stride2_restore_sizes(&[10.0, 20.0], &[30.0], 2),
        (2, 1, true)
    );
}

#[test]
fn stride2_size_mismatch_is_true_when_both_sizes_differ() {
    assert_eq!(
        non_diff_stride2_restore_sizes(&[10.0], &[30.0, 40.0, 50.0], 2),
        (1, 3, true)
    );
}

#[test]
fn stride2_size_mismatch_zero_expected_size_accepts_two_empty_vectors() {
    assert_eq!(non_diff_stride2_restore_sizes(&[], &[], 0), (0, 0, false));
}

#[test]
fn stride2_size_mismatch_zero_expected_size_rejects_non_empty_side() {
    assert_eq!(
        non_diff_stride2_restore_sizes(&[10.0], &[], 0),
        (1, 0, true)
    );
    assert_eq!(
        non_diff_stride2_restore_sizes(&[], &[20.0], 0),
        (0, 1, true)
    );
}
