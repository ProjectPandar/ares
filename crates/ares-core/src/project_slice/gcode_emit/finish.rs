use crate::{
    SliceError, project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{template, value};

pub(super) fn append(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    max_layer_z: f64,
) -> Result<(), SliceError> {
    let gcode = &traversal.resolved.views.runtime_gcode;
    let mut config =
        value::Config::from_block(traversal.config_block.as_deref().unwrap_or_default());
    config.insert("current_extruder", value::Value::number(0.0));
    config.insert(
        "layer_num",
        value::Value::number(layer_count(traversal) as f64),
    );
    config.insert("layer_z", value::Value::number(max_layer_z));
    config.insert("max_layer_z", value::Value::number(max_layer_z));
    config.insert("max_print_z", value::Value::number(max_layer_z));
    config.insert("timelapse_type", value::Value::number(0.0));
    config.insert("most_used_physical_extruder_id", value::Value::number(1.0));
    config.insert("curr_physical_extruder_id", value::Value::number(1.0));
    config.insert("has_timelapse_safe_pos", value::Value::Bool(false));

    append_template(output, &gcode.time_lapse_gcode.0, &config, "time-lapse")?;
    output.extend_from_slice(
        b"M106 S0\nM106 P2 S0\nM981 S0 P20000 ; close spaghetti detector\n; FEATURE: Custom\n",
    );
    if let Some(filament_end) = gcode.filament_end_gcode.0.first() {
        append_template(output, filament_end, &config, "filament-end")?;
    }
    append_template(output, &gcode.machine_end_gcode.0, &config, "machine-end")?;
    Ok(())
}

fn append_template(
    output: &mut Vec<u8>,
    source: &str,
    config: &value::Config,
    name: &str,
) -> Result<(), SliceError> {
    if source.is_empty() {
        return Ok(());
    }
    let rendered = template::render(source, config).map_err(|error| {
        SliceError::InvalidInput(format!("invalid project {name} G-code template: {error}"))
    })?;
    output.extend_from_slice(rendered.as_bytes());
    if !rendered.ends_with('\n') {
        output.push(b'\n');
    }
    Ok(())
}

fn layer_count(traversal: &PreparedPostClassicTraversal) -> usize {
    traversal
        .objects
        .first()
        .map_or(0, |object| object.records.len())
}
