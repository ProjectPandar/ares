//! The sequential by-object instance transition
//! (`GCode.cpp:3300-3372`): everything written between the previous
//! object's last layer flush and the next object's first layer.

use super::super::motion;
use super::super::motion::format::z as format_z;
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

/// The transition inputs beyond the writer state (`GCode.cpp:3309-3342`).
pub(in crate::project_slice::gcode_emit) struct Transition<'a> {
    pub(in crate::project_slice::gcode_emit) traversal: &'a PreparedPostClassicTraversal,
    pub(in crate::project_slice::gcode_emit) metadata: crate::GenerationMetadata,
    pub(in crate::project_slice::gcode_emit) first_layer_bounds:
        Option<super::super::footprint::FirstLayerBounds>,
    pub(in crate::project_slice::gcode_emit) finished_objects: usize,
    pub(in crate::project_slice::gcode_emit) initial_extruder: usize,
    pub(in crate::project_slice::gcode_emit) printing_extruders: Vec<usize>,
    pub(in crate::project_slice::gcode_emit) lift_z: f64,
    pub(in crate::project_slice::gcode_emit) origin: [f64; 2],
    /// The live writer bed-temperature cache
    /// (`GCodeWriter::m_last_bed_temperature`, GCodeWriter.cpp:168-175):
    /// `set_bed_temperature` refreshes it even when the custom G-code
    /// suppresses the write.
    pub(in crate::project_slice::gcode_emit) bed_cache: &'a mut i32,
}

