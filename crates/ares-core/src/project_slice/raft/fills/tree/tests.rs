use super::super::tests::{MM, sheath_spec, square};
use super::*;
use crate::project_slice::raft::fills::raft_layer_entities;

const SIDE: i64 = 20 * MM;

/// `with_sheath && density == 0` routes to `tree_supports_generate_paths`
/// (`SupportCommon.cpp:719-722`).
fn tree_spec(double_wall_area: f64) -> RaftFillSpec {
    RaftFillSpec {
        density: 0.0,
        tree_double_wall_area: double_wall_area,
        ..sheath_spec(0.4)
    }
}

/// Distance from the square `0..SIDE` boundary, mm.
fn inset(point: Point) -> f64 {
    let d = point
        .x()
        .min(point.y())
        .min(SIDE - point.x())
        .min(SIDE - point.y());
    d as f64 / MM as f64
}

fn length(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| ((pair[1].x() - pair[0].x()) as f64).hypot((pair[1].y() - pair[0].y()) as f64))
        .sum::<f64>()
        / MM as f64
}

/// A single wall: one oriented loop 0.5·width inside, which starts with an
/// anchor on the loop one width further in, `6·spacing` long in total
/// (`:614-697`).
#[test]
fn single_wall_loop_starts_with_an_anchor_from_the_inner_loop() {
    let out = raft_layer_entities(
        &[square(0, SIDE)],
        tree_spec(f64::MAX),
        CoordinateScale::Normal,
    )
    .unwrap();
    let [RaftEntity::Oriented(polyline)] = out.as_slice() else {
        panic!("expected one oriented loop, got {out:?}");
    };
    let points = polyline.points();
    let anchor = points
        .iter()
        .take_while(|&&point| (inset(point) - 0.63).abs() < 0.01)
        .count();
    assert!(anchor >= 2, "anchor prefix {anchor}");
    // Foot distance 0.42 + the cut along the inner loop = 6·0.4.
    let seam_gap = length(&points[anchor - 1..=anchor]);
    assert!((length(&points[..anchor]) + seam_gap - 2.4).abs() < 0.01);
    assert!(
        points[anchor..]
            .iter()
            .all(|&point| (inset(point) - 0.21).abs() < 0.01)
    );
}

/// A branch larger than the double-wall area keeps the outer loop and the
/// anchored inner loop in one `no_sort` collection, inner first
/// (`:593-609,699-702`).
#[test]
fn double_wall_prints_the_inner_anchored_loop_first() {
    let out =
        raft_layer_entities(&[square(0, SIDE)], tree_spec(0.1), CoordinateScale::Normal).unwrap();
    let [RaftEntity::NoSort(polylines)] = out.as_slice() else {
        panic!("expected one no_sort collection, got {out:?}");
    };
    assert_eq!(polylines.len(), 2);
    assert!((inset(polylines[0].points()[0]) - 1.05).abs() < 0.01);
    assert!(
        polylines[1]
            .points()
            .iter()
            .all(|&point| (inset(point) - 0.21).abs() < 0.01)
    );
}

/// A hole loop takes its anchor from the grown hole (`Z = 1`).
#[test]
fn hole_loop_anchors_on_the_grown_hole() {
    let mut hole = square(8 * MM, 12 * MM);
    hole.reverse();
    let out = raft_layer_entities(
        &[square(0, SIDE), hole],
        tree_spec(f64::MAX),
        CoordinateScale::Normal,
    )
    .unwrap();
    let [RaftEntity::Oriented(_), RaftEntity::Oriented(hole_loop)] = out.as_slice() else {
        panic!("expected contour and hole loops, got {out:?}");
    };
    let start = hole_loop.points()[0];
    let from_center = (start.x() - 10 * MM).abs().max((start.y() - 10 * MM).abs());
    assert!(
        (from_center as f64 / MM as f64 - 2.63).abs() < 0.01,
        "{start:?}"
    );
}
