//! FillAdaptive.cpp:1321–1440: profile clipping and bend geometry.
use super::{LevelPath, SweepLine, math::V};
struct Clip {
    c: V,
    side: i32,
    window: [f64; 2],
    limits: [f64; 2],
    cut_at_window: bool,
    drop_past_limits: bool,
}
fn interpolate(a: V, b: V, x: f64) -> f64 {
    if b.x > a.x {
        a.y + (x - a.x) / (b.x - a.x) * (b.y - a.y)
    } else {
        b.y
    }
}
fn push(out: &mut Vec<V>, x: f64, y: f64) {
    if out.last().is_none_or(|p| x > p.x || (y - p.y).abs() > 2.0) {
        out.push(V::new(x, y));
    }
}
fn y_local(local: &[V], x: f64) -> f64 {
    let mut i = 1;
    while i + 1 < local.len() && local[i].x < x {
        i += 1;
    }
    interpolate(local[i - 1], local[i], x)
}
fn stretch(out: &mut Vec<V>, local: &[V], c: V, x0: f64, x1: f64) {
    push(out, x0, y_local(local, x0));
    push(out, x0, c.x * x0 + c.y);
    push(out, x1, c.x * x1 + c.y);
    push(out, x1, y_local(local, x1));
}
fn clip_profile(f: &mut Vec<V>, clip: Clip) {
    let Clip {
        c,
        side,
        window: [window_start, window_end],
        limits: [lim0, lim1],
        cut_at_window,
        drop_past_limits,
    } = clip;
    let r0 = lim0.max(f[0].x);
    let r1 = lim1.min(f[f.len() - 1].x);
    if r1 <= r0 {
        return;
    }
    let ia = f.partition_point(|p| p.x <= r0);
    let ib = ia + f[ia..].partition_point(|p| p.x < r1);
    let mut local = vec![V::new(r0, interpolate(f[ia - 1], f[ia], r0))];
    local.extend_from_slice(&f[ia..ib]);
    local.push(V::new(r1, interpolate(f[ib - 1], f[ib], r1)));
    let beyond = |p: V| f64::from(side) * (c.x * p.x + c.y - p.y) - 1.0;
    let mut stretches: Vec<(f64, f64)> = Vec::new();
    let mut add = |x0, x1| {
        if let Some(last) = stretches.last_mut()
            && last.1 >= x0
        {
            last.1 = x1;
        } else {
            stretches.push((x0, x1));
        }
    };
    for pair in local.windows(2) {
        let [p, q] = [pair[0], pair[1]];
        if q.x <= p.x {
            continue;
        }
        let bp = beyond(p);
        let bq = beyond(q);
        if bp > 0.0 && bq > 0.0 {
            add(p.x, q.x);
        } else if bp > 0.0 || bq > 0.0 {
            let x = p.x + bp / (bp - bq) * (q.x - p.x);
            if bp > 0.0 {
                add(p.x, x);
            } else {
                add(x, q.x);
            }
        }
    }
    let mut keep = Vec::new();
    for (x0, x1) in stretches {
        if x1 < window_start || x0 > window_end {
            continue;
        }
        if drop_past_limits && ((x0 <= r0 && r0 == lim0) || (x1 >= r1 && r1 == lim1)) {
            continue;
        }
        keep.push(if cut_at_window {
            (x0.max(window_start), x1.min(window_end))
        } else {
            (x0, x1)
        });
    }
    if keep.is_empty() {
        return;
    }
    let mut out = f[..ia].to_vec();
    let mut k = 0;
    for p in &local {
        while k < keep.len() && keep[k].1 < p.x {
            stretch(&mut out, &local, c, keep[k].0, keep[k].1);
            k += 1;
        }
        if k < keep.len() && keep[k].0 <= p.x && p.x <= keep[k].1 {
            continue;
        }
        push(&mut out, p.x, p.y);
    }
    for &(x0, x1) in &keep[k..] {
        stretch(&mut out, &local, c, x0, x1);
    }
    for p in &f[ib..] {
        push(&mut out, p.x, p.y);
    }
    *f = out;
}
fn window(path: &LevelPath, b: usize, f: &[V]) -> [f64; 2] {
    let mut l = b as isize - 1;
    let mut r = b + 1;
    while l >= 0 && path.turn[l as usize] == path.turn[b] {
        l -= 1;
    }
    while r < path.junctions.len() && path.turn[r] == path.turn[b] {
        r += 1;
    }
    [
        if l >= 0 {
            path.verts[l as usize + 1].x
        } else {
            f[0].x
        },
        if r < path.junctions.len() {
            path.verts[r + 1].x
        } else {
            f[f.len() - 1].x
        },
    ]
}
pub(super) fn path_points(path: &LevelPath, lines: &[SweepLine], reach: f64) -> Vec<V> {
    let mut f = path.verts.clone();
    let sharp = |b: usize| lines[path.lines[b]].family != 0 && lines[path.lines[b + 1]].family != 0;
    for b in 0..path.junctions.len() {
        if !sharp(b) {
            let window = window(path, b, &f);
            let x = path.verts[b + 1].x;
            for &c in &path.cuts[b] {
                clip_profile(
                    &mut f,
                    Clip {
                        c,
                        side: path.turn[b],
                        window,
                        limits: [x - reach, x + reach],
                        cut_at_window: true,
                        drop_past_limits: false,
                    },
                );
            }
        }
    }
    for b in 0..path.junctions.len() {
        if sharp(b) {
            let x = path.verts[b + 1].x;
            for &c in &path.cuts[b] {
                clip_profile(
                    &mut f,
                    Clip {
                        c,
                        side: path.turn[b],
                        window: [x, x],
                        limits: [x - reach, x + reach],
                        cut_at_window: false,
                        drop_past_limits: true,
                    },
                );
            }
        }
    }
    for &(c, b) in &path.pushes {
        let window = window(path, b, &f);
        let x = path.verts[b + 1].x;
        clip_profile(
            &mut f,
            Clip {
                c,
                side: path.turn[b],
                window,
                limits: [x - reach, x + reach],
                cut_at_window: true,
                drop_past_limits: true,
            },
        );
    }
    let mut points: Vec<V> = Vec::new();
    for p in f {
        while points.len() >= 2 {
            let d0 = points[points.len() - 1] - points[points.len() - 2];
            let d1 = p - points[points.len() - 1];
            if (d0.x * d1.y - d0.y * d1.x).abs() > 1e-9 * d0.norm() * d1.norm() {
                break;
            }
            points.pop();
        }
        points.push(p);
    }
    points
}
