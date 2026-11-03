//! `GCode.cpp:8729-8801`: actual-width selection, pre-move and immediate zero-E wipe.

use super::{Params, WipeInwardSupport, apply, offset_direction};
use crate::geometry::Point;
use crate::project_slice::gcode_emit::motion::{EmitState, LayerGeometry, arc, travel};
use crate::project_slice::perimeters::classic::{
    chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
    materialize::{ExtrusionPath, ExtrusionRole},
};

pub(in crate::project_slice::gcode_emit::motion::loop_paths) fn finish_loop(
    output: &mut Vec<u8>,
    (loop_, paths, clipping, support): (
        &ExtrusionLoop,
        &[ExtrusionPath],
        (ExtrusionLoopRole, usize, f64),
        Option<&WipeInwardSupport>,
    ),
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let destination = super::super::inward::post_loop_destination(paths, clipping, geometry, state);
    let wipe_start = destination
        .unwrap_or_else(|| Point::new(state.last_scaled_position.0, state.last_scaled_position.1));
    let applied = support.is_some_and(|support| {
        replace_wipe_path(
            loop_,
            (&paths[..clipping.1], wipe_start),
            support,
            geometry,
            state,
        )
    });
    if let Some(point) = destination {
        super::super::inward::append_inward_move(output, point, geometry, state);
    }
    if applied {
        travel::wipe_without_retraction(output, state);
    }
}

fn replace_wipe_path(
    loop_: &ExtrusionLoop,
    (paths, wipe_start): (&[ExtrusionPath], Point),
    support: &WipeInwardSupport,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> bool {
    if !state.options.wipe
        || !state.options.wipe_inward
        || state.options.wipe_inward_distance.is_non_positive()
        || !support.has_inner()
        || !paths
            .iter()
            .any(|path| path.role == ExtrusionRole::ExternalPerimeter)
        || state.wipe_path.len() < 2
    {
        return false;
    }
    let width = f64::from(paths[0].width);
    let requested = match state.options.wipe_inward_distance {
        crate::FloatOrPercent::Float(value) => value,
        crate::FloatOrPercent::Percent(value) => width * value.0 / 100.0,
    };
    let factor = geometry.scale.factor();
    let epsilon = super::super::super::path::SOURCE_EPSILON_MM / factor;
    let offset = requested.min(width) / factor;
    if offset <= epsilon {
        return false;
    }
    let first = paths[0].polyline.points[0];
    let params = Params {
        seam_start: Point::new(first.x, first.y),
        seam_end: Point::new(state.last_scaled_position.0, state.last_scaled_position.1),
        wipe_start,
        direction: offset_direction(
            super::super::loop_is_counter_clockwise(&loop_.paths),
            loop_.role == ExtrusionLoopRole::Hole,
        ),
        offset,
        max_length: state.options.wipe_distance / factor,
        support_distance: state.options.nozzle_diameter.max(width) / factor,
        epsilon,
    };
    let mut path = state
        .wipe_path
        .iter()
        .map(|point| {
            Point::new(
                (point.x / factor).round() as i64,
                (point.y / factor).round() as i64,
            )
        })
        .collect();
    if !apply(&mut path, support, params) {
        return false;
    }
    state.wipe_path = path
        .into_iter()
        .map(|point| arc::Point {
            x: geometry.scale.unscale(point.x()),
            y: geometry.scale.unscale(point.y()),
        })
        .collect();
    true
}
