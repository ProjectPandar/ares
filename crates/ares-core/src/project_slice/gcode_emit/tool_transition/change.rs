//! Custom change-filament render lifecycle (`GCode.cpp:7954–7980`).

use super::super::{footprint, motion, placeholders, template, timelapse, value};
use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(in crate::project_slice::gcode_emit::tool_transition) mod dynamic;

#[derive(Clone, Copy)]
pub(super) struct Context<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) metadata: GenerationMetadata,
    pub(super) first_layer_bounds: Option<footprint::FirstLayerBounds>,
    pub(super) layer_num: i64,
    pub(super) print_z: f64,
    pub(super) max_z: f64,
    pub(super) temperature_override: Option<i32>,
}

pub(super) fn append(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    filament: usize,
    context: Context<'_>,
) -> Result<String, SliceError> {
    let old = if state.extruder_selected {
        Some(state.extruders.active_id())
    } else {
        state.start_gcode_filament
    };
    state.start_gcode_filament = None;
    let mut config = placeholders::base_config(
        context.traversal,
        context.metadata,
        context.first_layer_bounds,
        Some(&state.template_rng),
        state.total_layer_count,
    )?;
    dynamic::insert(&mut config, state, old, filament, context);
    let mut injected = Vec::new();
    motion::retract_for_custom_toolchange(&mut injected, state);
    let runtime = &context.traversal.resolved.views.runtime_gcode;
    injected.extend_from_slice(runtime.change_filament_gcode.0.as_bytes());
    if injected.is_empty() || (runtime.manual_filament_change.0 && state.toolchange_count == 1) {
        return Ok(String::new());
    }
    motion::refresh_writer_state(&mut config, state);
    if !state.extruder_selected {
        // `GCode.cpp:2856`: the parser retains the initial logical tool
        // until the first set_extruder render has completed.
        config.insert(
            "current_extruder",
            value::Value::Integer(state.initial_filaments.initial() as i64),
        );
    }
    let source = std::str::from_utf8(&injected).expect("G-code commands and templates are UTF-8");
    let name = "change_filament_gcode";
    let rendered = template::render_for_filament(source, &mut config, filament)
        .map_err(|error| error.to_string())
        .and_then(|text| motion::apply_writer_state(name, &config, &text, state).map(|()| text));
    let mut rendered = match rendered {
        Ok(text) => text,
        Err(error) => {
            state.record_template_failure(name, &error);
            format!(
                "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
            )
        }
    };
    if !rendered.is_empty() && !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    output.extend_from_slice(rendered.as_bytes());
    output.extend_from_slice(b";_FORCE_RESUME_FAN_SPEED\n");
    state.positioned = false;
    if let Some(z) = timelapse::last_motion_z(&rendered) {
        state.writer_z = Some(z);
    }
    Ok(rendered)
}

/// `GCodeProcessor::get_gcode_last_filament` (`2932–2965`): only complete
/// newline-terminated, space-trimmed T lines; retain the last valid ID.
pub(in crate::project_slice::gcode_emit) fn last_start_filament(source: &str) -> Option<usize> {
    source
        .split_inclusive('\n')
        .filter_map(|line| {
            let line = line.strip_suffix('\n')?.trim_matches(' ');
            let id = line.strip_prefix('T')?.parse::<usize>().ok()?;
            (id < 255).then_some(id)
        })
        .next_back()
}
