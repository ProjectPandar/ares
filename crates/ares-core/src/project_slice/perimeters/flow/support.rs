//! Support flow selection (`Flow.cpp:232-274`), shared by raft and normal support.

use super::{absolute, build_nonbridging_from_width, raw, selected_nozzle};
use crate::project_slice::perimeters::types::Flow;
use crate::{FloatOrPercent, ObjectOptions, ProjectSettings, SliceError};

pub(in crate::project_slice) struct SupportFlows {
    pub(in crate::project_slice) first_layer: Flow,
    pub(in crate::project_slice) base: Flow,
    pub(in crate::project_slice) interface: Flow,
}

pub(in crate::project_slice) fn resolve_support_flows(
    settings: &ProjectSettings,
    object: &ObjectOptions,
    first_layer_height: f64,
    layer_height: f64,
) -> Result<SupportFlows, SliceError> {
    let nozzle_diameters = &settings.project.print.nozzle_diameter;
    let nozzle = selected_nozzle(object.support_filament, nozzle_diameters)?;
    let interface_nozzle = selected_nozzle(object.support_interface_filament, nozzle_diameters)?;
    let width = if raw(object.support_line_width) > 0.0 {
        object.support_line_width
    } else {
        object.line_width
    };
    let first_width = if raw(settings.process.print.initial_layer_line_width) > 0.0 {
        settings.process.print.initial_layer_line_width
    } else {
        width
    };
    Ok(SupportFlows {
        first_layer: support_flow(first_width, first_layer_height as f32, nozzle)?,
        base: support_flow(width, layer_height as f32, nozzle)?,
        interface: support_flow(width, layer_height as f32, interface_nozzle)?,
    })
}

fn support_flow(width: FloatOrPercent, height: f32, nozzle: f32) -> Result<Flow, SliceError> {
    // Unlike perimeter auto width, support roles use exactly one nozzle diameter.
    let width = match width {
        FloatOrPercent::Float(value) if value <= 0.0 => nozzle,
        configured => absolute(configured, nozzle),
    };
    build_nonbridging_from_width(width, height, nozzle)
}
