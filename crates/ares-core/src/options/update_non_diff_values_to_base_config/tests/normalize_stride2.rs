use super::super::normalize_stride2_floats;

#[test]
fn normalize_stride2_clears_when_expected_size_is_zero() {
    let mut values = vec![1.0, 2.0, 3.0];
    normalize_stride2_floats(&mut values, 0);
    assert!(values.is_empty());
}

#[test]
fn normalize_stride2_empty_nonzero_expected_size_fills_zeroes() {
    let mut values = Vec::new();
    normalize_stride2_floats(&mut values, 4);
    assert_eq!(values, vec![0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn normalize_stride2_single_value_repeats_first_value_as_pair() {
    let mut values = vec![7.0];
    normalize_stride2_floats(&mut values, 6);
    assert_eq!(values, vec![7.0, 7.0, 7.0, 7.0, 7.0, 7.0]);
}

#[test]
fn normalize_stride2_odd_length_appends_original_second_then_replicates_first_pair() {
    let mut values = vec![1.0, 2.0, 3.0];
    normalize_stride2_floats(&mut values, 8);
    assert_eq!(values, vec![1.0, 2.0, 3.0, 2.0, 1.0, 2.0, 1.0, 2.0]);
}

#[test]
fn normalize_stride2_truncates_oversized_vectors() {
    let mut values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    normalize_stride2_floats(&mut values, 3);
    assert_eq!(values, vec![1.0, 2.0, 3.0]);
}

#[test]
fn normalize_stride2_preserves_existing_pairs_and_fills_missing_pairs() {
    let mut values = vec![1.0, 2.0, 3.0, 4.0];
    normalize_stride2_floats(&mut values, 8);
    assert_eq!(values, vec![1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 1.0, 2.0]);
}

#[test]
fn normalize_stride2_odd_expected_size_matches_upstream_integer_division() {
    let mut values = vec![1.0, 2.0];
    normalize_stride2_floats(&mut values, 5);
    assert_eq!(values, vec![1.0, 2.0, 1.0, 2.0, 0.0]);
}
