//! MultiMaterialSegmentation.cpp:36–57,958–975,1490–1523; Line.cpp121.

use crate::geometry::{ExPolygon, Line, Point, Polygon};

pub(super) fn paths(input: impl IntoIterator<Item = ExPolygon>) -> Vec<Polygon> {
    input
        .into_iter()
        .flat_map(|p| {
            let (contour, holes) = p.into_parts();
            std::iter::once(contour).chain(holes)
        })
        .collect()
}
pub(super) fn point(p: Point) -> [f64; 2] {
    [p.x() as f64, p.y() as f64]
}
pub(super) fn vector(a: Point, b: Point) -> [f64; 2] {
    [(b.x() - a.x()) as f64, (b.y() - a.y()) as f64]
}
pub(super) fn truncated(p: [f64; 2]) -> Point {
    Point::new(p[0] as i64, p[1] as i64)
}
pub(super) fn rounded(p: [f64; 2]) -> Point {
    Point::new(p[0].round() as i64, p[1].round() as i64)
}
pub(super) fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
pub(super) fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
pub(super) fn norm(a: [f64; 2]) -> f64 {
    dot(a, a).sqrt()
}
pub(super) fn unit(a: [f64; 2]) -> [f64; 2] {
    let n = norm(a);
    // Eigen's normalized() retains a zero vector.
    if n > 0.0 { [a[0] / n, a[1] / n] } else { a }
}
pub(super) fn angle(a: [f64; 2], b: [f64; 2]) -> f64 {
    let angle = dot(a, b).clamp(-1.0, 1.0).acos();
    if a[0] * b[1] - a[1] * b[0] < 0.0 {
        2.0 * std::f64::consts::PI - angle
    } else {
        angle
    }
}
pub(super) fn inside(first: Line, second: Line, p: Point) -> bool {
    let normal = |a, b| {
        let [x, y] = vector(a, b);
        unit([-y, x])
    };
    let a = normal(first.a, first.b);
    let b = normal(second.a, second.b);
    dot(unit([a[0] + b[0], a[1] + b[1]]), unit(vector(first.b, p))) > 0.0
}
pub(super) fn intersect(line: Line, other: Line, extension: f64) -> Option<Point> {
    let delta = unit(vector(line.a, line.b));
    let [x, y] = delta.map(|v| (extension * v) as i64);
    Line::new(
        Point::new(line.a.x() - x, line.a.y() - y),
        Point::new(line.b.x() + x, line.b.y() + y),
    )
    .intersection(other)
}
pub(super) fn at_start(line: Line, p: Point) -> bool {
    let a = vector(line.a, p);
    let b = vector(line.b, p);
    dot(a, a) < dot(b, b)
}
pub(super) fn ulp_equal(a: [f64; 2], b: [f64; 2]) -> bool {
    fn bits(x: f64) -> u64 {
        let bits = x.to_bits();
        if bits < 0x8000_0000_0000_0000 {
            0x8000_0000_0000_0000 - bits
        } else {
            bits
        }
    }
    // Boost voronoi_diagram_traits::vertex_equality_predicate_type::ULPS.
    (0..2).all(|i| bits(a[i]).abs_diff(bits(b[i])) <= 128)
}

pub(super) use crate::geometry::BoundingBox as Bounds;
