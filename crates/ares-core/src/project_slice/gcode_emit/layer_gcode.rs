//! Per-layer emission helpers split from the emit driver: fan allowance,
//! print preamble, and the layer-change G-code templates.

use crate::GenerationMetadata;

use super::super::gcode_emit::{SliceError, extruders, tags, template, value};
use super::PreparedPostClassicTraversal;

pub(super) struct LayerChangeTemplate {
    max_additional_fan: f64,
    metadata: GenerationMetadata,
}

impl LayerChangeTemplate {
    pub(super) fn new(
        traversal: &PreparedPostClassicTraversal,
        metadata: GenerationMetadata,
    ) -> Self {
        Self {
            max_additional_fan: max_additional_fan(traversal),
            metadata,
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
pub(super) fn append_print_preamble(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
) -> Result<(), SliceError> {
    if tags::Tags::of(traversal).is_bbl() {
        output.extend_from_slice(
            b"; filament start gcode\n;VT0\nG90\nG21\nM83 ; use relative distances for extrusion\n",
        );
        output.extend_from_slice(b"M981 S1 P20000 ;open spaghetti detector\nM106 S0\nM106 P2 S0\n");
        return Ok(());
    }
    // Compatible flavor renders the preset's filament start template
    // (`GCode.cpp` filament-start handling).
    if let Some(source) = traversal
        .resolved
        .views
        .runtime_gcode
        .filament_start_gcode
        .0
        .first()
    {
        let config = super::placeholders::base_config(traversal, metadata);
        let rendered = template::render(source, &config).map_err(|error| {
            SliceError::InvalidInput(format!(
                "invalid project filament-start G-code template: {error}"
            ))
        })?;
        output.extend_from_slice(rendered.as_bytes());
        if !rendered.ends_with('\n') {
            output.push(b'\n');
        }
    }
    Ok(())
}

/// Renders `before_layer_change_gcode` with `layer_num`, `layer_z`, and
/// `max_layer_z` in scope (`GCode.cpp:4631-4641`).
pub(super) fn append_before_layer_change_gcode(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    layer_index: usize,
    layer_z: f64,
    metadata: GenerationMetadata,
) -> Result<(), SliceError> {
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
    let mut config = super::placeholders::base_config(traversal, metadata);
    config.insert("layer_num", value::Value::number((layer_index + 1) as f64));
    config.insert("layer_z", value::Value::number(layer_z));
    config.insert("max_layer_z", value::Value::number(layer_z));
    let rendered = template::render(template, &config).map_err(|error| {
        SliceError::InvalidInput(format!(
            "invalid project before-layer-change G-code template: {error}"
        ))
    })?;
    output.extend_from_slice(rendered.as_bytes());
    output.push(b'\n');
    Ok(())
}

pub(super) fn append_layer_change(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    layer_index: usize,
    layer_z: f64,
    context: &LayerChangeTemplate,
) -> Result<(), SliceError> {
    let template = &traversal.resolved.views.runtime_gcode.layer_change_gcode.0;
    if !template.is_empty() {
        let mut config = super::placeholders::base_config(traversal, context.metadata);
        config.insert("current_extruder", value::Value::number(0.0));
        config.insert("layer_num", value::Value::number(layer_index as f64));
        config.insert("layer_z", value::Value::number(layer_z));
        config.insert("overall_chamber_temperature", value::Value::number(0.0));
        if let Some(value) = config
            .get("temperature_vitrification")
            .and_then(|value| value.index(0))
            .cloned()
        {
            config.insert("min_vitrification_temperature", value);
        }
        config.insert(
            "max_additional_fan",
            value::Value::number(context.max_additional_fan),
        );
        let rendered = template::render(template, &config).map_err(|error| {
            SliceError::InvalidInput(format!(
                "invalid project layer-change G-code template: {error}"
            ))
        })?;
        output.extend_from_slice(rendered.as_bytes());
        output.push(b'\n');
    }
    output.extend_from_slice(b";_SET_FAN_SPEED_CHANGING_LAYER\n");
    Ok(())
}
