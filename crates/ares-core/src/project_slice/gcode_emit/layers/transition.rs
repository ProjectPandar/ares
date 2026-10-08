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
    pub(in crate::project_slice::gcode_emit) bed_cache: i32,
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
    motion::tower::transition_retract(output, state);
    // `GCode.cpp:3326`: the safety lift above every printed layer.
    motion::tower::writer_z(output, state, lift_z, "", true);
    // `GCode.cpp:3327-3330`: the travel to the incoming instance's origin
    // with the external motion planner once; the writer lifts again for
    // the planar move (the native pair `G1 Z10.4` / `G1 Z10.8` /
    // `G1 X… Y… Z10.8`).
    let hop_z = lift_z + state.options.z_hop;
    motion::tower::writer_z(output, state, hop_z, "", true);
    let feedrate = motion::tower::travel_feedrate(state);
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} Z{} F{}\n",
            format_z(origin[0]),
            format_z(origin[1]),
            format_z(hop_z),
            motion::tower::format_feedrate(feedrate)
        )
        .as_bytes(),
    );
    state.x = origin[0];
    state.y = origin[1];
    state.positioned = true;
    // The transition's hop leaves the nozzle lifted above the layer
    // domain (`GCodeWriter` lifted state): the incoming instance's
    // layer change then skips its own lift and the first travel merges
    // the descent into the combined move (`GCodeWriter.cpp:782-806`,
    // the Ares `cancelled_lift` branch in `move_emit.rs`).
    state.lifted = true;
    state.lifted_amount = state.options.z_hop;
    // The transition's toolchange retract ran before the hop and its
    // tail scheduled a deferred lift for the pre-hop (zero-lift) state;
    // the explicit `travel_to_z` hop above has already materialized it
    // (`GCode.cpp:3316-3326`: the retract precedes the explicit hop —
    // the lazy slot is consumed by it, not re-fired on the incoming
    // instance's first travel).
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
    )?;
    motion::refresh_writer_state(&mut config, state);
    config.insert(
        "current_object_idx",
        super::super::value::Value::number(finished_objects as f64),
    );
    let name = "printing_by_object_gcode";
    let rendered = match super::super::template::render(raw, &mut config) {
        Ok(rendered) => rendered,
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
    let include_g10 = settings.printer.gcode.gcode_flavor == crate::GCodeFlavor::RepRapFirmware;
    let bed_set_by_gcode = machine_sets_temperature(&rendered, &["M140", "M190"], false);
    let bed_temperature = super::super::machine::first_layer_bed_temperature(traversal);
    if !bed_set_by_gcode && bed_temperature != bed_cache && bed_temperature > 0 {
        output.extend_from_slice(
            format!("M140 S{bed_temperature} ; set bed temperature\n").as_bytes(),
        );
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
