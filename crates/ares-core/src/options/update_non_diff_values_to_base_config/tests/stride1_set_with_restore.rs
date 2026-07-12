use super::super::apply_non_diff_stride1_set_with_restore;
use crate::SliceError;

#[test]
fn stride1_set_with_restore_restores_selected_source_values() {
    let mut source = vec![10, 20, 30];
    let target = vec![1, 2, 3];

    apply_non_diff_stride1_set_with_restore(&mut source, &target, &[2, -1, 0]).unwrap();

    assert_eq!(source, vec![30, 2, 10]);
}

#[test]
fn stride1_set_with_restore_all_negative_indexes_keep_target_values() {
    let mut source = vec![10, 20, 30];
    let target = vec![1, 2, 3];

    apply_non_diff_stride1_set_with_restore(&mut source, &target, &[-1, -1, -1]).unwrap();

    assert_eq!(source, target);
}

#[test]
fn stride1_set_with_restore_duplicate_indexes_restore_same_value() {
    let mut source = vec!["left".to_owned(), "right".to_owned()];
    let target = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];

    apply_non_diff_stride1_set_with_restore(&mut source, &target, &[1, -1, 1]).unwrap();

    assert_eq!(source, vec!["right", "b", "right"]);
}

#[test]
fn stride1_set_with_restore_invalid_target_size_errors_after_replacing_source() {
    let mut source = vec![10, 20];
    let target = vec![1];

    let error = apply_non_diff_stride1_set_with_restore(&mut source, &target, &[0, 1])
        .expect_err("target length must match restore index length");

    assert_eq!(source, target);
    assert!(
        matches!(error, SliceError::InvalidInput(message) if message == "ConfigOptionVector::set_with_restore(): Assigning from an vector with invalid restore_index size")
    );
}

#[test]
fn stride1_set_with_restore_accepts_non_numeric_vector_elements() {
    let mut source = vec![true, false];
    let target = vec![false, false];

    apply_non_diff_stride1_set_with_restore(&mut source, &target, &[0, -1]).unwrap();

    assert_eq!(source, vec![true, false]);
}