/// `GCode.cpp:3309-3342`: the transition block for the instance about to
/// print (`finished_objects > 0`). `lift_z` is
/// `m_max_layer_z + z_hop[initial_extruder]` (:3326); `origin` is the
/// incoming instance's origin — the upstream travels to `(0, 0)` after
/// `set_origin(shift)` (:3303/:3327), which is the shift point in plate
/// coordinates. Returns whether the incoming extruder was primed here
/// (`prime_extruder`, :3310-3325).
pub(in crate::project_slice::gcode_emit) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    transition: Transition<'_>,
) -> Result<bool, crate::SliceError> {
    let Transition {
        traversal,
        metadata,
        first_layer_bounds,
        finished_objects,
        initial_extruder,
        printing_extruders,
        lift_z,
        origin,
        bed_cache,
    } = transition;
    // `GCode.cpp:3310-3325`: `need_toolchange(initial_extruder_id)` primes
    // the incoming extruder before the travels. Ares defers the prime
    // flush to its toolchange machinery; until the multi-filament
    // by-object fixtures land, a priming boundary is an explicit error
    // instead of a silent retract.
    let prime_extruder = state.extruders.active_id() != initial_extruder;
    if prime_extruder {
        return Err(crate::SliceError::InvalidInput(
            "sequential by-object printing with a priming toolchange at the object boundary is not supported yet".into(),
        ));
    }
    // GCode.cpp:3298 sets the incoming instance frame before retract/travel.
    motion::set_origin(state, (origin[0], origin[1]));
    motion::tower::transition_retract(output, state);
    // `GCode.cpp:3326`: the safety lift above every printed layer.
    motion::tower::writer_z(output, state, lift_z, "", true);
    // `GCode.cpp:3327` runs `GCode::travel_to` (GCode.cpp:7350) for the
    // origin move: it selects the travel acceleration/jerk BEFORE the
    // retraction handling and the hop.
    motion::begin_path_travel(output, state, "", f64::INFINITY);
    // `GCode.cpp:7432-7462`: the retraction handling fires before the
    // label flush (:7465). The extruder is already retracted (the block's
    // own `retract()` did it), so the E part is a no-op, but the lift
    // still runs through `lazy_lift` (GCodeWriter.cpp:626-648) — and its
    // materialization point splits on vase mode:
    //
    // * spiral vase materializes instantly inside the retraction
    //   (`spiral_vase=true` → `_travel_to_z` "lift Z") — before the label
    //   flush; then `travel_to_xyz` branch 3 (GCodeWriter.cpp:785+) takes
    //   the finished object's nominal Z as the destination (`z ==
    //   DBL_MAX ? m_nominal_z`, GCode.cpp:7483) and the combined move
    //   cancels the lift (the native `G1 Z7 F42000` / `G1 X50 Y40 Z6.2
    //   F42000`), so the incoming instance's first-path travel performs
    //   its own entry hop pair.
    // * otherwise the lift defers once inside the block's own retract
    //   (`GCode.cpp:3308` `transition_retract` → `lazy_lift`,
    //   GCodeWriter.cpp:626-648, band checked at that moment's
    //   `m_pos.z()`); the travel never re-evaluates the band — it only
    //   consumes `m_to_lift`. With a deferral queued, `travel_to_xyz`
    //   branch 1 (GCodeWriter.cpp:700-737) raises the destination to
    //   `m_pos(2) + m_to_lift`, materializes the slop `G1 Z4.8 F9000 ;
    //   normal lift Z` and travels there — the arrival keeps the nozzle
    //   lifted (`m_lifted = z_hop`), so the incoming instance's first
    //   travel descends in the combined move without a new hop. With
    //   none queued (e.g. the retract's `m_pos.z()` sat outside the
    //   `retract_lift_above/below` band), the travel takes the
    //   nominal-Z destination like the vase branch.
    let hop_z = lift_z + state.options.z_hop;
    // `GCode.cpp:3322`: forward the sequential origin travel description.
    let comment = if state.options.gcode_comments {
        " ; move to origin position for next object"
    } else {
        ""
    };
    let feedrate = motion::tower::travel_feedrate(state);
    let destination_z;
    if state.spiral_vase {
        motion::tower::writer_z(output, state, hop_z, "lift Z", true);
        // GCode.cpp:7466-7467 consumes pending labels after the
        // retraction handling, before the travel move.
        motion::append_object_start(output, state);
        destination_z = state.layer_z;
        // Branch 3 ends at the nominal Z with the lift cancelled
        // (`m_lifted = 0`, GCodeWriter.cpp:810-815).
        state.lifted = false;
        state.lifted_amount = 0.0;
    } else {
        // GCode.cpp:7466-7467 consumes pending labels before the
        // deferred lift materializes in the travel's branch 1. The defer
        // decision itself happened once inside the block's retract
        // (`transition_retract` → `schedule_normal`, gated by
        // `is_allowed_at` at that writer z) — the travel only consumes
        // `m_to_lift`: queued → branch 1 hop, absent → nominal Z.
        motion::append_object_start(output, state);
        if state.pending_lift.is_some() {
            motion::tower::writer_z(output, state, hop_z, "normal lift Z", true);
            destination_z = hop_z;
            // Branch 1 keeps the arrival lifted (`m_lifted = z_hop`,
            // GCodeWriter.cpp:701-710).
            state.lifted = true;
            state.lifted_amount = state.options.z_hop;
        } else {
            // No deferred lift: `travel_to_xyz` keeps the finished
            // object's nominal Z (`z == DBL_MAX ? m_nominal_z`,
            // GCode.cpp:7483) and the arrival cancels the lift.
            destination_z = state.layer_z;
            state.lifted = false;
            state.lifted_amount = 0.0;
        }
    }
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} Z{} F{}{comment}\n",
            format_z(state.offset.0 - state.xy_offset.0),
            format_z(state.offset.1 - state.xy_offset.1),
            format_z(destination_z),
            motion::tower::format_feedrate(feedrate)
        )
        .as_bytes(),
    );
    state.x = state.offset.0;
    state.y = state.offset.1;
    // GCode.cpp:3322/7506: travel_to ends at (0, 0) in the incoming frame.
    state.last_scaled_position = (0, 0);
    state.last_position_defined = true;
    state.positioned = true;
    state.current_feedrate = feedrate;
    // The move's destination is the live writer Z in both branches.
    state.record_z(destination_z);
    state.template_lifted = false;
    // `GCode.cpp:3316-3326`: the block's explicit retract precedes the
    // origin travel — the deferred slot is consumed by it, not re-fired
    // on the incoming instance's first travel.
    state.pending_lift = None;
    // `GCode.cpp:3330`: the instance's first travel runs once without the
    // crossing avoidance — the direct combined move to the first print
    // point (the native `G1 X… Y… Z.2 F9000`).
    state.avoid_crossing_disabled_once = true;
    // `GCode.cpp:3333-3335`: the placeholder parser carries
    // `current_object_idx` while processing printing_by_object_gcode.
    let settings = &traversal.resolved.views.full;
    let raw = &settings.printer.gcode.printing_by_object_gcode.0;
    let mut config = super::super::placeholders::base_config(
        traversal,
        metadata,
        first_layer_bounds,
        Some(&state.template_rng),
        state.total_layer_count,
    )?;
    motion::refresh_writer_state(&mut config, state);
    config.insert(
        "current_object_idx",
        super::super::value::Value::Integer(finished_objects as i64),
    );
    let name = "printing_by_object_gcode";
    let rendered = match super::super::template::render(raw, &mut config) {
        Ok(rendered) => {
            motion::invalidate_after_custom_gcode(&rendered, state);
            rendered
        }
        Err(error) => {
            state.record_template_failure(name, &error);
            format!(
                "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
            )
        }
    };
    // `GCode.cpp:3337` (`_print_first_layer_bed_temperature`): the
    // first-layer bed temperature, skipped when the custom G-code already
    // sets it or the writer cache holds it (`set_bed_temperature`).
    // `GCodeWriter::set_bed_temperature` (`GCodeWriter.cpp:168-175`)
    // refreshes the writer cache even when the custom G-code suppresses
    // the write, so the NEXT object's transition compares against the
    // live value (the native skips object 2's M140 because object 1's
    // second layer already wrote it).
    let include_g10 = settings.printer.gcode.gcode_flavor == crate::GCodeFlavor::RepRapFirmware;
    let bed_set_by_gcode = machine_sets_temperature(&rendered, &["M140", "M190"], false);
    let bed_temperature = super::super::machine::first_layer_bed_temperature(
        traversal,
        initial_extruder,
        state.initial_filaments.first_layer_tools(),
    );
    if bed_temperature != *bed_cache {
        // No `> 0` guard on the bed path: `set_bed_temperature` emits
        // M140 S0 when the cache holds a nonzero value (the nozzle path
        // keeps its own `temp > 0` gate, `GCode.cpp:4058-4064`).
        if !bed_set_by_gcode {
            output.extend_from_slice(
                format!("M140 S{bed_temperature} ; set bed temperature\n").as_bytes(),
            );
        }
        *bed_cache = bed_temperature;
    }
    // `GCode.cpp:3338` (`_print_first_layer_extruder_temperatures`): the
    // first-layer nozzle temperatures without waiting — the current
    // extruder only under SEMM/ooze prevention, otherwise every printing
    // extruder with the ooze standby adjustment.
    let nozzle_set_by_gcode = machine_sets_temperature(&rendered, &["M104", "M109"], include_g10);
    if !nozzle_set_by_gcode {
        let single_extruder_multi_material = traversal
            .resolved
            .views
            .runtime_gcode
            .single_extruder_multi_material
            .0;
        let ooze_prevention = settings.process.print.ooze_prevention.0;
        let multiple_extruders = state.extruders.multiple() && !single_extruder_multi_material;
        if single_extruder_multi_material || ooze_prevention {
            write_nozzle_temperature(
                output,
                initial_extruder,
                super::super::machine::initial_layer_nozzle_temperature(settings, initial_extruder),
                multiple_extruders,
            );
        } else {
            for &tool in &printing_extruders {
                let temperature = ooze_adjusted(
                    settings,
                    super::super::machine::initial_layer_nozzle_temperature(settings, tool),
                    tool,
                    initial_extruder,
                    ooze_prevention,
                );
                write_nozzle_temperature(output, tool, temperature, multiple_extruders);
            }
        }
    }
    // `GCodeOutputStream::writeln` (`GCode.cpp:6266-6270`): nothing for an
    // empty rendered template, otherwise exactly one trailing newline.
    if !rendered.is_empty() {
        output.extend_from_slice(rendered.as_bytes());
        if !rendered.ends_with('\n') {
            output.push(b'\n');
        }
    }
    Ok(prime_extruder)
}

