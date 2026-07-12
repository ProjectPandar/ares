use crate::{
    HardwareOptions, LayerExtrusionMoves, LayerSpeedMoves, SliceError, SliceOptions,
    gcode_auxiliary_fan::AuxiliaryFanState,
    gcode_power_loss_recovery::PowerLossRecoveryState,
    gcode_writer::GCodeWriter,
    options::{ChamberTemperatureControl, ExhaustFanControl, GCodeFlavor},
};

pub(crate) struct FinishEmitCommand<'a> {
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

pub(crate) fn finish_gcode(command: FinishEmitCommand<'_>) -> Result<String, SliceError> {
    crate::gcode_finish::finish_gcode(crate::gcode_finish::FinishGCodeCommand {
        writer: command.writer,
        options: command.options,
        gcode_flavor: command.gcode_flavor,
        chamber_temperature_control: command.chamber_temperature_control,
        exhaust_fan_control: command.exhaust_fan_control,
        auxiliary_fan_completion_enabled: command.auxiliary_fan_completion_enabled,
        auxiliary_fan_state: command.auxiliary_fan_state,
        layer_extrusion_moves: command.layer_extrusion_moves,
        layer_speed_moves: command.layer_speed_moves,
        hardware_options: command.hardware_options,
        layer_num: command.layer_num,
        layer_z: command.layer_z,
    })
}

pub(crate) fn finish_output(
    gcode_comments: bool,
    power_loss_recovery_state: PowerLossRecoveryState,
    command: FinishEmitCommand<'_>,
) -> Result<String, SliceError> {
    let mut gcode = String::new();
    gcode.push_str(&crate::gcode_power_loss_recovery::finish_command(
        command.gcode_flavor,
        gcode_comments,
        power_loss_recovery_state,
    ));
    gcode.push_str(&crate::gcode_m73::last_progress_line(command.options)?);
    gcode.push_str(&finish_gcode(command)?);
    Ok(gcode)
}
