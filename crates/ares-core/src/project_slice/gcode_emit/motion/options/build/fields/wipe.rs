//! The wipe/region tail of `MotionOptions` field assembly
//! (`build::fields::assemble`) — grouped under the 400-LOC rule.

use super::super::context::Prelude;

pub(super) struct WipeTail {
    pub retract_before_wipe: f64,
    pub retract_after_wipe: f64,
    pub nozzle_diameter: f64,
}

pub(super) fn tail(prelude: &Prelude<'_>) -> WipeTail {
    let Prelude {
        full,
        gcode,
        filament,
        physical,
        ..
    } = prelude;
    let retract_before_wipe = gcode
        .retract_before_wipe
        .0
        .get(*filament)
        .or_else(|| gcode.retract_before_wipe.0.first())
        // `Extruder::retract_before_wipe` (`Extruder.cpp:171`):
        // `value * 0.01`, not `value / 100` — the inexact 0.01
        // shifts the wipe distribution by 1 ulp (a half-way E
        // rounds differently).
        .map_or(0.0, |value| (value.0 * 0.01).clamp(0.0, 1.0));
    WipeTail {
        retract_before_wipe,
        retract_after_wipe: gcode
            .retract_after_wipe
            .0
            .get(*filament)
            .or_else(|| gcode.retract_after_wipe.0.first())
            .map_or(0.0, |value| (value.0 * 0.01).clamp(0.0, 1.0))
            .min(1.0 - retract_before_wipe),
        nozzle_diameter: full
            .project
            .print
            .nozzle_diameter
            .0
            .get(*physical)
            .or_else(|| full.project.print.nozzle_diameter.0.first())
            .map_or(0.4, |value| value.0),
    }
}
