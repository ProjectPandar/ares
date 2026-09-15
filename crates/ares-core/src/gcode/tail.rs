//! The `format_gcode` tail — the after-last-object emission and the
//! finish/stat-placeholder passes.

use crate::SliceError;
use crate::SliceOptions;
use crate::gcode_object_labels::ObjectLabelState;
use crate::gcode_role_fan::RoleFanGCodeState;
use crate::gcode_writer::GCodeWriter;
use crate::options::GCodeFlavor;
use crate::options::HardwareOptions;

pub(super) struct FinishTail<'a> {
    pub(super) gcode: String,
    pub(super) writer: &'a GCodeWriter,
    pub(super) options: &'a SliceOptions,
    pub(super) gcode_comments: bool,
    pub(super) gcode_flavor: GCodeFlavor,
    pub(super) object_label_state: &'a mut ObjectLabelState,
    pub(super) role_fan_state: &'a mut RoleFanGCodeState,
    pub(super) power_loss_recovery_state: crate::gcode_power_loss_recovery::PowerLossRecoveryState,
    pub(super) chamber_temperature_control: crate::options::ChamberTemperatureControl,
    pub(super) exhaust_fan_control: crate::options::ExhaustFanControl,
    pub(super) auxiliary_fan_control: &'a crate::options::auxiliary_fan::AuxiliaryFanControl,
    pub(super) auxiliary_fan_state: crate::gcode_auxiliary_fan::AuxiliaryFanState,
    pub(super) layer_extrusion_moves: &'a [crate::LayerExtrusionMoves],
    pub(super) layer_speed_moves: &'a [crate::LayerSpeedMoves],
    pub(super) hardware_options: &'a HardwareOptions,
    pub(super) last_layer: (usize, String),
}

pub(super) fn finish(tail: FinishTail<'_>) -> Result<String, SliceError> {
    let FinishTail {
        mut gcode,
        writer,
        options,
        gcode_comments,
        gcode_flavor,
        object_label_state,
        role_fan_state,
        power_loss_recovery_state,
        chamber_temperature_control,
        exhaust_fan_control,
        auxiliary_fan_control,
        auxiliary_fan_state,
        layer_extrusion_moves,
        layer_speed_moves,
        hardware_options,
        last_layer,
    } = tail;
    gcode.push_str(object_label_state.after_last_object_move());
    let auxiliary_fan_completion_enabled =
        auxiliary_fan_control.completion_shutdown_speed().is_some();
    gcode.push_str(&role_fan_state.finish(writer));
    gcode.push_str(&crate::gcode_finish::finish_output(
        gcode_comments,
        power_loss_recovery_state,
        crate::gcode_finish::FinishGCodeCommand {
            writer,
            options,
            gcode_flavor,
            chamber_temperature_control,
            exhaust_fan_control,
            auxiliary_fan_completion_enabled,
            auxiliary_fan_state,
            layer_extrusion_moves,
            layer_speed_moves,
            hardware_options,
            layer_num: last_layer.0,
            layer_z: &last_layer.1,
        },
    )?);
    crate::gcode_stat_placeholders::finish(options, gcode, layer_extrusion_moves, layer_speed_moves)
}
