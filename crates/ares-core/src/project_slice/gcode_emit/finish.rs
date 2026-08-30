use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{template, value};

#[cfg(test)]
mod tests;

pub(super) fn append(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    max_layer_z: f64,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<super::footprint::FirstLayerBounds>,
) -> Result<(), SliceError> {
    let gcode = &traversal.resolved.views.runtime_gcode;
    let mut config = super::placeholders::base_config(traversal, metadata, first_layer_bounds)?;
    config.insert("current_extruder", value::Value::number(0.0));
    config.insert(
        "layer_num",
        value::Value::number(layer_count(traversal).saturating_sub(1) as f64),
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
    // GCode.cpp:3459-3465: set_fan(0) unconditionally, then the additional
    // (auxiliary) fan only when auxiliary_fan is enabled, then the BBL-only
    // spaghetti detector. The aux-fan off was wrongly gated on is_bbl.
    output.extend_from_slice(b"M106 S0\n");
    if traversal.resolved.views.runtime_gcode.auxiliary_fan.0 {
        output.extend_from_slice(b"M106 P2 S0\n");
    }
    if tags.is_bbl() {
        output.extend_from_slice(b"M981 S0 P20000 ; close spaghetti detector\n");
    }
    output.extend_from_slice(custom.as_bytes());
    if let Some(filament_end) = gcode.filament_end_gcode.0.first() {
        append_template(output, filament_end, &mut config, "filament-end")?;
    }
    append_template(
        output,
        &gcode.machine_end_gcode.0,
        &mut config,
        "machine-end",
    )?;
    Ok(())
}

/// Accounts used filament from the emitted G-code exactly like the upstream
/// Totals filament usage the way `GCodeProcessor` does: track the E
/// position (M82/M83, G90/G91, G92 resets), classify each move
/// (`move_type`, `GCodeProcessor.cpp:3834-3851`) and count only Extrude
/// moves — E-positive with an X or Y displacement. Retracts, unretracts
/// and E-only bowden re-primes do not count.
pub(super) fn account_used_filament(gcode: &[u8]) -> f64 {
    let text = std::str::from_utf8(gcode).unwrap_or_default();
    let mut e_position = 0.0_f64;
    let mut x_position = 0.0_f64;
    let mut y_position = 0.0_f64;
    let mut e_local_relative = false;
    let mut global_relative = false;
    let mut used = 0.0_f64;
    for line in text.lines() {
        let code = line.split_once(';').map_or(line, |(code, _)| code).trim();
        let mut words = code.split_ascii_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        let letter_value = |letter: char| {
            words.clone().find_map(|word| {
                word.strip_prefix(letter)
                    .and_then(|value| value.parse::<f64>().ok())
            })
        };
        match command {
            "M82" => e_local_relative = false,
            "M83" => e_local_relative = true,
            "G90" => global_relative = false,
            "G91" => global_relative = true,
            "G92" => {
                if let Some(value) = letter_value('E') {
                    e_position = value;
                }
            }
            "G0" | "G1" | "G2" | "G3" => {
                let x = letter_value('X');
                let y = letter_value('Y');
                let e = letter_value('E');
                let dx = match x {
                    Some(value) if global_relative => value,
                    Some(value) => value - x_position,
                    None => 0.0,
                };
                let dy = match y {
                    Some(value) if global_relative => value,
                    Some(value) => value - y_position,
                    None => 0.0,
                };
                let e_relative = global_relative || e_local_relative;
                let delta_e = match e {
                    Some(value) if e_relative => value,
                    Some(value) => value - e_position,
                    None => 0.0,
                };
                if let Some(value) = x {
                    x_position = apply_axis(x_position, value, global_relative);
                }
                if let Some(value) = y {
                    y_position = apply_axis(y_position, value, global_relative);
                }
                if let Some(value) = e {
                    e_position = apply_e(e_position, value, e_relative);
                }
                if delta_e > 0.0 && (dx != 0.0 || dy != 0.0) {
                    used += delta_e;
                }
            }
            _ => {}
        }
    }
    used
}

fn apply_e(current: f64, value: f64, relative: bool) -> f64 {
    if relative { current + value } else { value }
}

fn apply_axis(current: f64, value: f64, relative: bool) -> f64 {
    if relative { current + value } else { value }
}

pub(super) fn append_filament_stats(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    used_filament: f64,
) -> (f64, f64) {
    let filament = &traversal.resolved.views.full.filament.gcode;
    let mut used_mm = Vec::with_capacity(filament.filament_diameter.0.len());
    let mut used_cm3 = Vec::with_capacity(filament.filament_diameter.0.len());
    let mut used_g = Vec::with_capacity(filament.filament_diameter.0.len());
    let mut costs = Vec::with_capacity(filament.filament_diameter.0.len());
    let mut total_weight = 0.0;
    let mut total_cost = 0.0;
    for (index, diameter) in filament.filament_diameter.0.iter().enumerate() {
        let length = if index == 0 { used_filament } else { 0.0 };
        let volume = length * diameter.0.powi(2) * 0.25 * std::f64::consts::PI;
        let density = filament
            .filament_density
            .0
            .get(index)
            .or_else(|| filament.filament_density.0.first())
            .expect("validated filament density vector is nonempty")
            .0;
        let cost = filament
            .filament_cost
            .0
            .get(index)
            .or_else(|| filament.filament_cost.0.first())
            .expect("validated filament cost vector is nonempty")
            .0;
        let weight = volume * density * 0.001;
        let material_cost = weight * cost * 0.001;
        total_weight += weight;
        total_cost += material_cost;
        used_mm.push(format!("{length:.2}"));
        used_cm3.push(format!("{:.2}", volume * 0.001));
        used_g.push(format!("{weight:.2}"));
        costs.push(format!("{material_cost:.2}"));
    }
    output.extend_from_slice(
        format!(
            "; filament used [mm] = {}\n; filament used [cm3] = {}\n; filament used [g] = {}\n; filament cost = {}\n",
            used_mm.join(", "),
            used_cm3.join(", "),
            used_g.join(", "),
            costs.join(", "),
        )
        .as_bytes(),
    );
    (total_weight, total_cost)
}

fn append_template(
    output: &mut Vec<u8>,
    source: &str,
    config: &mut value::Config,
    name: &str,
) -> Result<(), SliceError> {
    if source.is_empty() {
        return Ok(());
    }
    let rendered = template::render(source, config).map_err(|error| {
        SliceError::InvalidInput(format!("invalid project {name} G-code template: {error}"))
    })?;
    let rendered = if name == "filament-end" {
        rendered.trim_start_matches([' ', '\t'])
    } else {
        &rendered
    };
    if rendered.is_empty() {
        return Ok(());
    }
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
