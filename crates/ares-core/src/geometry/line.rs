use super::Point;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Line {
    pub(crate) a: Point,
    pub(crate) b: Point,
}

impl Line {
    pub(crate) const fn new(a: Point, b: Point) -> Self {
        Self { a, b }
    }

    pub(crate) fn length(self) -> f64 {
        let dx = (self.b.x() - self.a.x()) as f64;
        let dy = (self.b.y() - self.a.y()) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    pub(crate) fn distance_to_squared(self, point: Point) -> f64 {
        super::simplification::distance_to_segment_squared(point, self.a, self.b)
    }

    /// `Line.cpp110–119`, `Geometry.hpp210–278` (Liang–Barsky).
    pub(crate) fn clip_with_bbox(self, bounds: super::BoundingBox) -> Option<Self> {
        let a = [self.a.x() as f64, self.a.y() as f64];
        let v = [self.b.x() as f64 - a[0], self.b.y() as f64 - a[1]];
        let min = [bounds.min().x() as f64, bounds.min().y() as f64];
        let max = [bounds.max().x() as f64, bounds.max().y() as f64];
        let (first, last) = liang_barsky_interval(a, v, min, max)?;
        Some(Self::new(
            Point::new((a[0] + first * v[0]) as i64, (a[1] + first * v[1]) as i64),
            Point::new((a[0] + last * v[0]) as i64, (a[1] + last * v[1]) as i64),
        ))
    }

    pub(crate) fn orientation(self) -> f64 {
        let angle = ((self.b.y() - self.a.y()) as f64).atan2((self.b.x() - self.a.x()) as f64);
        if angle < 0.0 {
            angle + 2.0 * std::f64::consts::PI
        } else {
            angle
        }
    }

    pub(crate) fn distance_to(self, point: Point) -> f64 {
        let vx = (self.b.x() - self.a.x()) as f64;
        let vy = (self.b.y() - self.a.y()) as f64;
        let vax = (point.x() - self.a.x()) as f64;
        let vay = (point.y() - self.a.y()) as f64;
        let l2 = vx * vx + vy * vy;
        if l2 == 0.0 {
            return (vax * vax + vay * vay).sqrt();
        }
        let t = (vax * vx + vay * vy) / l2;
        if t <= 0.0 {
            return (vax * vax + vay * vay).sqrt();
        }
        if t >= 1.0 {
            let vbx = (point.x() - self.b.x()) as f64;
            let vby = (point.y() - self.b.y()) as f64;
            return (vbx * vbx + vby * vby).sqrt();
        }
        let dx = t * vx - vax;
        let dy = t * vy - vay;
        (dx * dx + dy * dy).sqrt()
    }

    pub(crate) fn intersection(self, other: Self) -> Option<Point> {
        let v1 = (
            (self.b.x() - self.a.x()) as f64,
            (self.b.y() - self.a.y()) as f64,
        );
        let v2 = (
            (other.b.x() - other.a.x()) as f64,
            (other.b.y() - other.a.y()) as f64,
        );
        let denominator = cross(v1, v2);
        if denominator.abs() < 1e-4 {
            return None;
        }
        let v12 = (
            (self.a.x() - other.a.x()) as f64,
            (self.a.y() - other.a.y()) as f64,
        );
        let t1 = cross(v2, v12) / denominator;
        let t2 = cross(v1, v12) / denominator;
        if !(0.0..=1.0).contains(&t1) || !(0.0..=1.0).contains(&t2) {
            return None;
        }
        Some(Point::new(
            (self.a.x() as f64 + t1 * v1.0) as i64,
            (self.a.y() as f64 + t1 * v1.1) as i64,
        ))
    }
}

/// Geometry.hpp:210–278; shared by Line clipping and FillBase's collision interval.
pub(crate) fn liang_barsky_interval(
    point: [f64; 2],
    vector: [f64; 2],
    min: [f64; 2],
    max: [f64; 2],
) -> Option<(f64, f64)> {
    let mut interval = (0.0, 1.0);
    (clip_side(&mut interval, -vector[0], -min[0] + point[0])
        && clip_side(&mut interval, vector[0], max[0] - point[0])
        && clip_side(&mut interval, -vector[1], -min[1] + point[1])
        && clip_side(&mut interval, vector[1], max[1] - point[1]))
    .then_some(interval)
}

fn clip_side(interval: &mut (f64, f64), p: f64, q: f64) -> bool {
    if p == 0.0 {
        return q >= 0.0;
    }
    let ratio = q / p;
    if p < 0.0 {
        if ratio > interval.1 {
            return false;
        }
        if ratio > interval.0 {
            interval.0 = ratio;
        }
    } else {
        if ratio < interval.0 {
            return false;
        }
        if ratio < interval.1 {
            interval.1 = ratio;
        }
    }
    true
}

fn cross(left: (f64, f64), right: (f64, f64)) -> f64 {
    left.0 * right.1 - left.1 * right.0
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ThickLine {
    pub(crate) a: Point,
    pub(crate) b: Point,
    pub(crate) a_width: f64,
    pub(crate) b_width: f64,
}

impl ThickLine {
    pub(crate) const fn new(a: Point, b: Point) -> Self {
        Self {
            a,
            b,
            a_width: 0.0,
            b_width: 0.0,
        }
    }

    pub(crate) const fn with_widths(a: Point, b: Point, a_width: f64, b_width: f64) -> Self {
        Self {
            a,
            b,
            a_width,
            b_width,
        }
    }
}
