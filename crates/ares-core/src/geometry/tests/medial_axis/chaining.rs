use crate::geometry::medial_axis::{
    chaining::{
        NeighborSelection, directed_widths, select_active_neighbor, suppress_closed_endpoints,
    },
    validate::{EdgeData, rounded_point},
};
use crate::geometry::{Point, ThickPolyline};

fn edge(active: bool, width_start: f64, width_end: f64) -> EdgeData {
    EdgeData {
        active,
        width_start,
        width_end,
    }
}

#[test]
fn task22o13_seed_and_growth_points_use_source_rounding() {
    assert_eq!(rounded_point(2.5, -2.5), Point::new(3, -3));
}

#[test]
fn task22o13_zero_single_and_multiple_neighbor_selection_is_literal() {
    let select = |data: &[EdgeData]| {
        [0, 2, 4]
            .into_iter()
            .fold(NeighborSelection::None, |selection, index| {
                select_active_neighbor(selection, index, data)
            })
    };
    assert_eq!(
        select(&[
            edge(false, 0.0, 0.0),
            edge(false, 0.0, 0.0),
            edge(false, 0.0, 0.0),
        ]),
        NeighborSelection::None
    );
    assert_eq!(
        select(&[
            edge(false, 0.0, 0.0),
            edge(true, 0.0, 0.0),
            edge(false, 0.0, 0.0),
        ]),
        NeighborSelection::One(2)
    );
    assert_eq!(
        select(&[
            edge(true, 0.0, 0.0),
            edge(false, 0.0, 0.0),
            edge(true, 0.0, 0.0),
        ]),
        NeighborSelection::Multiple
    );
}

#[test]
fn task22o13_odd_reverse_edge_reverses_literal_width_direction() {
    let stored = edge(true, 1.25, 9.5);
    assert_eq!(directed_widths(8, stored), [1.25, 9.5]);
    assert_eq!(directed_widths(9, stored), [9.5, 1.25]);
}

#[test]
fn task22o13_closed_loop_suppresses_both_endpoint_flags() {
    let mut polyline = ThickPolyline {
        points: vec![Point::new(1, 2), Point::new(3, 4), Point::new(1, 2)],
        width: vec![1.0, 2.0, 3.0, 4.0],
        endpoints: (true, true),
    };
    suppress_closed_endpoints(&mut polyline);
    assert_eq!(polyline.endpoints, (false, false));
}

#[test]
fn task22o13_reused_reverse_scratch_clear_preserves_endpoint_flags() {
    let mut reverse = ThickPolyline {
        points: vec![Point::new(1, 2), Point::new(3, 4)],
        width: vec![5.0, 6.0],
        endpoints: (false, true),
    };
    reverse.clear();
    assert!(reverse.points.is_empty());
    assert!(reverse.width.is_empty());
    assert_eq!(reverse.endpoints, (false, true));
}
