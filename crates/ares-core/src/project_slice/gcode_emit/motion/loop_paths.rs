use super::{EmitState, LayerGeometry, format::axis as format_axis};
use crate::project_slice::perimeters::classic::{
    chained_loops::ExtrusionLoopRole,
    materialize::{ExtrusionPath, ExtrusionRole, Point3},
};

pub(super) fn emit(
    output: &mut Vec<u8>,
    paths: &[ExtrusionPath],
    loop_role: ExtrusionLoopRole,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let mut remaining_clip = state.options.seam_gap;
    let mut path_count = paths.len();
    while path_count > 0 {
        let length = path_length(&paths[path_count - 1], geometry);
        if length > remaining_clip {
            break;
        }
        path_count -= 1;
        remaining_clip -= length;
    }
    let mut emitted_loop_path = Vec::new();
    for (index, path) in paths[..path_count].iter().enumerate() {
        let end_clip = if index + 1 == path_count {
            remaining_clip
        } else {
            0.0
        };
        super::emit_materialized_path(output, path, end_clip, geometry, state);
        if emitted_loop_path.is_empty() {
            emitted_loop_path.extend(state.wipe_path.iter().rev().copied());
        } else {
            emitted_loop_path.extend(state.wipe_path.iter().rev().copied().skip(1));
        }
    }
    state.wipe_path = emitted_loop_path;
    append_inward_move(output, paths, loop_role, geometry, state);
}

fn append_inward_move(
    output: &mut Vec<u8>,
    paths: &[ExtrusionPath],
    loop_role: ExtrusionLoopRole,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let Some(first) = paths.first() else {
        return;
    };
    let last = paths.last().unwrap();
    if !state.options.wipe_on_loops
        || last.role != ExtrusionRole::ExternalPerimeter
        || state.options.wall_loops <= 1
        || first.polyline.points.len() < 2
        || last.polyline.points.len() < 3
    {
        return;
    }

    let point = inward_point(
        first.polyline.points[0],
        first.polyline.points[1],
        last.polyline.points[last.polyline.points.len() - 3],
        (
            loop_role == ExtrusionLoopRole::Hole,
            loop_is_counter_clockwise(paths),
        ),
        state.options.nozzle_diameter / geometry.scale.factor(),
    );
    let x = geometry.scale.unscale(point.x()) + state.offset.0;
    let y = geometry.scale.unscale(point.y()) + state.offset.1;
    output.extend_from_slice(format!("G1 X{} Y{}\n", format_axis(x), format_axis(y)).as_bytes());
    state.x = x;
    state.y = y;
    state.last_scaled_position = Some((point.x(), point.y()));
}

fn inward_point(
    center: Point3,
    mut first: Point3,
    mut second: Point3,
    orientation: (bool, bool),
    nozzle: f64,
) -> crate::geometry::Point {
    let (is_hole, counter_clockwise) = orientation;
    if is_hole == counter_clockwise {
        std::mem::swap(&mut first, &mut second);
    }
    let mut angle = ccw_angle(center, first, second) / 3.0;
    if is_hole == counter_clockwise {
        angle = -angle;
    }
    let dx = (first.x - center.x) as f64;
    let dy = (first.y - center.y) as f64;
    let length_squared = dx * dx + dy * dy;
    let factor = if nozzle * nozzle < length_squared {
        0.2 * nozzle / length_squared.sqrt()
    } else {
        0.2
    };
    let x = center.x as f64 + dx * factor;
    let y = center.y as f64 + dy * factor;
    let (sine, cosine) = angle.sin_cos();
    let rotated_x = center.x as f64 + cosine * (x - center.x as f64) - sine * (y - center.y as f64);
    let rotated_y = center.y as f64 + sine * (x - center.x as f64) + cosine * (y - center.y as f64);
    crate::geometry::Point::new(
        (rotated_x + 0.5).floor() as i64,
        (rotated_y + 0.5).floor() as i64,
    )
}

fn loop_is_counter_clockwise(paths: &[ExtrusionPath]) -> bool {
    let points = paths
        .iter()
        .flat_map(|path| path.polyline.points.iter())
        .collect::<Vec<_>>();
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(left, right)| left.x as i128 * right.y as i128 - right.x as i128 * left.y as i128)
        .sum::<i128>()
        > 0
}

fn ccw_angle(center: Point3, first: Point3, second: Point3) -> f64 {
    let angle = ((first.x - center.x) as f64).atan2((first.y - center.y) as f64)
        - ((second.x - center.x) as f64).atan2((second.y - center.y) as f64);
    if angle <= 0.0 {
        angle + std::f64::consts::TAU
    } else {
        angle
    }
}

fn path_length(path: &ExtrusionPath, geometry: LayerGeometry<'_>) -> f64 {
    path.polyline
        .points
        .windows(2)
        .map(|segment| {
            geometry
                .scale
                .unscale(segment[1].x - segment[0].x)
                .hypot(geometry.scale.unscale(segment[1].y - segment[0].y))
        })
        .sum()
}
