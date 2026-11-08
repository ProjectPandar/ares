//! FillAdaptive.cpp:1475–1523: clipped and merged sweep families.
use super::{
    SweepLine,
    math::{V, family_dir},
};
use crate::geometry::{BoundingBox, Line, fixed_gcc_sort_by, liang_barsky_interval};
struct Piece {
    c: f64,
    s0: f64,
    s1: f64,
}
pub(super) fn collect(
    raw: &[Line],
    cover: BoundingBox,
    angle: f64,
    eps: f64,
) -> Option<Vec<SweepLine>> {
    let min = [cover.min().x() as f64, cover.min().y() as f64];
    let max = [cover.max().x() as f64, cover.max().y() as f64];
    let mut pieces: [Vec<Piece>; 3] = std::array::from_fn(|_| Vec::new());
    for line in raw {
        let origin = V::new(line.a.x() as f64, line.a.y() as f64);
        let delta = V::new(line.b.x() as f64, line.b.y() as f64) - origin;
        let Some((t0, t1)) = liang_barsky_interval(origin.array(), delta.array(), min, max) else {
            continue;
        };
        let a = origin + delta * t0;
        let b = origin + delta * t1;
        if (b - a).norm() < 10.0 * eps {
            continue;
        }
        let a = a.rotate(angle);
        let b = b.rotate(angle);
        let direction = (b.y - a.y).atan2(b.x - a.x) / (std::f64::consts::PI / 3.0);
        if (direction - direction.round()).abs() > 0.01 {
            return None;
        }
        let f = (direction.round() as i32).rem_euclid(3) as usize;
        let d = family_dir(f);
        let n = V::new(-d.y, d.x);
        pieces[f].push(Piece {
            c: n.dot(a),
            s0: d.dot(a).min(d.dot(b)),
            s1: d.dot(a).max(d.dot(b)),
        });
    }
    let mut lines = Vec::new();
    for (f, ps) in pieces.iter_mut().enumerate() {
        merge(ps, f, eps, &mut lines);
    }
    Some(lines)
}

fn merge(ps: &mut [Piece], f: usize, eps: f64, lines: &mut Vec<SweepLine>) {
    let d = family_dir(f);
    let n = V::new(-d.y, d.x);
    let slope = d.y / d.x;
    fixed_gcc_sort_by(ps, |l, r| l.c < r.c);
    let mut i = 0;
    while i < ps.len() {
        let mut j = i + 1;
        while j < ps.len() && ps[j].c - ps[i].c < eps {
            j += 1;
        }
        fixed_gcc_sort_by(&mut ps[i..j], |l, r| l.s0 < r.s0);
        let mut c = 0.0;
        for p in &ps[i..j] {
            c += p.c / (j - i) as f64;
        }
        let mut s0 = ps[i].s0;
        let mut s1 = ps[i].s1;
        for next in ps[i + 1..j].iter().map(Some).chain(std::iter::once(None)) {
            if let Some(p) = next
                && p.s0 <= s1 + eps
            {
                s1 = s1.max(p.s1);
                continue;
            }
            let a = d * s0 + n * c;
            lines.push(SweepLine {
                a,
                b: d * s1 + n * c,
                family: f,
                lin: V::new(slope, a.y - slope * a.x),
                junctions: Vec::new(),
            });
            if let Some(p) = next {
                s0 = p.s0;
                s1 = p.s1;
            }
        }
        i = j;
    }
}
