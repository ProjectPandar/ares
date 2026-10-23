//! `GCode.cpp:777–794,1064–1093`: Type1-local template slots and mask lifecycle.
//! The mask is upstream `m_filament_instances_code` object-exclusion state:
//! consume it for M624 at the end slot, and close with M625 at the start slot.
//! This is emitted printer G-code, not comparison filtering or normalization.
use super::super::super::{
    motion, placeholders, template, timelapse,
    value::{Config, Value},
};
use crate::SliceError;

pub(super) fn config(
    state: &motion::EmitState,
    context: timelapse::Context<'_>,
) -> Result<Config, SliceError> {
    let mut config = placeholders::base_config(
        context.traversal,
        context.metadata,
        context.first_layer_bounds,
        Some(&state.template_rng),
        state.total_layer_count,
    )?;
    motion::refresh_writer_state(&mut config, state);
    Ok(config)
}

pub(super) fn render(
    state: &mut motion::EmitState,
    name: &str,
    source: &str,
    tool: usize,
    config: &mut Config,
) -> String {
    let rendered = template::render_for_filament(source, config, tool)
        .map_err(|e| e.to_string())
        .and_then(|text| motion::apply_writer_state(name, config, state).map(|()| text));
    let mut output = match rendered {
        Ok(text) => text,
        Err(error) => {
            state.record_template_failure(name, &error);
            format!(
                "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
            )
        }
    };
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

pub(super) fn end(
    state: &mut motion::EmitState,
    context: timelapse::Context<'_>,
    print_z: f32,
) -> Result<(String, bool), SliceError> {
    if !state.extruder_selected {
        return Ok((String::new(), false));
    }
    let tool = state.extruders.active_id();
    let sources = &context
        .traversal
        .resolved
        .views
        .runtime_gcode
        .filament_end_gcode
        .0;
    let source = sources.get(tool).unwrap_or(&sources[0]);
    if source.is_empty() {
        return Ok((String::new(), false));
    }
    let mut config = config(state, context)?;
    config.insert("layer_num", Value::Integer(context.layer.index as i64));
    config.insert("layer_z", Value::number(f64::from(print_z)));
    config.insert("filament_extruder_id", Value::Integer(tool as i64));
    let mask = state.filament_instances_code.take();
    let mut output = mask.as_ref().map_or_else(String::new, |code| {
        format!(
            "M624 {}\n",
            std::str::from_utf8(code).expect("mask is ASCII")
        )
    });
    output.push_str(&render(
        state,
        "filament_end_gcode",
        source,
        tool,
        &mut config,
    ));
    Ok((output, mask.is_some()))
}

pub(super) fn start(
    state: &mut motion::EmitState,
    context: timelapse::Context<'_>,
    tool: usize,
    close_mask: bool,
) -> Result<String, SliceError> {
    let sources = &context
        .traversal
        .resolved
        .views
        .runtime_gcode
        .filament_start_gcode
        .0;
    let source = sources.get(tool).unwrap_or(&sources[0]);
    if source.is_empty() {
        return Ok(String::new());
    }
    let mut config = config(state, context)?;
    placeholders::refresh_filament_aliases(&mut config, tool);
    config.insert("current_extruder", Value::Integer(tool as i64));
    config.insert("filament_extruder_id", Value::Integer(tool as i64));
    let mut output = render(state, "filament_start_gcode", source, tool, &mut config);
    if close_mask {
        output.push_str("M625\n");
    }
    Ok(output)
}
