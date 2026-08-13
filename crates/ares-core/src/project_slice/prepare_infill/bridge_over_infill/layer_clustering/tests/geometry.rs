use crate::geometry::{ClipperError, CoordinateScale, union_polygons_paths};

use super::{candidate, points, polygon, snapshot, square, view};
use crate::project_slice::prepare_infill::bridge_over_infill::layer_clustering::{
    cluster_candidate_layers, inflated_candidate_aabb, layer_coverage,
};

#[test]
fn task22o54_inflated_aabb_preserves_source_rounding_point_order_and_scales() {
    let candidate = candidate(0, 0, vec![square(1, 2, 10)]);
    assert_eq!(
        points(&inflated_candidate_aabb(
            &candidate,
            CoordinateScale::Normal
        )),
        vec![
            (-6_999_999, -6_999_998),
            (7_000_011, -6_999_998),
            (7_000_011, 7_000_012),
            (-6_999_999, 7_000_012),
        ]
    );
    assert_eq!(
        points(&inflated_candidate_aabb(
            &candidate,
            CoordinateScale::LargeBed
        )),
        vec![
            (-699_999, -699_998),
            (700_011, -699_998),
            (700_011, 700_012),
            (-699_999, 700_012),
        ]
    );
}

#[test]
fn task22o54_empty_and_degenerate_first_polygons_keep_bounding_box_semantics() {
    let empty = candidate(0, 0, Vec::new());
    assert_eq!(
        points(&inflated_candidate_aabb(&empty, CoordinateScale::Normal)),
        vec![
            (-7_000_000, -7_000_000),
            (7_000_000, -7_000_000),
            (7_000_000, 7_000_000),
            (-7_000_000, 7_000_000),
        ]
    );

    let degenerate = candidate(0, 0, vec![polygon(&[(100, 200), (110, 200)])]);
    assert_eq!(
        points(&inflated_candidate_aabb(
            &degenerate,
            CoordinateScale::LargeBed
        )),
        vec![
            (-699_900, -699_800),
            (700_110, -699_800),
            (700_110, 700_200),
            (-699_900, 700_200),
        ]
    );

    let replaced = candidate(
        0,
        0,
        vec![polygon(&[(100, 200), (110, 200)]), square(1_000, 2_000, 10)],
    );
    assert_eq!(
        points(&inflated_candidate_aabb(
            &replaced,
            CoordinateScale::LargeBed
        )),
        vec![
            (-699_000, -698_000),
            (701_010, -698_000),
            (701_010, 702_010),
            (-699_000, 702_010),
        ]
    );

    let ignored = candidate(
        0,
        0,
        vec![
            square(1_000, 2_000, 10),
            polygon(&[(-100, -200), (100, -200)]),
        ],
    );
    assert_eq!(
        points(&inflated_candidate_aabb(
            &ignored,
            CoordinateScale::LargeBed
        )),
        vec![
            (-699_000, -698_000),
            (701_010, -698_000),
            (701_010, 702_010),
            (-699_000, 702_010),
        ]
    );
}

#[test]
fn task22o54_disjoint_candidate_boxes_are_not_replaced_by_one_combined_box() {
    let first = [
        candidate(0, 0, vec![square(0, 0, 1_000_000)]),
        candidate(0, 0, vec![square(100_000_000, 0, 1_000_000)]),
    ];
    let second = [candidate(1, 0, vec![square(50_000_000, 0, 1_000_000)])];
    let layers = [view(2, 1.0, 1.0, &first), view(4, 1.1, 1.0, &second)];
    assert_eq!(
        cluster_candidate_layers(&layers, CoordinateScale::Normal).unwrap(),
        vec![vec![2], vec![4]]
    );
}

#[test]
fn task22o54_sequential_coverage_freezes_flat_order_and_differs_from_one_shot_union() {
    let candidates = [
        candidate(0, 0, vec![square(0, 0, 3)]),
        candidate(0, 0, vec![square(-14_000_001, -14_000_001, 5)]),
        candidate(0, 0, vec![square(-14_000_001, -14_000_001, 7)]),
    ];
    let sequential = layer_coverage(&candidates, CoordinateScale::Normal).unwrap();
    assert_eq!(
        sequential.iter().map(points).collect::<Vec<_>>(),
        vec![vec![
            (-6_999_994, -21_000_001),
            (-6_999_994, -7_000_000),
            (7_000_003, -7_000_000),
            (7_000_003, 7_000_003),
            (-7_000_000, 7_000_003),
            (-7_000_000, -6_999_994),
            (-21_000_001, -6_999_994),
            (-21_000_001, -21_000_001),
        ]]
    );
    let boxes = candidates
        .iter()
        .map(|candidate| inflated_candidate_aabb(candidate, CoordinateScale::Normal))
        .collect::<Vec<_>>();
    assert_eq!(
        union_polygons_paths(&boxes)
            .unwrap()
            .iter()
            .map(points)
            .collect::<Vec<_>>(),
        vec![vec![
            (-6_999_994, -7_000_000),
            (7_000_003, -7_000_000),
            (7_000_003, 7_000_003),
            (-7_000_000, 7_000_003),
            (-7_000_000, -6_999_994),
            (-21_000_001, -6_999_994),
            (-21_000_001, -21_000_001),
            (-6_999_994, -21_000_001),
        ]]
    );
}

#[test]
fn task22o54_empty_candidate_has_exact_zero_box_coverage() {
    let candidates = [candidate(0, 0, Vec::new())];
    assert_eq!(
        layer_coverage(&candidates, CoordinateScale::Normal)
            .unwrap()
            .iter()
            .map(points)
            .collect::<Vec<_>>(),
        vec![vec![
            (7_000_000, 7_000_000),
            (-7_000_000, 7_000_000),
            (-7_000_000, -7_000_000),
            (7_000_000, -7_000_000),
        ]]
    );
}

#[test]
fn task22o54_first_closed_path_range_error_is_atomic_and_inputs_are_unchanged() {
    let candidates = [
        candidate(0, 0, vec![square(1_i64 << 62, 0, 10)]),
        candidate(0, 0, vec![square(0, 0, 10)]),
    ];
    let before = snapshot(&candidates);
    let layers = [view(0, 1.0, 1.0, &candidates)];
    assert_eq!(
        cluster_candidate_layers(&layers, CoordinateScale::Normal),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(snapshot(&candidates), before);
}
