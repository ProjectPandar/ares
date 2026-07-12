// Source: OrcaSlicer/src/libslic3r/GCode.cpp:4669-4686

use crate::gcode_writer::GCodeWriter;
use crate::{SliceError, SliceOptions, options::GCodeFlavor};

pub(crate) fn second_layer_temperature_transition(
    writer: &GCodeWriter,
    gcode_flavor: GCodeFlavor,
    options: &SliceOptions,
    layer_num: usize,
) -> Result<String, SliceError> {
    if layer_num != 2 || gcode_flavor == GCodeFlavor::Klipper {
        return Ok(String::new());
    }

    let first_layer_nozzle = options.first_layer_nozzle_temperature()?.value();
    let other_layer_nozzle = options.other_layer_nozzle_temperature()?.value();
    let first_layer_bed = options.first_layer_bed_temperature()?.value();
    let other_layer_bed = options.other_layer_bed_temperature()?.value();

    let mut gcode = String::new();
    if other_layer_nozzle > 0 && other_layer_nozzle != first_layer_nozzle {
        gcode.push_str(&writer.set_nozzle_temperature(other_layer_nozzle, false, None));
    }
    if other_layer_bed != first_layer_bed {
        gcode.push_str(&writer.set_bed_temperature(other_layer_bed, false));
    }
    Ok(gcode)
}
