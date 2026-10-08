//! Print.cpp:3777-3797 Print::object_skirt_offset and :2028 skirt_flow.

use crate::{
    ProcessDraftShield, ProcessSkirtType, SliceError,
    project::effective_config::types::BoundedResolvedProjectConfig,
    project_slice::skirt_flow_width_spacing,
};

pub(super) fn offset(
    resolved: &BoundedResolvedProjectConfig,
    short: bool,
) -> Result<f32, SliceError> {
    let settings = &resolved.views.full;
    let print = &settings.process.print;
    if print.skirt_loops.0 == 0 || print.skirt_type != ProcessSkirtType::PerObject {
        return Ok(0.0);
    }
    let first = &resolved.objects[0];
    let nozzle_index = usize::try_from(first.object.support_filament.0 - 1).unwrap_or(usize::MAX);
    let nozzles = &settings.project.print.nozzle_diameter.0;
    let nozzle = nozzles.get(nozzle_index).unwrap_or(&nozzles[0]).0 as f32;
    let width = if print.initial_layer_line_width.is_non_positive() {
        first.object.line_width
    } else {
        print.initial_layer_line_width
    };
    let (width, spacing) =
        skirt_flow_width_spacing(width, print.initial_layer_print_height.0 as f32, nozzle)?;
    let skirt_width = width + (print.skirt_loops.0 - 1) as f32 * spacing;
    let max_nozzle = nozzles.iter().map(|n| n.0).fold(0.0_f64, f64::max);
    let max_layer = settings
        .project
        .print
        .max_layer_height
        .0
        .iter()
        .map(|h| h.0)
        .fold(0.0_f64, f64::max);
    let line_width = match print.initial_layer_line_width {
        crate::FloatOrPercent::Float(value) => value,
        crate::FloatOrPercent::Percent(value) => value.0 * max_nozzle * 0.01,
    };
    let distance = print.skirt_distance.0;
    let radius = settings.printer.remaining.extruder_clearance_radius.0;
    let value = if short {
        distance + f64::from(skirt_width)
    } else if print.draft_shield == ProcessDraftShield::Enabled
        || f64::from(print.skirt_height.0) * max_layer > settings.printer.remaining.nozzle_height.0
    {
        distance + line_width
    } else if distance + f64::from(skirt_width) > radius * 0.5 {
        distance + f64::from(skirt_width) - radius * 0.5
    } else {
        0.0
    };
    Ok(value as f32)
}
