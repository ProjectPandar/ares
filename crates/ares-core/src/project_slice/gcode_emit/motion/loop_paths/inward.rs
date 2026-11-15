//! Beta GCode.cpp:8730-8736,8790-8794 and WipePathHelpers.cpp:61-92,886-926.

use std::borrow::Cow;

use crate::geometry::CoordinateScale;

use super::*;

pub(super) fn post_loop_destination(
    paths: &[ExtrusionPath],
    clipping: (ExtrusionLoopRole, usize, f64),
    geometry: LayerGeometry<'_>,
    state: &EmitState,
) -> Option<crate::geometry::Point> {
    let (loop_role, path_count, end_clip) = clipping;
    let original_paths = paths;
    let paths = &paths[..path_count];
    let last = &paths[path_count - 1];
    if !state.options.wipe_on_loops
        || last.role != ExtrusionRole::ExternalPerimeter
        || state.options.wall_loops <= 1
        || paths[0].polyline.points.len() < 2
    {
        return None;
    }
    let last_points = clipped_last(last, end_clip / geometry.scale.factor(), geometry.scale);
    if last_points.len() < 2 {
        return None;
    }
    let orientation = (
        loop_role == ExtrusionLoopRole::Hole,
        loop_is_counter_clockwise(original_paths),
    );
    inward_destination(
        paths,
        &last_points,
        orientation,
        state.options.nozzle_diameter / geometry.scale.factor(),
        super::super::path::SOURCE_EPSILON_MM / geometry.scale.factor(),
    )
}

pub(super) fn append_inward_move(
    output: &mut Vec<u8>,
    point: crate::geometry::Point,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let x = geometry.scale.unscale(point.x()) + state.origin.0 - state.extruder_offset.0;
    let y = geometry.scale.unscale(point.y()) + state.origin.1 - state.extruder_offset.1;
    let comment = if state.options.gcode_comments {
        " ; move inwards before travel"
    } else {
        ""
    };
    output.extend_from_slice(
        format!(
            "G1 X{} Y{}{comment}\n",
            format_axis(x - state.xy_offset.0),
            format_axis(y - state.xy_offset.1)
        )
        .as_bytes(),
    );
    state.x = x;
    state.y = y;
    state.last_scaled_position = (point.x(), point.y());
    state.last_position_defined = true;
}

fn clipped_last(path: &ExtrusionPath, distance: f64, scale: CoordinateScale) -> Cow<'_, [Point3]> {
    let points = &path.polyline.points;
    if path.z_contoured {
        return super::super::path::contour::clipped(points, distance);
    }
    if distance <= 0.0 {
        return Cow::Borrowed(points);
    }
    let (count, endpoint) =
        super::super::clip::clipped_tail(points.iter().map(|point| (point.x, point.y)), distance);
    let mut points = points[..count].to_vec();
    if let Some((x, y)) = endpoint {
        let last = points.last_mut().unwrap();
        last.x = x;
        last.y = y;
    }
    if !path.polyline.fitting.is_empty() {
        let mut local_points = points
            .iter()
            .map(|point| (scale.unscale(point.x), scale.unscale(point.y)))
            .collect::<Vec<_>>();
        let mut fitting = path.polyline.fitting.clone();
        let pre_projected = local_points.last().copied();
        super::super::arc::clip_fitting_end(&mut local_points, &mut fitting, scale);
        if let Some(&(x, y)) = local_points.last()
            && Some((x, y)) != pre_projected
        {
            let last = points.last_mut().unwrap();
            last.x = (x / scale.factor()).round() as i64;
            last.y = (y / scale.factor()).round() as i64;
        }
    }
    Cow::Owned(points)
}

fn inward_destination(
    paths: &[ExtrusionPath],
    last_points: &[Point3],
    orientation: (bool, bool),
    nozzle: f64,
    scaled_epsilon: f64,
) -> Option<crate::geometry::Point> {
    let total_length = (0..paths.len())
        .map(|index| scaled_length(path_points(paths, last_points, index)))
        .sum::<f64>();
    let sample_distance = nozzle.min(total_length * 0.25);
    let mut a = sample_path_at_distance(paths, last_points, true, sample_distance, scaled_epsilon);
    let mut b = sample_path_at_distance(paths, last_points, false, sample_distance, scaled_epsilon);
    let mut center = paths[0].polyline.points[0];
    center.z = 0;
    if a == b || a == center || b == center {
        return None;
    }
    let (is_hole, is_ccw) = orientation;
    let reverse_turn = is_hole == is_ccw;
    if reverse_turn {
        std::mem::swap(&mut a, &mut b);
    }
    let mut angle = ccw_angle(center, a, b) / 3.0;
    let angle_epsilon = 0.01;
    if angle < angle_epsilon || angle > 2.0 * std::f64::consts::PI / 3.0 - angle_epsilon {
        return None;
    }
    if reverse_turn {
        angle = -angle;
    }
    let point = sample_path_at_distance(
        paths,
        last_points,
        true,
        (0.2 * nozzle).min(sample_distance),
        scaled_epsilon,
    );
    Some(rotate_point(point, center, angle))
}

