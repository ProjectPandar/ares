//! Common dynamic fields from `GCode::set_extruder` (`7858–7898`).

mod flush;
mod travel;

use flush::{FlushContext, insert_flush};

use super::{
    Context, motion,
    value::{Config, Value},
};

pub(in crate::project_slice::gcode_emit::tool_transition) fn insert(
    config: &mut Config,
    state: &motion::EmitState,
    old: Option<usize>,
    filament: usize,
    context: Context<'_>,
) {
    let full = &context.traversal.resolved.views.full;
    let runtime = &context.traversal.resolved.views.runtime_gcode;
    let map = &full.project.gcode.filament_map.0;
    // Static BBL hotend selectors are -1, distinct from physical nozzle IDs
    // (`GCode.cpp:223-247,11068-11082`).
    let hotend = |id: usize| {
        if full
            .printer
            .remaining
            .printer_model
            .0
            .starts_with("Bambu Lab")
        {
            -1
        } else {
            extruder_of(map, id) as i64
        }
    };
    for (name, number) in [
        ("previous_extruder", old.map_or(-1, |id| id as i64)),
        ("next_extruder", filament as i64),
        ("current_hotend", old.map_or(-1, hotend)),
        ("next_hotend", hotend(filament)),
        ("current_filament_id", old.map_or(-1, |id| id as i64)),
        ("next_filament_id", filament as i64),
        (
            "current_nozzle_id",
            old.map_or(-1, |id| state.initial_filaments.nozzle_id(id)),
        ),
        (
            "next_nozzle_id",
            state.initial_filaments.nozzle_id(filament),
        ),
        ("layer_num", context.layer_num),
        ("toolchange_count", state.toolchange_count as i64),
        ("fan_speed", 0),
    ] {
        config.insert(name, Value::Integer(number));
    }
    config.insert(
        "relative_e_axis",
        Value::OptionBool(runtime.use_relative_e_distances.0),
    );
    for (name, number) in [
        ("layer_z", context.print_z),
        ("toolchange_z", context.print_z),
        ("max_layer_z", context.max_z),
        ("x_after_toolchange", state.x),
        ("y_after_toolchange", state.y),
        ("z_after_toolchange", state.writer_z.unwrap_or(0.0)),
        (
            "old_retract_length",
            old.map_or(0.0, |id| float(&runtime.retraction_length, id)),
        ),
        (
            "new_retract_length",
            float(&runtime.retraction_length, filament),
        ),
        (
            "old_retract_length_toolchange",
            old.map_or(0.0, |id| float(&runtime.retract_length_toolchange, id)),
        ),
        (
            "new_retract_length_toolchange",
            float(&runtime.retract_length_toolchange, filament),
        ),
    ] {
        config.insert(name, Value::number(number));
    }
    let print = &full.filament.print;
    let temps = if state.on_first_layer {
        &print.nozzle_temperature_initial_layer
    } else {
        &print.nozzle_temperature
    };
    let new_temps = if state.on_first_layer || context.print_z.abs() < 1e-4 {
        &print.nozzle_temperature_initial_layer
    } else {
        &print.nozzle_temperature
    };
    config.insert(
        "old_filament_temp",
        Value::Integer(old.map_or(0, |id| int(temps, id))),
    );
    config.insert(
        "new_filament_temp",
        Value::Integer(
            context
                .temperature_override
                .map_or_else(|| int(new_temps, filament), i64::from),
        ),
    );
    let diameter = runtime
        .filament_diameter
        .0
        .get(filament)
        .unwrap_or(&runtime.filament_diameter.0[0])
        .0;
    let area = (std::f64::consts::FRAC_PI_4 * diameter.powi(2)) as f32;
    let speed = |id| {
        let maximum = &full.filament.gcode.filament_max_volumetric_speed.0;
        let speed = (60.0 * maximum.get(id).unwrap_or(&maximum[0]).0 / f64::from(area)) as i64;
        if speed == 0 { 100 } else { speed }
    };
    config.insert(
        "old_filament_e_feedrate",
        Value::Integer(old.map_or(200, speed)),
    );
    config.insert("new_filament_e_feedrate", Value::Integer(speed(filament)));
    // `z_hop_types[filament]` inside the template reads the merged
    // (printer base + `filament_z_hop_types` overlay) per-filament vector
    // as enum indices — `PrintConfig.cpp:7336-7390` moves it into the
    // filament option set and `GCode.cpp:800` compares against
    // `ZHopType` ordinals.
    let z_hop_types: Vec<Value> = runtime
        .z_hop_types
        .0
        .iter()
        .map(|value| Value::Integer(i64::from(*value as u8)))
        .collect();
    config.insert("z_hop_types", Value::List(z_hop_types));
    insert_flush(
        config,
        FlushContext {
            state,
            traversal: context.traversal,
            old,
            filament,
            filament_area: area,
        },
    );
}

fn float(values: &crate::OrcaFloats, id: usize) -> f64 {
    f64::from(values.0.get(id).unwrap_or(&values.0[0]).0 as f32)
}

/// `Print::get_extruder_id` (`Print.cpp:3280-3288`): `filament_map[id]-1`
/// (1-based map), or 0 when the filament id is outside the map.
pub(super) fn extruder_of(map: &[crate::OrcaInt], filament: usize) -> usize {
    if filament < map.len() {
        (map[filament].0 - 1) as usize
    } else {
        0
    }
}

fn int(values: &crate::OrcaInts, id: usize) -> i64 {
    i64::from(values.0.get(id).unwrap_or(&values.0[0]).0)
}
