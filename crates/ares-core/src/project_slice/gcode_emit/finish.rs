use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{template, value};

#[cfg(test)]
mod tests;

pub(super) fn append(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    traversal: &PreparedPostClassicTraversal,
    max_layer_z: f64,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<super::footprint::FirstLayerBounds>,
    position: (f64, f64, f64),
) -> Result<(), SliceError> {
    let gcode = &traversal.resolved.views.runtime_gcode;
    let mut config = super::placeholders::base_config(traversal, metadata, first_layer_bounds)?;
    // `ppi.update_from_gcodewriter(m_writer)` refreshes `position` and the
    // E counters before each template (`GCode.cpp:3896`).
    super::motion::refresh_writer_state(&mut config, state);
    // The `position` placeholder carries the writer's position at print
    // end (`GCode.cpp:3470-3493` end g-code — the parser's live position,
    // e.g. SPARKX's `G2 Z{position[2]+3}` spiral lift resolves against the
    // final layer z, not zero).
    config.insert(
        "position",
        value::Value::List(vec![
            value::Value::number(position.0),
            value::Value::number(position.1),
            value::Value::number(position.2),
        ]),
    );
    config.insert(
        "layer_num",
        value::Value::number(layer_count(traversal).saturating_sub(1) as f64),
    );
    // `GCode.cpp:3478`: `layer_z = get_position()(2) - z_offset` — the
    // model-domain writer z at print end, NOT the float-domain
    // `max_layer_z` (which stays a separate placeholder).
    let layer_z_placeholder =
        position.2 - traversal.resolved.views.full.printer.remaining.z_offset.0;
    config.insert("layer_z", value::Value::number(layer_z_placeholder));
    config.insert("max_layer_z", value::Value::number(max_layer_z));
    config.insert("max_print_z", value::Value::number(max_layer_z));
    config.insert("timelapse_type", value::Value::number(0.0));
    config.insert("most_used_physical_extruder_id", value::Value::number(1.0));
    config.insert("curr_physical_extruder_id", value::Value::number(1.0));
    config.insert("has_timelapse_safe_pos", value::Value::option_bool(false));

    let tags = super::tags::Tags::of(traversal);
    let custom = tags.custom() + "\n";
    // GCode.cpp:3459-3465: set_fan(0) unconditionally, then the additional
    // (auxiliary) fan only when auxiliary_fan is enabled, then the BBL-only
    // spaghetti detector. The aux-fan off was wrongly gated on is_bbl.
    let comments = traversal.resolved.views.full.process.print.gcode_comments.0;
    let fan_comment = if comments { " ; disable fan" } else { "" };
    let aux_comment = if comments {
        " ; disable additional fan "
    } else {
        ""
    };
    output.extend_from_slice(format!("M106 S0{fan_comment}\n").as_bytes());
    if traversal.resolved.views.runtime_gcode.auxiliary_fan.0 {
        output.extend_from_slice(format!("M106 P2 S0{aux_comment}\n").as_bytes());
    }

    if tags.is_bbl() {
        output.extend_from_slice(b"M981 S0 P20000 ; close spaghetti detector\n");
    }
    output.extend_from_slice(custom.as_bytes());
    if let Some(filament_end) = gcode.filament_end_gcode.0.first() {
        append_template(
            output,
            state,
            filament_end,
            &mut config,
            "filament-end",
            "filament_end_gcode",
        )?;
        // The machine-end template reseeds from the post-assignment
        // writer state (`GCode.cpp:3896` refresh per process call).
        super::motion::refresh_writer_state(&mut config, state);
    }
    append_template(
        output,
        state,
        &gcode.machine_end_gcode.0,
        &mut config,
        "machine-end",
        "machine_end_gcode",
    )?;
    Ok(())
}

pub(super) fn append_compatible_stats(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    (total_weight, total_cost): (f64, f64),
    layer_count: usize,
    total_toolchanges: usize,
) {
    if super::tags::Tags::of(traversal).is_bbl() {
        return;
    }
    output.extend_from_slice(
        format!(
            "; total filament used [g] = {total_weight:.2}\n\
; total filament cost = {total_cost:.2}\n\
"
        )
        .as_bytes(),
    );
    // `GCode.cpp:3539–3542` uses the same non-tower forecast as templates.
    if total_toolchanges > 0 {
        output.extend_from_slice(
            format!("; total filament change = {total_toolchanges}\n").as_bytes(),
        );
    }
    output.extend_from_slice(
        format!(
            "\
; total layers count = {layer_count}\n\
; estimated printing time (normal mode) = 0s\n\
; estimated first layer printing time (normal mode) = 0s\n\n"
        )
        .as_bytes(),
    );
    if let Some(config) = &traversal.config_block {
        output.extend_from_slice(config);
    }
}

