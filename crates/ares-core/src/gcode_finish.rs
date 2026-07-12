use crate::gcode_auxiliary_fan::AuxiliaryFanState;
use crate::gcode_writer::GCodeWriter;
use crate::options::{ChamberTemperatureControl, ExhaustFanControl, GCodeFlavor};
use crate::{HardwareOptions, LayerExtrusionMoves, LayerSpeedMoves, SliceError, SliceOptions};

pub(crate) struct FinishGCodeCommand<'a> {
    pub(crate) writer: &'a GCodeWriter,
    pub(crate) options: &'a SliceOptions,
    pub(crate) gcode_flavor: GCodeFlavor,
    pub(crate) chamber_temperature_control: ChamberTemperatureControl,
    pub(crate) exhaust_fan_control: ExhaustFanControl,
    pub(crate) auxiliary_fan_completion_enabled: bool,
    pub(crate) auxiliary_fan_state: AuxiliaryFanState,
    pub(crate) layer_extrusion_moves: &'a [LayerExtrusionMoves],
    pub(crate) layer_speed_moves: &'a [LayerSpeedMoves],
    pub(crate) hardware_options: &'a HardwareOptions,
    pub(crate) layer_num: usize,
    pub(crate) layer_z: &'a str,
}

pub(crate) fn finish_gcode(command: FinishGCodeCommand<'_>) -> Result<String, SliceError> {
    let mut gcode = String::new();
    if command.gcode_flavor != GCodeFlavor::Klipper
        && command.chamber_temperature_control.temperature().is_some()
    {
        gcode.push_str(&command.writer.set_chamber_temperature(0, false));
    }
    gcode.push_str(&crate::gcode_startup::exhaust_fan_completion_command(
        command.writer,
        command.gcode_flavor,
        command.exhaust_fan_control,
    ));
    gcode.push_str(&crate::gcode_auxiliary_fan::completion_command(
        command.writer,
        command.gcode_flavor,
        command.auxiliary_fan_completion_enabled,
        command.auxiliary_fan_state,
    ));
    gcode.push_str(&crate::gcode_placeholders::filament_end_gcode(
        command.options,
        command.layer_num,
        command.layer_z,
        command.layer_z,
        0,
    )?);
    gcode.push_str(&crate::gcode_placeholders::machine_end_gcode(
        command.options,
        command.layer_num,
        command.layer_z,
        command.layer_z,
        0,
    )?);
    gcode.push_str(&crate::gcode_filament_stats::format_filament_stats(
        command.layer_extrusion_moves,
        command.layer_speed_moves,
        command.hardware_options,
        command.options,
    )?);
    gcode.push_str("M2\n");
    Ok(gcode)
}
