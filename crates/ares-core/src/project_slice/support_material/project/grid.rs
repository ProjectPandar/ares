use std::borrow::Cow;

use crate::project_slice::{
    belt::{EPSILON, floor_context::BeltFloorContext},
    layers::{PlannedLayer, PlannedPrintObject},
    perimeters::{flow::with_flow_ratio, types::Flow},
    support_material::{
        install_layers::InstalledSupportLayer, toolpath_parameters::ToolpathParameters,
    },
};
use crate::{
    FloatOrPercent, ObjectOptions, ProjectSettings, geometry::CoordinateScale,
    options::PrinterRemainingOptions,
};

pub(super) struct SupportFrame<'a> {
    pub(super) layers: Cow<'a, [PlannedLayer]>,
    pub(super) floor: Option<BeltFloorContext>,
    pub(super) tilt_slope: [f64; 2],
    offset: f64,
}

impl<'a> SupportFrame<'a> {
    pub(super) fn new(plan: &'a PlannedPrintObject, printer: &PrinterRemainingOptions) -> Self {
        // PrintObject.cpp:5095–5127 temporarily runs normal support in local Z.
        let offset = plan.belt_slice.map_or(0.0, |slice| slice.print_z_offset);
        let offset = if offset.abs() > EPSILON { offset } else { 0.0 };
        let mut layers = Cow::Borrowed(plan.layers.as_slice());
        if offset != 0.0 {
            for layer in layers.to_mut() {
                layer.print_z -= offset;
            }
        }
        let floor = plan.belt_slice.and_then(|slice| slice.floor).map(|floor| {
            BeltFloorContext::new(floor, printer.belt_support_floor_offset.0).local(offset)
        });
        let tilt_slope = if plan.belt_slice.is_some() {
            [printer.build_plate_tilt_y.0, printer.build_plate_tilt_x.0]
                .map(|tilt| (std::f64::consts::PI * tilt.clamp(-89.0, 89.0) / 180.0).tan())
        } else {
            [0.0; 2]
        };
        Self {
            layers,
            floor,
            tilt_slope,
            offset,
        }
    }

    pub(super) fn restore(&self, layers: &mut [InstalledSupportLayer]) {
        if self.offset != 0.0 {
            for layer in layers {
                layer.print_z += self.offset;
            }
        }
    }
}

pub(super) fn dimensions(
    settings: &ProjectSettings,
    object: &ObjectOptions,
    plan: &PlannedPrintObject,
    parameters: &ToolpathParameters,
    bridge_flow_ratio: f64,
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
            with_flow_ratio(parameters.interface_flow, bridge_flow_ratio)
        } else {
            Flow::bridging_flow(
                (bridge_flow_ratio * f64::from(parameters.interface_flow.nozzle_diameter)) as f32,
                parameters.interface_flow.nozzle_diameter,
            )
        };
    (
        height_min,
        extrusion_width,
        f64::from(bottom_interface_flow.height),
    )
}
