use super::super::{CoordinateScale, Line, Point, Polygon, detect_bridging_direction};

mod pca;
mod selection;

type DetectBridgingDirection = fn(&[Line], &[Polygon], CoordinateScale) -> ((f64, f64), f64);

fn point(x: i64, y: i64) -> Point {
    Point::new(x, y)
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| point(x, y)).collect())
}

fn line(ax: i64, ay: i64, bx: i64, by: i64) -> Line {
    Line::new(point(ax, ay), point(bx, by))
}

fn detect_polygon(points: &[(i64, i64)], scale: CoordinateScale) -> ((f64, f64), f64) {
    detect_bridging_direction(&[], &[polygon(points)], scale)
}

fn assert_output_bits(actual: ((f64, f64), f64), expected: (u64, u64, u64)) {
    assert_eq!(
        (
            actual.0.0.to_bits(),
            actual.0.1.to_bits(),
            actual.1.to_bits()
        ),
        expected
    );
}

#[test]
fn task22o38_exact_crate_private_function_shape_is_reachable() {
    let function: DetectBridgingDirection = detect_bridging_direction;
    let _: ((f64, f64), f64) = function(&[], &[], CoordinateScale::Normal);
}
