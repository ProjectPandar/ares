//! Once-per-layer wrapping-detection retract and template insert.
//!
//! `GCode.cpp:5281-5284` / `5302-5305`: when `enable_wrapping_detection`
//! is on, the per-extruder loop emits `retract(false, false,
//! auto_lift_type, true)` + `insert_wrapping_detection_gcode()` before
//! the toolchange and the instance markers — once per layer, before the
//! layer's first object block. The retract is not gated on the template
//! being non-empty or on `wrapping_detection_layers` (the layer-count
//! gate lives only in the wipe-tower path, `WipeTower.cpp:2577`).
//!
//! `insert_wrapping_detection_gcode` (`GCode.cpp:5215-5236`) renders the
//! `wrapping_detection_gcode` printer template with layer placeholders,
//! appends a newline when non-empty, always marks the writer's current
//! position as not-clear (`set_current_position_clear(false)`), and
//! adopts the template's last Z move as the writer position when present
//! (`get_last_z_from_gcode`).

use crate::SliceError;

use super::super::{motion, placeholders, template, timelapse, value};

/// Emits the wrapping-detection block for a layer chunk. Runs once per
/// layer, after the change-layer block and before skirt/object content.
pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    context: &timelapse::Context<'_>,
) -> Result<(), SliceError> {
    let traversal = context.traversal;
    if !traversal
        .resolved
        .views
        .full
        .process
        .gcode
        .enable_wrapping_detection
        .0
    {
        return Ok(());
    }
    // `retract(false, false, auto_lift_type, true)`: full retract+wipe
    // with an eager lift, identical to the timelapse-insert retract
    // (`GCode.cpp:7666-7697`, `apply_instantly` → `eager_lift`).
    motion::retract_for_wrapping(output, state);

    let source = &traversal
        .resolved
        .views
        .runtime_gcode
        .wrapping_detection_gcode
        .0;
    let mut rendered = String::new();
    if !source.is_empty() {
        let mut config = placeholders::base_config(
            traversal,
            context.metadata,
            context.first_layer_bounds,
            Some(&state.template_rng),
            state.total_layer_count,
        )?;
        motion::refresh_writer_state(&mut config, state);
        config.insert(
            "layer_num",
            value::Value::Integer(context.layer.index as i64),
        );
        config.insert("layer_z", value::Value::number(context.layer.z));
        config.insert("max_layer_z", value::Value::number(context.layer.max_z));
        timelapse::insert_physical_extruder_ids(&mut config, traversal, state);
        // Write-back validation precedes emission
        // (`GCode.cpp:3920-3933`).
        match template::render(source, &mut config) {
            Ok(text) => {
                rendered = text;
                if let Err(error) = motion::apply_writer_state(
                    "wrapping_detection_gcode",
                    &config,
                    &rendered,
                    state,
                ) {
                    let name = "wrapping_detection_gcode";
                    output.extend_from_slice(
                        format!(
                            "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
                        )
                        .as_bytes(),
                    );
                    rendered.clear();
                } else {
                    rendered.push('\n');
                    output.extend_from_slice(rendered.as_bytes());
                }
            }
            Err(error) => {
                let name = "wrapping_detection_gcode";
                let error = error.to_string();
                state.record_template_failure(name, &error);
                output.extend_from_slice(
                    format!(
                        "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
                    )
                    .as_bytes(),
                );
            }
        }
    }
    // `set_current_position_clear(false)` (`GCode.cpp:5227`) runs even for
    // an empty template; `get_last_z_from_gcode` then adopts the
    // template's last Z only when it emitted a Z move (`GCode.cpp:5229-5233`).
    state.positioned = false;
    if let Some(z) = timelapse::last_motion_z(&rendered) {
        state.writer_z = Some(z);
    }
    Ok(())
}
