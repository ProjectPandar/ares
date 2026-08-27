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
    let mut config = super::placeholders::base_config(traversal);
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

    let tags = super::tags::Tags::of(traversal);
    let custom = tags.custom() + "\n";
    output.extend_from_slice(b"M106 S0\nM106 P2 S0\nM981 S0 P20000 ; close spaghetti detector\n");
    output.extend_from_slice(custom.as_bytes());
    if let Some(filament_end) = gcode.filament_end_gcode.0.first() {
        append_template(output, filament_end, &config, "filament-end")?;
    }
    append_template(output, &gcode.machine_end_gcode.0, &config, "machine-end")?;
    Ok(())
}

pub(super) fn append_filament_stats(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    used_filament: f64,
) {
    let filament = &traversal.resolved.views.full.filament.gcode;
    let mut used_mm = Vec::with_capacity(filament.filament_diameter.0.len());
    let mut used_cm3 = Vec::with_capacity(filament.filament_diameter.0.len());
    let mut used_g = Vec::with_capacity(filament.filament_diameter.0.len());
    let mut costs = Vec::with_capacity(filament.filament_diameter.0.len());
    for (index, diameter) in filament.filament_diameter.0.iter().enumerate() {
        let length = if index == 0 { used_filament } else { 0.0 };
        let volume = length * diameter.0.powi(2) * 0.25 * std::f64::consts::PI;
        let density = filament.filament_density.0[index].0;
        let cost = filament.filament_cost.0[index].0;
        let weight = volume * density * 0.001;
        used_mm.push(format!("{length:.2}"));
        used_cm3.push(format!("{:.2}", volume * 0.001));
        used_g.push(format!("{weight:.2}"));
        costs.push(format!("{:.2}", weight * cost * 0.001));
    }
    output.extend_from_slice(
        format!(
            "; filament used [mm] = {}\n; filament used [cm3] = {}\n; filament used [g] = {}\n; filament cost = {}\n\n",
            used_mm.join(", "),
            used_cm3.join(", "),
            used_g.join(", "),
            costs.join(", "),
        )
        .as_bytes(),
    );
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
