//! Isolated RED approved after failure-mode inventory112aa0f87.
use super::triangulate_expolygon_3d;
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};
#[path = "../../../../../tests/support/glu_native_oracle.rs"]
mod native_oracle;
use native_oracle::{NativeCase, NativeOracle};

fn compare(oracle: &mut NativeOracle, input: &str) -> Vec<[u64; 3]> {
    let case: NativeCase = serde_json::from_str(input).unwrap();
    let mut contours = case
        .contours
        .iter()
        .map(|contour| Polygon::new(contour.iter().map(|&[x, y]| Point::new(x, y)).collect()));
    let expolygon = ExPolygon::new(contours.next().unwrap(), contours.collect());
    let expected = oracle.run(&case).unwrap();
    let actual = triangulate_expolygon_3d(&expolygon, case.z, case.flip, CoordinateScale::Normal);
    let actual: Vec<_> = actual
        .into_iter()
        .map(|point| point.map(f64::to_bits))
        .collect();
    oracle.record_bits(&case, "rust", &actual).unwrap();
    assert_eq!(actual, expected, "{}", case.name);
    actual
}

fn check(input: &str) {
    let Some(mut oracle) = NativeOracle::from_env().unwrap() else {
        return;
    };
    compare(&mut oracle, input);
}

#[test]
fn glu_tessellation_quad_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/quad.json"
    ));
}

#[test]
fn glu_tessellation_translated_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/translated.json"
    ));
}

#[test]
fn glu_tessellation_clockwise_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/clockwise.json"
    ));
}

#[test]
fn glu_tessellation_concave_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/concave.json"
    ));
}

#[test]
fn glu_tessellation_one_hole_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/one_hole.json"
    ));
}

#[test]
fn glu_tessellation_two_holes_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/two_holes.json"
    ));
}

#[test]
fn glu_tessellation_holes_reordered_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/holes_reordered.json"
    ));
}

#[test]
fn glu_tessellation_convex_six_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/convex_six.json"
    ));
}

#[test]
fn glu_tessellation_flip_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/flip.json"
    ));
}

#[test]
fn glu_tessellation_float_z_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/float_z.json"
    ));
}

#[test]
fn glu_tessellation_collinear_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/collinear.json"
    ));
}

#[test]
fn glu_tessellation_duplicate_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/duplicate.json"
    ));
}

#[test]
fn glu_tessellation_empty_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/empty.json"
    ));
}

#[test]
fn glu_tessellation_point_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/point.json"
    ));
}

#[test]
fn glu_tessellation_line_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/line.json"
    ));
}

#[test]
fn glu_tessellation_crossing_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/crossing.json"
    ));
}

#[test]
fn glu_tessellation_concave_u_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/concave_u.json"
    ));
}

#[test]
fn glu_tessellation_cache_flush_101_ordered_native_triangles() {
    check(include_str!(
        "../../../../../tests/parity/glu-tessellation/inputs/cache_flush_101.json"
    ));
}

#[test]
fn glu_tessellation_repeated_a_b_a_has_no_state_leak() {
    let Some(mut oracle) = NativeOracle::from_env().unwrap() else {
        return;
    };
    let a = include_str!("../../../../../tests/parity/glu-tessellation/inputs/quad.json");
    let b = include_str!("../../../../../tests/parity/glu-tessellation/inputs/crossing.json");
    let first = compare(&mut oracle, a);
    compare(&mut oracle, b);
    let last = compare(&mut oracle, a);
    assert_eq!(first, last);
}
