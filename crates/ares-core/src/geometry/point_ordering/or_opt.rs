//! ShortestPath.cpp:1578–1622::improve_cycle_by_or_opt at
//! OrcaSlicer 345efec64c7b95a05ce5cf96df0dde66a158835c.

use super::{Point, nearest_points::NearestPoints, position, squared};

pub(super) fn improve(path: &mut Vec<usize>, points: &[Point], epsilon: f64) -> bool {
    let coords = points.iter().copied().map(position).collect::<Vec<_>>();
    let grid = NearestPoints::<8>::new(&coords);
    let mut positions = vec![0; points.len()];
    refresh(path, &mut positions);
    let mut improved = false;
    loop {
        let mut moved = false;
        for length in 1..=3 {
            for first in 1..=path.len() - length {
                moved |= move_run(
                    path,
                    (first, first + length - 1),
                    (&grid, &coords),
                    &mut positions,
                    epsilon,
                );
            }
        }
        improved |= moved;
        if !moved {
            return improved;
        }
    }
}

fn move_run(
    path: &mut Vec<usize>,
    (first, last): (usize, usize),
    (grid, coords): (&NearestPoints<'_, 8>, &[[f64; 2]]),
    positions: &mut [usize],
    epsilon: f64,
) -> bool {
    let removal = distance(path, coords, first - 1, first) + distance(path, coords, last, last + 1)
        - distance(path, coords, first - 1, last + 1);
    let mut applied = removal <= epsilon;
    let mut moved = false;
    for end in [path[first], path[last]] {
        for candidate in grid.query(coords[end], |index| index != end) {
            let gaps = if candidate == usize::MAX {
                [path.len(); 2]
            } else {
                [
                    (positions[candidate] + path.len() - 1) % path.len(),
                    positions[candidate],
                ]
            };
            if !applied && move_at_gaps(path, coords, (first, last), gaps, removal - epsilon) {
                refresh(path, positions);
                applied = true;
                moved = true;
            }
        }
    }
    moved
}

fn move_at_gaps(
    path: &mut Vec<usize>,
    coords: &[[f64; 2]],
    (first, last): (usize, usize),
    gaps: [usize; 2],
    required: f64,
) -> bool {
    for gap in gaps {
        if gap == path.len() || (gap + 1 >= first && gap <= last) {
            continue;
        }
        if relocate(path, coords, (first, last, gap), required) {
            return true;
        }
    }
    false
}

fn relocate(
    path: &mut Vec<usize>,
    coords: &[[f64; 2]],
    (first, last, gap): (usize, usize, usize),
    required: f64,
) -> bool {
    let base = distance(path, coords, gap, gap + 1);
    let kept = distance(path, coords, gap, first) + distance(path, coords, last, gap + 1) - base;
    let reversed =
        distance(path, coords, gap, last) + distance(path, coords, first, gap + 1) - base;
    let smallest = if reversed < kept { reversed } else { kept };
    if smallest > required {
        return false;
    }
    let mut run = path[first..=last].to_vec();
    if reversed < kept {
        run.reverse();
    }
    drop(path.drain(first..=last));
    let insertion = if gap < first {
        gap + 1
    } else {
        gap + 1 - run.len()
    };
    path.splice(insertion..insertion, run);
    true
}

fn distance(path: &[usize], coords: &[[f64; 2]], a: usize, b: usize) -> f64 {
    squared(coords[path[a % path.len()]], coords[path[b % path.len()]]).sqrt()
}

fn refresh(path: &[usize], positions: &mut [usize]) {
    for (index, &point) in path.iter().enumerate() {
        positions[point] = index;
    }
}
