//! The wipe/region tail of `MotionOptions` field assembly
//! (`build::fields::assemble`) — grouped under the 400-LOC rule.

use super::super::super::helpers::absolute;
use super::super::context::Prelude;

pub(super) struct WipeTail {
    pub retract_before_wipe: f64,
    pub role_based_wipe_speed: bool,
    pub wipe_speed: f64,
    pub wipe_on_loops: bool,
    pub wipe_before_external_loop: bool,
    pub wall_loops: i32,
    pub nozzle_diameter: f64,
}

pub(super) fn tail(prelude: &Prelude<'_>) -> WipeTail {
    let Prelude {
        full,
        gcode,
        region,
        travel_speed,
        filament,
        physical,
        ..
    } = prelude;
    WipeTail {
        retract_before_wipe: gcode
            .retract_before_wipe
            .0
            .get(*filament)
            .or_else(|| gcode.retract_before_wipe.0.first())
            // `Extruder::retract_before_wipe` (`Extruder.cpp:171`):
            // `value * 0.01`, not `value / 100` — the inexact 0.01
            // shifts the wipe distribution by 1 ulp (a half-way E
            // rounds differently).
            .map_or(0.0, |value| (value.0 * 0.01).clamp(0.0, 1.0)),
        role_based_wipe_speed: full.process.region.role_based_wipe_speed.0,
        wipe_speed: absolute(
            region.map_or(full.process.region.wipe_speed, |value| value.wipe_speed),
            *travel_speed,
        ),
        wipe_on_loops: region.map_or(full.process.region.wipe_on_loops.0, |value| {
            value.wipe_on_loops.0
        }),
        wipe_before_external_loop: region
            .map_or(full.process.region.wipe_before_external_loop.0, |value| {
                value.wipe_before_external_loop.0
            }),
        wall_loops: region.map_or(full.process.region.wall_loops.0, |value| value.wall_loops.0),
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
