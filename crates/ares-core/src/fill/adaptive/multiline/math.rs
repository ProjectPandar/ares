//! Double-precision vectors used by `FillAdaptive.cpp::noncrossing`.
use std::ops::{Add, Mul, Sub};
#[derive(Clone, Copy, Default)]
pub(super) struct V {
    pub(super) x: f64,
    pub(super) y: f64,
}
impl V {
    pub(super) const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub(super) fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
    pub(super) fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }
    pub(super) fn rotate(self, angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self::new(c * self.x - s * self.y, s * self.x + c * self.y)
    }
    pub(super) fn array(self) -> [f64; 2] {
        [self.x, self.y]
    }
}
impl Add for V {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Sub for V {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Mul<f64> for V {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}
pub(super) fn family_dir(family: usize) -> V {
    let h = 0.5 * 3.0_f64.sqrt();
    [V::new(1.0, 0.0), V::new(0.5, h), V::new(0.5, -h)][family]
}
pub(super) fn length(points: &[V]) -> f64 {
    points.windows(2).map(|p| (p[1] - p[0]).norm()).sum()
}
pub(super) fn point_along(points: &[V], mut t: f64) -> (V, usize) {
    for i in 1..points.len() {
        let d = points[i] - points[i - 1];
        let len = d.norm();
        if t <= len {
            return (points[i - 1] + d * if len > 0.0 { t / len } else { 0.0 }, i);
        }
        t -= len;
    }
    (points[points.len() - 1], points.len() - 1)
}
/// Line.hpp::line_alg::distance_to on trimmed floating endpoints.
pub(super) fn distance(a: V, b: V, p: V) -> f64 {
    let v = b - a;
    let va = p - a;
    let l2 = v.dot(v);
    if l2 == 0.0 {
        return va.norm();
    }
    let t = va.dot(v) / l2;
    if t <= 0.0 {
        va.norm()
    } else if t >= 1.0 {
        (p - b).norm()
    } else {
        (v * t - va).norm()
    }
}
