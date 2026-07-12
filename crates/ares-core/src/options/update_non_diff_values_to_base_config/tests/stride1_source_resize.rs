use super::super::resize_non_diff_stride1_source;

#[test]
fn stride1_source_resize_leaves_matching_size_unchanged() {
    let mut source = vec!["a".to_owned(), "b".to_owned()];
    let original = source.clone();

    resize_non_diff_stride1_source(&mut source, &["target".to_owned()], 2);

    assert_eq!(source, original);
}

#[test]
fn stride1_source_resize_zero_expected_size_clears_source() {
    let mut source = vec![1, 2, 3];

    resize_non_diff_stride1_source(&mut source, &[9], 0);

    assert!(source.is_empty());
}

#[test]
fn stride1_source_resize_truncates_oversized_source() {
    let mut source = vec![1, 2, 3, 4];

    resize_non_diff_stride1_source(&mut source, &[9], 2);

    assert_eq!(source, vec![1, 2]);
}

#[test]
fn stride1_source_resize_extends_non_empty_source_with_first_source_value() {
    let mut source = vec![7, 8];

    resize_non_diff_stride1_source(&mut source, &[99], 5);

    assert_eq!(source, vec![7, 8, 7, 7, 7]);
}

#[test]
fn stride1_source_resize_extends_empty_source_with_first_target_value() {
    let mut source = Vec::new();

    resize_non_diff_stride1_source(&mut source, &[42, 43], 3);

    assert_eq!(source, vec![42, 42, 42]);
}

#[test]
fn stride1_source_resize_accepts_non_float_vector_elements_without_inspection() {
    let mut source = vec![true];

    resize_non_diff_stride1_source(&mut source, &[false], 3);

    assert_eq!(source, vec![true, true, true]);
}
