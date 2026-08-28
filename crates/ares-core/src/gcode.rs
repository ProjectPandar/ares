mod context;

use crate::gcode_layer_change_retraction::{
    LayerChangeLiftCommand, LayerChangeLiftState, LayerChangeResumeCommand,
    LayerChangeRetractCommand, layer_change_resume_before_print, layer_change_retract_gcode,
};
use crate::gcode_layer_diagnostics::{LayerDiagnosticCommand, layer_diagnostics};
use crate::gcode_move_buffer::{BufferedMove, flush};
use crate::gcode_object_labels::{ObjectLabelConfig, ObjectLabelState};
use crate::gcode_pressure_advance::{PressureAdvanceMoveState, startup_command};
use crate::gcode_travel_retraction::{
    TravelRetractionCommand, TravelRetractionState, TravelUnretractCommand,
};
use crate::gcode_wipe_before_external_loop::WipeBeforeExternalLoop;
use crate::gcode_wipe_on_loops::{WipeOnLoops, WipeOnLoopsCommand};
use crate::{PrintPathRole, SliceError, SliceOptions, SlicingPipeline, ToolpathMoveKind};
use crate::{gcode_format::format_decimal, gcode_layer_custom::after_z_gcode};
pub(crate) fn format_gcode(
    pipeline: &SlicingPipeline,
    options: &SliceOptions,
) -> Result<Vec<u8>, SliceError> {
    let layers = pipeline.layers();
    let layer_slices = pipeline.layer_slices();
    let layer_contours = pipeline.layer_contours();
    let layer_perimeters = pipeline.layer_perimeters();
    let layer_infills = pipeline.layer_infills();
    let layer_skirts = pipeline.layer_skirts();
    let layer_brims = pipeline.layer_brims();
    let layer_print_paths = pipeline.layer_print_paths();
    let layer_toolpath_moves = pipeline.layer_toolpath_moves();
    let layer_extrusion_moves = pipeline.layer_extrusion_moves();
    let layer_speed_moves = pipeline.layer_speed_moves();
    let layer_height = options.layer_height()?;
    let initial_layer_height = options.initial_layer_print_height()?;
    let hardware_options = options.hardware_options()?;
    let speed_options = options.speed_options()?;
    let gcode_comments = options.gcode_comments()?;
    let wipe_before_external_loop = options.wipe_before_external_loop()?;
    let wipe_on_loops = options.wipe_on_loops()?;
    let perimeter_options = options.perimeter_options()?;
    let gcode_flavor = options.gcode_flavor()?;
    let accel_to_decel_config = options.accel_to_decel_config()?;
    let object_label_config = ObjectLabelConfig::from_options(options, gcode_flavor)?;
    let chamber_temperature_control = options.chamber_temperature_control()?;
    let exhaust_fan_control = options.exhaust_fan_control()?;
    let auxiliary_fan_control = options.auxiliary_fan_control()?;
    let part_cooling_fan_ramp = options.part_cooling_fan_ramp()?;
    let part_cooling_fan_min_pwm = options.part_cooling_fan_min_pwm()?;
    let fan_speedup_control = options.fan_speedup_control()?;
    let role_fan_control = options.role_fan_control()?;
    let first_layer_bed_temperature = options.first_layer_bed_temperature()?;
    let machine_limits = options.machine_limits()?;
    let input_shaping = options.input_shaping_config()?;
    let mut spiral_vase = crate::gcode_spiral_vase::SpiralVaseRunState::from_options(options)?;
    options.filament_config_exports()?;
    crate::gcode_runtime_options::consume(options)?;
    let power_loss_recovery_mode = options.power_loss_recovery_mode()?;
    let z_offset = options.z_offset()?;
    let layer_change_retraction = options.layer_change_retraction()?;
    let sparse_infill_density_positive = options.effective_sparse_infill_density_percent()? > 0.0;
    let first_nozzle_diameter = hardware_options.nozzle_diameters()[0];
    let comments = context::MoveComments::new(gcode_comments);
    let mut writer = crate::gcode_writer_setup::configured_writer(
        gcode_flavor,
        accel_to_decel_config,
        part_cooling_fan_min_pwm,
        options.use_relative_e_distances()?,
    );
    let max_print_z = context::max_print_z(layers);
    let mut gcode =
        crate::gcode_file_start::file_start(crate::gcode_file_start::FileStartCommand {
            writer: &mut writer,
            pipeline,
            options,
            gcode_flavor,
            first_layer_bed_temperature,
            chamber_temperature_control,
            exhaust_fan_control,
            hardware_options: &hardware_options,
            layer_height,
            initial_layer_height,
            machine_limits,
            input_shaping,
            max_print_z,
        })?;
    gcode.push_str(&startup_command(&writer, options)?);
    let mut pressure_advance_move_state = PressureAdvanceMoveState::from_options(options)?;
    gcode.push_str(&object_label_config.object_definition(layer_print_paths));
    gcode.push_str(&crate::gcode_m73::first_progress_line(
        options,
        layer_speed_moves,
    )?);
    let mut role_fan_state = crate::gcode_role_fan::RoleFanGCodeState::new(
        Some(0),
        part_cooling_fan_ramp.fan_kickstart_s(),
        fan_speedup_control,
    );
    let mut auxiliary_fan_state = crate::gcode_auxiliary_fan::AuxiliaryFanState::new();
    let mut power_loss_recovery_state =
        crate::gcode_power_loss_recovery::PowerLossRecoveryState::new();
    let mut role_change_state = crate::gcode_role_change::RoleChangeGCodeState::new();
    let mut object_label_state = ObjectLabelState::new(object_label_config);
    let mut last_layer = (0, String::new());
    let mut pending_layer_change_unretract = false;
    let mut layer_change_lift_state = LayerChangeLiftState::new();
    let mut travel_retraction_state = TravelRetractionState::new(
        layer_change_retraction.z_hop_lift,
        layer_change_retraction.resolution,
    );
    let mut last_non_gap_fill_print_role = None;
    let mut e_position_offset = 0.0;
    for (layer_index, layer) in layers.iter().enumerate() {
        let layer_slice = &layer_slices[layer_index];
        let layer_contours = &layer_contours[layer_index];
        let layer_perimeters = &layer_perimeters[layer_index];
        let layer_infills = &layer_infills[layer_index];
        let layer_skirts = &layer_skirts[layer_index];
        let layer_brims = &layer_brims[layer_index];
        let layer_print_paths = &layer_print_paths[layer_index];
        let layer_toolpath_moves = &layer_toolpath_moves[layer_index];
        let layer_extrusion_moves = &layer_extrusion_moves[layer_index];
        let final_layer = layer_index + 1 == layers.len();
        let mut spiral_vase_layer_state =
            spiral_vase.layer_state(layer_index, final_layer, layer_extrusion_moves);
        let layer_speed_moves = &layer_speed_moves[layer_index];
        let z_output = layer.print_z() + z_offset;
        let z = format_decimal(layer.print_z());
        let layer_num = layer_index + 1;
        last_layer = (layer_num, z.clone());
        let z_feedrate = speed_options.z_travel_feedrate_for_layer(layer.id() == 0);
        gcode.push_str(&format!(";LAYER_CHANGE\n;LAYER:{}\n;Z:{}\n", layer.id(), z));
        gcode.push_str(
            &crate::gcode_temperature_transition::second_layer_temperature_transition(
                &writer,
                gcode_flavor,
                options,
                layer_num,
            )?,
        );
        gcode.push_str(&crate::gcode_layer_custom::before_layer_change_gcode(
            options, layer_num, &z,
        )?);
        let will_change_layer_with_retraction = layer_index > 0
            && layer_change_retraction.is_enabled()
            && !travel_retraction_state.pending_unretract()
            && (writer.current_position().2 - z_output).abs() > f64::EPSILON;
        let layer_change_z_hop = if will_change_layer_with_retraction {
            layer_change_retraction.z_hop_for_layer_change(
                writer.current_position().2,
                layer_index == 1,
                last_non_gap_fill_print_role,
            )
        } else {
            0.0
        };
        if will_change_layer_with_retraction {
            gcode.push_str(&layer_change_retract_gcode(
                &mut writer,
                LayerChangeRetractCommand {
                    use_firmware: layer_change_retraction.use_firmware,
                    length: layer_change_retraction.length,
                    feedrate: layer_change_retraction.retract_feedrate,
                },
                comments.retract,
            ));
            pending_layer_change_unretract = true;
        }
        gcode.push_str(&writer.travel_to_z_with_comment(z_output, z_feedrate, comments.z_travel));
        travel_retraction_state.clear_z_restore_after_layer_z_move();
        gcode.push_str(
            &layer_change_lift_state.schedule_lift(LayerChangeLiftCommand {
                writer: &mut writer,
                z_hop: layer_change_z_hop,
                z_hop_lift: layer_change_retraction.z_hop_lift,
                resolution: layer_change_retraction.resolution,
                feedrate: z_feedrate,
                comment: comments.z_lift,
            }),
        );
        gcode.push_str(&after_z_gcode(options, layer_num, &z)?);
        gcode.push_str(&crate::gcode_power_loss_recovery::layer_command(
            gcode_flavor,
            power_loss_recovery_mode,
            layer_index,
            gcode_comments,
            &mut power_loss_recovery_state,
        ));
        crate::gcode_layer_markers::push(
            &mut gcode,
            &writer,
            options,
            (layer_index, layer_num, &z),
        )?;
        let layer_part_cooling_fan_speed = crate::gcode_layer_fan::baseline_speed(
            part_cooling_fan_ramp,
            layer_index,
            layer_speed_moves,
        );
        if let Some(speed) = layer_part_cooling_fan_speed {
            gcode.push_str(&role_fan_state.layer_baseline_command(&writer, speed));
        }
        let layer_role_fan_control = role_fan_control.for_layer(
            part_cooling_fan_ramp,
            layer_index,
            layer_part_cooling_fan_speed,
        );
        gcode.push_str(&crate::gcode_auxiliary_fan::layer_command(
            &writer,
            gcode_flavor,
            auxiliary_fan_control.speed_for_layer(layer_index),
            &mut auxiliary_fan_state,
        ));
        gcode.push_str(&layer_diagnostics(LayerDiagnosticCommand {
            layer_slice,
            layer_contours,
            layer_perimeters,
            layer_infills,
            layer_skirts,
            layer_brims,
            layer_print_paths,
            layer_toolpath_moves,
            layer_extrusion_moves,
            layer_speed_moves,
        }));
        let mut buffered_move = None;
        let external_loop_wipe = WipeBeforeExternalLoop::new(
            wipe_before_external_loop,
            layer_print_paths,
            layer_toolpath_moves.moves(),
            gcode_comments.then_some("wipe before external loop"),
        );
        let loop_end_wipe = WipeOnLoops::new(WipeOnLoopsCommand {
            enabled: wipe_on_loops,
            wall_loops: perimeter_options.wall_loops(),
            nozzle_diameter: first_nozzle_diameter,
            layer_print_paths,
            toolpath_moves: layer_toolpath_moves.moves(),
            comment: gcode_comments.then_some("move inwards before travel"),
        });
        for (move_index, ((extrusion_move, _), speed_move)) in layer_extrusion_moves
            .moves()
            .iter()
            .zip(layer_toolpath_moves.moves().iter())
            .zip(layer_speed_moves.moves().iter())
            .enumerate()
        {
            gcode.push_str(object_label_state.before_first_object_move());
            let fan_move_command = crate::gcode_role_fan::RoleFanMoveCommand {
                writer: &writer,
                role_fan_control: layer_role_fan_control,
                baseline_speed: layer_part_cooling_fan_speed,
                move_kind: extrusion_move.kind(),
                role: extrusion_move.role(),
            };
            let move_fan_before_buffered = buffered_move.is_some()
                && role_fan_state.can_speedup_before_move(&fan_move_command);
            let mut move_output = String::new();
            if move_fan_before_buffered {
                let fan_command = role_fan_state.before_move(fan_move_command);
                debug_assert!(fan_command.speedup_eligible);
                gcode.push_str(&fan_command.gcode);
            }
            flush(&mut gcode, &writer, &mut role_fan_state, &mut buffered_move);
            if !move_fan_before_buffered {
                let fan_command = role_fan_state.before_move(fan_move_command);
                move_output.push_str(&fan_command.gcode);
            }
            move_output.push_str(&travel_retraction_state.retract_before_travel(
                TravelRetractionCommand {
                    z_hop: layer_change_retraction.z_hop_for_z(writer.current_position().2),
                    writer: &mut writer,
                    use_firmware: layer_change_retraction.use_firmware,
                    length: layer_change_retraction.length,
                    retract_feedrate: layer_change_retraction.retract_feedrate,
                    minimum_travel: layer_change_retraction.minimum_travel,
                    lift_enforce: layer_change_retraction.lift_enforce,
                    current_layer_is_first: layer_index == 0,
                    previous_non_gap_fill_role: last_non_gap_fill_print_role,
                    kind: extrusion_move.kind(),
                    role: extrusion_move.role(),
                    target: extrusion_move.point(),
                    pending_layer_change_unretract,
                    travel_retraction_enabled: layer_change_retraction.travel_retraction_enabled(),
                    reduce_infill_retraction: layer_change_retraction.reduce_infill_retraction,
                    sparse_infill_density_positive,
                    wipe: layer_change_retraction.wipe,
                    wipe_distance: layer_change_retraction.wipe_distance,
                    retract_before_wipe: layer_change_retraction.retract_before_wipe,
                    role_based_wipe_speed: layer_change_retraction.role_based_wipe_speed,
                    wipe_feedrate: layer_change_retraction.wipe_feedrate,
                    z_feedrate,
                    retract_comment: comments.retract,
                    z_lift_comment: comments.z_lift,
                },
            ));
            let (resume_gcode, e_offset_delta) =
                layer_change_resume_before_print(LayerChangeResumeCommand {
                    writer: &mut writer,
                    lift_state: &mut layer_change_lift_state,
                    pending_unretract: &mut pending_layer_change_unretract,
                    use_firmware: layer_change_retraction.use_firmware,
                    length: layer_change_retraction.length,
                    unretract_length: layer_change_retraction.unretract_length,
                    unretract_feedrate: layer_change_retraction.unretract_feedrate,
                    kind: extrusion_move.kind(),
                    z_feedrate,
                    lift_comment: comments.z_lift,
                    restore_comment: comments.z_restore,
                    unretract_comment: comments.unretract,
                });
            move_output.push_str(&resume_gcode);
            e_position_offset += e_offset_delta;
            let (travel_unretract_gcode, travel_e_offset_delta) = travel_retraction_state
                .unretract_before_print(TravelUnretractCommand {
                    writer: &mut writer,
                    use_firmware: layer_change_retraction.use_firmware,
                    length: layer_change_retraction.length,
                    unretract_length: layer_change_retraction.unretract_length,
                    unretract_feedrate: layer_change_retraction.unretract_feedrate,
                    kind: extrusion_move.kind(),
                    z_feedrate,
                    z_restore_comment: comments.z_restore,
                    unretract_comment: comments.unretract,
                });
            move_output.push_str(&travel_unretract_gcode);
            e_position_offset += travel_e_offset_delta;
            let move_start =
                crate::Point2::new(writer.current_position().0, writer.current_position().1);
            let travel_lift = layer_change_lift_state
                .consume_travel_lift(&writer, extrusion_move.kind(), extrusion_move.point())
                .or_else(|| travel_retraction_state.consume_travel_lift());
            move_output.push_str(&external_loop_wipe.gcode(&mut writer, move_index, speed_move));
            let (emitted_move_gcode, spiral_e_offset_delta) =
                crate::gcode_move_emit::move_gcode(crate::gcode_move_emit::MoveGCodeCommand {
                    writer: &mut writer,
                    options,
                    role_change_state: &mut role_change_state,
                    pressure_advance_state: &mut pressure_advance_move_state,
                    spiral_vase_layer_state: &mut spiral_vase_layer_state,
                    extrusion_move,
                    speed_move,
                    e_position_offset,
                    layer_num,
                    layer_z: &z,
                    speed_comment: comments.speed,
                    acceleration_comment: comments.acceleration,
                    jerk_comment: comments.jerk,
                    travel_comment: comments.travel,
                    extrude_comment: comments.extrude,
                    travel_lift,
                })?;
            move_output.push_str(&emitted_move_gcode);
            e_position_offset += spiral_e_offset_delta;
            let emitted_end =
                crate::Point2::new(writer.current_position().0, writer.current_position().1);
            move_output.push_str(&loop_end_wipe.gcode(&mut writer, move_index, speed_move));
            if extrusion_move.kind() == ToolpathMoveKind::Print
                && extrusion_move.role() != PrintPathRole::GapFill
            {
                last_non_gap_fill_print_role = Some(extrusion_move.role());
            }
            #[rustfmt::skip]
            travel_retraction_state.observe_completed_move(extrusion_move.kind(), move_start, emitted_end, speed_move.feedrate_mm_min());
            buffered_move = Some(BufferedMove::new(move_output, *speed_move));
        }
        flush(&mut gcode, &writer, &mut role_fan_state, &mut buffered_move);
        #[rustfmt::skip]
        gcode.push_str(&spiral_vase.finish_layer(spiral_vase_layer_state, &mut writer, gcode_comments));
    }
    gcode.push_str(object_label_state.after_last_object_move());
    let auxiliary_fan_completion_enabled =
        auxiliary_fan_control.completion_shutdown_speed().is_some();
    gcode.push_str(&role_fan_state.finish(&writer));
    gcode.push_str(&crate::gcode_finish::finish_output(
        gcode_comments,
        power_loss_recovery_state,
        crate::gcode_finish::FinishGCodeCommand {
            writer: &writer,
            options,
            gcode_flavor,
            chamber_temperature_control,
            exhaust_fan_control,
            auxiliary_fan_completion_enabled,
            auxiliary_fan_state,
            layer_extrusion_moves,
            layer_speed_moves,
            hardware_options: &hardware_options,
            layer_num: last_layer.0,
            layer_z: &last_layer.1,
        },
    )?);
    let gcode = crate::gcode_stat_placeholders::finish(
        options,
        gcode,
        layer_extrusion_moves,
        layer_speed_moves,
    )?;
    Ok(gcode.into_bytes())
}
