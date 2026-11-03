//! `Geometry/Circle.hpp` at v2.5.0-beta `345efec64`: `circle_center` (`:18-36`) and the smallest
//! enclosing circle by Welzl's algorithm (`smallest_enclosing_circle2_welzl`, `:112-141`;
//! `smallest_enclosing_circle_welzl`, `:145-154`).

use super::Point;
use crate::arrangement::SCALING_FACTOR;

/// The circumcentre of `a`, `b` and `c` (`circle_center`, `:18-36`); for collinear points within
/// `epsilon`, the centre of the two furthest apart.
pub(crate) fn circle_center(
    a: [f64; 2],
    b_src: [f64; 2],
    c_src: [f64; 2],
    epsilon: f64,
) -> [f64; 2] {
    let b = [b_src[0] - a[0], b_src[1] - a[1]];
    let c = [c_src[0] - a[0], c_src[1] - a[1]];
    let lb = b[0] * b[0] + b[1] * b[1];
    let lc = c[0] * c[0] + c[1] * c[1];
    let d = b[0] * c[1] - b[1] * c[0];
    if d.abs() < epsilon {
        let bc = [c_src[0] - b_src[0], c_src[1] - b_src[1]];
        let lbc = bc[0] * bc[0] + bc[1] * bc[1];
        let (first, last) = if lb > lc && lb > lbc {
            (a, b_src)
        } else if lc > lb && lc > lbc {
            (a, c_src)
        } else {
            (b_src, c_src)
        };
        return [0.5 * (first[0] + last[0]), 0.5 * (first[1] + last[1])];
    }
    let v = [lc * b[0] - lb * c[0], lc * b[1] - lb * c[1]];
    [a[0] + -v[1] / (2.0 * d), a[1] + v[0] / (2.0 * d)]
}

/// `Circled`: a centre and radius, here in scaled coordinates.
pub(crate) struct Circle {
    pub(crate) center: [f64; 2],
    pub(crate) radius: f64,
}

/// `CircleSq<Vec2d>` (`:40-63`): a centre and squared radius.
struct CircleSq {
    center: [f64; 2],
    radius2: f64,
}

impl CircleSq {
    fn pair(a: [f64; 2], b: [f64; 2]) -> Self {
        let center = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
        Self {
            center,
            radius2: squared_distance(a, center),
        }
    }

    fn triple(a: [f64; 2], b: [f64; 2], c: [f64; 2], epsilon: f64) -> Self {
        let center = circle_center(a, b, c, epsilon);
        Self {
            center,
            radius2: squared_distance(a, center),
        }
    }

    fn contains(&self, p: [f64; 2]) -> bool {
        squared_distance(p, self.center) < self.radius2
    }

    fn inflated(self, epsilon: f64) -> Self {
        let r = self.radius2.sqrt() + epsilon;
        Self {
            center: self.center,
            radius2: r * r,
        }
    }
}

fn squared_distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (x, y) = (a[0] - b[0], a[1] - b[1]);
    x * x + y * y
}

/// `smallest_enclosing_circle_welzl(points)` (`:151-154`): the unshuffled Welzl loop
/// (`:112-141`) inflated by `SCALED_EPSILON` (`libslic3r.h:109`), as a radius (`Circle(CircleSq)`,
/// `:83-84`). The caller passes at least one point.
pub(crate) fn smallest_enclosing_circle_welzl(points: &[Point]) -> Circle {
    let epsilon = 1e-4 / SCALING_FACTOR;
    let points: Vec<[f64; 2]> = points
        .iter()
        .map(|p| [p.x() as f64, p.y() as f64])
        .collect();
    let p0 = points[0];
    let circle = if points.len() == 1 {
        CircleSq {
            center: p0,
            radius2: epsilon * epsilon,
        }
    } else {
        let mut circle = CircleSq::pair(p0, points[1]).inflated(epsilon);
        for (i, &p) in points.iter().enumerate().skip(2) {
            if !circle.contains(p) {
                // p is the first point on the smallest circle enclosing points[0..i].
                circle = with_point(&points[..i], p, epsilon);
            }
        }
        circle
    };
    Circle {
        center: circle.center,
        radius: if circle.radius2 <= 0.0 {
            circle.radius2
        } else {
            circle.radius2.sqrt()
        },
    }
}

/// The smallest circle enclosing `points` with `p` on it (`:121-134`).
fn with_point(points: &[[f64; 2]], p: [f64; 2], epsilon: f64) -> CircleSq {
    let mut circle = CircleSq::pair(points[0], p).inflated(epsilon);
    for (j, &q) in points.iter().enumerate().skip(1) {
        if !circle.contains(q) {
            // q is the second point on the smallest circle enclosing points[0..j] and p.
            circle = with_two_points(&points[..j], p, q, epsilon);
        }
    }
    circle
}

/// The smallest circle enclosing `points` with `p` and `q` on it (`:127-133`).
fn with_two_points(points: &[[f64; 2]], p: [f64; 2], q: [f64; 2], epsilon: f64) -> CircleSq {
    let mut circle = CircleSq::pair(p, q).inflated(epsilon);
    for &r in points {
        if !circle.contains(r) {
            circle = CircleSq::triple(p, q, r, epsilon).inflated(epsilon);
        }
    }
    circle
}
