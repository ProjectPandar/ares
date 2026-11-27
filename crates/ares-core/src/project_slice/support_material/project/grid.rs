use crate::project_slice::{
    layers::PlannedPrintObject,
    perimeters::{flow::with_flow_ratio, types::Flow},
};
use crate::{FloatOrPercent, ObjectOptions, ProjectSettings, geometry::CoordinateScale};

pub(super) fn dimensions(
    settings: &ProjectSettings,
    object: &ObjectOptions,
    plan: &PlannedPrintObject,
    (interface_flow, bridge_flow_ratio): (Flow, f64),
    scale: CoordinateScale,
) -> (f64, f64, f64) {
    let height_min = settings
        .project
        .print
        .min_layer_height
        .0
        .iter()
        .map(|height| height.0.max(0.01))
        .chain(plan.layers.iter().map(|layer| layer.height.max(0.01)))
        .fold(0.01 / scale.factor(), f64::min);
    let interface_nozzle = super::region::nozzle(
        object.support_interface_filament.0,
        &settings.project.print.nozzle_diameter,
    );
    let extrusion_width = match object.line_width {
        FloatOrPercent::Float(width) => width,
        FloatOrPercent::Percent(percent) => interface_nozzle * percent.0 / 100.0,
    };
    let bottom_interface_flow =
        if plan.slicing_parameters.zero_gap_interface_bottom || !object.thick_bridges.0 {
            with_flow_ratio(interface_flow, bridge_flow_ratio)
        } else {
            Flow::bridging_flow(
                (bridge_flow_ratio * f64::from(interface_flow.nozzle_diameter)) as f32,
                interface_flow.nozzle_diameter,
            )
        };
    (
        height_min,
        extrusion_width,
        f64::from(bottom_interface_flow.height),
    )
}
