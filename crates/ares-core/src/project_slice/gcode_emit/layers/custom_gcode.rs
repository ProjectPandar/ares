//! `ProcessLayer::emit_custom_gcode_per_print_z` (`GCode.cpp:4174-4258`),
//! with the color change block `:4203-4231` compiled out (`#if 0`) as
//! upstream ships it.
use super::super::{SliceError, motion, placeholders, template, timelapse};
use crate::gcode_processor::{CustomGCodeItem, CustomGCodeType};

pub(super) fn append(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    item: &CustomGCodeItem,
    context: &timelapse::Context<'_>,
) -> Result<(), SliceError> {
    // `GCodeProcessor::reserved_tag` Pause_Print / Custom_Code
    // (`GCodeProcessor.cpp:57-102`).
    let (pause_tag, custom_tag) = if state.tags.is_bbl() {
        ("; PAUSE_PRINTING\n", "; CUSTOM_GCODE\n")
    } else {
        (";PAUSE_PRINT\n", ";CUSTOM_GCODE\n")
    };
    let gcode = &context.traversal.resolved.views.full.printer.gcode;
    match item.kind {
        // The pause message `M117` is commented out (`:4237-4239`).
        CustomGCodeType::PausePrint => {
            output.extend_from_slice(pause_tag.as_bytes());
            render(
                output,
                state,
                context,
                "machine_pause_gcode",
                &gcode.machine_pause_gcode.0,
            )?;
            output.push(b'\n');
        }
        CustomGCodeType::Template => {
            output.extend_from_slice(custom_tag.as_bytes());
            render(
                output,
                state,
                context,
                "template_custom_gcode",
                &gcode.template_custom_gcode.0,
            )?;
        }
        CustomGCodeType::ColorChange | CustomGCodeType::Custom | CustomGCodeType::Unknown => {
            output.extend_from_slice(custom_tag.as_bytes());
            output.extend_from_slice(item.extra.as_bytes());
        }
        CustomGCodeType::ToolChange => {
            unreachable!("assign_custom_gcodes never stores a ToolChange")
        }
    }
    output.push(b'\n');
    Ok(())
}

/// `GCode::placeholder_parser_process(name, template, current_extruder_id)`
/// with no extra config.
fn render(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    context: &timelapse::Context<'_>,
    name: &str,
    source: &str,
) -> Result<(), SliceError> {
    let mut config = placeholders::base_config(
        context.traversal,
        context.metadata,
        context.first_layer_bounds,
        Some(&state.template_rng),
        state.total_layer_count,
    )?;
    motion::refresh_writer_state(&mut config, state);
    match template::render_for_filament(source, &mut config, state.extruders.active_id()) {
        Ok(rendered) => {
            motion::invalidate_after_custom_gcode(&rendered, state);
            output.extend_from_slice(rendered.as_bytes());
        }
        Err(error) => {
            state.record_template_failure(name, &error);
            output.extend_from_slice(
                format!(
                    "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
                )
                .as_bytes(),
            );
        }
    }
    Ok(())
}
