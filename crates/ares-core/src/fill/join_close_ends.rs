//! FillBase.cpp:3001–3117 at beta 345efec64: sparse-fill endpoint joining.
use crate::geometry::{Line, LineDistanceTree, Point, Polyline};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
struct Gap {
    apart: bool,
    trailing: bool,
    distance: f64,
    first: usize,
    second: usize,
}

fn cell(point: Point, max_gap: f64) -> [i64; 2] {
    [
        (point.x() as f64 / max_gap).floor() as i64,
        (point.y() as f64 / max_gap).floor() as i64,
    ]
}
fn key(c: [i64; 2]) -> u64 {
    ((c[0] as u64) << 32) ^ u64::from(c[1] as u32)
}
fn neighbors(c: [i64; 2], radius: i64) -> impl Iterator<Item = [i64; 2]> {
    (-radius..=radius).flat_map(move |dx| (-radius..=radius).map(move |dy| [c[0] + dx, c[1] + dy]))
}
fn root(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

pub(crate) fn join_close_ends(
    polylines: Vec<Polyline>,
    max_gap: f64,
    can_reverse: bool,
    epsilon: f64,
) -> Vec<Polyline> {
    let n = polylines.len();
    if n < 2 || max_gap <= 0.0 {
        return polylines;
    }
    let end = |e: usize| {
        let p = polylines[e / 2].points();
        if e.is_multiple_of(2) {
            p[0]
        } else {
            p[p.len() - 1]
        }
    };
    let mut grid: HashMap<u64, Vec<usize>> = HashMap::new();
    for e in 0..2 * n {
        if polylines[e / 2].points().len() >= 2 {
            grid.entry(key(cell(end(e), max_gap))).or_default().push(e);
        }
    }
    let backwards: Vec<_> = polylines
        .iter()
        .map(|p| {
            let p = p.points();
            if p.len() < 2 {
                return false;
            }
            let dx = (p[p.len() - 1].x() - p[0].x()) as f64;
            let dy = (p[p.len() - 1].y() - p[0].y()) as f64;
            if dx.abs() >= dy.abs() {
                dx < 0.0
            } else {
                dy < 0.0
            }
        })
        .collect();
    let trailing = |e: usize| !e.is_multiple_of(2) != backwards[e / 2];
    let mut gaps = Vec::new();
    for (a, b) in grid.values().flat_map(|ends| {
        neighbors(cell(end(ends[0]), max_gap), 1)
            .filter_map(|c| grid.get(&key(c)))
            .flatten()
            .flat_map(move |&b| ends.iter().map(move |&a| (a, b)))
    }) {
        if a >= b || a / 2 == b / 2 || (!can_reverse && a % 2 == b % 2) {
            continue;
        }
        let d = Line::new(end(a), end(b)).length();
        if d <= max_gap {
            gaps.push(Gap {
                apart: d > epsilon,
                trailing: d > epsilon && (trailing(a) || trailing(b)),
                distance: d,
                first: a,
                second: b,
            });
        }
    }
    if gaps.is_empty() {
        return polylines;
    }
    gaps.sort_unstable_by(|a, b| {
        a.apart
            .cmp(&b.apart)
            .then(a.trailing.cmp(&b.trailing))
            .then(a.distance.total_cmp(&b.distance))
            .then(a.first.cmp(&b.first))
            .then(a.second.cmp(&b.second))
    });
    let mut near_gaps = HashSet::new();
    for e in gaps.iter().flat_map(|gap| [gap.first, gap.second]) {
        near_gaps.extend(neighbors(cell(end(e), max_gap), 1).map(key));
    }
    let mut near_lines = Vec::new();
    for p in &polylines {
        for pair in p.points().windows(2) {
            let c0 = cell(pair[0], max_gap);
            let c1 = cell(pair[1], max_gap);
            let lo = [c0[0].min(c1[0]), c0[1].min(c1[1])];
            let hi = [c0[0].max(c1[0]), c0[1].max(c1[1])];
            let near = (hi[0] - lo[0]).max(hi[1] - lo[1]) > 3
                || (lo[0]..=hi[0])
                    .any(|x| (lo[1]..=hi[1]).any(|y| near_gaps.contains(&key([x, y]))));
            if near {
                near_lines.push(Line::new(pair[0], pair[1]));
            }
        }
    }
    let tree = LineDistanceTree::new(&near_lines);
    let mut partner = vec![usize::MAX; 2 * n];
    let mut parent: Vec<_> = (0..n).collect();
    for gap in gaps {
        let (a, b) = (gap.first, gap.second);
        if partner[a] != usize::MAX
            || partner[b] != usize::MAX
            || root(&mut parent, a / 2) == root(&mut parent, b / 2)
        {
            continue;
        }
        let p = end(a);
        let q = end(b);
        let length = Line::new(p, q).length();
        let crosses = if length <= 2.0 * epsilon {
            false
        } else {
            let dir = [
                (q.x() - p.x()) as f64 / length,
                (q.y() - p.y()) as f64 / length,
            ];
            let inner = Line::new(
                Point::new(
                    (p.x() as f64 + epsilon * dir[0]) as i64,
                    (p.y() as f64 + epsilon * dir[1]) as i64,
                ),
                Point::new(
                    (q.x() as f64 - epsilon * dir[0]) as i64,
                    (q.y() as f64 - epsilon * dir[1]) as i64,
                ),
            );
            !tree.intersections_sorted(inner).is_empty()
                || neighbors(cell(p, max_gap), 3)
                    .filter_map(|c| grid.get(&key(c)))
                    .flatten()
                    .any(|&e| {
                        partner[e] != usize::MAX
                            && intersects(inner, Line::new(end(e), end(partner[e])))
                    })
        };
        if !crosses {
            partner[a] = b;
            partner[b] = a;
            let a = root(&mut parent, a / 2);
            let b = root(&mut parent, b / 2);
            parent[a] = b;
        }
    }
    let mut polylines: Vec<_> = polylines.into_iter().map(Polyline::into_points).collect();
    let mut done = vec![false; n];
    let mut output = Vec::new();
    for first in [0, 1]
        .into_iter()
        .flat_map(|parity| (0..n).map(move |i| 2 * i + parity))
    {
        if done[first / 2] || partner[first] != usize::MAX {
            continue;
        }
        let mut joined = Vec::new();
        let mut e = first;
        while e != usize::MAX {
            let index = e / 2;
            done[index] = true;
            let mut p = std::mem::take(&mut polylines[index]);
            if !e.is_multiple_of(2) {
                p.reverse();
            }
            let skip = usize::from(!joined.is_empty() && joined.last() == p.first());
            joined.extend(p.into_iter().skip(skip));
            e = partner[e ^ 1];
        }
        output.push(Polyline::new(joined));
    }
    output
}

// Geometry.hpp:134–184, including collinear overlap.
fn intersects(a: Line, b: Line) -> bool {
    let side = |a: Point, b: Point, p: Point| {
        let v = (b.x() - a.x()) as i128 * (p.y() - a.y()) as i128
            - (b.y() - a.y()) as i128 * (p.x() - a.x()) as i128;
        v.signum()
    };
    let s = [side(a.a, a.b, b.a), side(a.a, a.b, b.b)];
    let t = [side(b.a, b.b, a.a), side(b.a, b.b, a.b)];
    if s[0] * s[1] > 0 || t[0] * t[1] > 0 {
        return false;
    }
    if s[0] * s[1] != 0 || t[0] * t[1] != 0 {
        return true;
    }
    if s != [0, 0] {
        return false;
    }
    let x = (a.b.x() - a.a.x()).abs() > (a.b.y() - a.a.y()).abs();
    let c = |p: Point| if x { p.x() } else { p.y() };
    let (i, j) = (c(a.a).min(c(a.b)), c(a.a).max(c(a.b)));
    let (k, l) = (c(b.a).min(c(b.b)), c(b.a).max(c(b.b)));
    (k >= i && k <= j) || (i >= k && i <= l)
}
