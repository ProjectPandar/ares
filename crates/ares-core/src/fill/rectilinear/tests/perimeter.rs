use crate::geometry::{Point, Polygon};

use super::super::{
    append_contour_segment, contour_segment_length, directed_segment_distance, emit_horizontal_arc,
    emit_vertical_arc, measure_horizontal_arc, measure_vertical_arc, prepare_rectilinear_slice,
};
use super::rectangle;

fn square() -> Polygon {
    Polygon::new(vec![
        Point::new(0, 0),
        Point::new(10, 0),
        Point::new(10, 10),
        Point::new(0, 10),
    ])
}

#[test]
fn task22o83_directed_distance_length_and_vertices_preserve_wrap_direction() {
    let polygon = square();
    assert_eq!(directed_segment_distance(4, 1, 3, true), 2);
    assert_eq!(directed_segment_distance(4, 3, 1, true), 2);
    assert_eq!(directed_segment_distance(4, 1, 3, false), 2);
    assert_eq!(
        contour_segment_length(&polygon, 1, Point::new(5, 0), 3, Point::new(5, 10)).to_bits(),
        20.0_f64.to_bits()
    );

    let mut forward = Vec::new();
    append_contour_segment(&mut forward, &polygon, 1, 3, true);
    assert_eq!(forward, vec![Point::new(10, 0), Point::new(10, 10)]);

    let mut reverse = Vec::new();
    append_contour_segment(&mut reverse, &polygon, 3, 1, false);
    assert_eq!(reverse, vec![Point::new(10, 10), Point::new(10, 0)]);

    let mut same_segment = Vec::new();
    append_contour_segment(&mut same_segment, &polygon, 2, 2, false);
    assert!(same_segment.is_empty());
}

#[test]
fn task22o83_indexed_horizontal_and_vertical_arcs_include_only_source_vertices_and_end() {
    let slice = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 2, 10, 80).unwrap();
    assert_eq!(
        measure_horizontal_arc(&slice, 0, 0, 0).to_bits(),
        80.0_f64.to_bits()
    );

    let mut horizontal = Vec::new();
    emit_horizontal_arc(&slice, 0, 0, 0, true, &mut horizontal);
    assert_eq!(horizontal, vec![Point::new(90, 0)]);

    let vertical_length = measure_vertical_arc(&slice, 0, 0, 1, true);
    assert_eq!(vertical_length.to_bits(), 260.0_f64.to_bits());
    let mut vertical = Vec::new();
    emit_vertical_arc(&slice, 0, 0, 1, true, &mut vertical);
    assert_eq!(
        vertical,
        vec![Point::new(100, 0), Point::new(100, 80), Point::new(10, 80)]
    );
}
