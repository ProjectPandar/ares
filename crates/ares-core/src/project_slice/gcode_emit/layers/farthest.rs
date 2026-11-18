//! `GCode.cpp:6683–6687,9831–9853`: per-layer farthest-point photo context.

mod target;
pub(in crate::project_slice::gcode_emit) use target::ObjectTargets;

use super::{instances::Instance, schedule::LayerToPrint};
use crate::geometry::{CoordinateScale, Point};
use crate::project_slice::{
    gcode_emit::{motion::EmitState, timelapse, tool_ordering::LayerTools},
    island_print_order::OrderedExtrusionLayer,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

#[derive(Default)]
pub(in crate::project_slice::gcode_emit) struct Context {
    pub(in crate::project_slice::gcode_emit) enabled: bool,
    farthest_gcode_pos: (f64, f64),
    /// Plate-relative native-scale target (`GCode.cpp:6526–6531`).
    pub(in crate::project_slice::gcode_emit) farthest_point: Option<Point>,
    pub(in crate::project_slice::gcode_emit) farthest_is_photo_head: bool,
    pub(in crate::project_slice::gcode_emit) inserted_this_layer: bool,
}

pub(super) struct LayerViews<'a> {
    pub(super) entries: &'a [LayerToPrint],
    pub(super) object_targets: Option<&'a ObjectTargets>,
    pub(super) support: &'a [Vec<InstalledSupportLayer>],
    pub(super) raft: &'a [Vec<OrderedExtrusionLayer>],
    pub(super) instances: &'a [Vec<Instance>],
}

pub(super) fn begin_layer(
    state: &mut EmitState,
    traversal: &PreparedPostClassicTraversal,
    views: LayerViews<'_>,
    tools: &LayerTools,
) {
    let full = &traversal.resolved.views.full;
    let enabled = views.object_targets.is_some();
    let single_nozzle = full.project.print.nozzle_diameter.0.len() <= 1;
    let native_scale = CoordinateScale::Normal;
    let origin = traversal.project.plate_origin();
    let origin = Point::new(
        native_scale.scaled_delta(origin.0).round() as i64,
        native_scale.scaled_delta(origin.1).round() as i64,
    );
    state.farthest_point = Context {
        enabled,
        // compute_farthest_point resets the global point to zero (:6307).
        farthest_point: enabled.then_some(Point::new(-origin.x(), -origin.y())),
        ..Context::default()
    };
    if let Some(object_targets) = views.object_targets
        && let Some(candidate) = object_targets.select(traversal, views, tools)
    {
        state.farthest_point.farthest_gcode_pos = (
            traversal.scale.unscale(candidate.point.x()),
            traversal.scale.unscale(candidate.point.y()),
        );
        let (x, y) = state.farthest_point.farthest_gcode_pos;
        state.farthest_point.farthest_point = Some(Point::new(
            native_scale.scaled_delta(x).round() as i64 - origin.x(),
            native_scale.scaled_delta(y).round() as i64 - origin.y(),
        ));
        state.farthest_point.farthest_is_photo_head = single_nozzle
            || (full.project.gcode.filament_map.0[candidate.filament].0 - 1) as usize
                == state.most_used_extruder;
    }
}

pub(in crate::project_slice::gcode_emit) fn check_and_insert(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    endpoint: (f64, f64),
    context: timelapse::Context<'_>,
) -> Result<(), crate::SliceError> {
    let photo = &state.farthest_point;
    if !photo.enabled
        || !photo.farthest_is_photo_head
        || photo.inserted_this_layer
        || !state.extruder_selected
        || state.extruders.active_physical() != state.most_used_extruder
    {
        return Ok(());
    }
    let dx = endpoint.0 - photo.farthest_gcode_pos.0;
    let dy = endpoint.1 - photo.farthest_gcode_pos.1;
    if (dx * dx + dy * dy).sqrt() >= 0.5 {
        return Ok(());
    }
    if timelapse::append_inline(output, state, context)? {
        state.farthest_point.inserted_this_layer = true;
    }
    Ok(())
}
