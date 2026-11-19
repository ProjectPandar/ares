//! Non-tower `wipe_volume`/`flush_length` family from `GCode::set_extruder`
//! (`GCode.cpp:11151-11207, 11263-11341`). `wipe_volume` comes from the flush
//! matrix for the new filament's physical extruder, scaled by
//! `flush_multiplier[new_extruder]` and reduced by `grab_length*2.4`;
//! `wipe_length = wipe_volume / filament_area`, `flush_count =
//! min(4, round(wipe_volume / 135))`, `flush_length_{1..count}` share
//! `wipe_length / flush_count` and the remaining slots are zeroed.
//! Tower-branch values (`tcr.purge_volume`, `wipe_avoid_*` from the rotated
//! tower bbox, `is_prime_tower_interface`) are deferred with the tower
//! slice — the non-tower branch writes `is_prime_tower_interface=false`,
//! `wipe_avoid_perimeter=false` and `wipe_avoid_pos_x=110` (`:7895-7897`).

use super::{Config, Value, extruder_of, travel::travel_points};
use crate::project_slice::{
    gcode_emit::motion, perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(super) struct FlushContext<'a> {
    pub state: &'a motion::EmitState,
    pub traversal: &'a PreparedPostClassicTraversal,
    pub old: Option<usize>,
    pub filament: usize,
    pub filament_area: f32,
}
pub(super) fn insert_flush(config: &mut Config, context: FlushContext<'_>) {
    let FlushContext {
        state,
        traversal,
        old,
        filament,
        filament_area,
    } = context;
    const PURGE_VOLUME_ONE_TIME: f32 = 135.0; // g_purge_volume_one_time, GCode.cpp:209
    const MAX_FLUSH_COUNT: usize = 4; // g_max_flush_count

    let full = &traversal.resolved.views.full;
    // `number_of_extruders` upstream is `filament_colour.size()` — the
    // logical filament count (`GCode.cpp:11160`).
    let filament_count = full.filament.gcode.filament_colour.0.len();
    // `get_extruder_id(filament)` — `filament_map` is 1-based.
    let map = &full.project.gcode.filament_map.0;
    let new_extruder = extruder_of(map, filament);

    let wipe_volume = match old {
        None => 0.0_f32,
        Some(old_filament) => {
            // `get_flush_volumes_matrix(values, new_extruder_id,
            // nozzle_nums)` slices a `filament_count^2` block per nozzle.
            // `get_flush_volumes_matrix(values, new_extruder_id,
            // nozzle_nums)` slices a `filament_count^2` block per nozzle;
            // out-of-range is an upstream assert, not a fallback.
            let matrix = &full.project.print;
            let values = &matrix.flush_volumes_matrix.0;
            let base = new_extruder * filament_count * filament_count;
            let cell = |from: usize, to: usize| values[base + from * filament_count + to] as f32;
            let old_extruder = extruder_of(map, old_filament);
            let volume = if old_extruder != new_extruder {
                // `GCode.cpp:11178-11187`: filament last loaded in the new
                // extruder, or 0 when it was never used.
                match state.extruders.loaded_filament(new_extruder) {
                    None => 0.0,
                    Some(old_in_new) => cell(old_in_new, filament),
                }
            } else {
                cell(old_filament, filament)
            };
            // `get_at(i)` — `Config.hpp:676-680`: `values[i]`, else
            // `values.front()`; non-empty is a prepare-stage invariant.
            let multiplier = matrix
                .flush_multiplier
                .0
                .get(new_extruder)
                .unwrap_or(&matrix.flush_multiplier.0[0])
                .0;
            // Upstream widens the matrix cell before multiplying, then stores a float.
            let volume = (f64::from(volume) * multiplier) as f32;
            let grab = full
                .printer
                .remaining
                .grab_length
                .0
                .get(new_extruder)
                .unwrap_or(&full.printer.remaining.grab_length.0[0])
                .0;
            let grab_purge_volume = (grab * 2.4) as f32;
            (volume - grab_purge_volume).max(0.0)
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
    // `GCode.cpp:11329-11330`: `flush_unit` divides by the rounded count
    // directly; the `flush_length_*` loop never reads it when the count
    // is zero, matching upstream's unused-NaN/inf case.
    let flush_count = MAX_FLUSH_COUNT.min((wipe_volume / PURGE_VOLUME_ONE_TIME).round() as usize);
    let flush_unit = wipe_length / flush_count as f32;
    for index in 0..MAX_FLUSH_COUNT {
        let value = if index < flush_count { flush_unit } else { 0.0 };
        config.insert(
            format!("flush_length_{}", index + 1),
            Value::number(f64::from(value)),
        );
    }
    // `travel_point_{1..3}` is the path upstream computes once per print
    // in `get_path_of_change_filament` (`GCode.cpp:110-197`, bound at
    // `:7888-7894`): a three-point detour around the model's X-extent to
    // reach `start_end_points[1]` (the wipe/cutter station) without
    // crossing printed objects. Guarded by `m_writer.multiple_extruders`
    // (`:3213`) — any filament id > 0.
    let travel = if state.extruders.multiple() {
        travel_points(traversal)
    } else {
        [(0.0, 0.0); 3]
    };
    for (index, (x, y)) in travel.iter().enumerate() {
        config.insert(format!("travel_point_{}_x", index + 1), Value::number(*x));
        config.insert(format!("travel_point_{}_y", index + 1), Value::number(*y));
    }
    config.insert("wipe_avoid_pos_x", Value::number(110.0));
    config.insert("wipe_avoid_perimeter", Value::OptionBool(false));
    config.insert("is_prime_tower_interface", Value::OptionBool(false));
    let interface = &full.filament.gcode;
    config.insert(
        "filament_tower_interface_purge_volume",
        Value::number(
            interface
                .filament_tower_interface_purge_volume
                .0
                .get(filament)
                .unwrap_or(&interface.filament_tower_interface_purge_volume.0[0])
                .0,
        ),
    );
    let mut interface_temp = interface
        .filament_tower_interface_print_temp
        .0
        .get(filament)
        .unwrap_or(&interface.filament_tower_interface_print_temp.0[0])
        .0;
    if interface_temp == -1 {
        interface_temp = full
            .filament
            .print
            .nozzle_temperature_range_high
            .0
            .get(filament)
            .unwrap_or(&full.filament.print.nozzle_temperature_range_high.0[0])
            .0;
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
