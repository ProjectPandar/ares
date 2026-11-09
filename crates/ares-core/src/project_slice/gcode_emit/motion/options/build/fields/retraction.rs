//! Selected-filament getters (`Extruder.cpp:172–235`, GCode FILAMENT_CONFIG).

use super::super::super::MotionOptions;
use super::super::context::Prelude;
use crate::{OrcaFloats, ProjectSettings};

fn at<T>(values: &[T], index: usize) -> Option<&T> {
    values.get(index).or_else(|| values.first())
}

fn float(values: &OrcaFloats, index: usize) -> f64 {
    at(&values.0, index).map_or(0.0, |value| value.0)
}

pub(super) fn apply(options: &mut MotionOptions, prelude: &Prelude<'_>, runtime: &ProjectSettings) {
    let Prelude {
        gcode,
        filament,
        physical,
        retraction_speed,
        deretraction_speed,
        ..
    } = prelude;
    let filament = *filament;
    options.retraction_length = float(&gcode.retraction_length, filament);
    options.retract_length_toolchange = float(&gcode.retract_length_toolchange, filament);
    options.retract_restart_extra_toolchange =
        float(&gcode.retract_restart_extra_toolchange, filament);
    options.retract_restart_extra = float(&gcode.retract_restart_extra, filament);
    options.deretraction_feedrate = deretraction_speed * 60.0;
    options.retraction_feedrate = retraction_speed * 60.0;
    // Wipe::calculateWipeRetractionLengths reads config_index() (GCode.cpp:584).
    options.retraction_during_wipe_speed = float(&gcode.retraction_speed, filament);
    options.z_hop = float(&gcode.z_hop, filament);
    options.retract_lift_above = float(&gcode.retract_lift_above, *physical);
    options.retract_lift_below = float(&gcode.retract_lift_below, *physical);
    options.retract_lift_enforce = at(&gcode.retract_lift_enforce.0, *physical)
        .copied()
        .unwrap_or_default();
    options.z_hop_type = at(&gcode.z_hop_types.0, filament)
        .copied()
        .unwrap_or_default();
    // Extruder::travel_slope alone uses physical extruder_id(), not m_id.
    options.travel_slope_radians =
        float(&gcode.travel_slope, *physical) * std::f64::consts::PI / 180.0;
    // The runtime view has the overrides and the filament map applied (Print.cpp:4730-4749).
    let print = &runtime.project.print;
    options.wipe = at(&print.wipe.0, filament).is_some_and(|value| value.0);
    options.wipe_distance = float(&print.wipe_distance, filament);
    options.retraction_minimum_travel = float(&print.retraction_minimum_travel, filament);
    options.short_travel_distance = float(&print.retraction_minimum_travel, *physical);
    options.retract_when_changing_layer =
        at(&print.retract_when_changing_layer.0, filament).is_some_and(|value| value.0);
}
