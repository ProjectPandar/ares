//! `PerimeterGenerator.cpp:677-881`, reconnection and anchored dependency order.

use std::collections::HashSet;

use crate::geometry::{Line, LineDistanceTree, Point, Polyline};

fn squared(a: Point, b: Point) -> f64 {
    let x = (a.x() - b.x()) as f64;
    let y = (a.y() - b.y()) as f64;
    x * x + y * y
}

pub(super) fn reconnect(paths: Vec<Polyline>, limit: f64) -> Vec<Polyline> {
    enum Join {
        TailHead,
        TailTail,
        HeadTail,
        HeadHead,
    }
    let mut paths = paths
        .into_iter()
        .map(|p| (!p.points().is_empty()).then_some(p))
        .collect::<Vec<_>>();
    for a in 0..paths.len() {
        for b in a + 1..paths.len() {
            let (left, right) = paths.split_at_mut(b);
            let Some(base) = left[a].as_mut() else { break };
            let Some(next) = right[0].as_ref() else {
                continue;
            };
            let side = if squared(base.back().unwrap(), next.front().unwrap()) < limit * limit {
                Join::TailHead
            } else if squared(base.back().unwrap(), next.back().unwrap()) < limit * limit {
                Join::TailTail
            } else if squared(base.front().unwrap(), next.back().unwrap()) < limit * limit {
                Join::HeadTail
            } else if squared(base.front().unwrap(), next.front().unwrap()) < limit * limit {
                Join::HeadHead
            } else {
                continue;
            };
            let mut next = right[0].take().unwrap();
            match side {
                Join::TailHead => append(base, next),
                Join::TailTail => {
                    next.reverse();
                    base.append_points(next.into_points());
                }
                Join::HeadTail => {
                    append(&mut next, Polyline::new(std::mem::take(base.points_mut())));
                    next.reverse();
                    *base = next;
                }
                Join::HeadHead => {
                    base.reverse();
                    append(base, next);
                    base.reverse();
                }
            }
        }
    }
    // Source unordered_map reserves the initial count and uses identity-hashed
    // contiguous size_t keys: no collisions, reverse insertion iteration order.
    paths.into_iter().rev().flatten().collect()
}

// `Polyline.cpp:383-405`: unlike raw point insertion, append drops a shared endpoint.
fn append(base: &mut Polyline, next: Polyline) {
    if !next.is_valid() {
        return;
    }
    let skip = usize::from(base.back() == next.front());
    base.append_points(next.into_points().into_iter().skip(skip));
}

fn touch(a: &Polyline, b: &Polyline, limit: f64) -> bool {
    fn near(a: &Polyline, b: &Polyline, limit: f64) -> bool {
        let lines = b
            .points()
            .windows(2)
            .map(|p| Line::new(p[0], p[1]))
            .collect::<Vec<_>>();
        let tree = LineDistanceTree::new(&lines);
        a.points().iter().any(|p| {
            tree.nearest(*p)
                .is_some_and(|nearest| nearest.squared_distance.sqrt() < limit)
        })
    }
    near(a, b, limit) || near(b, a, limit)
}

pub(super) fn start_at_anchor(path: &mut Polyline, lower: &LineDistanceTree<'_>) {
    let mut closest = 0;
    let mut minimum = f64::MAX;
    for (index, point) in path.points().iter().enumerate() {
        // Source assigns the comparison's bool to d (not the distance).
        if super::signed_distance(lower, *point) < minimum {
            minimum = 1.0;
            closest = index;
        }
    }
    path.points_mut().rotate_left(closest);
}

pub(super) fn sort(paths: Vec<Polyline>, first_unanchored: usize, spacing: f64) -> Vec<Polyline> {
    let mut dependencies = (0..paths.len())
        .map(|index| {
            (0..index)
                .filter(|previous| touch(&paths[index], &paths[*previous], spacing * 1.5))
                .collect::<HashSet<_>>()
        })
        .collect::<Vec<_>>();
    let mut processed = vec![false; paths.len()];
    processed[..first_unanchored].fill(true);
    for _ in first_unanchored..paths.len() {
        let mut changed = false;
        for index in first_unanchored..paths.len() {
            if processed[index] || !dependencies[index].iter().any(|dep| processed[*dep]) {
                continue;
            }
            let reverse = dependencies[index]
                .iter()
                .copied()
                .filter(|dep| !processed[*dep])
                .collect::<Vec<_>>();
            for dep in reverse {
                dependencies[dep].insert(index);
                dependencies[index].remove(&dep);
            }
            processed[index] = true;
            changed = true;
        }
        if !changed {
            break;
        }
    }
    let mut current = paths[0].front().unwrap();
    let mut paths = paths.into_iter().map(Some).collect::<Vec<_>>();
    let mut ordered = Vec::new();
    loop {
        let closest = dependencies
            .iter()
            .enumerate()
            .filter(|(_, deps)| deps.is_empty())
            .flat_map(|(index, _)| {
                let path = paths[index].as_ref().unwrap();
                [
                    (squared(path.front().unwrap(), current), index, false),
                    (squared(path.back().unwrap(), current), index, true),
                ]
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let Some((_, index, reverse)) = closest else {
            break;
        };
        let mut path = paths[index].take().unwrap();
        if reverse {
            path.reverse();
        }
        current = path.back().unwrap();
        ordered.push(path);
        dependencies[index].insert(usize::MAX);
        for deps in &mut dependencies {
            deps.remove(&index);
        }
        // Source's distance > scaled(5) resets next_idx, then repeats the same
        // nearest eligible search. A single search preserves that result.
    }
    let mut reconnected: Vec<Polyline> = Vec::with_capacity(ordered.len());
    for path in ordered {
        if let Some(last) = reconnected.last_mut()
            && squared(last.back().unwrap(), path.front().unwrap()) < spacing * spacing * 4.0
        {
            last.append_points(path.into_points());
        } else {
            reconnected.push(path);
        }
    }
    reconnected.retain(|path| {
        path.points()
            .windows(2)
            .map(|p| squared(p[0], p[1]).sqrt())
            .sum::<f64>()
            > 3.0 * spacing
    });
    reconnected
}
