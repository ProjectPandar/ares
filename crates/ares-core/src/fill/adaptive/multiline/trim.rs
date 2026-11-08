//! FillAdaptive.cpp:1752–1827: indexed clearance and dependency-ordered trimming.
use super::{
    Junction, LevelPath, Routing, SweepLine,
    math::{V, distance, length, point_along},
    profile::path_points,
};
use crate::geometry::FloatLineDistanceTree;
#[derive(Clone, Copy)]
struct Clearance<'a> {
    tree: &'a FloatLineDistanceTree,
    starts: &'a [(usize, f64)],
    kept: &'a [(f64, f64)],
    d1: f64,
}
fn clearance(pi: usize, q: V, context: &Clearance<'_>) -> f64 {
    let Clearance {
        tree,
        starts,
        kept,
        d1,
    } = *context;
    let mut dist = f64::MAX;
    for s in tree.all_lines_in_radius(q.array(), d1) {
        let (pj, start) = starts[s];
        let [a, b] = tree.get_line(s);
        let a = V::new(a[0], a[1]);
        let d = V::new(b[0], b[1]) - a;
        let len = d.norm();
        let t0 = 0.0_f64.max(kept[pj].0 - start);
        let t1 = len.min(kept[pj].1 - start);
        if pj != pi && len > 0.0 && t0 <= t1 {
            dist = dist.min(distance(a + d * (t0 / len), a + d * (t1 / len), q));
        }
    }
    dist
}
fn trim_front(pi: usize, pts: &mut Vec<V>, context: &Clearance<'_>, end_clearance: f64) -> f64 {
    let d1 = context.d1;
    let total = length(pts);
    let step = d1 / 32.0;
    let mut t = 0.0;
    while t <= total && clearance(pi, point_along(pts, t).0, context) < end_clearance {
        t += step;
    }
    if t > total {
        pts.clear();
        return total;
    }
    if t == 0.0 {
        return 0.0;
    }
    let mut lo = 0.0_f64.max(t - step);
    while t - lo > step / 256.0 {
        let mid = 0.5 * (lo + t);
        if clearance(pi, point_along(pts, mid).0, context) < end_clearance {
            lo = mid;
        } else {
            t = mid;
        }
    }
    let (q, seg) = point_along(pts, t);
    pts.drain(..seg - 1);
    pts[0] = q;
    t
}
fn mark_stopping(
    (pi, ji, own): (usize, usize, usize),
    junctions: &[Junction],
    carried: &[Vec<(f64, f64, usize)>],
    eps: f64,
    stopping: &mut [Vec<usize>],
) {
    for &li in &junctions[ji].lines {
        if li == own {
            continue;
        }
        for &(x0, x1, pj) in &carried[li] {
            if pj != pi && x0 - eps <= junctions[ji].p.x && junctions[ji].p.x <= x1 + eps {
                stopping[pj].push(pi);
            }
        }
    }
}

fn visit(pi: usize, stopping_on: &[Vec<usize>], visited: &mut [bool], order: &mut Vec<usize>) {
    if visited[pi] {
        return;
    }
    visited[pi] = true;
    for &child in &stopping_on[pi] {
        visit(child, stopping_on, visited, order);
    }
    order.push(pi);
}
pub(super) fn trim(
    paths: &[LevelPath],
    lines: &[SweepLine],
    junctions: &[Junction],
    Routing {
        d1,
        end_overlap,
        eps,
    }: Routing,
) -> Vec<Vec<V>> {
    let mut geometry = Vec::with_capacity(paths.len());
    let mut segments = Vec::new();
    let mut starts = Vec::new();
    let mut kept = Vec::new();
    for path in paths {
        let points = if path.junctions.is_empty() {
            vec![path.verts[0], path.verts[path.verts.len() - 1]]
        } else {
            path_points(path, lines, 4.0 * d1)
        };
        let mut along = 0.0;
        for pair in points.windows(2) {
            segments.push([pair[0].array(), pair[1].array()]);
            starts.push((geometry.len(), along));
            along += (pair[1] - pair[0]).norm();
        }
        geometry.push(points);
        kept.push((0.0, along));
    }
    let tree = FloatLineDistanceTree::new(segments);
    let mut carried = vec![Vec::new(); lines.len()];
    for (pi, path) in paths.iter().enumerate() {
        for (i, &line) in path.lines.iter().enumerate() {
            carried[line].push((path.verts[i].x, path.verts[i + 1].x, pi));
        }
    }
    let mut stopping_on = vec![Vec::new(); paths.len()];
    for (pi, path) in paths.iter().enumerate() {
        for (term, own) in [
            (path.start_term, path.lines[0]),
            (path.end_term, path.lines[path.lines.len() - 1]),
        ] {
            if let Some(ji) = term {
                mark_stopping((pi, ji, own), junctions, &carried, eps, &mut stopping_on);
            }
        }
    }
    let mut order = Vec::new();
    let mut visited = vec![false; paths.len()];
    for pi in 0..paths.len() {
        visit(pi, &stopping_on, &mut visited, &mut order);
    }
    let mut trimmed = vec![Vec::new(); paths.len()];
    for pass in 0..3 {
        for &pi in &order {
            if pass < 2 {
                trimmed[pi].clone_from(&geometry[pi]);
                kept[pi] = (0.0, length(&geometry[pi]));
            } else if trimmed[pi].is_empty() {
                continue;
            }
            let pts = &mut trimmed[pi];
            if paths[pi].start_term.is_some() {
                let context = Clearance {
                    tree: &tree,
                    starts: &starts,
                    kept: &kept,
                    d1,
                };
                let n = trim_front(pi, pts, &context, d1 - end_overlap);
                kept[pi].0 += n;
            }
            if paths[pi].end_term.is_some() && !pts.is_empty() {
                pts.reverse();
                let context = Clearance {
                    tree: &tree,
                    starts: &starts,
                    kept: &kept,
                    d1,
                };
                let n = trim_front(pi, pts, &context, d1 - end_overlap);
                kept[pi].1 -= n;
                pts.reverse();
            }
            if pts.len() < 2 || length(pts) < d1 {
                pts.clear();
                kept[pi] = (0.0, -1.0);
            }
        }
    }
    trimmed
}
