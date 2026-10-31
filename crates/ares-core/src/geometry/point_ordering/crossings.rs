//! OrderingStrategies.cpp:104–228::tsp_remove_crossings and
//! Geometry.hpp:134–184::segments_intersect at
//! OrcaSlicer 345efec64c7b95a05ce5cf96df0dde66a158835c.

use std::collections::{HashMap, hash_map::Entry};

use super::{Point, crossing_grid::CrossingGrid};

pub(super) fn remove(path: &mut [usize], points: &[Point]) -> bool {
    let mut grid = CrossingGrid::new(path, points);
    // Source size_t multiplication followed by static_cast<int>, including
    // the signed cap; keep the same low 32 bits on browser WASM as on Native.
    let max_iterations = path.len().wrapping_mul(path.len()) as i32;
    let mut seen = HashMap::new();
    seen.insert(path_hash(path), 0);
    let mut improved = false;
    for iteration in 1..=max_iterations {
        if !reverse_first(path, points, &mut grid) {
            break;
        }
        improved = true;
        match seen.entry(path_hash(path)) {
            Entry::Vacant(entry) => {
                entry.insert(iteration);
            }
            Entry::Occupied(entry) => {
                let steps = (max_iterations - iteration) % (iteration - entry.get());
                for _ in 0..steps {
                    reverse_first(path, points, &mut grid);
                }
                break;
            }
        }
    }
    improved
}

fn path_hash(path: &[usize]) -> u64 {
    path.iter().fold(1_469_598_103_934_665_603, |hash, &index| {
        (hash ^ index as u64).wrapping_mul(1_099_511_628_211)
    })
}

fn reverse_first(path: &mut [usize], points: &[Point], grid: &mut CrossingGrid) -> bool {
    let found = if path.len() >= 500 {
        grid.find(path, points)
    } else {
        find(path, points)
    };
    let Some((first, last)) = found else {
        return false;
    };
    path[first + 1..last + 1].reverse();
    true
}

fn find(path: &[usize], points: &[Point]) -> Option<(usize, usize)> {
    for first in 0..path.len() {
        for last in first + 2..path.len() {
            if first == 0 && last + 1 == path.len() {
                continue;
            }
            if edges_cross(path, points, first, last) {
                return Some((first, last));
            }
        }
    }
    None
}

pub(super) fn edges_cross(path: &[usize], points: &[Point], first: usize, last: usize) -> bool {
    let a = points[path[first]];
    let b = points[path[(first + 1) % path.len()]];
    let c = points[path[last]];
    let d = points[path[(last + 1) % path.len()]];
    boxes_overlap(a, b, c, d) && segments_intersect(a, b, c, d)
}

fn boxes_overlap(a: Point, b: Point, c: Point, d: Point) -> bool {
    !(a.x().max(b.x()) < c.x().min(d.x())
        || c.x().max(d.x()) < a.x().min(b.x())
        || a.y().max(b.y()) < c.y().min(d.y())
        || c.y().max(d.y()) < a.y().min(b.y()))
}

fn segments_intersect(a: Point, b: Point, c: Point, d: Point) -> bool {
    let signs1 = [side(a, b, c), side(a, b, d)];
    let signs2 = [side(c, d, a), side(c, d, b)];
    let test1 = signs1[0] * signs1[1];
    let test2 = signs2[0] * signs2[1];
    if test1 > 0 || test2 > 0 {
        return false;
    }
    if test1 != 0 || test2 != 0 {
        return true;
    }
    if signs1 != [0, 0] {
        return false;
    }
    let x_axis = (b.x() - a.x()).abs() > (b.y() - a.y()).abs();
    let coordinate = |point: Point| if x_axis { point.x() } else { point.y() };
    let (i, j, k, l) = (coordinate(a), coordinate(b), coordinate(c), coordinate(d));
    let (i, j) = (i.min(j), i.max(j));
    let (k, l) = (k.min(l), k.max(l));
    (k >= i && k <= j) || (i >= k && i <= l)
}

fn side(a: Point, b: Point, point: Point) -> i64 {
    let dx = b.x() - a.x();
    let dy = b.y() - a.y();
    (dx * (point.y() - a.y()) - dy * (point.x() - a.x())).signum()
}
