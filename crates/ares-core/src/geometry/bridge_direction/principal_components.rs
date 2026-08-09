use super::super::{CoordinateScale, Polygon};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Vec2f {
    x: f32,
    y: f32,
}

impl Vec2f {
    pub(super) const ZERO: Self = Self { x: 0.0, y: 0.0 };

    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub(super) fn normalized(self) -> Self {
        let z = self.x * self.x + self.y * self.y;
        if z > 0.0 {
            let norm = z.sqrt();
            Self::new(self.x / norm, self.y / norm)
        } else {
            self
        }
    }

    pub(super) fn as_f64(self) -> (f64, f64) {
        (self.x as f64, self.y as f64)
    }
}

fn compute_moments_of_area_of_triangle(a: Vec2f, b: Vec2f, c: Vec2f) -> (f32, Vec2f, Vec2f, f32) {
    let jacobian_determinant_abs = ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs();
    let second_moment_of_area_xy = Vec2f::new(
        jacobian_determinant_abs
            * (a.x * a.x + b.x * b.x + b.x * c.x + c.x * c.x + a.x * (b.x + c.x))
            / 12.0,
        jacobian_determinant_abs
            * (a.y * a.y + b.y * b.y + b.y * c.y + c.y * c.y + a.y * (b.y + c.y))
            / 12.0,
    );
    let second_moment_of_area_covariance = jacobian_determinant_abs
        * (1.0 / 24.0)
        * (a.y * (b.x + c.x)
            + a.x * (2.0 * a.y + b.y + c.y)
            + b.y * c.x
            + b.x * c.y
            + 2.0 * b.x * b.y
            + 2.0 * c.x * c.y);
    let area = jacobian_determinant_abs * 0.5;
    let first_moment_of_area_xy = Vec2f::new(
        jacobian_determinant_abs * (a.x + b.x + c.x) / 6.0,
        jacobian_determinant_abs * (a.y + b.y + c.y) / 6.0,
    );
    (
        area,
        first_moment_of_area_xy,
        second_moment_of_area_xy,
        second_moment_of_area_covariance,
    )
}

fn unscaled(point: super::super::Point, scale: CoordinateScale) -> Vec2f {
    Vec2f::new(
        (point.x() as f64 * scale.factor()) as f32,
        (point.y() as f64 * scale.factor()) as f32,
    )
}

pub(super) fn compute_principal_components(
    polygons: &[Polygon],
    scale: CoordinateScale,
) -> (Vec2f, Vec2f) {
    let mut centroid_accumulator = Vec2f::ZERO;
    let mut second_moment_of_area_accumulator = Vec2f::ZERO;
    let mut second_moment_of_area_covariance_accumulator = 0.0_f32;
    let mut area = 0.0_f32;

    for polygon in polygons {
        let points = polygon.points();
        let p0 = unscaled(points[0], scale);
        for index in 2..points.len() {
            let p1 = unscaled(points[index - 1], scale);
            let p2 = unscaled(points[index], scale);
            let sign = if (p1.x - p0.x) * (p2.y - p1.y) - (p1.y - p0.y) * (p2.x - p1.x) > 0.0 {
                1.0
            } else {
                -1.0
            };
            let (triangle_area, first_moment, second_moment, covariance) =
                compute_moments_of_area_of_triangle(p0, p1, p2);
            area += sign * triangle_area;
            centroid_accumulator.x += sign * first_moment.x;
            centroid_accumulator.y += sign * first_moment.y;
            second_moment_of_area_accumulator.x += sign * second_moment.x;
            second_moment_of_area_accumulator.y += sign * second_moment.y;
            second_moment_of_area_covariance_accumulator += sign * covariance;
        }
    }

    if area <= 0.0 {
        return (Vec2f::ZERO, Vec2f::ZERO);
    }

    let centroid = Vec2f::new(centroid_accumulator.x / area, centroid_accumulator.y / area);
    let variance = Vec2f::new(
        second_moment_of_area_accumulator.x / area - centroid.x * centroid.x,
        second_moment_of_area_accumulator.y / area - centroid.y * centroid.y,
    );
    let covariance =
        (second_moment_of_area_covariance_accumulator / area - centroid.x * centroid.y) as f64;
    if covariance.abs() < 1e-4 {
        let result = (Vec2f::new(variance.x, 0.0), Vec2f::new(0.0, variance.y));
        return if variance.y > variance.x {
            (result.1, result.0)
        } else {
            result
        };
    }

    let difference = variance.x - variance.y;
    let root = ((difference * difference) as f64 + 4.0 * covariance * covariance).sqrt();
    let sum = variance.x + variance.y;
    let eigenvalue_a = (0.5 * (sum as f64 + root)) as f32;
    let eigenvalue_b = (0.5 * (sum as f64 - root)) as f32;
    let eigenvector_a = Vec2f::new(
        ((eigenvalue_a - variance.y) as f64 / covariance) as f32,
        1.0,
    );
    let eigenvector_b = Vec2f::new(
        ((eigenvalue_b - variance.y) as f64 / covariance) as f32,
        1.0,
    );
    if eigenvalue_a > eigenvalue_b {
        (eigenvector_a, eigenvector_b)
    } else {
        (eigenvector_b, eigenvector_a)
    }
}