/// Accounts used filament from the emitted G-code exactly like the upstream
/// Totals filament usage the way `GCodeProcessor` does: track the E
/// position (M82/M83, G90/G91, G92 resets), classify each move
/// (`move_type`, `GCodeProcessor.cpp:3834-3851`) and count only Extrude
/// moves — E-positive with an X or Y displacement. Retracts, unretracts
/// and E-only bowden re-primes do not count.
pub(super) fn account_used_filament(gcode: &[u8], filament_diameter: f64) -> f64 {
    let diameter = filament_diameter as f32;
    let radius = 0.5_f32 * diameter;
    let area = std::f32::consts::PI * (radius * radius);
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
                    .map(|value| f64::from(value as f32))
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
                let start = (x_position, y_position, e_position);
                let e_relative = global_relative || e_local_relative;
                if let Some(value) = x {
                    x_position = apply_axis(x_position, value, global_relative);
                }
                if let Some(value) = y {
                    y_position = apply_axis(y_position, value, global_relative);
                }
                if let Some(value) = e {
                    e_position = apply_e(e_position, value, e_relative);
                }
                let dx = x_position - start.0;
                let dy = y_position - start.1;
                let delta_e = e_position - start.2;
                if delta_e > 0.0 && (dx != 0.0 || dy != 0.0) {
                    // GCodeProcessor::process_G1 stores each extruded volume as float.
                    used += f64::from((f64::from(area) * delta_e) as f32);
                }
            }
            _ => {}
        }
    }
    used / (std::f64::consts::PI * (0.5 * f64::from(diameter)).powi(2))
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
    writer_used_filament: f64,
) -> (f64, f64) {
    let filament = &traversal.resolved.views.full.filament.gcode;
    let mut used_mm = Vec::with_capacity(filament.filament_diameter.0.len());
    let mut used_cm3 = Vec::with_capacity(filament.filament_diameter.0.len());
    // `GCode.cpp:2349-2368` gates line presence; the processor replaces
    // present lines with configured-width vectors, including trailing zeros.
    let mut used_g = vec![None; filament.filament_diameter.0.len()];
    let mut costs = vec![None; filament.filament_diameter.0.len()];
    let mut total_weight = 0.0;
    let mut total_cost = 0.0;
    for (index, diameter) in filament.filament_diameter.0.iter().enumerate() {
        // `GCode.cpp:2330-2354`: the `[g]`/`cost` lines are gated on the
        // WRITER-side weight (`Extruder::used_filament()` = signed
        // `m_absolute_E + m_retracted` — wipe negatives drain it past the
        // restart, so extreme flow ratios leave it <= 0 and drop both
        // lines) while the emitted numbers come from the processor's
        // `total_volumes_per_extruder` accumulation, which `used_filament`
        // here already models (Extrude-only E).
        let displayed_length = if index == 0 { used_filament } else { 0.0 };
        let gate_length = if index == 0 {
            writer_used_filament
        } else {
            0.0
        };
        let volume = displayed_length * diameter.0.powi(2) * 0.25 * std::f64::consts::PI;
        let gate_volume = gate_length * diameter.0.powi(2) * 0.25 * std::f64::consts::PI;
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
        // Writer-side weight decides whether upstream's `append` ran
        // (`filament_weight > 0`, GCode.cpp:2349); the printed values are
        // the processor's per-extruder accumulations.
        let gate_weight = gate_volume * density * 0.001;
        used_mm.push(format!("{displayed_length:.2}"));
        used_cm3.push(format!("{:.2}", volume * 0.001));
        // `; total filament used [g]` is `print_statistics.total_weight`,
        // overwritten at `GCode.cpp:1915-1937` from processor volumes — it
        // is printed even when the writer-side gate dropped the per-extruder
        // `[g]`/`cost` lines.
        total_weight += weight;
        total_cost += material_cost;
        if gate_weight > 0.0 {
            used_g[index] = Some(format!("{weight:.2}"));
            if material_cost > 0.0 {
                costs[index] = Some(format!("{material_cost:.2}"));
            }
        }
    }
    let mut stats = format!(
        "; filament used [mm] = {}\n; filament used [cm3] = {}\n",
        used_mm.join(", "),
        used_cm3.join(", "),
    );
    if let Some(grams) = join_postprocessed(&used_g) {
        stats.push_str(&format!("; filament used [g] = {grams}\n"));
    }
    if let Some(costs) = join_postprocessed(&costs) {
        stats.push_str(&format!("; filament cost = {costs}\n"));
    }
    output.extend_from_slice(stats.as_bytes());
    (total_weight, total_cost)
}

/// `GCodeProcessor.cpp:1148-1175` replaces existing lines at full vector width.
fn join_postprocessed(values: &[Option<String>]) -> Option<String> {
    values.iter().find(|value| value.is_some())?;
    let parts = values
        .iter()
        .map(|value| value.as_deref().unwrap_or("0.00"))
        .collect::<Vec<_>>();
    Some(parts.join(", "))
}

fn append_template(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    source: &str,
    config: &mut value::Config,
    name: &str,
    template_name: &str,
) -> Result<(), SliceError> {
    if source.is_empty() {
        return Ok(());
    }
    // Render or write-back failure emits only the inline report — the
    // rendered text is discarded (`GCode.cpp:3920-3933`).
    let rendered = match template::render(source, config) {
        Ok(rendered) => Some(rendered),
        Err(error) => {
            let error = error.to_string();
            state.record_template_failure(template_name, &error);
            output.extend_from_slice(
                format!(
                    "\n!!!!! Failed to process the custom G-code template {template_name}\n{error}!!!!! End of an error report for the custom G-code template {template_name}\n\n\n"
                )
                .as_bytes(),
            );
            None
        }
    };
    let Some(rendered) = rendered else {
        return Ok(());
    };
    if let Err(error) = super::motion::apply_writer_state(template_name, config, state) {
        output.extend_from_slice(
            format!(
                "\n!!!!! Failed to process the custom G-code template {template_name}\n{error}!!!!! End of an error report for the custom G-code template {template_name}\n\n\n"
            )
            .as_bytes(),
        );
        return Ok(());
    }
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
