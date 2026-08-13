use super::*;

fn point(x: f64, y: f64) -> F64Point {
    F64Point::new(x, y)
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64) -> F64Segment {
    F64Segment::new(point(ax, ay), point(bx, by))
}

#[test]
fn task22o44_clips_surviving_subsegments_in_source_order() {
    let points = [Point::new(0, 0), Point::new(10, 0), Point::new(10, 10)];
    assert_eq!(
        clipped_infill_segments(&points, 3.0),
        vec![segment(3.0, 0.0, 10.0, 0.0), segment(10.0, 0.0, 10.0, 7.0)]
    );
}

#[test]
fn task22o44_clipping_exactly_consuming_the_path_is_empty() {
    let points = [Point::new(0, 0), Point::new(10, 0)];
    assert!(clipped_infill_segments(&points, 5.0).is_empty());
}

#[test]
fn task22o44_rounded_segment_unions_circle_and_rectangle_intervals() {
    assert_eq!(
        rounded_thick_segment_collision(
            segment(0.0, 0.0, 10.0, 0.0),
            segment(5.0, -1.0, 5.0, 1.0),
            2.0,
            0.0001,
        ),
        Some(EuclideanInterval {
            start: 3.0,
            end: 7.0,
        })
    );
}

#[test]
fn task22o44_very_short_boundary_uses_strict_squared_distance() {
    let line = segment(0.0, 0.0, 0.5, 0.0);
    assert_eq!(
        rounded_thick_segment_collision(line, segment(0.25, -10.0, 0.25, 10.0), 1.0, 1.0),
        Some(EuclideanInterval {
            start: 0.0,
            end: 0.5,
        })
    );
    assert_eq!(
        rounded_thick_segment_collision(line, segment(1.25, -10.0, 1.25, 10.0), 1.0, 1.0),
        None
    );
}

#[test]
fn task22o44_short_nonzero_thick_segment_passes_offset_as_squared_radius() {
    assert_eq!(
        collision_interval_prefiltered(
            segment(-10.0, 3.0, 10.0, 3.0),
            segment(0.0, 0.0, 0.05, 0.0),
            4.0,
            0.1,
        ),
        None
    );
}

#[test]
fn task22o44_fractional_prefilter_overlap_is_inclusive() {
    let boundary = segment(1.1, -1.0, 1.1, 1.0);
    let infill = segment(0.0, -1.0, 0.0, 1.0);
    assert!(fractional_bounds_overlap(boundary, infill, 1.1));
    assert!(!fractional_bounds_overlap(boundary, infill, 1.099));
}

#[test]
fn task22o44_traces_select_negative_then_positive_and_truncate_toward_zero() {
    let infill = segment(0.75, 0.0, 10.75, 0.0);
    assert_eq!(
        thick_trace_line(infill, 1.5, true).unwrap(),
        (Point::new(0, -1), Point::new(12, -1))
    );
    assert_eq!(
        thick_trace_line(infill, 1.5, false).unwrap(),
        (Point::new(0, 1), Point::new(12, 1))
    );
}

#[test]
fn task22o44_trace_conversion_reports_out_of_range_coordinates() {
    let max_exclusive = -(i64::MIN as f64);
    assert_eq!(
        thick_trace_line(segment(max_exclusive, 0.0, max_exclusive, 1.0), 1.0, true),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
