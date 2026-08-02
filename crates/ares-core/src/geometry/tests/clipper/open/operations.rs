use super::{coordinates, polyline, square};
use crate::geometry::clipper::{ClipOperation, Clipper, ClipperOptions, FillRule, PathRole};
use crate::geometry::{Polygon, Polyline};

pub(super) fn execute(
    operation: ClipOperation,
    subject: &[Polyline],
    clip: &[Polygon],
) -> Vec<Polyline> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper
        .add_open_paths(subject, PathRole::Subject)
        .expect("fixed subject coordinates are valid");
    clipper
        .add_closed_paths(clip, PathRole::Clip)
        .expect("fixed clip coordinates are valid");
    clipper
        .execute_polytree(operation, FillRule::NonZero, FillRule::NonZero)
        .into_open_polylines()
}

#[test]
fn task22o6_intersection_and_difference_preserve_exact_record_order_and_orientation() {
    let subject = [polyline(&[(-5, 5), (15, 5)])];
    let clip = [square()];

    assert_eq!(
        coordinates(&execute(ClipOperation::Intersection, &subject, &clip)),
        vec![vec![(10, 5), (0, 5)]]
    );
    assert_eq!(
        coordinates(&execute(ClipOperation::Difference, &subject, &clip)),
        vec![vec![(0, 5), (-5, 5)], vec![(15, 5), (10, 5)]]
    );
}

#[test]
fn task22o6_open_paths_wholly_inside_and_outside_are_not_fabricated_or_dropped() {
    let paths = [polyline(&[(2, 3), (8, 3)]), polyline(&[(20, 4), (30, 4)])];
    let clip = [square()];

    assert_eq!(
        coordinates(&execute(ClipOperation::Intersection, &paths, &clip)),
        vec![vec![(8, 3), (2, 3)]]
    );
    assert_eq!(
        coordinates(&execute(ClipOperation::Difference, &paths, &clip)),
        vec![vec![(30, 4), (20, 4)]]
    );
}

#[test]
fn task22o6_open_path_multiple_crossings_emit_source_order_records() {
    let subject = [polyline(&[(-5, 5), (25, 5)])];
    let clip = [
        square(),
        super::polygon(&[(15, 0), (20, 0), (20, 10), (15, 10)]),
    ];

    assert_eq!(
        coordinates(&execute(ClipOperation::Intersection, &subject, &clip)),
        vec![vec![(10, 5), (0, 5)], vec![(20, 5), (15, 5)]]
    );
    assert_eq!(
        coordinates(&execute(ClipOperation::Difference, &subject, &clip)),
        vec![
            vec![(0, 5), (-5, 5)],
            vec![(15, 5), (10, 5)],
            vec![(25, 5), (20, 5)],
        ]
    );
}
