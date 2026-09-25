//! Loop splitting geometry — scale/normalize helpers, seam-point
//! projection, and the loop split/reorder (`seam placement` tail).

pub(super) const SEAM_VERTEX_SNAP_MM: f64 = 0.0015;

use super::fitting;
use crate::geometry::CoordinateScale;
use crate::project_slice::perimeters::classic::chained_loops::ExtrusionLoop;
use crate::project_slice::perimeters::classic::materialize::{ExtrusionPath, Point3};

pub(super) fn scale_position(position: (f32, f32), scale: CoordinateScale) -> (i64, i64) {
    (
        (f64::from(position.0) / scale.factor()) as i64,
        (f64::from(position.1) / scale.factor()) as i64,
    )
}

pub(super) fn normalized(vector: (f32, f32)) -> (f32, f32) {
    let length = vector.0.hypot(vector.1);
    (vector.0 / length, vector.1 / length)
}

#[derive(Clone, Copy)]
pub(super) struct Projection {
    pub(super) path: usize,
    pub(super) segment: usize,
    pub(super) x: i64,
    pub(super) y: i64,
}

/// `get_next_loop_point` (SeamPlacer.cpp:1511-1519): advance to the next
/// polyline point, wrapping across paths.
pub(super) fn next_loop_point(loop_: &ExtrusionLoop, mut current: &Projection) -> Projection {
    let mut segment = current.segment + 1;
    let mut path = current.path;
    if segment >= loop_.paths[path].polyline.points.len() {
        path = (path + 1) % loop_.paths.len();
        segment = 0;
    }
    let point = loop_.paths[path].polyline.points[segment];
    Projection {
        path,
        segment,
        x: point.x,
        y: point.y,
    }
}

pub(super) fn closest_projection(
    paths: &[ExtrusionPath],
    target: (i64, i64),
    prefer_non_overhang: bool,
) -> Projection {
    // `get_closest_path_and_point` (`ExtrusionEntity.cpp:230-253`): track
    // the closest foot overall and, when prefer_non_overhang, the closest
    // foot on a non-bridge path — the non-overhang foot wins when present.
    // Only `OverhangPerimeter` is bridge inside a loop (`is_bridge`
    // `ExtrusionEntity.hpp:99-103`).
    let mut best = None::<(Projection, f64)>;
    let mut best_non_overhang = None::<(Projection, f64)>;
    for (path_index, path) in paths.iter().enumerate() {
        let mut path_best = None::<(Projection, f64)>;
        for (segment_index, segment) in path.polyline.points.windows(2).enumerate() {
            let a = (segment[0].x, segment[0].y);
            let b = (segment[1].x, segment[1].y);
            let (x, y) = project_onto_segment(a, b, target);
            let distance = squared_distance((x, y), target);
            if path_best.as_ref().is_none_or(|(_, best)| distance < *best) {
                path_best = Some((
                    Projection {
                        path: path_index,
                        segment: segment_index,
                        x,
                        y,
                    },
                    distance,
                ));
            }
        }
        if let Some((projection, distance)) = path_best {
            if best.as_ref().is_none_or(|(_, best)| distance < *best) {
                best = Some((projection, distance));
            }
            if prefer_non_overhang
                && path.role != crate::project_slice::perimeters::classic::materialize::ExtrusionRole::OverhangPerimeter
                && best_non_overhang
                    .as_ref()
                    .is_none_or(|(_, best)| distance < *best)
            {
                best_non_overhang = Some((projection, distance));
            }
        }
    }
    if prefer_non_overhang {
        if let Some((projection, _)) = best_non_overhang {
            return projection;
        }
    }
    best.expect("an extrusion loop has a segment").0
}

pub(super) fn project_onto_segment(a: (i64, i64), b: (i64, i64), target: (i64, i64)) -> (i64, i64) {
    let lx = (b.0 - a.0) as f64;
    let ly = (b.1 - a.1) as f64;
    let denominator = lx.mul_add(lx, ly * ly);
    if denominator == 0.0 {
        return a;
    }
    let theta = (((b.0 - target.0) as f64) * lx + ((b.1 - target.1) as f64) * ly) / denominator;
    if !(0.0..=1.0).contains(&theta) {
        return if squared_distance(a, target) < squared_distance(b, target) {
            a
        } else {
            b
        };
    }
    (
        (theta * a.0 as f64 + (1.0 - theta) * b.0 as f64) as i64,
        (theta * a.1 as f64 + (1.0 - theta) * b.1 as f64) as i64,
    )
}

pub(super) fn squared_distance(left: (i64, i64), right: (i64, i64)) -> f64 {
    let dx = (left.0 - right.0) as f64;
    let dy = (left.1 - right.1) as f64;
    dx.mul_add(dx, dy * dy)
}

