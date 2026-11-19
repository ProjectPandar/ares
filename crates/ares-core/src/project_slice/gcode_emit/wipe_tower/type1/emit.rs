//! `GCode.cpp:712–1117`: Type1 append_tcr; never calls append_tcr2/set_extruder.
use super::super::super::{motion, pressure_advance, timelapse, value::Value};
use super::super::{emit::Append, filament::at, postprocess};
use super::{dynamic, templates, travel};
use crate::SliceError;

pub(in crate::project_slice::gcode_emit::wipe_tower) fn append(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    input: Append<'_>,
) -> Result<(), SliceError> {
    let Append {
        result,
        new_tool,
        z,
        context,
        geometry,
    } = input;
    let metadata = result.type1.as_ref().expect("Type1 result metadata");
    let tool = new_tool.expect("Type1 final purge is not emitted");
    let tower = state.wipe_tower.as_ref().unwrap();
    let angle = if result.priming { 0.0 } else { tower.angle };
    let offset = [
        if result.priming {
            0.0
        } else {
            tower.position[0]
        } + tower.data.rib_offset[0],
        if result.priming {
            0.0
        } else {
            tower.position[1]
        } + tower.data.rib_offset[1],
    ];
    let transform = |point| {
        if result.priming {
            point
        } else {
            postprocess::transform(point, offset, angle)
        }
    };
    let start = transform(result.start);
    let end = transform(result.end);
    let change_start = transform(if metadata.tool_change {
        metadata.change_start
    } else {
        result.start
    });
    let code = postprocess::moves(result, offset, angle, &tower.offsets, tower.semm);
    let settings = &context.traversal.resolved.views.full;
    let nozzle = &metadata.nozzle_change;
    let nozzle_change =
        !nozzle.gcode.is_empty() && settings.project.print.nozzle_diameter.0.len() > 1;
    if !result.priming && metadata.finish_first {
        motion::tower::retract(output, state);
        motion::tower::travel_to(output, state, start, |_| Ok(Vec::new()), geometry)?;
    }
    let current_z = state.writer_z.unwrap_or(0.0);
    let z = z.unwrap_or(current_z);
    let different_z = (z - current_z).abs() >= 1e-4;
    let sparse_skipped = super::super::sparse_layers_skipped(settings);
    let compacted_below_object = sparse_skipped && z >= 0.0 && f64::from(result.print_z) - z > 1e-4;
    let defer_compacted_descend =
        sparse_skipped && !result.priming && !metadata.finish_first && current_z - z > 1e-4;
    if different_z && !defer_compacted_descend {
        motion::tower::change_z(
            output,
            state,
            z,
            "Travel down to the last wipe tower layer.",
        );
    }
    let (mut end_filament, close_mask) = templates::end(state, context, result.print_z)?;
    state.toolchange_count += 1;
    let mut retract = Vec::new();
    motion::tower::type1_retract(&mut retract, state, metadata.tool_change && !nozzle_change);
    motion::append_exclude_end(&mut retract, state);
    if nozzle_change {
        let mut before = Vec::new();
        let compact_intower_nc_travel = compacted_below_object
            && f64::from(result.print_z) - state.writer_z.expect("tower travel positions writer")
                > 1e-4;
        motion::tower::type1_travel(
            &mut before,
            state,
            transform(nozzle.start),
            geometry,
            compact_intower_nc_travel.then_some(z),
        )?;
        if !compact_intower_nc_travel && compacted_below_object {
            motion::tower::writer_z(
                &mut before,
                state,
                z,
                "Descend to compacted wipe tower z (no sparse layers)",
                false,
            );
        }
        motion::tower::type1_unretract(&mut before, state);
        before.extend_from_slice(
            postprocess::nozzle_moves(&nozzle.gcode, nozzle.start, offset, angle).as_bytes(),
        );
        motion::tower::finish(
            state,
            geometry.scale,
            transform(nozzle.end),
            nozzle.wipe_path.iter().copied().map(transform),
        );
        motion::tower::type1_retract(&mut before, state, metadata.tool_change);
        end_filament.insert_str(
            0,
            std::str::from_utf8(&before).expect("writer G-code is UTF-8"),
        );
    }
    end_filament.insert_str(
        0,
        std::str::from_utf8(&retract).expect("writer G-code is UTF-8"),
    );
    let source = &context
        .traversal
        .resolved
        .views
        .runtime_gcode
        .change_filament_gcode
        .0;
    let new_physical = (at(&settings.project.gcode.filament_map.0, tool).0 - 1) as usize;
    let tool_may_travel = !source.is_empty()
        || (state.extruder_selected && state.extruders.active_physical() != new_physical);
    let mut restore_layer_z = Vec::new();
    if compacted_below_object
        && tool_may_travel
        && state.writer_z.expect("tower travel positions writer") < current_z - 1e-4
    {
        motion::tower::writer_z(
            &mut restore_layer_z,
            state,
            current_z,
            "Restore layer Z before toolchange",
            false,
        );
    }
    let mut change = String::new();
    if !source.is_empty() {
        let mut config = templates::config(state, context)?;
        dynamic::insert(
            &mut config,
            state,
            result,
            context,
            (change_start, z, geometry.scale),
        )?;
        change = templates::render(state, "change_filament_gcode", source, tool, &mut config);
        change.push_str(";_FORCE_RESUME_FAN_SPEED\n");
        state.positioned = false;
        if let Some(z) = timelapse::last_motion_z(&change) {
            state.writer_z = Some(z);
        }
    }
    if result.priming || !state.extruder_selected || state.extruders.active_id() != tool {
        let mut command = Vec::new();
        motion::tool_selection::toolchange(&mut command, state, context.traversal, tool, &change);
        change.push_str(std::str::from_utf8(&command).expect("writer G-code is UTF-8"));
    }
    let mut last_travel = Vec::new();
    if !source.is_empty() {
        let path = if settings.process.print.prime_tower_skip_points.0 {
            travel::path(&change, change_start, state, context, geometry.scale)?
        } else {
            vec![motion::tower::object_point_grid(
                change_start,
                state,
                geometry.scale,
            )]
        };
        let last_index = path.len() - 1;
        for (i, point) in path.into_iter().enumerate() {
            let mut moves = Vec::new();
            motion::tower::type1_travel_scaled(&mut moves, state, point, geometry)?;
            if i == last_index {
                last_travel = moves;
            } else {
                change.push_str(std::str::from_utf8(&moves).expect("writer G-code is UTF-8"));
            }
        }
    }
    let mut unretract = Vec::new();
    motion::tower::type1_unretract(&mut unretract, state);
    let mut start_filament = templates::start(state, context, tool, close_mask)?;
    start_filament.push_str(std::str::from_utf8(&last_travel).expect("writer G-code is UTF-8"));
    start_filament.push_str(std::str::from_utf8(&unretract).expect("writer G-code is UTF-8"));
    if compacted_below_object {
        let mut descend = Vec::new();
        motion::tower::writer_z(
            &mut descend,
            state,
            z,
            "Descend to compacted wipe tower z (no sparse layers)",
            false,
        );
        start_filament.push_str(std::str::from_utf8(&descend).expect("writer G-code is UTF-8"));
    }
    let mut config = templates::config(state, context)?;
    config.insert("filament_end_gcode", Value::String(end_filament));
    config.insert(
        "restore_layer_z_before_toolchange",
        Value::String(String::from_utf8(restore_layer_z).expect("writer G-code is UTF-8")),
    );
    config.insert("change_filament_gcode", Value::String(change));
    config.insert("filament_start_gcode", Value::String(start_filament));
    let text = templates::render(state, "tcr_rotated_gcode", &code, tool, &mut config);
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        let ch = if ch == '\\' {
            match chars.next().expect("tower result ends in newline") {
                'r' => '\r',
                'n' => '\n',
                other => other,
            }
        } else {
            ch
        };
        let mut bytes = [0; 4];
        output.extend_from_slice(ch.encode_utf8(&mut bytes).as_bytes());
    }
    pressure_advance::append(output, state, context.traversal, tool);
    motion::tower::set_exit_position(state, geometry.scale, end);
    if different_z {
        motion::tower::change_z(
            output,
            state,
            current_z,
            "Travel back up to the topmost object layer.",
        );
    } else {
        // GCode.cpp:1652-1657 prepares the future wipe only on the same Z.
        motion::tower::prepare_wipe(
            state,
            geometry.scale,
            result.wipe_path.iter().copied().map(transform),
        );
        motion::tower::type1_retract(output, state, false);
    }
    state.use_external_mp_once = true;
    Ok(())
}