fn path_points<'a>(
    paths: &'a [ExtrusionPath],
    last_points: &'a [Point3],
    index: usize,
) -> &'a [Point3] {
    if index + 1 == paths.len() {
        last_points
    } else {
        &paths[index].polyline.points
    }
}

fn scaled_length(points: &[Point3]) -> f64 {
    points
        .windows(2)
        .map(|pair| {
            let dx = (pair[1].x - pair[0].x) as f64;
            let dy = (pair[1].y - pair[0].y) as f64;
            let dz = (pair[1].z - pair[0].z) as f64;
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

fn sample_path_at_distance(
    paths: &[ExtrusionPath],
    last_points: &[Point3],
    forward: bool,
    mut remaining: f64,
    scaled_epsilon: f64,
) -> Point3 {
    let endpoint = if forward {
        paths[0].polyline.points[0]
    } else {
        *last_points.last().unwrap()
    };
    let mut result = Point3 { z: 0, ..endpoint };
    for step in 0..paths.len() {
        if remaining <= 0.0 {
            break;
        }
        let index = if forward {
            step
        } else {
            paths.len() - 1 - step
        };
        if let Some(point) = sample_polyline(
            path_points(paths, last_points, index),
            forward,
            &mut remaining,
            &mut result,
            scaled_epsilon,
        ) {
            return point;
        }
    }
    result
}

fn sample_polyline(
    points: &[Point3],
    forward: bool,
    remaining: &mut f64,
    result: &mut Point3,
    scaled_epsilon: f64,
) -> Option<Point3> {
    for step in 0..points.len() - 1 {
        let (cur, next) = if forward {
            (points[step], points[step + 1])
        } else {
            (
                points[points.len() - 1 - step],
                points[points.len() - 2 - step],
            )
        };
        let dx = (next.x - cur.x) as f64;
        let dy = (next.y - cur.y) as f64;
        let length = (dx * dx + dy * dy).sqrt();
        if length < scaled_epsilon {
            continue;
        }
        if *remaining <= length {
            let ratio = *remaining / length;
            return Some(Point3 {
                x: (cur.x as f64 + ratio * dx) as i64,
                y: (cur.y as f64 + ratio * dy) as i64,
                z: 0,
            });
        }
        *remaining -= length;
        *result = Point3 { z: 0, ..next };
    }
    None
}

fn rotate_point(point: Point3, center: Point3, angle: f64) -> crate::geometry::Point {
    let sine = angle.sin();
    let cosine = angle.cos();
    let dx = (point.x - center.x) as f64;
    let dy = (point.y - center.y) as f64;
    // Point.cpp:57-64 uses libslic3r.h::fast_round_up, not interpolation's cast.
    let round = |value: f64| {
        if value == 0.49999999999999994 {
            0
        } else {
            (value + 0.5).floor() as i64
        }
    };
    crate::geometry::Point::new(
        round(center.x as f64 + cosine * dx - sine * dy),
        round(center.y as f64 + sine * dx + cosine * dy),
    )
}

pub(super) fn loop_is_counter_clockwise(paths: &[ExtrusionPath]) -> bool {
    let points = paths
        .iter()
        .flat_map(|path| path.polyline.points.iter())
        .collect::<Vec<_>>();
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(left, right)| left.x as i128 * right.y as i128 - right.x as i128 * left.y as i128)
        .sum::<i128>()
        >= 0
}

pub(super) fn ccw_angle(center: Point3, first: Point3, second: Point3) -> f64 {
    let angle = ((first.x - center.x) as f64).atan2((first.y - center.y) as f64)
        - ((second.x - center.x) as f64).atan2((second.y - center.y) as f64);
    if angle <= 0.0 {
        angle + std::f64::consts::TAU
    } else {
        angle
    }
}

pub(super) fn path_length(path: &ExtrusionPath, geometry: LayerGeometry<'_>) -> f64 {
    path.polyline
        .points
        .windows(2)
        .map(|segment| {
            if path.z_contoured {
                let dx = (segment[1].x - segment[0].x) as f64;
                let dy = (segment[1].y - segment[0].y) as f64;
                let dz = (segment[1].z - segment[0].z) as f64;
                (dx * dx + dy * dy + dz * dz).sqrt() * geometry.scale.factor()
            } else {
                geometry
                    .scale
                    .unscale(segment[1].x - segment[0].x)
                    .hypot(geometry.scale.unscale(segment[1].y - segment[0].y))
            }
        })
        .sum()
}
