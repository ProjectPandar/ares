//! FillAdaptive.cpp:1528–1626: crossing detection and short dead arms.
use super::{Junction, SweepLine, math::V};
use crate::geometry::fixed_gcc_sort_by;
pub(super) fn detect(lines: &mut [SweepLine], junctions: &mut Vec<Junction>, eps: f64) {
    let mut hits = Vec::new();
    let mut order: Vec<_> = (0..lines.len()).collect();
    fixed_gcc_sort_by(&mut order, |&l, &r| lines[l].a.x < lines[r].a.x);
    for (oi, &i) in order.iter().enumerate() {
        let li = &lines[i];
        for &j in &order[oi + 1..] {
            let lj = &lines[j];
            if lj.a.x > li.b.x + eps {
                break;
            }
            if li.family == lj.family {
                continue;
            }
            let x = (lj.lin.y - li.lin.y) / (li.lin.x - lj.lin.x);
            let p = V::new(x, li.lin.x * x + li.lin.y);
            let ti = li.along(p);
            let tj = lj.along(p);
            if ti > -eps && ti < li.length() + eps && tj > -eps && tj < lj.length() + eps {
                hits.push((p, i, j));
            }
        }
    }
    fixed_gcc_sort_by(&mut hits, |l, r| l.0.x < r.0.x);
    junctions.clear();
    for (p, i, j) in hits {
        let mut found = None;
        for k in (0..junctions.len()).rev() {
            if junctions[k].p.x <= p.x - eps {
                break;
            }
            if (junctions[k].p.y - p.y).abs() < eps {
                found = Some(k);
                break;
            }
        }
        let k = found.unwrap_or_else(|| {
            let k = junctions.len();
            junctions.push(Junction {
                p,
                lines: Vec::new(),
                pairs: Vec::new(),
                bends: Vec::new(),
            });
            k
        });
        for li in [i, j] {
            if !junctions[k].lines.contains(&li) {
                junctions[k].lines.push(li);
            }
        }
    }
    for line in lines.iter_mut() {
        line.junctions.clear();
    }
    for (ji, j) in junctions.iter().enumerate() {
        for &li in &j.lines {
            lines[li].junctions.push((j.p.x, ji));
        }
    }
    for line in lines {
        fixed_gcc_sort_by(&mut line.junctions, |l, r| {
            l.0 < r.0 || (l.0 == r.0 && l.1 < r.1)
        });
    }
}
pub(super) fn has_arm(line: &SweepLine, p: V, right: bool, eps: f64) -> bool {
    let t = line.along(p);
    if right {
        t < line.length() - eps
    } else {
        t > eps
    }
}
fn dead_arm(
    l: &SweepLine,
    junctions: &[Junction],
    ji: usize,
    p: V,
    eps: f64,
) -> (f64, bool, usize) {
    let mut dead = (f64::MAX, false, 0);
    let at = l
        .junctions
        .iter()
        .position(|&(_, id)| id == ji)
        .expect("line contains its junction");
    let t = l.along(p);
    if at + 1 < l.junctions.len() && l.length() - l.along(junctions[l.junctions[at + 1].1].p) < eps
    {
        dead = (l.length() - t, true, l.junctions[at + 1].1);
    }
    if at > 0 && l.along(junctions[l.junctions[at - 1].1].p) < eps && t < dead.0 {
        dead = (t, false, l.junctions[at - 1].1);
    }
    dead
}

pub(super) fn shorten_dead_arms(
    lines: &mut [SweepLine],
    junctions: &mut Vec<Junction>,
    d1: f64,
    eps: f64,
) {
    for _ in 0..3 {
        let mut touched = vec![false; lines.len()];
        let mut changed = false;
        for (ji, j) in junctions.iter().enumerate() {
            if j.lines.len() != 2
                || j.lines.iter().any(|&li| {
                    touched[li]
                        || !has_arm(&lines[li], j.p, false, eps)
                        || !has_arm(&lines[li], j.p, true, eps)
                })
            {
                continue;
            }
            let mut dead = [(f64::MAX, false, 0); 2];
            for (k, &li) in j.lines.iter().enumerate() {
                dead[k] = dead_arm(&lines[li], junctions, ji, j.p, eps);
            }
            let k = usize::from(dead[0].0 > dead[1].0);
            if dead[k].0 >= 2.0 * d1 {
                continue;
            }
            let line = &mut lines[j.lines[k]];
            if dead[k].1 {
                line.b = j.p;
            } else {
                line.a = j.p;
            }
            for &li in &junctions[dead[k].2].lines {
                touched[li] = true;
            }
            changed = true;
        }
        if !changed {
            break;
        }
        detect(lines, junctions, eps);
    }
}
