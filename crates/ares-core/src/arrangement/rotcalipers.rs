//! libnest2d's rotating calipers (`utils/rotcalipers.hpp` at `345efec64`) as `Arrange.cpp:1105-1122`
//! instantiates them: `i64` units and `boost::rational<__int128>` ratios, which compare exactly.

use super::nest::geometry::shape_box;
use super::nest::item::{Item, rotated};
use super::nest::{Bin, geometry::NBox};
use crate::geometry::{ExPolygon, Point};
use std::cmp::Ordering;

/// `boost::rational<__int128>`, kept normalized with a positive denominator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Rational {
    num: i128,
    den: i128,
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.abs()
}

impl Rational {
    const fn int(value: i128) -> Self {
        Self { num: value, den: 1 }
    }

    /// `operator/=(int)`; `None` where boost throws `bad_rational`.
    fn div_int(self, divisor: i128) -> Option<Self> {
        if divisor == 0 {
            return None;
        }
        if self.num == 0 {
            return Some(self);
        }
        let g = gcd(self.num, divisor);
        let (num, divisor) = (self.num / g, divisor / g);
        let den = self.den * divisor;
        Some(if den < 0 {
            Self {
                num: -num,
                den: -den,
            }
        } else {
            Self { num, den }
        })
    }

    /// `operator*=(int)`.
    fn mul_int(self, factor: i128) -> Self {
        let g = gcd(factor, self.den);
        Self {
            num: self.num * (factor / g),
            den: self.den / g,
        }
    }

    /// Exact `a/b` against `c/d` by continued fractions, without overflowing.
    fn compare(self, other: Self) -> Ordering {
        let (mut a, mut b, mut c, mut d) = (self.num, self.den, other.num, other.den);
        loop {
            let (q1, r1, q2, r2) = (
                a.div_euclid(b),
                a.rem_euclid(b),
                c.div_euclid(d),
                c.rem_euclid(d),
            );
            if q1 != q2 {
                return q1.cmp(&q2);
            }
            match (r1 == 0, r2 == 0) {
                (true, true) => return Ordering::Equal,
                (true, false) => return Ordering::Less,
                (false, true) => return Ordering::Greater,
                (false, false) => (a, b, c, d) = (d, r2, b, r1),
            }
        }
    }
}

fn dot(a: Point, b: Point) -> i64 {
    a.x() * b.x() + a.y() * b.y()
}

fn perp(a: Point) -> Point {
    Point::new(a.y(), -a.x())
}

fn neg(a: Point) -> Point {
    Point::new(-a.x(), -a.y())
}

fn sub(a: Point, b: Point) -> Point {
    Point::new(a.x() - b.x(), a.y() - b.y())
}

/// `RotatedBox`: an axis and the two extents along and across it, unnormalized.
#[derive(Clone, Copy, Debug)]
struct RotatedBox {
    axis: Point,
    bottom: i64,
    right: i64,
}

impl RotatedBox {
    fn magnitude(self) -> f64 {
        (self.axis.x() as f64 * self.axis.x() as f64 + self.axis.y() as f64 * self.axis.y() as f64)
            .sqrt()
    }

    fn width(self) -> f64 {
        self.bottom.abs() as f64 / self.magnitude()
    }

    fn height(self) -> f64 {
        self.right.abs() as f64 / self.magnitude()
    }

    fn angle_to_x(self) -> f64 {
        let mut ret = (self.axis.y() as f64).atan2(self.axis.x() as f64);
        if ret.is_sign_negative() {
            ret += 2.0 * std::f64::consts::PI;
        }
        -ret
    }

    /// `rectarea<Ratio>(rbox)`.
    fn area(self) -> Option<Rational> {
        Some(
            Rational::int(i128::from(self.bottom))
                .div_int(i128::from(dot(self.axis, self.axis)))?
                .mul_int(i128::from(self.right)),
        )
    }
}

/// `rotcalipers` over a convex contour; `visit` returns whether to continue. `None` where boost
/// throws on a zero-length edge.
fn rotcalipers(contour: &[Point], mut visit: impl FnMut(RotatedBox) -> Option<bool>) -> Option<()> {
    if contour.len() < 2 {
        return Some(());
    }
    let mut last = contour.len() - 1;
    if contour[0] == contour[last] {
        last -= 1;
    }
    if last < 2 {
        return Some(());
    }
    let (p, q, r) = (contour[0], contour[1], contour[last]);
    let d = (i128::from(q.y()) - i128::from(p.y())) * (i128::from(r.x()) - i128::from(p.x()))
        - (i128::from(q.x()) - i128::from(p.x())) * (i128::from(r.y()) - i128::from(p.y()));
    let points: Vec<Point> = if d > 0 {
        contour[1..=last]
            .iter()
            .rev()
            .chain(std::iter::once(&contour[0]))
            .copied()
            .collect()
    } else {
        contour[..=last].to_vec()
    };
    let count = points.len();
    let next = |index: usize| if index == count - 1 { 0 } else { index + 1 };
    let prev = |index: usize| if index == 0 { count - 1 } else { index - 1 };
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (0, 0, 0, 0);
    for (index, &v) in points.iter().enumerate() {
        let d = sub(v, points[min_x]);
        if d.x() < 0 || (d.x() == 0 && d.y() < 0) {
            min_x = index;
        }
        let d = sub(v, points[max_x]);
        if d.x() > 0 || (d.x() == 0 && d.y() > 0) {
            max_x = index;
        }
        let d = sub(v, points[min_y]);
        if d.y() < 0 || (d.y() == 0 && d.x() > 0) {
            min_y = index;
        }
        let d = sub(v, points[max_y]);
        if d.y() > 0 || (d.y() == 0 && d.x() < 0) {
            max_y = index;
        }
    }
    let rotated_box = |w: Point, rect: &[usize; 4]| RotatedBox {
        axis: w,
        bottom: dot(w, sub(points[rect[1]], points[rect[3]])),
        right: dot(neg(perp(w)), sub(points[rect[2]], points[rect[0]])),
    };
    let mut w = Point::new(1, 0);
    let mut rect = [min_y, max_x, max_y, min_x];
    if !visit(rotated_box(w, &rect))? {
        return Some(());
    }
    let mut edgemask = vec![false; count];
    for _ in 0..count {
        if !update(&points, w, &mut rect, next)? {
            break;
        }
        if edgemask[rect[0]] {
            break;
        }
        edgemask[rect[0]] = true;
        w = sub(points[rect[0]], points[prev(rect[0])]);
        if !visit(rotated_box(w, &rect))? {
            break;
        }
    }
    Some(())
}

