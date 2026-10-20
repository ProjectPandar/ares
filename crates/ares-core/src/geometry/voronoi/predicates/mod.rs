//! VoronoiUtilsCgal.cpp40-138: filtered signs, exact binary-float fallback.
mod exact;
mod interval;
use exact::Exact;
use interval::Interval;
use std::ops::{Add, Mul, Neg, Sub};

type Point = [f64; 2];
#[derive(Clone, Copy)]
pub(super) struct Parabola {
    pub focus: Point,
    pub from: Point,
    pub to: Point,
    pub side: i8,
}
enum Predicate {
    Orientation(Point, Point, Point),
    TangentLine(Point, Point, Parabola),
    Tangents(Point, Parabola, Parabola),
}
trait Scalar:
    Clone + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Neg<Output = Self>
{
    fn from_double(value: f64) -> Self;
}
fn point<S: Scalar>(p: Point) -> [S; 2] {
    p.map(S::from_double)
}
fn difference<S: Scalar>(a: &[S; 2], b: &[S; 2]) -> [S; 2] {
    [a[0].clone() - b[0].clone(), a[1].clone() - b[1].clone()]
}
fn dot<S: Scalar>(a: &[S; 2], b: &[S; 2]) -> S {
    a[0].clone() * b[0].clone() + a[1].clone() * b[1].clone()
}
fn cross<S: Scalar>(a: &[S; 2], b: &[S; 2]) -> S {
    a[0].clone() * b[1].clone() - a[1].clone() * b[0].clone()
}
fn tangent<S: Scalar>(p: Point, source: Parabola) -> [S; 2] {
    let p = point::<S>(p);
    let f = point::<S>(source.focus);
    let u = point::<S>(source.from);
    let v = point::<S>(source.to);
    let d = difference(&v, &u);
    let len = dot(&d, &d);
    let fu = difference(&f, &u);
    let pu = difference(&p, &u);
    let proj = dot(&d, &pu);
    let x = fu[0].clone() * len.clone() - d[0].clone() * proj.clone();
    let y = fu[1].clone() * len - d[1].clone() * proj;
    if source.side > 0 { [-y, x] } else { [y, -x] }
}
fn evaluate<S: Scalar>(p: &Predicate) -> S {
    match *p {
        Predicate::Orientation(a, b, c) => cross(
            &difference(&point::<S>(b), &point::<S>(a)),
            &difference(&point::<S>(c), &point::<S>(a)),
        ),
        Predicate::TangentLine(p, q, source) => cross(
            &tangent::<S>(p, source),
            &difference(&point::<S>(q), &point::<S>(p)),
        ),
        Predicate::Tangents(p, a, b) => cross(&tangent::<S>(p, a), &tangent::<S>(p, b)),
    }
}
fn sign(predicate: Predicate) -> i8 {
    if let Some(sign) = evaluate::<Interval>(&predicate).sign() {
        return sign;
    }
    evaluate::<Exact>(&predicate).sign()
}
pub(super) fn orientation(a: Point, b: Point, c: Point) -> i8 {
    sign(Predicate::Orientation(a, b, c))
}
pub(super) fn tangent_line(p: Point, q: Point, source: Parabola) -> i8 {
    sign(Predicate::TangentLine(p, q, source))
}
pub(super) fn tangents(p: Point, a: Parabola, b: Parabola) -> i8 {
    sign(Predicate::Tangents(p, a, b))
}
