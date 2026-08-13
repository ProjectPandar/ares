use crate::geometry::Point;

use super::super::tracing as production_tracing;
use super::line;

#[test]
fn task22o53_trace_bounds_exclude_low_equality_and_include_high_equality() {
    let slice = vec![line(0, 0, 0, 10), line(0, 10, 0, 20), line(0, 20, 0, 30)];
    assert_eq!(
        production_tracing::candidate_range_for_test(&slice, Point::new(0, 10), Point::new(0, 20)),
        (1, 3)
    );
}

#[test]
fn task22o53_trace_connection_is_strict_and_uses_integer_half_spacing() {
    assert_eq!(
        production_tracing::distance_squared_bits_for_test(
            Point::new(9_007_199_254_740_993, 0),
            Point::new(9_007_199_254_740_994, 0),
        ),
        1.0_f64.to_bits()
    );
    assert_eq!(
        production_tracing::connect_points_for_test(Point::new(0, 0), Point::new(59, 0), 10),
        vec![Point::new(0, 0), Point::new(59, 0)]
    );
    assert_eq!(
        production_tracing::connect_points_for_test(Point::new(0, 0), Point::new(60, 0), 10,),
        vec![
            Point::new(0, 0),
            Point::new(5, 0),
            Point::new(55, 0),
            Point::new(60, 0),
        ]
    );
    assert_eq!(
        production_tracing::connect_points_for_test(Point::new(0, 0), Point::new(66, 0), 11,),
        vec![
            Point::new(0, 0),
            Point::new(5, 0),
            Point::new(61, 0),
            Point::new(66, 0),
        ]
    );
}

#[test]
fn task22o53_used_segment_identity_closes_the_second_competing_trace() {
    let output = production_tracing::trace_sections(
        &[
            vec![line(0, 0, 0, 10), line(0, 20, 0, 30)],
            vec![line(10, 0, 10, 30)],
        ],
        10,
    );
    assert_eq!(
        output[0].points(),
        &[
            Point::new(-5, 20),
            Point::new(0, 20),
            Point::new(5, 20),
            Point::new(5, 30),
            Point::new(0, 30),
            Point::new(-5, 30),
        ]
    );
    assert_eq!(
        output[1].points(),
        &[
            Point::new(-5, 0),
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 30),
            Point::new(0, 10),
            Point::new(-5, 10),
        ]
    );
}

#[test]
fn task22o53_one_trace_splits_and_seeds_the_unused_segment() {
    let output = production_tracing::trace_sections(
        &[
            vec![line(0, 0, 0, 30)],
            vec![line(10, 0, 10, 10), line(10, 20, 10, 30)],
        ],
        10,
    );
    assert_eq!(
        output[0].points(),
        &[
            Point::new(-5, 0),
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 30),
            Point::new(-5, 30),
        ]
    );
    assert_eq!(
        output[1].points(),
        &[
            Point::new(5, 20),
            Point::new(10, 20),
            Point::new(10, 30),
            Point::new(5, 30),
        ]
    );
}

#[test]
fn task22o53_unmatched_and_final_traces_have_distinct_closing_points() {
    let first = vec![line(5, 0, 5, 20)];
    let second = vec![line(15, 0, 15, 20)];
    let closed =
        production_tracing::trace_sections(&[first.clone(), second.clone(), Vec::new()], 10);
    assert_eq!(
        closed[0].points(),
        &[
            Point::new(0, 0),
            Point::new(5, 0),
            Point::new(15, 0),
            Point::new(20, 0),
            Point::new(20, 20),
            Point::new(15, 20),
            Point::new(5, 20),
            Point::new(0, 20),
        ]
    );
    let final_open = production_tracing::trace_sections(&[first, second], 10);
    assert_eq!(
        final_open[0].points(),
        &[
            Point::new(0, 0),
            Point::new(5, 0),
            Point::new(15, 0),
            Point::new(15, 20),
            Point::new(5, 20),
            Point::new(0, 20),
        ]
    );
}
