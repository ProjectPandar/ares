//! The local Z frame used by normal and Organic support on a belt.

use std::borrow::Cow;

use crate::{
    options::PrinterRemainingOptions,
    project_slice::{
        belt::{EPSILON, floor_context::BeltFloorContext},
        layers::{PlannedLayer, PlannedPrintObject},
        support_material::install_layers::InstalledSupportLayer,
    },
};

pub(super) struct SupportFrame<'a> {
    pub(super) layers: Cow<'a, [PlannedLayer]>,
    pub(super) floor: Option<BeltFloorContext>,
    pub(super) tilt_slope: [f64; 2],
    offset: f64,
}

impl<'a> SupportFrame<'a> {
    pub(super) fn new(plan: &'a PlannedPrintObject, printer: &PrinterRemainingOptions) -> Self {
        // PrintObject.cpp:5095–5144 installs both normal and Organic layers in global Z.
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
