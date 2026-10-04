//! `--dump-moves` CSV of `GCodeProcessorResult::moves`, in the column order
//! of the `tools/chain-probe/procgcode.patch` oracle dump.

use ares_core::GCodeProcessorResult;
use std::fmt::Write as _;

const HEADER: &str = "gcode_id,type,role,extruder_id,cp_color_id,x,y,z,delta_extruder,feedrate,\
width,height,mm3_per_mm,travel_dist,fan_speed,temperature,time0,layer_id,internal_only\n";

pub(crate) fn format(result: &GCodeProcessorResult) -> String {
    let mut csv = String::from(HEADER);
    for vertex in &result.moves {
        let [x, y, z] = vertex.position;
        writeln!(
            csv,
            "{},{},{},{},{},{x},{y},{z},{},{},{},{},{},{},{},{},{},{},{}",
            vertex.gcode_id,
            vertex.move_type as u8,
            vertex.extrusion_role as u8,
            vertex.extruder_id,
            vertex.cp_color_id,
            vertex.delta_extruder,
            vertex.feedrate,
            vertex.width,
            vertex.height,
            vertex.mm3_per_mm,
            vertex.travel_dist,
            vertex.fan_speed,
            vertex.temperature,
            vertex.time,
            vertex.layer_id,
            u8::from(vertex.internal_only),
        )
        .expect("writing to a String cannot fail");
    }
    csv
}
