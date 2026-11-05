//! `GCode.cpp:6683–6687,9831–9853`: per-layer farthest-point photo context.

mod target;

use super::{instances::Instance, schedule::LayerToPrint};
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
    farthest_is_photo_head: bool,
    pub(super) inserted_this_layer: bool,
}

pub(super) struct LayerViews<'a> {
    pub(super) entries: &'a [LayerToPrint],
    pub(super) objects: &'a [Vec<OrderedExtrusionLayer>],
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
    let enabled = full.printer.gcode.farthest_point_timelapse.0
        && full.process.print.timelapse_type == crate::ProcessTimelapseType::Traditional
        && traversal.resolved.views.runtime_gcode.printer_structure != crate::PrinterStructure::I3;
    let single_nozzle = full.project.print.nozzle_diameter.0.len() <= 1;
    state.farthest_point = Context {
        enabled,
        ..Context::default()
    };
    if enabled && let Some(candidate) = target::compute(traversal, views, tools) {
        state.farthest_point.farthest_gcode_pos = (
            traversal.scale.unscale(candidate.point.x()),
            traversal.scale.unscale(candidate.point.y()),
        );
        state.farthest_point.farthest_is_photo_head =
            single_nozzle || candidate.physical_extruder == state.most_used_extruder;
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
