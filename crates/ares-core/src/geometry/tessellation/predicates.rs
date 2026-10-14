//! glu-libtess geom.c:47–264, ordered and stable sweep predicates.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
use super::mesh::Vertex;
pub(super) fn leq(u: Vertex, v: Vertex) -> bool {
    u.s < v.s || (u.s == v.s && u.t <= v.t)
}
pub(super) fn eq(u: Vertex, v: Vertex) -> bool {
    u.s == v.s && u.t == v.t
}
pub(super) fn eval(u: Vertex, v: Vertex, w: Vertex) -> f64 {
    let l = v.s - u.s;
    let r = w.s - v.s;
    if l + r > 0.0 {
        if l < r {
            (v.t - u.t) + (u.t - w.t) * (l / (l + r))
        } else {
            (v.t - w.t) + (w.t - u.t) * (r / (l + r))
        }
    } else {
        0.0
    }
}
pub(super) fn sign(u: Vertex, v: Vertex, w: Vertex) -> f64 {
    let l = v.s - u.s;
    let r = w.s - v.s;
    if l + r > 0.0 {
        (v.t - w.t) * l + (v.t - u.t) * r
    } else {
        0.0
    }
}
fn trans(v: Vertex) -> Vertex {
    Vertex {
        s: v.t,
        t: v.s,
        ..v
    }
}
fn interpolate(mut a: f64, x: f64, mut b: f64, y: f64) -> f64 {
    if a < 0.0 {
        a = 0.0;
    }
    if b < 0.0 {
        b = 0.0;
    }
    if a <= b {
        if b == 0.0 {
            (x + y) / 2.0
        } else {
            x + (y - x) * (a / (a + b))
        }
    } else {
        y + (x - y) * (b / (a + b))
    }
}
pub(super) fn intersect(mut o1: Vertex, mut d1: Vertex, mut o2: Vertex, mut d2: Vertex) -> Vertex {
    if !leq(o1, d1) {
        std::mem::swap(&mut o1, &mut d1);
    }
    if !leq(o2, d2) {
        std::mem::swap(&mut o2, &mut d2);
    }
    if !leq(o1, o2) {
        std::mem::swap(&mut o1, &mut o2);
        std::mem::swap(&mut d1, &mut d2);
    }
    let s = coordinate(o1, d1, o2, d2);
    if !leq(trans(o1), trans(d1)) {
        std::mem::swap(&mut o1, &mut d1);
    }
    if !leq(trans(o2), trans(d2)) {
        std::mem::swap(&mut o2, &mut d2);
    }
    if !leq(trans(o1), trans(o2)) {
        std::mem::swap(&mut o1, &mut o2);
        std::mem::swap(&mut d1, &mut d2);
    }
    let t = coordinate(trans(o1), trans(d1), trans(o2), trans(d2));
    Vertex {
        s,
        t,
        ..Vertex::default()
    }
}
fn coordinate(o1: Vertex, d1: Vertex, o2: Vertex, d2: Vertex) -> f64 {
    if !leq(o2, d1) {
        return (o2.s + d1.s) / 2.0;
    }
    let (mut z1, mut z2, y) = if leq(d1, d2) {
        (eval(o1, o2, d1), eval(o2, d1, d2), d1.s)
    } else {
        (sign(o1, o2, d1), -sign(o1, d2, d1), d2.s)
    };
    if z1 + z2 < 0.0 {
        z1 = -z1;
        z2 = -z2;
    }
    interpolate(z1, o2.s, z2, y)
}
