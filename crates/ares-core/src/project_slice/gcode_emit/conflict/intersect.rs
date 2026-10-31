//! `ConflictChecker.cpp::RasterizationImpl`, `find_inter_of_lines`,
//! and `line_intersect`; `Line.cpp:73–92` owns collinear overlap.
use crate::geometry::{CoordinateScale, Line};
use std::collections::BTreeMap;

pub(super) struct OwnedLine {
    pub(super) line: Line,
    pub(super) owner: usize,
    pub(super) support: bool,
}

pub(super) fn find(lines: &[OwnedLine], scale: CoordinateScale) -> Option<(usize, usize)> {
    let mut grid: BTreeMap<(i64, i64), Vec<usize>> = BTreeMap::new();
    let mut cells = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        rasterize(line.line, (1.0 / scale.factor()) as i64, &mut cells);
        for cell in &cells {
            let candidates = grid.entry(*cell).or_default();
            if let Some(candidate) = candidates.iter().copied().find(|&candidate| {
                let other = &lines[candidate];
                line.owner != other.owner && intersects(line, other, scale)
            }) {
                return Some((line.owner, lines[candidate].owner));
            }
            candidates.push(index);
        }
    }
    None
}

fn intersects(first: &OwnedLine, second: &OwnedLine, scale: CoordinateScale) -> bool {
    if overlap(first.line, second.line, scale) > 0.01 / scale.factor() {
        return true;
    }
    let Some(point) = first.line.intersection(second.line) else {
        return false;
    };
    let distance = [first.line.a, first.line.b, second.line.a, second.line.b]
        .into_iter()
        .map(|end| {
            let x = scale.unscale(end.x() - point.x());
            let y = scale.unscale(end.y() - point.y());
            (x * x + y * y).sqrt()
        })
        .fold(f64::MAX, f64::min);
    distance
        > if first.support && second.support {
            100.0
        } else {
            0.01
        }
}

fn overlap(first: Line, second: Line, scale: CoordinateScale) -> f64 {
    if !parallel(first, second) {
        return 0.0;
    }
    let between = Line::new(first.a, second.a);
    if between.length() > 1e-4 / scale.factor() && !parallel(first, between) {
        return 0.0;
    }
    let (a_min, a_max) = (first.a.x().min(first.b.x()), first.a.x().max(first.b.x()));
    let (b_min, b_max) = (
        second.a.x().min(second.b.x()),
        second.a.x().max(second.b.x()),
    );
    if a_min > b_max || a_max < b_min {
        return 0.0;
    }
    // Preserve the source's X projection, including NaN for vertical overlap.
    (a_max.min(b_max) - a_min.max(b_min)).max(0) as f64 / ((a_max - a_min) as f64 / first.length())
}
fn parallel(first: Line, second: Line) -> bool {
    let a = [
        (first.b.x() - first.a.x()) as f64,
        (first.b.y() - first.a.y()) as f64,
    ];
    let b = [
        (second.b.x() - second.a.x()) as f64,
        (second.b.y() - second.a.y()) as f64,
    ];
    let cross = a[0] * b[1] - a[1] * b[0];
    cross * cross < (1e-4_f64 * 1e-4) * (a[0] * a[0] + a[1] * a[1]) * (b[0] * b[0] + b[1] * b[1])
}

fn rasterize(line: Line, cell: i64, output: &mut Vec<(i64, i64)>) {
    output.clear();
    let start = [line.a.x(), line.a.y()];
    let end = [line.b.x(), line.b.y()];
    let mut current = start.map(|v| v / cell);
    let last = end.map(|v| v / cell);
    let ray = [end[0] - start[0], end[1] - start[1]];
    let step = ray.map(|v| if v >= 0 { 1 } else { -1 });
    let mut time = [f64::MAX; 2];
    let mut delta = [f64::MAX; 2];
    for axis in 0..2 {
        if ray[axis] != 0 {
            let boundary = (current[axis] + step[axis] + i64::from(step[axis] < 0)) * cell;
            time[axis] = (boundary - start[axis]) as f64 / ray[axis] as f64;
            delta[axis] = cell as f64 / ray[axis] as f64 * step[axis] as f64;
        }
    }
    output.push((current[0], current[1]));
    while current != last {
        let axis = if current[0] == last[0] {
            1
        } else if current[1] == last[1] || time[0] < time[1] {
            0
        } else {
            1
        };
        current[axis] += step[axis];
        time[axis] += delta[axis];
        output.push((current[0], current[1]));
    }
}
