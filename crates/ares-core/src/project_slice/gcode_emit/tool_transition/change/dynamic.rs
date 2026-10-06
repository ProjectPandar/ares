//! Common dynamic fields from `GCode::set_extruder` (`7858–7898`).

use super::{
    Context, motion,
    value::{Config, Value},
};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) fn insert(
    config: &mut Config,
    state: &motion::EmitState,
    old: Option<usize>,
    filament: usize,
    context: Context<'_>,
) {
    let full = &context.traversal.resolved.views.full;
    let runtime = &context.traversal.resolved.views.runtime_gcode;
    let map = &full.project.gcode.filament_map.0;
    let hotend = |id: usize| {
        if full.printer.remaining.printer_model.0 == "Bambu Lab X2D" {
            -1
        } else {
            map.get(id).map_or(0, |value| value.0 - 1)
        }
    };
    for (name, number) in [
        ("previous_extruder", old.map_or(-1, |id| id as i64)),
        ("next_extruder", filament as i64),
        ("current_hotend", old.map_or(-1, |id| i64::from(hotend(id)))),
        ("next_hotend", i64::from(hotend(filament))),
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

/// Non-tower `wipe_volume`/`flush_length` family from `GCode::set_extruder`
/// (`GCode.cpp:7814-7853, 7884-7948`). `wipe_volume` comes from the flush
/// matrix for the new filament's physical extruder, scaled by
/// `flush_multiplier[new_extruder]` and reduced by `grab_length*2.4`;
/// `wipe_length = wipe_volume / filament_area`, `flush_count =
/// min(4, round(wipe_volume / 135))`, `flush_length_{1..count}` share
/// `wipe_length / flush_count` and the remaining slots are zeroed.
/// Tower-branch values (`tcr.purge_volume`, `wipe_avoid_*` from the rotated
/// tower bbox, `is_prime_tower_interface`) are deferred with the tower
/// slice — the non-tower branch writes `is_prime_tower_interface=false`,
/// `wipe_avoid_perimeter=false` and `wipe_avoid_pos_x=110` (`:7895-7897`).
struct FlushContext<'a> {
    state: &'a motion::EmitState,
    traversal: &'a PreparedPostClassicTraversal,
    old: Option<usize>,
    filament: usize,
    filament_area: f32,
}

fn insert_flush(config: &mut Config, context: FlushContext<'_>) {
    let FlushContext {
        state,
        traversal,
        old,
        filament,
        filament_area,
    } = context;
    const PURGE_VOLUME_ONE_TIME: f32 = 135.0; // g_purge_volume_one_time, GCode.cpp:93
    const MAX_FLUSH_COUNT: usize = 4; // g_max_flush_count

    let full = &traversal.resolved.views.full;
    // `number_of_extruders` upstream is `filament_colour.size()` — the
    // logical filament count (`GCode.cpp:7811`).
    let filament_count = full.filament.gcode.filament_colour.0.len().max(1);
    let nozzle_count = full.project.print.nozzle_diameter.0.len().max(1);
    // `get_extruder_id(filament)` — `filament_map` is 1-based.
    let map = &full.project.gcode.filament_map.0;
    let extruder_of = |id: usize| map.get(id).map_or(0, |value| (value.0 - 1).max(0) as usize);
    let new_extruder = extruder_of(filament);

    let wipe_volume = match old {
        None => 0.0_f32,
        Some(old_filament) => {
            // `get_flush_volumes_matrix(values, new_extruder_id,
            // nozzle_nums)` slices a `filament_count^2` block per nozzle.
            let matrix = &full.project.print;
            let values = &matrix.flush_volumes_matrix.0;
            let base = new_extruder.min(nozzle_count - 1) * filament_count * filament_count;
            let cell = |from: usize, to: usize| {
                values
                    .get(base + from * filament_count + to)
                    .map_or(0.0, |value| *value as f32)
            };
            let old_extruder = extruder_of(old_filament);
            let mut volume = if old_extruder != new_extruder {
                // `GCode.cpp:7827-7836`: filament last loaded in the new
                // extruder, or 0 when it was never used.
                match state.extruders.loaded_filament(new_extruder) {
                    None => 0.0,
                    Some(old_in_new) => cell(old_in_new, filament),
                }
            } else {
                cell(old_filament, filament)
            };
            volume *= matrix
                .flush_multiplier
                .0
                .get(new_extruder)
                .or_else(|| matrix.flush_multiplier.0.first())
                .map_or(1.0, |value| value.0 as f32);
            let grab = full
                .printer
                .remaining
                .grab_length
                .0
                .get(new_extruder)
                .or_else(|| full.printer.remaining.grab_length.0.first())
                .map_or(0.0, |value| value.0 as f32);
            (volume - grab * 2.4).max(0.0)
        }
    };
    let wipe_length = wipe_volume / filament_area;
    config.insert(
        "first_flush_volume",
        Value::number(f64::from(wipe_length / 2.0)),
    );
    config.insert(
        "second_flush_volume",
        Value::number(f64::from(wipe_length / 2.0)),
    );
    config.insert("flush_length", Value::number(f64::from(wipe_length)));
    let flush_count = MAX_FLUSH_COUNT.min((wipe_volume / PURGE_VOLUME_ONE_TIME).round() as usize);
    let flush_unit = wipe_length / flush_count.max(1) as f32;
    for index in 0..MAX_FLUSH_COUNT {
        let value = if index < flush_count { flush_unit } else { 0.0 };
        config.insert(
            format!("flush_length_{}", index + 1),
            Value::number(f64::from(value)),
        );
    }
    for (name, value) in [
        ("travel_point_1_x", 0.0),
        ("travel_point_1_y", 0.0),
        ("travel_point_2_x", 0.0),
        ("travel_point_2_y", 0.0),
        ("travel_point_3_x", 0.0),
        ("travel_point_3_y", 0.0),
        ("wipe_avoid_pos_x", 110.0),
    ] {
        config.insert(name, Value::number(value));
    }
    config.insert("wipe_avoid_perimeter", Value::OptionBool(false));
    config.insert("is_prime_tower_interface", Value::OptionBool(false));
    let interface = &full.filament.gcode;
    config.insert(
        "filament_tower_interface_purge_volume",
        Value::number(f64::from(
            interface
                .filament_tower_interface_purge_volume
                .0
                .get(filament)
                .or_else(|| interface.filament_tower_interface_purge_volume.0.first())
                .map_or(0.0, |value| value.0) as f32,
        )),
    );
    let mut interface_temp = interface
        .filament_tower_interface_print_temp
        .0
        .get(filament)
        .or_else(|| interface.filament_tower_interface_print_temp.0.first())
        .map_or(-1, |value| value.0);
    if interface_temp == -1 {
        interface_temp = full
            .filament
            .print
            .nozzle_temperature_range_high
            .0
            .get(filament)
            .or_else(|| full.filament.print.nozzle_temperature_range_high.0.first())
            .map_or(0, |value| value.0);
    }
    config.insert(
        "filament_tower_interface_print_temp",
        Value::Integer(i64::from(interface_temp)),
    );
    // `:7921-7931`: the list is resized to `filament_type` length and the
    // non-tower branch zeroes it unconditionally.
    let cooling_len = full
        .filament
        .gcode
        .filament_type
        .0
        .len()
        .max(interface.filament_cooling_before_tower.len());
    config.insert(
        "filament_cooling_before_tower",
        Value::List(
            std::iter::repeat_with(|| Value::number(0.0))
                .take(cooling_len)
                .collect(),
        ),
    );
}

fn float(values: &crate::OrcaFloats, id: usize) -> f64 {
    f64::from(values.0.get(id).unwrap_or(&values.0[0]).0 as f32)
}

fn int(values: &crate::OrcaInts, id: usize) -> i64 {
    i64::from(values.0.get(id).unwrap_or(&values.0[0]).0)
}
