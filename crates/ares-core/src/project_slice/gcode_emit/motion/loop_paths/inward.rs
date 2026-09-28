//! Loop-end inward moves (`GCode.cpp:6020-6033` "move inwards before travel")
//! and shared loop geometry helpers.

use super::*;

pub(super) fn append_inward_move(
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
    let x = geometry.scale.unscale(point.x()) + state.origin.0 - state.extruder_offset.0;
    let y = geometry.scale.unscale(point.y()) + state.origin.1 - state.extruder_offset.1;
    output.extend_from_slice(format!("G1 X{} Y{}\n", format_axis(x), format_axis(y)).as_bytes());
    state.x = x;
    state.y = y;
    // `GCode::extrude_loop` (GCode.cpp:6032) moves only the writer here;
    // m_last_pos remains the clipped extrusion endpoint for subsequent chaining.
}

pub(super) fn inward_point(
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
        > 0
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
            geometry
                .scale
                .unscale(segment[1].x - segment[0].x)
                .hypot(geometry.scale.unscale(segment[1].y - segment[0].y))
        })
        .sum()
}
