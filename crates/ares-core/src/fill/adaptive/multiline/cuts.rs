//! FillAdaptive.cpp:1691–1750: bend clearance cuts and neighbouring-wall pushes.
use super::{
    Junction, LevelPath, SweepLine,
    math::{V, family_dir},
};
struct Crossing<'a> {
    j: &'a Junction,
    n: V,
    junctions: &'a [Junction],
    sharp: bool,
}
impl Crossing<'_> {
    fn room(&self, path: &LevelPath, b: usize, away: i32, sign: f64) -> f64 {
        let Self {
            j,
            n,
            junctions,
            sharp,
        } = self;
        let sharp = *sharp;
        let mut c = f64::MAX;
        for nb in [
            b.checked_sub(1),
            (b + 1 < path.junctions.len()).then_some(b + 1),
        ]
        .into_iter()
        .flatten()
        {
            if sharp && path.turn[nb] == away {
                c = c.min(sign * n.dot(junctions[path.junctions[nb]].p - j.p));
            }
        }
        if b == 0 && path.start_term.is_some() {
            c = c.min(sign * n.dot(path.verts[0] - j.p));
        }
        if b + 1 == path.junctions.len() && path.end_term.is_some() {
            c = c.min(sign * n.dot(path.verts[path.verts.len() - 1] - j.p));
        }
        c.max(0.0)
    }
}

fn propagate(
    path: &LevelPath,
    b: usize,
    (away, step): (i32, isize),
    cut: V,
    junctions: &[Junction],
) -> Vec<(usize, V, usize)> {
    let mut pushes = Vec::new();
    for nb in [
        b.checked_sub(1),
        (b + 1 < path.junctions.len()).then_some(b + 1),
    ]
    .into_iter()
    .flatten()
    {
        if path.turn[nb] == away {
            let w = &junctions[path.junctions[nb]];
            let k = w
                .pairs
                .iter()
                .position(|&(left, _)| left == path.lines[nb])
                .unwrap_or(w.pairs.len()) as isize
                + step;
            if k >= 0
                && (k as usize) < w.bends.len()
                && let Some((pi, bend)) = w.bends[k as usize]
            {
                pushes.push((pi, cut, bend));
            }
        }
    }
    pushes
}
pub(super) fn apply(lines: &[SweepLine], junctions: &[Junction], paths: &mut [LevelPath], d1: f64) {
    for j in junctions {
        if j.pairs.len() < 2 {
            continue;
        }
        let (Some((lo, lb)), Some((hi, hb))) = (j.bends[0], j.bends[j.bends.len() - 1]) else {
            continue;
        };
        let pair = j.pairs[j.pairs.len() - 1];
        let delta = family_dir(lines[pair.1].family) - family_dir(lines[pair.0].family);
        let norm = delta.norm();
        let n = V::new(delta.x / norm, delta.y / norm);
        let cut = |offset| {
            let q = j.p + n * offset;
            let slope = -n.x / n.y;
            V::new(slope, q.y - slope * q.x)
        };
        if j.pairs.len() == 3 {
            paths[lo].cuts[lb].push(cut(-d1));
            paths[hi].cuts[hb].push(cut(d1));
            continue;
        }
        let sharp = lines[j.pairs[0].0].family != 0 && lines[j.pairs[0].1].family != 0;
        let crossing = Crossing {
            j,
            n,
            junctions,
            sharp,
        };
        let c_lo = crossing.room(&paths[lo], lb, 1, -1.0);
        let c_hi = crossing.room(&paths[hi], hb, -1, 1.0);
        let mut d_lo = 0.5 * d1;
        if d1 - c_hi <= c_lo {
            d_lo = d_lo.clamp(d1 - c_hi, c_lo);
        } else if c_lo + c_hi > 0.0 {
            d_lo = d1 * c_lo / (c_lo + c_hi);
        }
        let d_hi = d1 - d_lo;
        paths[lo].cuts[lb].push(cut(-d_lo));
        paths[hi].cuts[hb].push(cut(d_hi));
        if sharp {
            let lower = propagate(&paths[lo], lb, (1, -1), cut(-d_lo - d1), junctions);
            for (pi, c, bend) in lower {
                paths[pi].pushes.push((c, bend));
            }
            let upper = propagate(&paths[hi], hb, (-1, 1), cut(d_hi + d1), junctions);
            for (pi, c, bend) in upper {
                paths[pi].pushes.push((c, bend));
            }
        }
    }
}
