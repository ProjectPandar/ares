//! Filament template lifecycle from `GCode::set_extruder` (7759–7777,8018–8043).

use super::{motion, placeholders, template, timelapse, value};
use crate::SliceError;

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    filament: usize,
    context: timelapse::Context<'_>,
) -> Result<(), SliceError> {
    if state.extruders.multiple() && state.extruder_selected {
        append_filament(output, state, state.extruders.active_id(), false, context)?;
    }
    // Higher-level retract/change_filament/temperature remain the next coupled
    // source slices; this consumer installs their end/start lifecycle boundary.
    motion::tool_selection::toolchange(output, state, context.traversal, filament);
    append_filament(output, state, filament, true, context)
}

fn append_filament(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    filament: usize,
    start: bool,
    context: timelapse::Context<'_>,
) -> Result<(), SliceError> {
    let runtime = &context.traversal.resolved.views.runtime_gcode;
    let (name, sources) = if start {
        ("filament_start_gcode", &runtime.filament_start_gcode.0)
    } else {
        ("filament_end_gcode", &runtime.filament_end_gcode.0)
    };
    let source = sources.get(filament).unwrap_or(&sources[0]);
    if source.is_empty() {
        return Ok(());
    }
    let mut config = placeholders::base_config(
        context.traversal,
        context.metadata,
        context.first_layer_bounds,
    )?;
    motion::refresh_writer_state(&mut config, state);
    config.insert(
        "layer_num",
        value::Value::Integer(context.layer.index as i64),
    );
    config.insert(
        "layer_z",
        value::Value::number(state.writer_z.unwrap_or(0.0) - state.options.z_offset),
    );
    config.insert("max_layer_z", value::Value::number(context.layer.max_z));
    let template_filament = if start {
        filament as i64
    } else {
        // The end-template argument is the old physical extruder ID; start
        // uses the new logical filament ID (`GCode.cpp:7770,8027`).
        let map = &context
            .traversal
            .resolved
            .views
            .full
            .project
            .gcode
            .filament_map
            .0;
        map.get(filament).map_or(0, |value| i64::from(value.0 - 1))
    };
    config.insert(
        "filament_extruder_id",
        value::Value::Integer(template_filament),
    );
    let rendered = match template::render_for_filament(source, &mut config, filament) {
        Ok(rendered) => match motion::apply_writer_state(name, &config, state) {
            Ok(()) => Ok(rendered),
            Err(error) => Err(error),
        },
        Err(error) => {
            let error = error.to_string();
            state.record_template_failure(name, &error);
            Err(error)
        }
    };
    match rendered {
        Ok(rendered) => {
            output.extend_from_slice(rendered.as_bytes());
            if !rendered.is_empty() && !rendered.ends_with('\n') {
                output.push(b'\n');
            }
        }
        Err(error) => output.extend_from_slice(
            format!("\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n").as_bytes(),
        ),
    }
    Ok(())
}