pub(super) fn split_at(loop_: &mut ExtrusionLoop, seam: (i64, i64), scale: CoordinateScale) {
    if let Ok(path) = std::env::var("ARES_DUMP_SEAM") {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(f, "SEAM {},{}", seam.0, seam.1);
        }
    }
    if let Ok(path) = std::env::var("ARES_DUMP_LOOP") {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = write!(f, "L[ ");
            for p in &loop_.paths {
                for q in &p.polyline.points {
                    let _ = write!(f, "{},{};", q.x, q.y);
                }
                let _ = write!(f, "|");
            }
            let _ = writeln!(f, "]");
        }
    }
    let snap_distance = SEAM_VERTEX_SNAP_MM / scale.factor();
    let snap_distance_squared = snap_distance * snap_distance;
    // `split_at_vertex` fast path (`ExtrusionEntity.cpp:1622-1626`): if an
    // existing loop vertex is within scaled_epsilon of the seam, the split
    // lands exactly on that vertex — projection never runs.
    let seam = loop_
        .paths
        .iter()
        .flat_map(|path| &path.polyline.points)
        .find(|point| squared_distance((point.x, point.y), seam) < snap_distance_squared)
        .map_or(seam, |point| (point.x, point.y));
    // `ExtrusionLoop::split_at` (`ExtrusionEntity.cpp:266-284`): project
    // onto the loop preferring non-overhang feet, THEN snap the projected
    // foot to the projected segment's endpoints when closer than
    // scaled_epsilon — never to an unrelated vertex elsewhere.
    let mut projection = closest_projection(&loop_.paths, seam, true);
    {
        let p1 = loop_.paths[projection.path].polyline.points[projection.segment];
        let p2 = loop_.paths[projection.path].polyline.points[projection.segment + 1];
        let d2_1 = squared_distance(seam, (p1.x, p1.y));
        let d2_2 = squared_distance(seam, (p2.x, p2.y));
        if d2_1 < d2_2 {
            if d2_1 < snap_distance_squared {
                projection.x = p1.x;
                projection.y = p1.y;
            }
        } else if d2_2 < snap_distance_squared {
            projection.x = p2.x;
            projection.y = p2.y;
        }
    }
    let single_path = loop_.paths.len() == 1;
    let mut paths = std::mem::take(&mut loop_.paths);
    let following = paths.split_off(projection.path + 1);
    let target = paths.pop().expect("projected path exists");
    let split = projected_parts(target, &projection, scale);
    if single_path {
        match split {
            (Some(mut suffix), Some(prefix)) => {
                fitting::append_polyline(&mut suffix.polyline, prefix.polyline);
                loop_.paths.push(suffix);
            }
            (Some(path), None) | (None, Some(path)) => loop_.paths.push(path),
            (None, None) => unreachable!("a projected loop path has a valid side"),
        }
        return;
    }
    loop_.paths.extend(split.0);
    loop_.paths.extend(following);
    loop_.paths.extend(paths);
    loop_.paths.extend(split.1);
}

pub(super) fn projected_parts(
    path: ExtrusionPath,
    projection: &Projection,
    scale: CoordinateScale,
) -> (Option<ExtrusionPath>, Option<ExtrusionPath>) {
    let point = Point3 {
        x: projection.x,
        y: projection.y,
        z: path.polyline.points[projection.segment].z,
    };
    let start = path.polyline.points[projection.segment];
    let end = path.polyline.points[projection.segment + 1];
    let (suffix, prefix) = if point.x == start.x && point.y == start.y {
        let (prefix, suffix) = fitting::split_at_index(&path.polyline, projection.segment, scale);
        (suffix, prefix)
    } else if point.x == end.x && point.y == end.y {
        let (prefix, suffix) =
            fitting::split_at_index(&path.polyline, projection.segment + 1, scale);
        (suffix, prefix)
    } else {
        let (mut prefix, _) = fitting::split_at_index(&path.polyline, projection.segment, scale);
        fitting::append(&mut prefix, point);
        let (_, mut suffix) =
            fitting::split_at_index(&path.polyline, projection.segment + 1, scale);
        fitting::prepend(&mut suffix, point);
        (suffix, prefix)
    };
    let make = |polyline: crate::project_slice::perimeters::classic::materialize::Polyline3| {
        (polyline.points.len() >= 2).then_some(ExtrusionPath {
            polyline,
            role: path.role,
            can_reverse: path.can_reverse,
            mm3_per_mm: path.mm3_per_mm,
            width: path.width,
            height: path.height,
        })
    };
    (make(suffix), make(prefix))
}