fn write_nozzle_temperature(
    output: &mut Vec<u8>,
    tool: usize,
    temperature: i32,
    multiple_extruders: bool,
) {
    if temperature <= 0 {
        return;
    }
    let tool_suffix = if multiple_extruders {
        format!(" T{tool}")
    } else {
        String::new()
    };
    output.extend_from_slice(
        format!("M104 S{temperature}{tool_suffix} ; set nozzle temperature\n").as_bytes(),
    );
}

/// The ooze-prevention standby adjustment
/// (`GCode.cpp:4087-4094`: the configured idle temperature wins over the
/// standby delta).
fn ooze_adjusted(
    settings: &crate::ProjectSettings,
    temperature: i32,
    tool: usize,
    initial_extruder: usize,
    ooze_prevention: bool,
) -> i32 {
    if !(ooze_prevention && tool != initial_extruder) {
        return temperature;
    }
    let idle = settings
        .filament
        .print
        .idle_temperature
        .0
        .get(tool)
        .map(|value| value.0)
        .unwrap_or(0);
    if idle != 0 {
        idle
    } else {
        temperature + settings.process.print.standby_temperature_delta.0
    }
}

/// `custom_gcode_sets_temperature` (`GCode.cpp:6212-6240`): whether the
/// rendered custom G-code already controls the temperature with the given
/// M-codes.
fn machine_sets_temperature(gcode: &str, codes: &[&str], include_g10: bool) -> bool {
    gcode.lines().any(|line| {
        let trimmed = line.trim_start();
        codes.iter().any(|code| trimmed.starts_with(code))
            || (include_g10 && trimmed.starts_with("G10") && trimmed.contains('S'))
    })
}