/// The `update` lambda: advances the support vertices whose edge makes the smallest angle with
/// the axis; `Some(false)` when the polygon is a rectangle.
fn update(
    points: &[Point],
    w: Point,
    rect: &mut [usize; 4],
    next: impl Fn(usize) -> usize,
) -> Option<bool> {
    let edge = |index: usize| sub(points[next(index)], points[index]);
    let pw = perp(w);
    let dots = [
        dot(w, edge(rect[0])),
        dot(neg(pw), edge(rect[1])),
        dot(neg(w), edge(rect[2])),
        dot(pw, edge(rect[3])),
    ];
    let dw = Rational::int(i128::from(dot(w, w)));
    let mut angles = [Rational::int(0); 4];
    for (slot, (&value, &support)) in angles.iter_mut().zip(dots.iter().zip(rect.iter())) {
        let e = edge(support);
        *slot = Rational::int(i128::from(value))
            .div_int(i128::from(dot(e, e)))?
            .mul_int(i128::from(value));
    }
    let mut ordered: Vec<(Rational, usize)> = Vec::with_capacity(4);
    for (i, j) in [(3, 0), (0, 1), (1, 2), (2, 3)] {
        if rect[i] != rect[j] && angles[i].compare(dw) == Ordering::Less {
            let at = ordered
                .partition_point(|&(angle, _)| angle.compare(angles[i]) == Ordering::Greater);
            ordered.insert(at, (angles[i], i));
        }
    }
    let Some(&(amin, imin)) = ordered.first() else {
        return Some(false);
    };
    for &(angle, index) in &ordered {
        if angle == amin {
            rect[index] = next(rect[index]);
        }
    }
    rect.rotate_left(imin);
    Some(true)
}

/// `min_area_boundingbox_rotation` (`Arrange.cpp:1105-1116`): the rotation of the smallest-area
/// box round the shape, 0 where the calipers throw.
fn min_area_rotation(shape: &ExPolygon) -> f64 {
    let mut minbox = RotatedBox {
        axis: Point::new(0, 0),
        bottom: 0,
        right: 0,
    };
    let mut minarea = Rational::int(i128::from(i64::MAX));
    let search = rotcalipers(shape.contour().points(), |rbox| {
        let area = rbox.area()?;
        if area.compare(minarea) != Ordering::Greater {
            minarea = area;
            minbox = rbox;
        }
        Some(true)
    });
    if search.is_some() {
        minbox.angle_to_x()
    } else {
        0.0
    }
}

/// `fitIntoBoxRotation` (`rotcalipers.hpp:316-367`, eps 1e-4): bisects the caliper angles where
/// the box aspect ratio crosses the bed's until the shape fits. Its `prev_rbox` is never
/// updated, so the interval always starts at its default box's angle, 0.
fn fit_into_box_rotation(shape: &ExPolygon, bin: NBox) -> f64 {
    let aspect = |width: f64, height: f64| width / height;
    let aspect_r = aspect(bin.width() as f64, bin.height() as f64);
    let (mut a_from, mut a_to) = (0.0_f64, 0.0_f64);
    let _ = rotcalipers(shape.contour().points(), |rbox| {
        if aspect(rbox.width(), rbox.height()) < aspect_r {
            a_from = -0.0;
            a_to = rbox.angle_to_x();
            return Some(false);
        }
        Some(true)
    });
    let rot_box = |angle: f64| shape_box(&rotated(shape, angle));
    let rot_aspect = |angle: f64| {
        let bb = rot_box(angle);
        aspect(bb.width() as f64, bb.height() as f64)
    };
    let would_fit = |bb: NBox| bb.width() < bin.width() && bb.height() < bin.height();
    let mut ar_from = rot_aspect(a_from);
    let mut middle = (a_from + a_to) / 2.0;
    let mut box_middle = rot_box(middle);
    while !would_fit(box_middle) && (a_to - a_from).abs() > 1e-4 {
        let ar_middle = aspect(box_middle.width() as f64, box_middle.height() as f64);
        if (ar_from < aspect_r) != (ar_middle < aspect_r) {
            a_to = middle;
        } else {
            a_from = middle;
        }
        ar_from = rot_aspect(a_from);
        middle = (a_from + a_to) / 2.0;
        box_middle = rot_box(middle);
    }
    middle
}

/// `_arrange`'s starting rotation (`Arrange.cpp:1150-1162`): the minimum-area box rotation, and on
/// a box bed an extra turn when the item still does not fit.
pub(super) fn start_rotation(item: &mut Item, bin: &Bin) {
    item.set_rotation(min_area_rotation(item.transformed_shape()));
    if let Bin::Box(bed) = bin {
        let bb = item.bounding_box();
        if bb.width() >= bed.width() || bb.height() >= bed.height() {
            let turn = fit_into_box_rotation(item.transformed_shape(), *bed);
            item.rotate(turn);
        }
    }
}
