//! FillAdaptive.cpp:1628–1689: ordered crossing pairs and left-to-right paths.
use super::{Junction, LevelPath, SweepLine, junctions::has_arm};
use crate::geometry::fixed_gcc_sort_by;
pub(super) fn trace(lines: &[SweepLine], junctions: &mut [Junction], eps: f64) -> Vec<LevelPath> {
    for j in junctions.iter_mut() {
        let mut left: Vec<_> = j
            .lines
            .iter()
            .copied()
            .filter(|&li| {
                has_arm(&lines[li], j.p, false, eps) && has_arm(&lines[li], j.p, true, eps)
            })
            .collect();
        let mut right = left.clone();
        fixed_gcc_sort_by(&mut left, |&l, &r| lines[l].lin.x > lines[r].lin.x);
        fixed_gcc_sort_by(&mut right, |&l, &r| lines[l].lin.x < lines[r].lin.x);
        j.pairs.extend(left.into_iter().zip(right));
        j.bends.resize(j.pairs.len(), None);
    }
    let mut paths = Vec::new();
    for (li, l) in lines.iter().enumerate() {
        let start = l
            .junctions
            .first()
            .and_then(|&(_, ji)| (!has_arm(l, junctions[ji].p, false, eps)).then_some(ji));
        let mut path = LevelPath {
            start_term: start,
            verts: vec![start.map_or(l.a, |ji| junctions[ji].p)],
            lines: vec![li],
            ..LevelPath::default()
        };
        let mut cur = li;
        let mut x = path.verts[0].x;
        loop {
            let Some(&(at_x, ji)) = lines[cur]
                .junctions
                .iter()
                .find(|&&(at_x, _)| at_x > x + 0.25 * eps)
            else {
                path.verts.push(lines[cur].b);
                break;
            };
            let j = &mut junctions[ji];
            let Some(k) = j.pairs.iter().position(|&(left, _)| left == cur) else {
                path.verts.push(j.p);
                path.end_term = Some(ji);
                break;
            };
            let next = j.pairs[k].1;
            if next != cur {
                j.bends[k] = Some((paths.len(), path.junctions.len()));
                path.verts.push(j.p);
                path.lines.push(next);
                path.junctions.push(ji);
                path.turn
                    .push(2 * i32::from(lines[next].lin.x > lines[cur].lin.x) - 1);
                cur = next;
            }
            x = at_x;
        }
        path.cuts.resize_with(path.junctions.len(), Vec::new);
        paths.push(path);
    }
    paths
}
