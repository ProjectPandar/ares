use crate::geometry::Polygon;
use crate::geometry::clipper::{ClipperOffset, JoinType};

use super::helpers::{coordinates, polygon};

#[test]
fn task22g_closed_offset_discards_short_paths_but_keeps_flat_three_points() {
    let mut offset = ClipperOffset::default();
    for path in [
        Polygon::new(Vec::new()),
        polygon(&[(7, 7)]),
        polygon(&[(0, 0), (10, 0)]),
        polygon(&[(0, 0), (10, 0), (20, 0)]),
    ] {
        offset.add_closed_path(&path, JoinType::Miter);
    }

    assert_eq!(
        offset
            .generate_raw(0.0)
            .iter()
            .map(coordinates)
            .collect::<Vec<_>>(),
        vec![vec![(0, 0), (10, 0), (20, 0)]]
    );
}

#[test]
fn task22g_closed_offset_removes_terminal_and_consecutive_duplicates() {
    let mut offset = ClipperOffset::default();
    for path in [
        polygon(&[(0, 0), (100, 0), (100, 100), (0, 100), (0, 0), (0, 0)]),
        polygon(&[(200, 0), (300, 0), (300, 0), (300, 100), (200, 100)]),
    ] {
        offset.add_closed_path(&path, JoinType::Miter);
    }

    assert_eq!(
        offset
            .generate_raw(0.0)
            .iter()
            .map(coordinates)
            .collect::<Vec<_>>(),
        vec![
            vec![(0, 0), (100, 0), (100, 100), (0, 100)],
            vec![(200, 0), (300, 0), (300, 100), (200, 100)],
        ]
    );
}

#[test]
fn task22g_zero_shortest_length_uses_equality_only() {
    let input = polygon(&[(0, 0), (1, 0), (100, 0), (100, 100), (0, 100)]);
    let mut offset = ClipperOffset::default();
    offset.add_closed_path(&input, JoinType::Miter);

    assert_eq!(
        offset
            .generate_raw(0.0)
            .iter()
            .map(coordinates)
            .collect::<Vec<_>>(),
        vec![coordinates(&input)]
    );
}

#[test]
fn task22g_shortest_edge_comparison_is_strict() {
    let mut offset = ClipperOffset::default();
    offset.set_shortest_edge_length(245.0);
    for path in [
        polygon(&[(0, 0), (1000, 0), (1000, 1000), (0, 1000), (244, 22)]),
        polygon(&[(2000, 0), (3000, 0), (3000, 1000), (2000, 1000), (2245, 0)]),
        polygon(&[
            (4000, 0),
            (4244, 22),
            (4245, 0),
            (5000, 0),
            (5000, 1000),
            (4000, 1000),
        ]),
    ] {
        offset.add_closed_path(&path, JoinType::Miter);
    }

    assert_eq!(
        offset
            .generate_raw(0.0)
            .iter()
            .map(coordinates)
            .collect::<Vec<_>>(),
        vec![
            vec![(0, 0), (1000, 0), (1000, 1000), (0, 1000)],
            vec![(2000, 0), (3000, 0), (3000, 1000), (2000, 1000), (2245, 0)],
            vec![(4000, 0), (4245, 0), (5000, 0), (5000, 1000), (4000, 1000)],
        ]
    );
}

#[test]
fn task22g_global_lowest_clockwise_contour_reverses_every_closed_path() {
    let mut offset = ClipperOffset::default();
    offset.add_closed_path(
        &polygon(&[(0, 0), (10, 0), (10, 10), (0, 10)]),
        JoinType::Miter,
    );
    offset.add_closed_path(
        &polygon(&[(100, 20), (100, 40), (120, 40), (120, 20)]),
        JoinType::Miter,
    );

    assert_eq!(
        offset
            .generate_raw(0.0)
            .iter()
            .map(coordinates)
            .collect::<Vec<_>>(),
        vec![
            vec![(0, 10), (10, 10), (10, 0), (0, 0)],
            vec![(120, 20), (120, 40), (100, 40), (100, 20)],
        ]
    );
}
