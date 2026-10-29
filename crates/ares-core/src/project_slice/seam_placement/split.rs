//! Loop splitting geometry — scale/normalize helpers, seam-point
//! projection, and the loop split/reorder (`seam placement` tail).

mod vertex;

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
    // `SeamPlacer.cpp:1584-1587`: Eigen leaves a zero direction unchanged.
    let squared_norm = vector.0 * vector.0 + vector.1 * vector.1;
    if squared_norm > 0.0 {
        let length = squared_norm.sqrt();
        (vector.0 / length, vector.1 / length)
    } else {
        vector
    }
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
pub(super) fn next_loop_point(loop_: &ExtrusionLoop, current: &Projection) -> Projection {
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
    if prefer_non_overhang && let Some((projection, _)) = best_non_overhang {
        return projection;
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
    if !vertex::split_at_vertex(loop_, seam, SEAM_VERTEX_SNAP_MM / scale.factor(), scale) {
        split_at_projected(loop_, seam, true, scale);
    }
}

pub(super) fn split_at_projected(
    loop_: &mut ExtrusionLoop,
    seam: (i64, i64),
    prefer_non_overhang: bool,
    scale: CoordinateScale,
) {
    let snap_distance = 0.001 / scale.factor();
    let snap_distance_squared = snap_distance * snap_distance;
    // `ExtrusionEntity.cpp:266–284`: projection owns its endpoint snap;
    // this is distinct from SeamPlacer's initial vertex search.
    let mut projection = closest_projection(&loop_.paths, seam, prefer_non_overhang);
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
    let target = loop_.paths.remove(projection.path);
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
    } else {
        // Source replaces the selected path in place before its separate
        // vertex re-search; another path may own the same shared vertex.
        loop_.paths.splice(
            projection.path..projection.path,
            split.1.into_iter().chain(split.0),
        );
    }
    vertex::split_at_vertex(loop_, (projection.x, projection.y), snap_distance, scale);
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
        let suffix_point = Point3 { z: end.z, ..point };
        fitting::prepend(&mut suffix, suffix_point);
        (suffix, prefix)
    };
    let make = |polyline: crate::project_slice::perimeters::classic::materialize::Polyline3| {
        (polyline.points.len() >= 2).then_some(ExtrusionPath {
            z_contoured: path.z_contoured,
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
