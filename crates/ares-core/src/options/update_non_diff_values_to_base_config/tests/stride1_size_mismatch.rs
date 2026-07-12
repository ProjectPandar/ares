use super::super::non_diff_stride1_restore_sizes;

#[test]
fn stride1_size_mismatch_is_false_when_both_sizes_match_expected_size() {
    let source = ["source-a", "source-b"];
    let target = ["target-a", "target-b"];

    assert_eq!(
        non_diff_stride1_restore_sizes(&source, &target, 2),
        (2, 2, false)
    );
}

#[test]
fn stride1_size_mismatch_is_true_when_source_size_differs() {
    let source = [1, 2, 3];
    let target = [4, 5];

    assert_eq!(
        non_diff_stride1_restore_sizes(&source, &target, 2),
        (3, 2, true)
    );
}

#[test]
fn stride1_size_mismatch_is_true_when_target_size_differs() {
    let source = [1, 2];
    let target = [3, 4, 5];

    assert_eq!(
        non_diff_stride1_restore_sizes(&source, &target, 2),
        (2, 3, true)
    );
}

#[test]
fn stride1_size_mismatch_is_true_when_both_sizes_differ() {
    let source = [1];
    let target = [2, 3, 4];

    assert_eq!(
        non_diff_stride1_restore_sizes(&source, &target, 2),
        (1, 3, true)
    );
}

#[test]
fn stride1_size_mismatch_zero_expected_size_accepts_two_empty_vectors() {
    let source: [i32; 0] = [];
    let target: [i32; 0] = [];

    assert_eq!(
        non_diff_stride1_restore_sizes(&source, &target, 0),
        (0, 0, false)
    );
}

#[test]
fn stride1_size_mismatch_zero_expected_size_rejects_non_empty_side() {
    let source = [1];
    let target: [i32; 0] = [];

    assert_eq!(
        non_diff_stride1_restore_sizes(&source, &target, 0),
        (1, 0, true)
    );
}

#[test]
fn stride1_size_mismatch_accepts_non_float_vector_elements_without_inspection() {
    let source = [true, false];
    let target = ["left", "right"];

    assert_eq!(
        non_diff_stride1_restore_sizes(&source, &target, 2),
        (2, 2, false)
    );
}
