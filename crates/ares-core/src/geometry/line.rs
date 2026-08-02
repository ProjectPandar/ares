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

    #[cfg(test)]
    pub(crate) fn projection(self, point: Point) -> Point {
        let (x, y) = self.projection_f64(point.x() as f64, point.y() as f64);
        Point::new(x as i64, y as i64)
    }

    #[cfg(test)]
    pub(crate) fn projection_f64(self, x: f64, y: f64) -> (f64, f64) {
        let ax = self.a.x() as f64;
        let ay = self.a.y() as f64;
        let dx = (self.b.x() - self.a.x()) as f64;
        let dy = (self.b.y() - self.a.y()) as f64;
        let t = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        (ax + t * dx, ay + t * dy)
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
