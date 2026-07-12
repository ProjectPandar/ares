use crate::{
    ExtrusionMove, SliceError, SliceOptions, SpeedMove, ToolpathMoveKind,
    gcode_format::format_decimal,
    gcode_lift::TravelLiftMove,
    gcode_writer::{GCodeWriter, SpiralLiftCommand},
};

pub(crate) struct MoveGCodeCommand<'a> {
    pub(crate) writer: &'a mut GCodeWriter,
    pub(crate) options: &'a SliceOptions,
    pub(crate) role_change_state: &'a mut crate::gcode_role_change::RoleChangeGCodeState,
    pub(crate) pressure_advance_state:
        &'a mut crate::gcode_pressure_advance::PressureAdvanceMoveState,
    pub(crate) spiral_vase_layer_state: &'a mut crate::gcode_spiral_vase::SpiralVaseLayerState,
    pub(crate) extrusion_move: &'a ExtrusionMove,
    pub(crate) speed_move: &'a SpeedMove,
    pub(crate) e_position_offset: f64,
    pub(crate) layer_num: usize,
    pub(crate) layer_z: &'a str,
    pub(crate) speed_comment: Option<&'a str>,
    pub(crate) acceleration_comment: Option<&'a str>,
    pub(crate) jerk_comment: Option<&'a str>,
    pub(crate) travel_comment: Option<&'a str>,
    pub(crate) extrude_comment: Option<&'a str>,
    pub(crate) travel_lift: Option<TravelLiftMove>,
}

pub(crate) fn move_gcode(command: MoveGCodeCommand<'_>) -> Result<(String, f64), SliceError> {
    let point = command.extrusion_move.point();
    let target_e = command
        .extrusion_move
        .e_position()
        .map(|e_position| e_position + command.e_position_offset)
        .unwrap_or_else(|| command.writer.current_e());
    let adjusted_move = command.spiral_vase_layer_state.adjusted_move(
        crate::gcode_spiral_vase::SpiralVaseMoveCommand {
            kind: command.extrusion_move.kind(),
            point,
            current_e: command.writer.current_e(),
            target_e,
        },
    );
    let emitted_point = adjusted_move.point;
    let x = format_decimal(emitted_point.x());
    let y = format_decimal(emitted_point.y());
    let feedrate = format_decimal(command.speed_move.feedrate_mm_min());
    let speed_role_label = crate::print_paths::diagnostic_role_label(
        command.speed_move.role(),
        command.speed_move.extrusion_role(),
    );
    let extrusion_role_label = crate::print_paths::diagnostic_role_label(
        command.extrusion_move.role(),
        command.extrusion_move.extrusion_role(),
    );
    let mut gcode = format!(
        ";SPEED:{}:{}:{},{}:{}\n",
        command.speed_move.kind().as_str(),
        speed_role_label,
        x,
        y,
        feedrate
    );
    gcode.push_str(&command.pressure_advance_state.before_move(
        command.writer,
        command.extrusion_move.kind(),
        command.extrusion_move.role(),
    ));
    gcode.push_str(&command.role_change_state.before_move(
        crate::gcode_role_change::RoleChangeGCodeCommand {
            options: command.options,
            move_kind: command.extrusion_move.kind(),
            role: command.extrusion_move.role(),
            layer_num: command.layer_num,
            layer_z: command.layer_z,
        },
    )?);
    let move_start_e = command.writer.current_e();

    match command.extrusion_move.kind() {
        ToolpathMoveKind::Travel => {
            let acceleration_command = command.writer.set_travel_acceleration_with_comment(
                command.speed_move.acceleration_mm_s2(),
                command.acceleration_comment,
            );
            let jerk_command = command
                .writer
                .set_jerk_xy_with_comment(command.speed_move.jerk_mm_s(), command.jerk_comment);
            let move_command = match command.travel_lift {
                Some(TravelLiftMove::SlopeTop {
                    point: slope_top,
                    z,
                }) => {
                    let mut gcode = command.writer.travel_to_xyz_with_comment(
                        slope_top,
                        z,
                        command.speed_move.feedrate_mm_min(),
                        command.travel_comment,
                    );
                    gcode.push_str(&command.writer.travel_to_xy_with_comment(
                        point,
                        command.speed_move.feedrate_mm_min(),
                        command.travel_comment,
                    ));
                    gcode
                }
                Some(TravelLiftMove::Spiral {
                    start,
                    z_start,
                    z,
                    slope_radians,
                    resolution,
                    target,
                }) => {
                    let mut gcode = command.writer.spiral_lift_with_comment(SpiralLiftCommand {
                        start,
                        z_start,
                        z,
                        slope_radians,
                        resolution,
                        target,
                        feedrate: command.speed_move.feedrate_mm_min(),
                        comment: command.travel_comment.map(|_| "spiral lift Z"),
                    });
                    gcode.push_str(&command.writer.travel_to_xy_with_comment(
                        point,
                        command.speed_move.feedrate_mm_min(),
                        command.travel_comment,
                    ));
                    gcode
                }
                Some(TravelLiftMove::Target { z }) => command.writer.travel_to_xyz_with_comment(
                    point,
                    z,
                    command.speed_move.feedrate_mm_min(),
                    command.travel_comment,
                ),
                None => command.writer.travel_to_xy_with_comment(
                    point,
                    command.speed_move.feedrate_mm_min(),
                    command.travel_comment,
                ),
            };
            gcode.push_str(&format!(
                ";EXTRUSION:travel:{}:{},{}:\n;MOVE:travel:{}:{},{}\n{}{}{}",
                extrusion_role_label,
                x,
                y,
                extrusion_role_label,
                x,
                y,
                acceleration_command,
                jerk_command,
                move_command
            ));
        }
        ToolpathMoveKind::Print => {
            let e_position = adjusted_move.e_position;
            let e = format_decimal(e_position);
            let acceleration_command = command.writer.set_print_acceleration_with_comment(
                command.speed_move.acceleration_mm_s2(),
                command.acceleration_comment,
            );
            let jerk_command = command
                .writer
                .set_jerk_xy_with_comment(command.speed_move.jerk_mm_s(), command.jerk_comment);
            let move_command = crate::gcode_print_move::print_move_command(
                command.writer,
                crate::gcode_print_move::PrintMoveCommand::new(
                    emitted_point,
                    e_position,
                    command.speed_move.feedrate_mm_min(),
                    command.speed_comment,
                    command.extrude_comment,
                ),
            );
            gcode.push_str(&format!(
                ";EXTRUSION:print:{}:{},{}:{}\n;MOVE:print:{}:{},{}\n{}{}{}",
                extrusion_role_label,
                x,
                y,
                e,
                extrusion_role_label,
                x,
                y,
                acceleration_command,
                jerk_command,
                move_command
            ));
        }
    }
    command.spiral_vase_layer_state.observe_transition_out(
        crate::gcode_spiral_vase_transition::TransitionOutMoveCommand {
            kind: command.extrusion_move.kind(),
            role: command.extrusion_move.role(),
            point: emitted_point,
            speed_move: *command.speed_move,
            emitted_e_delta: command.writer.current_e() - move_start_e,
        },
    );

    Ok((gcode, adjusted_move.e_offset_delta))
}
