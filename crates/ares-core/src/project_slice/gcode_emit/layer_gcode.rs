//! Per-layer emission helpers split from the emit driver: fan allowance,
//! print preamble, and the layer-change G-code templates.

use crate::{GenerationMetadata, project_slice::extruders};

use super::super::gcode_emit::{SliceError, template, value};
use super::PreparedPostClassicTraversal;

pub(super) struct LayerChangeTemplate {
    max_additional_fan: f64,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<super::footprint::FirstLayerBounds>,
}

impl LayerChangeTemplate {
    pub(super) fn new(
        traversal: &PreparedPostClassicTraversal,
        metadata: GenerationMetadata,
        first_layer_bounds: Option<super::footprint::FirstLayerBounds>,
    ) -> Self {
        Self {
            max_additional_fan: max_additional_fan(traversal),
            metadata,
            first_layer_bounds,
        }
    }
}

fn max_additional_fan(traversal: &PreparedPostClassicTraversal) -> f64 {
    let used = extruders::collect_project_object_extruders(
        traversal.project.objects(),
        &traversal.resolved.objects,
        traversal.resolved.logical_filament_count,
    );
    let speeds = &traversal
        .resolved
        .views
        .full
        .filament
        .print
        .additional_cooling_fan_speed
        .0;
    used.into_iter()
        .flatten()
        .filter_map(|extruder| speeds.get(extruder).or_else(|| speeds.first()))
        .map(|speed| speed.0)
        .max()
        .map_or(0.0, f64::from)
}
mod startup;
pub(super) use startup::append_print_preamble;

pub(super) struct LayerTemplateContext<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) layer_index: usize,
    pub(super) layer_z: f64,
    /// `m_max_layer_z` (`GCode.cpp:4638`): running max of the OFFSET
    /// writer z (`print_z + z_offset`) — layers ascend, so this equals
    /// the current layer's offset z.
    pub(super) max_layer_z: f64,
    pub(super) context: &'a LayerChangeTemplate,
}

/// Renders `before_layer_change_gcode` with `layer_num`, `layer_z`, and
/// `max_layer_z` in scope (`GCode.cpp:4631-4641`).
pub(super) fn append_before_layer_change_gcode(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    template_context: LayerTemplateContext<'_>,
) -> Result<(), SliceError> {
    let LayerTemplateContext {
        traversal,
        layer_index,
        layer_z,
        max_layer_z,
        context,
    } = template_context;
    let template = traversal
        .resolved
        .views
        .runtime_gcode
        .before_layer_change_gcode
        .0
        .as_str();
    if template.is_empty() {
        return Ok(());
    }
    if template.trim().is_empty() {
        output.push(b'\n');
        return Ok(());
    }
    let mut config = super::placeholders::base_config(
        traversal,
        context.metadata,
        context.first_layer_bounds,
        Some(&state.template_rng),
        state.total_layer_count,
    )?;
    // `ppi.update_from_gcodewriter(m_writer)` refreshes `position`/
    // E-counters before each template (`GCode.cpp:3896`).
    super::motion::refresh_writer_state(&mut config, state);
    config.insert("layer_num", value::Value::number(layer_index as f64));
    config.insert("layer_z", value::Value::number(layer_z));
    config.insert("max_layer_z", value::Value::number(max_layer_z));
    // Render or write-back failure both reduce to the inline report —
    // the processed text is discarded (`GCode.cpp:3920-3933` returns the
    // block instead of the template output; `slic3r_errors` via
    // `record_template_failure` abort the export at the end).
    let name = "before_layer_change_gcode";
    let mut failure_error = String::new();
    // GCode.cpp:4639-4641 renders with the current logical writer filament.
    let rendered =
        match template::render_for_filament(template, &mut config, state.extruders.active_id()) {
            Ok(rendered) => Some(rendered),
            Err(error) => {
                failure_error = error.to_string();
                state.record_template_failure(name, &failure_error);
                None
            }
        };
    let failed = rendered.is_none()
        || super::motion::apply_writer_state(name, &config, state)
            .map_err(|error| failure_error = error)
            .is_err();
    if failed {
        let error = failure_error;
        output.extend_from_slice(
            format!(
                "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
            )
            .as_bytes(),
        );
        return Ok(());
    }
    output.extend_from_slice(rendered.unwrap().as_bytes());
    output.push(b'\n');
    Ok(())
}

pub(super) fn append_layer_change(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    template_context: LayerTemplateContext<'_>,
) -> Result<(), SliceError> {
    let LayerTemplateContext {
        traversal,
        layer_index,
        layer_z,
        max_layer_z: _,
        context,
    } = template_context;
    let template = &traversal.resolved.views.runtime_gcode.layer_change_gcode.0;
    if !template.is_empty() {
        let mut config = super::placeholders::base_config(
            traversal,
            context.metadata,
            context.first_layer_bounds,
            Some(&state.template_rng),
            state.total_layer_count,
        )?;
        super::motion::refresh_writer_state(&mut config, state);
        config.insert("layer_num", value::Value::number(layer_index as f64));
        config.insert("layer_z", value::Value::number(layer_z));
        config.insert("overall_chamber_temperature", value::Value::number(0.0));
        if let Some(value) = config
            .get("temperature_vitrification")
            .and_then(|value| value.index(0))
            .and_then(value::Value::as_number)
        {
            config.insert(
                "min_vitrification_temperature",
                value::Value::Integer(value as i64),
            );
        }
        config.insert(
            "max_additional_fan",
            value::Value::number(context.max_additional_fan),
        );
        // `GCode.cpp:4679`: `most_used_physical_extruder_id` =
        // `physical_extruder_map.get_at(most_used_extruder)` — the layer's
        // most-used logical extruder mapped to its physical extruder
        // (`ToolOrdering.cpp:955-976` counts; `state.most_used_extruder`).
        let physical_extruder_map = &traversal
            .resolved
            .views
            .runtime_gcode
            .physical_extruder_map
            .0;
        let most_used_physical_extruder_id = physical_extruder_map
            .get(state.most_used_extruder)
            .or_else(|| physical_extruder_map.first())
            .map_or(0, |value| value.0);
        config.insert(
            "most_used_physical_extruder_id",
            value::Value::number(most_used_physical_extruder_id as f64),
        );
        // Render or write-back failure both produce only the inline
        // report (`GCode.cpp:3920-3933`).
        let mut failure_error = String::new();
        // GCode.cpp:4682-4684 keeps the current logical writer filament.
        let rendered =
            match template::render_for_filament(template, &mut config, state.extruders.active_id())
            {
                Ok(rendered) => Some(rendered),
                Err(error) => {
                    failure_error = error.to_string();
                    state.record_template_failure("layer_change_gcode", &failure_error);
                    None
                }
            };
        let name = "layer_change_gcode";
        let failed = rendered.is_none()
            || super::motion::apply_writer_state(name, &config, state)
                .map_err(|error| failure_error = error)
                .is_err();
        if failed {
            let error = failure_error;
            output.extend_from_slice(
                format!(
                    "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
                )
                .as_bytes(),
            );
        } else {
            output.extend_from_slice(rendered.unwrap().as_bytes());
            output.push(b'\n');
        }
    }
    output.extend_from_slice(b";_SET_FAN_SPEED_CHANGING_LAYER\n");
    Ok(())
}
