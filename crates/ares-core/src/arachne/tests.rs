use crate::{
    arachne::{ExtrusionJunction, ExtrusionLine},
    geometry::{CoordinateScale, Point},
};

use super::extrusion_line::{five_micron_tolerances, infinite_intersection};

fn junction(x: i64, y: i64, width: i64, perimeter: usize) -> ExtrusionJunction {
    ExtrusionJunction::new(Point::new(x, y), width, perimeter)
}

#[test]
fn task22o98_line_mutation_and_reversal_preserve_junction_payloads() {
    let first = junction(0, 0, 10, 2);
    let middle = junction(3, 4, 20, 3);
    let last = junction(6, 8, 30, 4);
    let mut line = ExtrusionLine::new(7, false);

    line.push(first);
    line.push(last);
    line.insert(1, middle);
    assert_eq!(line.remove(2), last);
    line.push(last);
    line.reverse();

    assert_eq!(line.inset_index, 7);
    assert!(!line.is_odd);
    assert_eq!(line.junctions, vec![last, middle, first]);
    line.clear();
    assert!(line.junctions.is_empty());
}

#[test]
fn task22o98_length_polygon_and_thick_polyline_match_source_order() {
    let mut line = ExtrusionLine::new(0, true);
    line.junctions = vec![
        junction(0, 0, 10, 0),
        junction(3, 4, 20, 0),
        junction(3, 8, 30, 0),
    ];

    assert_eq!(line.length(), 9);
    assert_eq!(
        line.to_polygon().points(),
        [Point::new(0, 0), Point::new(3, 4), Point::new(3, 8)]
    );
    let thick = line.to_thick_polyline();
    assert_eq!(thick.points, line.to_polygon().points());
    assert_eq!(thick.width, vec![10.0, 20.0, 20.0, 30.0]);
    assert_eq!(thick.endpoints, (false, false));

    line.is_closed = true;
    assert_eq!(line.length(), 17);
}

#[test]
fn task22o98_closed_orientation_and_area_match_arachne_signs() {
    let mut clockwise = ExtrusionLine::new(0, false);
    clockwise.is_closed = true;
    clockwise.junctions = vec![
        junction(0, 0, 10, 0),
        junction(0, 10, 10, 0),
        junction(10, 10, 10, 0),
        junction(10, 0, 10, 0),
        junction(0, 0, 10, 0),
    ];

    assert_eq!(clockwise.area(), -100.0);
    assert!(clockwise.is_contour());
    clockwise.reverse();
    assert_eq!(clockwise.area(), 100.0);
    assert!(!clockwise.is_contour());
}

#[test]
fn task22o98_area_deviation_uses_integer_weighted_width() {
    assert_eq!(
        ExtrusionLine::extrusion_area_deviation(
            junction(0, 0, 10, 0),
            junction(3, 0, 20, 0),
            junction(3, 4, 30, 0),
        ),
        45
    );
    assert_eq!(
        ExtrusionLine::extrusion_area_deviation(
            junction(0, 0, 10, 0),
            junction(3, 0, 11, 0),
            junction(3, 4, 10, 0),
        ),
        3
    );
}

#[test]
fn task22o98_simplification_removes_tiny_and_collinear_vertices_but_keeps_width_error() {
    let mut tiny = ExtrusionLine::new(0, false);
    tiny.junctions = vec![
        junction(0, 0, 10, 0),
        junction(1_000, 0, 100, 0),
        junction(20_000, 0, 10, 0),
    ];
    tiny.simplify(1, 0, 0, CoordinateScale::Normal);
    assert_eq!(
        tiny.junctions
            .iter()
            .map(|item| item.point)
            .collect::<Vec<_>>(),
        vec![Point::new(0, 0), Point::new(20_000, 0)]
    );

    let mut collinear = ExtrusionLine::new(0, false);
    collinear.junctions = vec![
        junction(0, 0, 10, 0),
        junction(10_000, 0, 10, 0),
        junction(20_000, 0, 10, 0),
    ];
    collinear.simplify(1, 0, 0, CoordinateScale::Normal);
    assert_eq!(collinear.junctions.len(), 2);

    let mut width_change = ExtrusionLine::new(0, false);
    width_change.junctions = vec![
        junction(0, 0, 10, 0),
        junction(10_000, 4_000, 30, 0),
        junction(20_000, 0, 10, 0),
    ];
    width_change.simplify(1, 0, 0, CoordinateScale::Normal);
    assert_eq!(width_change.junctions.len(), 3);
}

#[test]
fn task22o98_large_bed_keeps_integer_and_floating_five_micron_tolerances_distinct() {
    let (integer, floating) = five_micron_tolerances(CoordinateScale::LargeBed);

    assert_eq!(integer, 499);
    assert!(floating > 499.0);
    assert!(floating < 500.0);
}

#[test]
fn task22o98_infinite_intersection_rejects_out_of_range_result() {
    let max = i64::MAX;
    assert_eq!(
        infinite_intersection(
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(max - 10, max),
            Point::new(max - 9, max - 1),
        ),
        None
    );
    assert_eq!(
        infinite_intersection(
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(max, 1),
            Point::new(max - 1, 2),
        ),
        None
    );
}

#[test]
fn task22o98_length_truncates_each_non_pythagorean_segment_independently() {
    let mut line = ExtrusionLine::new(0, false);
    line.junctions = vec![
        junction(0, 0, 10, 0),
        junction(1, 1, 10, 0),
        junction(3, 2, 10, 0),
    ];

    assert_eq!(line.length(), 3);
}

#[test]
fn task22o98_simplification_replaces_short_corner_with_source_intersection_payload() {
    let mut line = ExtrusionLine::new(0, false);
    line.junctions = vec![
        junction(0, 0, 10, 0),
        junction(100_000, 0, 20, 1),
        junction(105_000, 20_000, 30, 2),
        junction(1_005_000, -3_580_000, 40, 3),
    ];

    line.simplify(500_000_000, 500_000_000, i64::MAX, CoordinateScale::Normal);

    assert_eq!(line.junctions.len(), 3);
    assert_eq!(line.junctions[1], junction(110_000, 0, 30, 2));
}

#[test]
fn task22o98_closed_simplification_preserves_duplicate_closure_after_spill() {
    let mut line = ExtrusionLine::new(0, false);
    line.is_closed = true;
    line.junctions = vec![
        junction(0, 0, 10, 0),
        junction(10_000, 0, 10, 0),
        junction(20_000, 0, 10, 0),
        junction(20_000, 20_000, 10, 0),
        junction(0, 20_000, 10, 0),
        junction(0, 0, 10, 0),
    ];

    line.simplify(1, 0, 0, CoordinateScale::Normal);

    assert_eq!(line.junctions.first(), line.junctions.last());
    assert_eq!(line.junctions.len(), 5);
}
