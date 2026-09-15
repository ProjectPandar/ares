//! FanMover construction gate (`GCode.cpp:3727-3740`).

use crate::GCodeFlavor;
use crate::project_slice::gcode_emit::fan_mover::FanMover;
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) fn gate(traversal: &PreparedPostClassicTraversal) -> Option<FanMover> {
    let gcode = &traversal.resolved.views.full.printer.gcode;
    let speedup_time = gcode.fan_speedup_time.0;
    let kickstart = gcode.fan_kickstart.0;
    (speedup_time != 0.0 || kickstart > 0.0).then(|| {
        let relative_e = gcode.use_relative_e_distances.0;
        let flavor: GCodeFlavor = gcode.gcode_flavor;
        FanMover::new(
            speedup_time,
            kickstart,
            gcode.fan_speedup_overhangs.0,
            relative_e,
            flavor,
        )
    })
}
