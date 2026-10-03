//! Single-path `ExtrusionLoop::split_at(false)` and `Polyline3::split_at(Point)`.

use super::SupportExtrusionPath;
use crate::{
    ProcessWallDirection,
    geometry::{CoordinateScale, Point, Polygon},
    project_slice::{
        perimeters::classic::materialize::{Point3, Polyline3},
        seam_placement::fitting,
    },
};

impl SupportExtrusionPath {
    pub(in crate::project_slice) fn loop_polyline(
        &self,
        cursor: Point,
        scale: CoordinateScale,
        direction: Option<ProcessWallDirection>,
    ) -> Polyline3 {
        let reversed = direction.and_then(|direction| {
            let polygon = Polygon::new(
                self.polyline.points[..self.polyline.points.len() - 1]
                    .iter()
                    .map(|point| Point::new(point.x, point.y))
                    .collect(),
            );
            let reverse = match direction {
                ProcessWallDirection::CounterClockwise => !polygon.is_counter_clockwise(),
                ProcessWallDirection::Clockwise => polygon.is_counter_clockwise(),
            };
            if !reverse {
                return None;
            }
            let mut polyline = self.polyline.clone();
            super::chain::reverse_polyline(&mut polyline);
            Some(polyline)
        });
        let source = reversed.as_ref().unwrap_or(&self.polyline);
        let target = (cursor.x(), cursor.y());
        let (segment, mut point) = closest(&source.points, target);
        let first = xy(source.points[segment]);
        let last = xy(source.points[segment + 1]);
        let first_distance = distance2(target, first);
        let last_distance = distance2(target, last);
        let epsilon2 = (0.001 / scale.factor()).powi(2);
        if first_distance < last_distance {
            if first_distance < epsilon2 {
                point = first;
            }
        } else if last_distance < epsilon2 {
            point = last;
        }
        let (prefix, suffix) = split(source, &mut point, scale);
        let joined = join(prefix, suffix);
        // `split_at_vertex`: nearest vertex, strict epsilon and first minimum.
        let mut minimum = f64::MAX;
        let mut index = 0;
        for (candidate, vertex) in joined.points.iter().enumerate() {
            let distance = distance2(xy(*vertex), point);
            if distance < minimum {
                minimum = distance;
                index = candidate;
            }
        }
        if minimum < epsilon2 && index != 0 {
            let (prefix, suffix) = fitting::split_at_index(&joined, index, scale);
            if prefix.points.len() >= 2 && suffix.points.len() >= 2 {
                return join(prefix, suffix);
            }
        }
        joined
    }
}

fn split(
    source: &Polyline3,
    point: &mut (i64, i64),
    scale: CoordinateScale,
) -> (Polyline3, Polyline3) {
    if let Some(index) = source
        .points
        .iter()
        .position(|vertex| xy(*vertex) == *point)
    {
        let (prefix, suffix) = fitting::split_at_index(source, index, scale);
        *point = if prefix.points.len() >= 2 {
            xy(*prefix.points.last().unwrap())
        } else {
            xy(suffix.points[0])
        };
        return (prefix, suffix);
    }
    // Polyline3 rescans the projected point, starting at its first vertex.
    let mut projected = xy(source.points[0]);
    let mut minimum = distance2(projected, *point).sqrt();
    let mut segment = 0;
    for (index, pair) in source.points.windows(2).enumerate() {
        let candidate = project(xy(pair[0]), xy(pair[1]), *point);
        let distance = distance2(candidate, *point).sqrt();
        if distance < minimum {
            minimum = distance;
            projected = candidate;
            segment = index;
        }
    }
    let (mut prefix, mut suffix) = if let Some(index) = source
        .points
        .iter()
        .position(|vertex| xy(*vertex) == projected)
    {
        fitting::split_at_index(source, index, scale)
    } else {
        let (prefix, _) = fitting::split_at_index(source, segment, scale);
        let (_, suffix) = fitting::split_at_index(source, segment + 1, scale);
        (prefix, suffix)
    };
    let prefix_z = prefix.points.last().unwrap().z;
    let suffix_z = suffix.points[0].z;
    fitting::append(
        &mut prefix,
        Point3 {
            x: point.0,
            y: point.1,
            z: prefix_z,
        },
    );
    fitting::prepend(
        &mut suffix,
        Point3 {
            x: point.0,
            y: point.1,
            z: suffix_z,
        },
    );
    (prefix, suffix)
}

fn join(prefix: Polyline3, mut suffix: Polyline3) -> Polyline3 {
    if prefix.points.len() < 2 {
        return suffix;
    }
    if suffix.points.len() < 2 {
        return prefix;
    }
    fitting::append_polyline(&mut suffix, prefix);
    suffix
}

fn closest(points: &[Point3], target: (i64, i64)) -> (usize, (i64, i64)) {
    let mut minimum = f64::MAX;
    let mut best = (0, xy(points[0]));
    for (index, pair) in points.windows(2).enumerate() {
        let point = project(xy(pair[0]), xy(pair[1]), target);
        let distance = distance2(point, target);
        if distance < minimum {
            minimum = distance;
            best = (index, point);
        }
    }
    best
}

fn project(first: (i64, i64), last: (i64, i64), target: (i64, i64)) -> (i64, i64) {
    if first == last {
        return first;
    }
    let dx = (last.0 - first.0) as f64;
    let dy = (last.1 - first.1) as f64;
    let theta =
        ((last.0 - target.0) as f64 * dx + (last.1 - target.1) as f64 * dy) / (dx * dx + dy * dy);
    if (0.0..=1.0).contains(&theta) {
        return (
            (theta * first.0 as f64 + (1.0 - theta) * last.0 as f64) as i64,
            (theta * first.1 as f64 + (1.0 - theta) * last.1 as f64) as i64,
        );
    }
    if distance2(first, target) < distance2(last, target) {
        first
    } else {
        last
    }
}

fn xy(point: Point3) -> (i64, i64) {
    (point.x, point.y)
}

fn distance2(first: (i64, i64), last: (i64, i64)) -> f64 {
    let dx = (first.0 - last.0) as f64;
    let dy = (first.1 - last.1) as f64;
    dx * dx + dy * dy
}
