use super::{candidate, ids, polygon, rectangle};
use crate::project_slice::prepare_infill::bridge_over_infill::candidate_ordering::{
    candidate_bounds, order_candidate_surfaces,
};

#[test]
fn task22o55_empty_single_and_two_candidates_use_minimum_x_then_y() {
    assert!(order_candidate_surfaces(Vec::new()).is_empty());

    let one = order_candidate_surfaces(vec![candidate(4, vec![rectangle(30, 40, 50, 60)])]);
    assert_eq!(ids(&one), vec![4]);

    let two = order_candidate_surfaces(vec![
        candidate(0, vec![rectangle(10, -20, 20, -10)]),
        candidate(1, vec![rectangle(10, -30, 20, -20)]),
    ]);
    assert_eq!(ids(&two), vec![1, 0]);
}

#[test]
fn task22o55_source_extent_definition_replaces_or_ignores_boxes_exactly() {
    let ordered = order_candidate_surfaces(vec![
        candidate(0, Vec::new()),
        candidate(
            1,
            vec![
                polygon(&[(-100, -100), (-100, 100)]),
                rectangle(5, 5, 10, 10),
            ],
        ),
        candidate(
            2,
            vec![
                rectangle(-5, -5, 0, 0),
                polygon(&[(-1_000, -1_000), (-1_000, 1_000)]),
            ],
        ),
        candidate(
            3,
            vec![
                polygon(&[(-10, -10), (0, 0)]),
                rectangle(100, 100, 110, 110),
            ],
        ),
    ]);
    assert_eq!(ids(&ordered), vec![3, 1, 0, 2]);
}

#[test]
fn task22o55_extent_definition_requires_positive_x_and_y() {
    let zero_width = candidate(0, vec![polygon(&[(4, -20), (4, 30)])]);
    let zero_height = candidate(1, vec![polygon(&[(-20, 4), (30, 4)])]);
    let diagonal = candidate(2, vec![polygon(&[(-10, -10), (0, 0)])]);

    let width = candidate_bounds(&zero_width);
    assert!(!width.defined);
    assert_eq!(
        (width.min_x, width.min_y, width.max_x, width.max_y),
        (4, -20, 4, 30)
    );
    let height = candidate_bounds(&zero_height);
    assert!(!height.defined);
    assert_eq!(
        (height.min_x, height.min_y, height.max_x, height.max_y),
        (-20, 4, 30, 4)
    );
    assert!(candidate_bounds(&diagonal).defined);
}
