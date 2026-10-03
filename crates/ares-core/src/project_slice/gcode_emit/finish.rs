use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{template, value};

mod accounting;
pub(super) use accounting::account_volumes;

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

pub(super) fn append_filament_stats(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    volumes: &[f64],
    state: &super::motion::EmitState,
) -> (f64, f64) {
    let filament = &traversal.resolved.views.full.filament.gcode;
    let mut used_mm = Vec::with_capacity(volumes.len());
    let mut used_cm3 = Vec::with_capacity(volumes.len());
    let mut used_g = Vec::with_capacity(volumes.len());
    let mut costs = Vec::with_capacity(volumes.len());
    let mut total_weight = 0.0;
    let mut total_cost = 0.0;
    // Processor properties are floats; each output is derived from original
    // volume independently (`GCodeProcessor.cpp:997–1009`).
    for (index, &volume) in volumes.iter().enumerate() {
        let diameter = f64::from(filament.filament_diameter.0[index].0 as f32);
        let density = f64::from(float_at(&filament.filament_density, index) as f32);
        let cost = f64::from(float_at(&filament.filament_cost, index) as f32);
        let length = volume / (std::f64::consts::PI * (0.5 * diameter).powi(2));
        let cm3 = volume * 0.001;
        let weight = cm3 * density;
        let material_cost = weight * cost * 0.001;
        used_mm.push(format!("{length:.2}"));
        used_cm3.push(format!("{cm3:.2}"));
        used_g.push(format!("{weight:.2}"));
        costs.push(format!("{material_cost:.2}"));
        total_weight += weight;
        total_cost += material_cost;
    }
    // Writer signed counters decide line presence; once present, processor
    // post-processing replaces the entire configured-width vector.
    let mut weight_line = false;
    let mut cost_line = false;
    for &id in state.extruders.ids() {
        let (_, weight) = state.extruders.statistics(
            id,
            &state.extruder,
            (state.options.filament_area, state.options.filament_density),
            state.options.single_extruder_multi_material,
        );
        if weight > 0.0 {
            weight_line = true;
            cost_line |= weight * float_at(&filament.filament_cost, id) * 0.001 > 0.0;
        }
    }
    if !state.extruders.ids().is_empty() {
        let mut stats = format!(
            "; filament used [mm] = {}\n; filament used [cm3] = {}\n",
            used_mm.join(", "),
            used_cm3.join(", "),
        );
        if weight_line {
            stats.push_str(&format!("; filament used [g] = {}\n", used_g.join(", ")));
        }
        if cost_line {
            stats.push_str(&format!("; filament cost = {}\n", costs.join(", ")));
        }
        output.extend_from_slice(stats.as_bytes());
    }
    (total_weight, total_cost)
}

fn float_at(values: &crate::OrcaFloats, index: usize) -> f64 {
    values.0.get(index).unwrap_or(&values.0[0]).0
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
