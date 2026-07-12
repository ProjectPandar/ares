use crate::{
    gcode_writer::{ExtrusionAxisMode, GCodeWriter},
    options::{AccelToDecelConfig, GCodeFlavor},
};

pub(crate) fn configured_writer(
    gcode_flavor: GCodeFlavor,
    accel_to_decel_config: AccelToDecelConfig,
    part_cooling_fan_min_pwm: u8,
    use_relative_e_distances: bool,
) -> GCodeWriter {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(gcode_flavor);
    writer.set_accel_to_decel_config(accel_to_decel_config);
    writer.set_part_cooling_fan_min_pwm(part_cooling_fan_min_pwm);
    writer.set_extrusion_axis_mode(if use_relative_e_distances {
        ExtrusionAxisMode::Relative
    } else {
        ExtrusionAxisMode::Absolute
    });
    writer
}
