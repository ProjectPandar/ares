//! Filament template lifecycle from `GCode::set_extruder` (7759–7777,8018–8043).

use super::{motion, placeholders, template, timelapse, value};
use crate::SliceError;

mod change;
mod temperature;
pub(super) use change::last_start_filament;

pub(super) fn initial(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
    metadata: crate::GenerationMetadata,
    first_layer_bounds: Option<super::footprint::FirstLayerBounds>,
) -> Result<(), SliceError> {
    emit_inner(
        output,
        state,
        state.initial_filaments.initial(),
        change::Context {
            traversal,
            tool_ordering: None,
            metadata,
            first_layer_bounds,
            layer_num: -1,
            print_z: 0.0,
            max_z: 0.0,
            temperature_override: None,
        },
    )
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    filament: usize,
    context: timelapse::Context<'_>,
) -> Result<(), SliceError> {
    // GCode.cpp:7476-7478,7513-7539: synchronize inline insertion before
    // the ordinary pre-toolchange timelapse gate.
    let settings = &context.traversal.resolved.views.full;
    let current_extruder = settings
        .project
        .gcode
        .filament_map
        .0
        .get(state.extruders.active_id())
        .map_or(0, |value| (value.0 - 1).max(0) as usize);
    let photo = &state.farthest_point;
    state.has_insert_timelapse_gcode |= photo.inserted_this_layer;
    let current_is_photo_head = current_extruder == state.most_used_extruder;
    let is_case_b = photo.enabled && !photo.farthest_is_photo_head;
    let should_insert = if is_case_b {
        !current_is_photo_head
    } else {
        current_is_photo_head
    };
    if state.traditional_timelapse
        && !state.has_insert_timelapse_gcode
        && !(photo.enabled && photo.farthest_is_photo_head)
        && state.extruders.active_id() != filament
        && settings.project.print.nozzle_diameter.0.len() == 2
        && state.extruder_selected
        && should_insert
    {
        motion::prepare_traditional_timelapse(output, state);
        motion::append_exclude_end(output, state);
        timelapse::append_and_track(output, state, context)?;
        state.has_insert_timelapse_gcode = true;
    }
    emit_inner(
        output,
        state,
        filament,
        change::Context {
            traversal: context.traversal,
            tool_ordering: Some(context.tool_ordering),
            metadata: context.metadata,
            first_layer_bounds: context.first_layer_bounds,
            layer_num: context.layer.index as i64,
            print_z: context.layer.z,
            max_z: context.layer.max_z,
            temperature_override: None,
        },
    )
}

pub(super) struct TowerChange {
    pub(super) filament: usize,
    pub(super) print_z: f64,
    pub(super) priming: bool,
    pub(super) temperature_override: Option<i32>,
}

/// `append_tcr2` invokes `set_extruder` at the result's Z, including priming.
pub(super) fn tower(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    context: timelapse::Context<'_>,
    change: TowerChange,
) -> Result<(), SliceError> {
    emit_inner(
        output,
        state,
        change.filament,
        change::Context {
            traversal: context.traversal,
            tool_ordering: Some(context.tool_ordering),
            metadata: context.metadata,
            first_layer_bounds: context.first_layer_bounds,
            layer_num: if change.priming {
                -1
            } else {
                context.layer.index as i64
            },
            print_z: change.print_z,
            max_z: context.layer.max_z,
            temperature_override: change.temperature_override,
        },
    )
}

pub(super) fn type1_config(
    config: &mut value::Config,
    state: &motion::EmitState,
    filament: usize,
    context: timelapse::Context<'_>,
    print_z: f32,
) {
    change::dynamic::insert(
        config,
        state,
        state.extruder_selected.then(|| state.extruders.active_id()),
        filament,
        change::Context {
            traversal: context.traversal,
            tool_ordering: Some(context.tool_ordering),
            metadata: context.metadata,
            first_layer_bounds: context.first_layer_bounds,
            layer_num: context.layer.index as i64,
            print_z: f64::from(print_z),
            max_z: context.layer.max_z,
            temperature_override: None,
        },
    );
}

