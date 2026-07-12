use super::super::apply_non_diff_stride2_set_with_restore;
use crate::SliceError;

#[test]
fn stride2_set_with_restore_all_negative_indexes_keep_target_pairs() {
    let mut source = vec![10.0, 11.0, 20.0, 21.0];
    let target = vec![30.0, 31.0, 40.0, 41.0];

    apply_non_diff_stride2_set_with_restore(&mut source, &target, &[-1, -1]).unwrap();

    assert_eq!(source, target);
}

#[test]
fn stride2_set_with_restore_restores_selected_source_pairs() {
    let mut source = vec![10.0, 11.0, 20.0, 21.0, 30.0, 31.0];
    let target = vec![100.0, 101.0, 200.0, 201.0, 300.0, 301.0];

    apply_non_diff_stride2_set_with_restore(&mut source, &target, &[2, -1, 0]).unwrap();

    assert_eq!(source, vec![30.0, 31.0, 200.0, 201.0, 10.0, 11.0]);
}

#[test]
fn stride2_set_with_restore_duplicate_indexes_restore_same_pair() {
    let mut source = vec![10.0, 11.0, 20.0, 21.0];
    let target = vec![100.0, 101.0, 200.0, 201.0];

    apply_non_diff_stride2_set_with_restore(&mut source, &target, &[1, 1]).unwrap();

    assert_eq!(source, vec![20.0, 21.0, 20.0, 21.0]);
}

#[test]
fn stride2_set_with_restore_invalid_target_size_errors_after_replacing_source() {
    let mut source = vec![10.0, 11.0, 20.0, 21.0];
    let target = vec![100.0, 101.0];
    let error =
        apply_non_diff_stride2_set_with_restore(&mut source, &target, &[0, -1]).unwrap_err();

    assert_eq!(source, target);
    let SliceError::InvalidInput(message) = error else {
        panic!("expected InvalidInput");
    };
    assert!(message.contains("set_with_restore"));
    assert!(message.contains("invalid restore_index size"));
}
