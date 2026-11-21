//! `WipeTower.cpp:1489-1551,2921-2932,4513-4525`: writer acceleration state.
use super::Writer;
use crate::{FloatOrPercent, GCodeFlavor, ProjectSettings};
use std::fmt::Write;

impl Writer {
    pub(in super::super) fn configure_accelerations(
        &mut self,
        settings: &ProjectSettings,
        first_layer: bool,
    ) {
        let object = &settings.process.object;
        let gcode = &settings.process.gcode;
        self.accelerations = (0..settings.filament.gcode.filament_colour.0.len())
            .map(|tool| {
                let slot = crate::options::mapped_extruder_id(settings, tool);
                let mut travel = object.travel_acceleration.get_at(slot);
                let normal = if first_layer {
                    travel = match gcode.initial_layer_travel_acceleration.get_at(slot) {
                        FloatOrPercent::Float(value) => value,
                        FloatOrPercent::Percent(value) => travel * value.0 / 100.0,
                    };
                    object.initial_layer_acceleration.get_at(slot)
                } else {
                    object.default_acceleration.get_at(slot)
                };
                [normal, travel].map(|value| (value + 0.5).floor() as u32)
            })
            .collect();
        self.max_acceleration = settings
            .printer
            .machine
            .machine_max_acceleration_extruding
            .0[0]
            .0 as u32;
        self.flavor = settings.printer.gcode.gcode_flavor;
        self.accel_to_decel = gcode.accel_to_decel_enable.0;
        self.accel_to_decel_factor = gcode.accel_to_decel_factor.0 as f32;
    }

    pub(super) fn set_acceleration(&mut self, travel: bool) {
        if self.accelerations.is_empty() {
            return;
        }
        let mut acceleration = self.accelerations[self.tool][usize::from(travel)];
        if self.max_acceleration > 0 {
            acceleration = acceleration.min(self.max_acceleration);
        }
        if acceleration == 0 || acceleration == self.last_acceleration {
            return;
        }
        self.last_acceleration = acceleration;
        match self.flavor {
            GCodeFlavor::Repetier => {
                writeln!(self.gcode, "M201 X{acceleration} Y{acceleration}").unwrap();
                writeln!(self.gcode, "M202 X{acceleration} Y{acceleration}").unwrap();
            }
            GCodeFlavor::RepRapFirmware | GCodeFlavor::MarlinFirmware => {
                writeln!(self.gcode, "M204 P{acceleration}").unwrap();
            }
            GCodeFlavor::Klipper if self.accel_to_decel => {
                let mut command = format!("SET_VELOCITY_LIMIT ACCEL={acceleration}");
                crate::project_slice::gcode_emit::klipper_acceleration::append_klipper_accel_to_decel(
                    &mut command,
                    acceleration,
                    f64::from(self.accel_to_decel_factor),
                );
                writeln!(self.gcode, "{command}").unwrap();
            }
            _ => writeln!(self.gcode, "M204 S{acceleration}").unwrap(),
        }
    }
}