fn emit_inner(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    filament: usize,
    context: change::Context<'_>,
) -> Result<(), SliceError> {
    if !state.extruders.multiple() {
        append_filament(
            output,
            state,
            filament,
            FilamentTemplate::Start { close_mask: false },
            context,
        )?;
        super::pressure_advance::append(output, state, context.traversal, filament);
        motion::tool_selection::toolchange(output, state, context.traversal, filament, "");
        return Ok(());
    }
    state.toolchange_count += 1;
    motion::retract_for_toolchange(output, state);
    let close_mask = if state.extruder_selected {
        let inserted = append_filament(
            output,
            state,
            state.extruders.active_id(),
            FilamentTemplate::End,
            context,
        )?;
        temperature::pre(output, state, context);
        inserted
    } else {
        false
    };
    let custom = change::append(output, state, filament, context)?;
    motion::tool_selection::toolchange(output, state, context.traversal, filament, &custom);
    // `GCode.cpp:8000-8004` (`set_extruder` tail): SEMM without a prime
    // tower re-emits the nozzle temperature after every filament change —
    // the initial-layer value while `m_layer_index <= 0`, the ordinary one
    // after — through the writer overload that drops the tool index for
    // SEMM (`GCodeWriter.cpp:159-164`).
    if state.options.single_extruder_multi_material
        && !context
            .traversal
            .resolved
            .views
            .full
            .process
            .print
            .enable_prime_tower
            .0
    {
        let filament_settings = &context.traversal.resolved.views.full.filament.print;
        // `m_layer_index <= 0` covers the pre-first-layer (-1) and first
        // layer (0) states; Ares' `layer_index` is a `usize` whose 0 covers
        // both (the startup toolchange emits the initial-layer value under
        // `== 0`, matching the native `<= 0` behavior).
        let values = if state.layer_index == 0 {
            &filament_settings.nozzle_temperature_initial_layer
        } else {
            &filament_settings.nozzle_temperature
        };
        let temperature = values.0.get(filament).unwrap_or(&values.0[0]).0;
        super::temperature::append(
            output,
            context.traversal.resolved.views.runtime_gcode.gcode_flavor,
            temperature as u32,
            false,
            None,
        );
    }
    append_filament(
        output,
        state,
        filament,
        FilamentTemplate::Start { close_mask },
        context,
    )?;
    temperature::post(output, state, context);
    super::pressure_advance::append(output, state, context.traversal, filament);
    state.last_position_defined = false;
    Ok(())
}

enum FilamentTemplate {
    End,
    Start { close_mask: bool },
}

fn append_filament(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    filament: usize,
    invocation: FilamentTemplate,
    context: change::Context<'_>,
) -> Result<bool, SliceError> {
    let start = matches!(invocation, FilamentTemplate::Start { .. });
    let runtime = &context.traversal.resolved.views.runtime_gcode;
    let (name, sources) = if start {
        ("filament_start_gcode", &runtime.filament_start_gcode.0)
    } else {
        ("filament_end_gcode", &runtime.filament_end_gcode.0)
    };
    let source = sources.get(filament).unwrap_or(&sources[0]);
    if source.is_empty() {
        return Ok(false);
    }
    // GCode.cpp:7775-7779 consumes the mask only at an old end invocation.
    let inserted_mask = if !start && let Some(code) = state.filament_instances_code.take() {
        output.extend_from_slice(b"M624 ");
        output.extend_from_slice(&code);
        output.push(b'\n');
        true
    } else {
        false
    };
    let mut config = placeholders::base_config(
        context.traversal,
        context.metadata,
        context.first_layer_bounds,
        Some(&state.template_rng),
        state.total_layer_count,
    )?;
    motion::refresh_writer_state(&mut config, state);
    if start {
        placeholders::refresh_filament_aliases(&mut config, filament);
        config.insert("current_extruder", value::Value::Integer(filament as i64));
    }
    config.insert("layer_num", value::Value::Integer(context.layer_num));
    config.insert(
        "layer_z",
        value::Value::number(state.writer_z.unwrap_or(0.0) - state.options.z_offset),
    );
    config.insert("max_layer_z", value::Value::number(context.max_z));
    let template_filament = if start {
        filament as i64
    } else {
        // The end-template argument is the old physical extruder ID; start
        // uses the new logical filament ID (`GCode.cpp:7770,8027`).
        let map = &context
            .traversal
            .resolved
            .views
            .full
            .project
            .gcode
            .filament_map
            .0;
        map.get(filament).map_or(0, |value| i64::from(value.0 - 1))
    };
    config.insert(
        "filament_extruder_id",
        value::Value::Integer(template_filament),
    );
    let rendered = match template::render_for_filament(source, &mut config, filament) {
        Ok(rendered) => match motion::apply_writer_state(name, &config, &rendered, state) {
            Ok(()) => Ok(rendered),
            Err(error) => Err(error),
        },
        Err(error) => {
            let error = error.to_string();
            state.record_template_failure(name, &error);
            Err(error)
        }
    };
    match rendered {
        Ok(rendered) => {
            output.extend_from_slice(rendered.as_bytes());
            if !rendered.is_empty() && !rendered.ends_with('\n') {
                output.push(b'\n');
            }
        }
        Err(error) => output.extend_from_slice(
            format!("\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n").as_bytes(),
        ),
    }
    if let FilamentTemplate::Start { close_mask: true } = invocation {
        output.extend_from_slice(b"M625\n");
    }
    Ok(inserted_mask)
}
