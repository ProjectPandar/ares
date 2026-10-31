// ClipperUtils.cpp:append_offset_path, c2_offset_setup and c2_offset_closed_paths.
mod execute;
mod expolygon;
mod opening;

use super::{ClipperError, FillRule, PolyTree, backend};
use crate::geometry::{Point, Polygon, clipper2 as c2};
pub(crate) use execute::{
    offset_lines, offset_open_paths, offset_paths, offset_paths_tree, raw_offset_paths,
};
pub(crate) use expolygon::{
    closing_ex, offset_expolygon, offset_expolygon_refs_paths, offset_expolygons,
    offset_expolygons_paths, offset2_ex, offset2_ex_with_interstage, opening_ex,
};
pub(crate) use opening::opening_paths;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JoinType {
    Square,
    Round,
    Miter,
}
fn join(value: JoinType) -> c2::JoinType {
    match value {
        JoinType::Square => c2::JoinType::Square,
        JoinType::Round => c2::JoinType::Round,
        JoinType::Miter => c2::JoinType::Miter,
    }
}

trait OffsetPoint: Copy {
    fn x(self) -> i64;
    fn y(self) -> i64;
}
impl OffsetPoint for Point {
    fn x(self) -> i64 {
        self.x()
    }
    fn y(self) -> i64 {
        self.y()
    }
}
impl OffsetPoint for c2::Point64 {
    fn x(self) -> i64 {
        self.x
    }
    fn y(self) -> i64 {
        self.y
    }
}

// ClipperUtils.cpp:235–264: source PathT access without a second coordinate buffer.
fn append_path<P: OffsetPoint>(
    paths: &mut c2::Paths64,
    path: &[P],
    shortest: f64,
    end: c2::EndType,
) -> bool {
    if path.is_empty() {
        return false;
    }
    let squared = shortest * shortest;
    let same = |a: P, b: P| {
        let dx = (a.x() - b.x()) as f64;
        let dy = (a.y() - b.y()) as f64;
        if squared > 0.0 {
            dx * dx + dy * dy < squared
        } else {
            dx == 0.0 && dy == 0.0
        }
    };
    let mut last = path.len() - 1;
    if matches!(end, c2::EndType::Polygon | c2::EndType::Joined) {
        while last > 0 && same(path[last], path[0]) {
            last -= 1;
        }
    }
    let mut output = Vec::with_capacity(last + 1);
    output.push(c2::Point64::new(path[0].x(), path[0].y()));
    let mut previous = 0;
    for i in 1..=last {
        if !same(path[i], path[previous]) {
            output.push(c2::Point64::new(path[i].x(), path[i].y()));
            previous = i;
        }
    }
    if end == c2::EndType::Polygon && output.len() < 3 {
        return false;
    }
    paths.push(output);
    true
}

fn setup(delta: f32, kind: JoinType, limit: f64, cap_tolerance: f64) -> c2::ClipperOffset {
    let maximum = f64::from(delta).abs() * 0.25;
    let mut co = c2::ClipperOffset::new_default();
    if kind == JoinType::Round {
        co.set_arc_tolerance(if limit > 0.0 {
            limit.min(maximum)
        } else {
            0.25
        });
    } else {
        co.set_miter_limit(limit.max(2.0));
        co.set_arc_tolerance(
            (if cap_tolerance > 0.0 {
                cap_tolerance
            } else {
                0.25
            })
            .min(maximum),
        );
    }
    co
}

// Mirrors the upstream TOut overload, avoiding a second union to construct trees.
trait Output: Default {
    fn offset(&mut self, co: &mut c2::ClipperOffset, delta: f64);
    fn unite(&mut self, paths: &c2::Paths64, fill: FillRule);
}
impl Output for c2::Paths64 {
    fn offset(&mut self, co: &mut c2::ClipperOffset, delta: f64) {
        co.execute(delta, self);
    }
    fn unite(&mut self, paths: &c2::Paths64, fill: FillRule) {
        let mut clipper = backend::configured();
        clipper.add_subject(paths);
        clipper.execute(c2::ClipType::Union, fill, self, None);
    }
}
impl Output for c2::PolyTree64 {
    fn offset(&mut self, co: &mut c2::ClipperOffset, delta: f64) {
        co.execute_tree(delta, self);
    }
    fn unite(&mut self, paths: &c2::Paths64, fill: FillRule) {
        let mut clipper = backend::configured();
        clipper.add_subject(paths);
        clipper.execute_tree(c2::ClipType::Union, fill, self, &mut Vec::new());
    }
}

fn can_group(paths: &c2::Paths64, areas: &[f64]) -> bool {
    let contours: Vec<_> = paths
        .iter()
        .zip(areas)
        .filter(|(_, area)| **area > 0.0)
        .map(|(p, _)| c2::get_bounds_path(p))
        .collect();
    paths
        .iter()
        .zip(areas)
        .filter(|(_, area)| **area < 0.0)
        .all(|(p, _)| {
            let hole = c2::get_bounds_path(p);
            contours.iter().any(|r| {
                r.left < hole.left
                    && r.right > hole.right
                    && r.top < hole.top
                    && r.bottom > hole.bottom
            })
        })
}

fn closed<T: Output>(paths: c2::Paths64, delta: f32, kind: JoinType, limit: f64) -> T {
    let areas: Vec<_> = paths.iter().map(c2::area).collect();
    let mut co = setup(delta, kind, limit, 0.0);
    let mut output = T::default();
    if can_group(&paths, &areas) {
        if delta < 0.0 && areas.iter().all(|&area| area == 0.0) {
            return output;
        }
        co.add_paths(&paths, join(kind), c2::EndType::Polygon);
        output.offset(&mut co, f64::from(delta));
    } else {
        let mut offset = Vec::new();
        let mut single = Vec::new();
        for (path, area) in paths.iter().zip(areas) {
            if area == 0.0 && delta < 0.0 {
                continue;
            }
            co.clear();
            co.add_path(path, join(kind), c2::EndType::Polygon);
            co.execute(
                f64::from(if area < 0.0 { -delta } else { delta }),
                &mut single,
            );
            offset.append(&mut single);
        }
        output.unite(
            &offset,
            if delta > 0.0 {
                FillRule::NonZero
            } else {
                FillRule::Positive
            },
        );
    }
    output
}
fn prepare(paths: &[Polygon], delta: f32, end: c2::EndType) -> c2::Paths64 {
    let mut output = Vec::with_capacity(paths.len());
    let shortest = (f64::from(delta) * 0.005).abs();
    for path in paths {
        append_path(&mut output, path.points(), shortest, end);
    }
    output
}
