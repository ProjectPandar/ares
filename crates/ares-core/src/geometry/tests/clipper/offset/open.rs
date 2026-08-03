use super::helpers::{coordinates, polygon};
use crate::geometry::clipper::{
    ClipperError, ClipperOffset, JoinType, offset_open_paths, raw_offset_open_paths,
};

fn raw(points: &[(i64, i64)], delta: f64) -> Vec<(i64, i64)> {
    let mut offset = ClipperOffset::default();
    offset.add_open_path(&polygon(points), JoinType::Square);
    coordinates(&offset.generate_raw(delta)[0])
}

#[test]
fn task22o14_open_butt_one_point_and_straight_caps_are_literal() {
    assert_eq!(raw(&[(5, 7)], 2.0), vec![(3, 5), (7, 5), (7, 9), (3, 9)]);
    assert_eq!(
        raw(&[(0, 0), (100, 0)], 10.0),
        vec![(100, -10), (100, 10), (0, 10), (0, -10)]
    );
    assert_eq!(
        raw(&[(100, 0), (0, 0)], 10.0),
        vec![(0, 10), (0, -10), (100, -10), (100, 10)]
    );
}

#[test]
fn task22o14_open_butt_duplicate_and_shortest_edge_filter_is_strict() {
    let mut offset = ClipperOffset::default();
    offset.set_shortest_edge_length(5.0);
    offset.add_open_path(
        &polygon(&[(0, 0), (0, 0), (3, 0), (5, 0), (11, 0)]),
        JoinType::Square,
    );
    assert_eq!(
        coordinates(&offset.generate_raw(2.0)[0]),
        vec![(5, -2), (11, -2), (11, 2), (5, 2), (0, 2), (0, -2)]
    );
}

#[test]
fn task22o14_open_butt_square_bends_preserve_side_order() {
    assert_eq!(
        raw(&[(0, 0), (100, 0), (100, 100)], 10.0),
        vec![
            (104, -10),
            (110, -4),
            (110, 100),
            (90, 100),
            (90, 0),
            (100, 0),
            (100, 10),
            (0, 10),
            (0, -10)
        ]
    );
    assert_eq!(
        raw(&[(0, 0), (100, 0), (100, -100)], 10.0),
        vec![
            (100, -10),
            (100, 0),
            (90, 0),
            (90, -100),
            (110, -100),
            (110, 4),
            (104, 10),
            (0, 10),
            (0, -10)
        ]
    );
}

#[test]
fn task22o14_open_wrapper_handles_multiple_paths_and_range_errors() {
    let paths = [polygon(&[(0, 0), (100, 0)]), polygon(&[(0, 30), (100, 30)])];
    assert_eq!(
        offset_open_paths(&paths, 10.0, JoinType::Square, 0.0)
            .unwrap()
            .len(),
        2
    );
    let overlapping = [polygon(&[(0, 0), (100, 0)]), polygon(&[(0, 0), (100, 0)])];
    let per_path_positive =
        raw_offset_open_paths(&overlapping, 10.0, JoinType::Square, 0.0).unwrap();
    assert_eq!(per_path_positive.len(), 2);
    assert_eq!(per_path_positive[0], per_path_positive[1]);
    assert_eq!(
        offset_open_paths(&overlapping, 10.0, JoinType::Square, 0.0)
            .unwrap()
            .len(),
        1
    );
    let outside = polygon(&[(0x4000_0000_0000_0000, 0), (0x4000_0000_0000_1000, 0)]);
    assert_eq!(
        offset_open_paths(&[outside], 1_024.0, JoinType::Square, 0.0),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22o14_open_wrapper_accepts_normal_and_large_bed_scaled_deltas() {
    let path = polygon(&[(0, 0), (1_000_000, 0)]);
    assert_eq!(
        offset_open_paths(
            std::slice::from_ref(&path),
            250_010.0,
            JoinType::Square,
            0.0
        )
        .unwrap()
        .len(),
        1
    );
    assert_eq!(
        offset_open_paths(&[path], 25_010.0, JoinType::Square, 0.0)
            .unwrap()
            .len(),
        1
    );
}
