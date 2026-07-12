use super::super::normalized_non_diff_stride1_target_temp;

#[test]
fn stride1_target_temp_resize_leaves_matching_size_clone_unchanged() {
    let target = vec!["a".to_owned(), "b".to_owned()];

    let temporary = normalized_non_diff_stride1_target_temp(&target, 2);

    assert_eq!(temporary, target);
}

#[test]
fn stride1_target_temp_resize_does_not_mutate_original_target() {
    let target = vec![1, 2];

    let temporary = normalized_non_diff_stride1_target_temp(&target, 4);

    assert_eq!(target, vec![1, 2]);
    assert_eq!(temporary, vec![1, 2, 1, 1]);
}

#[test]
fn stride1_target_temp_resize_zero_expected_size_returns_empty_temporary() {
    let target = vec![1, 2, 3];

    let temporary = normalized_non_diff_stride1_target_temp(&target, 0);

    assert!(temporary.is_empty());
}

#[test]
fn stride1_target_temp_resize_truncates_oversized_target_temporary() {
    let target = vec![1, 2, 3, 4];

    let temporary = normalized_non_diff_stride1_target_temp(&target, 2);

    assert_eq!(temporary, vec![1, 2]);
}

#[test]
fn stride1_target_temp_resize_extends_with_first_target_value() {
    let target = vec![7, 8];

    let temporary = normalized_non_diff_stride1_target_temp(&target, 5);

    assert_eq!(temporary, vec![7, 8, 7, 7, 7]);
}

#[test]
fn stride1_target_temp_resize_accepts_non_float_vector_elements_without_inspection() {
    let target = vec![true];

    let temporary = normalized_non_diff_stride1_target_temp(&target, 3);

    assert_eq!(temporary, vec![true, true, true]);
}
