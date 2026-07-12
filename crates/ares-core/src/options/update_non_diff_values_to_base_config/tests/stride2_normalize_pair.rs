use super::super::normalize_non_diff_stride2_restore_pair;

#[test]
fn stride2_normalize_pair_normalizes_source_in_place_and_target_temporary() {
    let mut source = vec![10.0, 20.0];
    let target = vec![30.0, 40.0];

    let target_tmp = normalize_non_diff_stride2_restore_pair(&mut source, &target, 6);

    assert_eq!(source, vec![10.0, 20.0, 10.0, 20.0, 10.0, 20.0]);
    assert_eq!(target, vec![30.0, 40.0]);
    assert_eq!(target_tmp, vec![30.0, 40.0, 30.0, 40.0, 30.0, 40.0]);
}

#[test]
fn stride2_normalize_pair_uses_same_expected_size_for_source_and_target() {
    let mut source = vec![1.0];
    let target = vec![2.0];

    let target_tmp = normalize_non_diff_stride2_restore_pair(&mut source, &target, 4);

    assert_eq!(source, vec![1.0, 1.0, 1.0, 1.0]);
    assert_eq!(target_tmp, vec![2.0, 2.0, 2.0, 2.0]);
}

#[test]
fn stride2_normalize_pair_zero_expected_size_clears_source_and_target_temporary() {
    let mut source = vec![1.0, 2.0];
    let target = vec![3.0, 4.0];

    let target_tmp = normalize_non_diff_stride2_restore_pair(&mut source, &target, 0);

    assert!(source.is_empty());
    assert_eq!(target, vec![3.0, 4.0]);
    assert!(target_tmp.is_empty());
}

#[test]
fn stride2_normalize_pair_zero_fills_empty_vectors_for_nonzero_expected_size() {
    let mut source = Vec::new();
    let target = Vec::new();

    let target_tmp = normalize_non_diff_stride2_restore_pair(&mut source, &target, 4);

    assert_eq!(source, vec![0.0, 0.0, 0.0, 0.0]);
    assert_eq!(target_tmp, vec![0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn stride2_normalize_pair_truncates_oversized_vectors() {
    let mut source = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let target = vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0];

    let target_tmp = normalize_non_diff_stride2_restore_pair(&mut source, &target, 4);

    assert_eq!(source, vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(target_tmp, vec![7.0, 8.0, 9.0, 10.0]);
}
