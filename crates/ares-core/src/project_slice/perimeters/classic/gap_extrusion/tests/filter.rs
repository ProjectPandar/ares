use super::super::retention_mask;
use super::thick;

#[test]
fn task22o14_retention_is_strict_and_ordered_at_both_scaled_epsilons() {
    let normal = [
        thick(&[(0, 0), (99, 0)], &[1.0, 1.0]),
        thick(&[(0, 0), (100, 0)], &[2.0, 2.0]),
        thick(&[(0, 0), (101, 0)], &[3.0, 3.0]),
    ];
    assert_eq!(retention_mask(&normal, 100.0), vec![false, true, true]);

    let large = [
        thick(&[(0, 0), (9, 0)], &[4.0, 4.0]),
        thick(&[(0, 0), (10, 0)], &[5.0, 5.0]),
        thick(&[(0, 0), (11, 0)], &[6.0, 6.0]),
    ];
    assert_eq!(retention_mask(&large, 10.0), vec![false, true, true]);
}

#[test]
fn task22o14_zero_threshold_retains_zero_length_and_high_threshold_filters_all() {
    let polylines = [
        thick(&[(7, 9), (7, 9)], &[10.0, 20.0]),
        thick(&[(0, 0), (50, 0)], &[30.0, 40.0]),
    ];
    assert_eq!(retention_mask(&polylines, 0.0), vec![true, true]);
    assert_eq!(retention_mask(&polylines, 50.5), vec![false, false]);
}
