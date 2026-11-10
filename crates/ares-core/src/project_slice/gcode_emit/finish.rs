use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{template, value};

/// Print-end placeholder inputs (`GCode.cpp:3470-3493`).
pub(super) struct FinishContext {
    pub(super) metadata: GenerationMetadata,
    pub(super) first_layer_bounds: Option<super::footprint::FirstLayerBounds>,
    pub(super) position: (f64, f64, f64),
}

pub(super) fn append(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    traversal: &PreparedPostClassicTraversal,
    max_layer_z: f64,
    FinishContext {
        metadata,
        first_layer_bounds,
        position,
    }: FinishContext,
) -> Result<(), SliceError> {
    let gcode = &traversal.resolved.views.runtime_gcode;
    let mut config = super::placeholders::base_config(
        traversal,
        metadata,
        first_layer_bounds,
        Some(&state.template_rng),
        state.total_layer_count,
    )?;
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
        value::Value::Integer(state.layer_chunk_index as i64),
    );
    // `GCode.cpp:3478`: `layer_z = get_position()(2) - z_offset` — the
    // model-domain writer z at print end, NOT the float-domain
    // `max_layer_z` (which stays a separate placeholder).
    let layer_z_placeholder =
        position.2 - traversal.resolved.views.full.printer.remaining.z_offset.0;
    config.insert("layer_z", value::Value::number(layer_z_placeholder));
    config.insert("max_layer_z", value::Value::number(max_layer_z));
    config.insert("timelapse_type", value::Value::number(0.0));
    config.insert("most_used_physical_extruder_id", value::Value::Integer(1));
    config.insert("curr_physical_extruder_id", value::Value::Integer(1));
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
    if traversal.resolved.views.runtime_gcode.enable_pre_heating.0 {
        output.extend_from_slice(b"; MACHINE_END_GCODE_START\n");
    }
    if state.options.single_extruder_multi_material {
        let filament = state.extruders.active_id();
        let source = gcode
            .filament_end_gcode
            .0
            .get(filament)
            .unwrap_or(&gcode.filament_end_gcode.0[0]);
        append_template(
            output,
            state,
            source,
            &mut config,
            EndTemplate::Filament(filament),
        )?;
    } else {
        for (filament, source) in gcode.filament_end_gcode.0.iter().enumerate() {
            append_template(
                output,
                state,
                source,
                &mut config,
                EndTemplate::Filament(filament),
            )?;
        }
    }
    append_template(
        output,
        state,
        &gcode.machine_end_gcode.0,
        &mut config,
        EndTemplate::Machine,
    )?;
    Ok(())
}

pub(super) fn append_compatible_stats(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    layer_count: usize,
    total_toolchanges: usize,
) {
    if super::tags::Tags::of(traversal).is_bbl() {
        return;
    }
    output.extend_from_slice(b"; total filament used [g] =\n; total filament cost =\n");
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
}

pub(super) fn append_filament_stats(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    state: &super::motion::EmitState,
) {
    let filament = &traversal.resolved.views.full.filament.gcode;
    // Writer signed counters decide line presence; once present, processor
    // post-processing replaces the entire configured-width vector.
    let mut weight_line = false;
    let mut cost_line = false;
    for &id in state.extruders.ids() {
        let (_, mut weight) = state.extruders.statistics(
            id,
            &state.extruder,
            (state.options.filament_area, state.options.filament_density),
            state.options.single_extruder_multi_material,
        );
        if let Some(tower) = &state.wipe_tower {
            let used = state.extruders.used_filament(
                id,
                &state.extruder,
                state.options.single_extruder_multi_material,
            ) + tower.used_filament(id);
            let diameter = float_at(&filament.filament_diameter, id);
            let volume = used * (diameter * diameter * std::f64::consts::PI / 4.0);
            weight = volume * float_at(&filament.filament_density, id) * 0.001;
        }
        if weight > 0.0 {
            weight_line = true;
            cost_line |= weight * float_at(&filament.filament_cost, id) * 0.001 > 0.0;
        }
    }
    if !state.extruders.ids().is_empty() {
        let mut stats = String::from("; filament used [mm] =\n; filament used [cm3] =\n");
        if weight_line {
            stats.push_str("; filament used [g] =\n");
        }
        if cost_line {
            stats.push_str("; filament cost =\n");
        }
        output.extend_from_slice(stats.as_bytes());
    }
}

fn float_at(values: &crate::OrcaFloats, index: usize) -> f64 {
    values.0.get(index).unwrap_or(&values.0[0]).0
}

#[derive(Clone, Copy)]
enum EndTemplate {
    Filament(usize),
    Machine,
}

impl EndTemplate {
    fn name(self) -> &'static str {
        match self {
            Self::Filament(_) => "filament-end",
            Self::Machine => "machine-end",
        }
    }

    fn template_name(self) -> &'static str {
        match self {
            Self::Filament(_) => "filament_end_gcode",
            Self::Machine => "machine_end_gcode",
        }
    }
}

fn append_template(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    source: &str,
    config: &mut value::Config,
    template: EndTemplate,
) -> Result<(), SliceError> {
    let name = template.name();
    let template_name = template.template_name();
    if source.is_empty() {
        return Ok(());
    }
    super::motion::refresh_writer_state(config, state);
    let filament = match template {
        EndTemplate::Filament(filament) => filament,
        EndTemplate::Machine => state.extruders.active_id(),
    };
    if matches!(template, EndTemplate::Filament(_)) {
        config.insert(
            "filament_extruder_id",
            value::Value::Integer(filament as i64),
        );
    }
    // Render or write-back failure emits only the inline report — the
    // rendered text is discarded (`GCode.cpp:3920-3933`).
    let rendered = match template::render_for_filament(source, config, filament) {
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
    if let Err(error) = super::motion::apply_writer_state(template_name, config, &rendered, state) {
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
