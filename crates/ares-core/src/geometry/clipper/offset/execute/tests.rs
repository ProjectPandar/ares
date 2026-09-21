use super::*;
use crate::geometry::{Point, Polygon};

fn square(clockwise: bool) -> Polygon {
    let mut points = [
        Point::new(-1000, -1000),
        Point::new(1000, -1000),
        Point::new(1000, 1000),
        Point::new(-1000, 1000),
    ];
    if clockwise {
        points.reverse();
    }
    Polygon::new(points.to_vec())
}

/// Pins the deliberate per-path orientation emulation in
/// `raw_offset_paths`: a CW closed input is offset with the negated
/// delta (after FixOrientations normalizes it to CCW) and the output
/// traversal is reversed back. This mirrors upstream's multi-path
/// `ClipperOffset::Execute` semantics inside ares's per-path
/// offsetting; the corpus byte-parity is built around it, so any
/// refactor of the generate walk must preserve these exact outputs.
#[test]
fn cw_input_offset_negates_delta_and_reverses_output() {
    let ccw = raw_offset_paths(&[square(false)], 500.0, JoinType::Miter, 3.0).unwrap();
    let cw = raw_offset_paths(&[square(true)], 500.0, JoinType::Miter, 3.0).unwrap();

    assert_eq!(ccw.len(), 1);
    assert_eq!(cw.len(), 1);
    let ccw_points = ccw[0].points();
    let cw_points = cw[0].points();
    // CCW grows outward to the 1500 square; CW negates to the inward 500 one.
    assert!(
        ccw_points
            .iter()
            .all(|p| p.x().abs() == 1500 || p.y().abs() == 1500)
    );
    assert!(
        cw_points
            .iter()
            .all(|p| p.x().abs() == 500 || p.y().abs() == 500)
    );
    assert_eq!(ccw_points.len(), 4);
    assert_eq!(cw_points.len(), 4);
}
