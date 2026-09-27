//! `GCode.cpp::change_layer` boundary emission: the layer-change labels,
//! retraction deferral and state advance that open every layer chunk.
#[cfg(test)]
mod tests;

use super::super::{
    GenerationMetadata, PreparedPostClassicTraversal, SliceError, footprint,
    format_processor_float, layer_gcode, machine, motion, spiral_vase, timelapse,
};

pub(super) struct Boundary<'a> {
    pub traversal: &'a PreparedPostClassicTraversal,
    pub layer_change_template: &'a layer_gcode::LayerChangeTemplate,
    pub metadata: GenerationMetadata,
    pub first_layer_bounds: Option<footprint::FirstLayerBounds>,
}

/// Advances the emission state into the new layer. `change_layer`
/// increments `m_layer_index` BEFORE its change-layer retract
/// (`GCode.cpp:5690-5696`), so the deferred hop's
/// `retract_lift_enforce` bottom/top gate must see the NEW layer
/// index — the index advance strictly precedes the deferral.
pub(in crate::project_slice::gcode_emit::layers) fn advance_for_layer(
    state: &mut motion::EmitState,
    precise_layer_z: f64,
    layer_z: f64,
    layer_index: usize,
    object_layer_over_raft: bool,
    layer_retract_pending: bool,
) {
    // `GCode.cpp:5692`: `z = print_z + m_config.z_offset` — the writer's
    // Z domain is offset; `source_layer_z`/`;Z:`/comments stay model z.
    state.layer_z = f64::from(layer_z) + state.options.z_offset;
    state.source_layer_z = precise_layer_z;
    state.layer_index = layer_index;
    state.object_layer_over_raft = object_layer_over_raft;
    // `change_layer` retracts while the writer still sits at the previous
    // layer's z (`GCode.cpp:5693-5706` precede the z move), so the
    // deferred-lift gate must see the OLD `m_pos.z()`; only after the
    // deferral does `change_layer` silently set `m_pos.z() = z`
    // (`GCode.cpp:5712`), which post-change-layer `lazy_lift`/`eager_lift`
    // gating reads as the NEW layer z (`GCodeWriter.cpp:659-662`).
    if layer_retract_pending {
        motion::defer_layer_change_lift(state);
    }
    state.record_z(state.layer_z);
}

pub(super) struct BoundaryAdvance<'a> {
    pub layer_z: f64,
    pub layer_height: f64,
    pub timelapse_context: timelapse::Context<'a>,
}

/// Emits the `change_layer` block for `layer_index` (`GCode.cpp:5685-5718`
/// and its callers): Z/HEIGHT headers, the before-layer-change template,
/// the deferred change-layer retract, and the per-state layer advance.
pub(super) fn append<'a>(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    spiral: &mut spiral_vase::SpiralVaseFilter,
    Boundary {
        traversal,
        layer_change_template,
        metadata,
        first_layer_bounds,
    }: Boundary<'a>,
    layer_index: usize,
    object_layer_over_raft: bool,
    layer_z: f64,
    layer_height: f64,
    second_layer_done: &mut bool,
    bed_cache: i32,
) -> Result<BoundaryAdvance<'a>, SliceError> {
    state.physical_fan_speed = state.part_fan_speed;
    let tags = state.tags;
    output.extend_from_slice(tags.layer_change().as_bytes());
    output.push(b'\n');
    // `GCode.cpp:4629-4630`: `m_last_layer_z = float(print_z)` and
    // `m_max_layer_z = max(...)` update BEFORE the
    // before-layer-change/timelapse templates render (`GCode.cpp:4636`).
    state.max_layer_z = state.max_layer_z.max(layer_z as f32);
    let header = format!(
        "{}\n{}\n",
        tags.z(&format_processor_float(f64::from(layer_z))),
        tags.height(&format_processor_float(f64::from(layer_height))),
    );
    output.extend_from_slice(header.as_bytes());
    let timelapse_context = timelapse::Context {
        traversal,
        layer: timelapse::TimelapseLayer {
            // `{layer_num}`/`{layer_z}` placeholders and the wrapping
            // insert (`GCode.cpp:783`, :5302) scope the PHYSICAL index and
            // the running max writer z — `m_layer_index` after
            // `change_layer` and `m_max_layer_z` over the offset domain.
            index: layer_index,
            z: f64::from(layer_z),
            // `GCode.cpp:890`: `{max_layer_z}` = `m_max_layer_z`, the
            // float-domain running max over RAW print z (no `z_offset`).
            max_z: f64::from(state.max_layer_z),
        },
        metadata,
        first_layer_bounds,
    };
    let timelapse_at_layer_change = !tags.is_bbl()
        && !traversal
            .resolved
            .views
            .runtime_gcode
            .time_lapse_gcode
            .0
            .is_empty();
    layer_gcode::append_before_layer_change_gcode(
        output,
        state,
        layer_gcode::LayerTemplateContext {
            traversal,
            // `{layer_num}` = `m_layer_index + 1` pre-increment = the new
            // PHYSICAL layer index (`GCode.cpp:4636`, :5689).
            layer_index,
            layer_z: f64::from(layer_z),
            // `{max_layer_z}` = `m_max_layer_z` — float-domain running
            // max over raw print z (`GCode.cpp:4638`), not `layer_z`.
            max_layer_z: f64::from(state.max_layer_z),
            totals: state.extrusion_totals(),
            context: layer_change_template,
        },
    )?;
    // `GCodeWriter::will_move_z` (`GCodeWriter.cpp:916-929`) reads the
    // LIVE writer Z — a before-layer template `position[2]` assignment
    // (`GCode.cpp:3901-3903`) or a previous role hook must participate,
    // so evaluate after the template and before any nominal override
    // semantics. While lifted, a target inside [nominal, pos] does not
    // move; otherwise upstream suppresses only diff STRICTLY below
    // EPSILON (`GCodeWriter.cpp:927-930`), so equality still moves.
    let will_move_z = state.writer_z.is_none_or(|writer_z| {
        let target = f64::from(layer_z) + state.options.z_offset;
        if state.lifted_amount > 0.0 {
            let nominal = writer_z - state.lifted_amount;
            !(target >= nominal && target <= writer_z)
        } else {
            (target - writer_z).abs() >= 1.0e-4
        }
    });
    // Upstream's `change_layer` retract (`retract_when_changing_layer`)
    // also DEFERS the hop (`maybe_zlift`, `GCodeWriter.cpp:626-648`):
    // `m_to_lift` survives into the new layer's first travel, which
    // raises to layer+hop and descends at the target. Capture the
    // flag BEFORE the wipe flush consumes it; defer only here (the
    // c7ea935f lesson: no already-retracted mid-print deferrals).
    // The deferred lift exists only when the change-layer retract ran —
    // same `will_move_z` gate (`GCode.cpp:5693`).
    let layer_retract_pending = state.pending_layer_retract
        && will_move_z
        && state.options.z_hop > 0.0
        && state.options.retraction_length > 0.0
        && !state.lifted
        && state.pending_lift.is_none();
    // `change_layer` retract requires `will_move_z` (`GCode.cpp:5693`) —
    // on non-BBL paths a pending retract whose target Z equals the
    // (possibly template-assigned) writer Z performs no retract at all,
    // so its `restart_extra` assignment must not run either.
    motion::flush_pending_retract_wipe(output, state, will_move_z);
    // Pending object-end labels flush after the layer-change
    // retract/wipe, before the layer-change gcode
    // (`GCode.cpp:5699` `change_layer`).
    motion::append_exclude_end(output, state);
    // The layer-start retraction mirrors the `change_layer`/BBS
    // layer-start retract, which is gated on
    // `retract_when_changing_layer` (`GCode.cpp:5206`, `GCode.cpp:5693`);
    // the compatible-flavor `change_layer` variant additionally
    // requires `will_move_z` — the nozzle must actually change Z
    // from wherever the start g-code left it (`GCode.cpp:5693`).
    // BBL layer starts retract whenever the flag is set.
    if layer_index == 0
        && state.options.retract_when_changing_layer
        && (state.tags.is_bbl() || will_move_z)
    {
        motion::retract_before_layer(output, state);
    }
    // Spiral filter targets live in the writer (offset) Z domain
    // (`GCode.cpp:5692` emits the `print_z + z_offset` move, then
    // `SpiralVase.cpp` rewrites that move and derives its ramp).
    spiral.append_layer_z(
        output,
        layer_index,
        f64::from(layer_z) + state.options.z_offset,
    );
    // `change_layer`'s explicit `travel_to_z` (`GCode.cpp:5702-5704`) and
    // the silent `m_pos.z() = z` both leave the writer at the new layer z —
    // but that z change happens AFTER the deferred-lift gate reads the
    // previous writer z, so `advance_for_layer` below owns the record.
    // A deferred previous-layer retraction lifts above the new layer's
    // print Z (`GCodeWriter::travel_to_z` during layer transition).
    // The lift gate evaluates at the writer's z before the layer move
    // (`GCode.cpp:5690` change_layer retract precedes travel_to_z;
    // `GCodeWriter.cpp:633-639` gates on `m_pos.z()`).
    // The `retract_lift_above/below` gate reads the LIVE writer Z at
    // retract time — including any `position[2]` assigned by the
    // before-layer template (`GCodeWriter.cpp:633-639` reads `m_pos(2)`).
    let live_writer_z = f64::from(state.writer_z.unwrap_or(state.layer_z));
    advance_for_layer(
        state,
        f64::from(layer_z),
        layer_z,
        layer_index,
        object_layer_over_raft,
        layer_retract_pending,
    );
    motion::flush_pending_retract_lift(output, state, live_writer_z, will_move_z);
    if timelapse_at_layer_change {
        // Non-BBL layer-start render (`GCode.cpp:4667`): no position-clear,
        // no Z adoption — those belong to the lambda inserts only. It runs
        // AFTER `change_layer` (`process_layer`, `GCode.cpp:4663-4668`) so
        // `{position[2]}` reads the new layer's Z and assignments survive.
        timelapse::append_untracked(output, state, timelapse_context)?;
    }
    // Upstream renders the layer-change template AFTER `change_layer`
    // finalized writer state (`process_layer` at `GCode.cpp:4667-4685`,
    // after `change_layer` returns at `:5712`), so `{position[2]}` reads
    // the new layer's Z and assignments are not overwritten by the
    // transition bookkeeping.
    layer_gcode::append_layer_change(
        output,
        state,
        layer_gcode::LayerTemplateContext {
            traversal,
            // Post-`change_layer` `{layer_num}` = `m_layer_index` = the
            // new PHYSICAL layer index (`GCode.cpp:4669`, :5689).
            layer_index,
            layer_z: f64::from(layer_z),
            max_layer_z: f64::from(state.max_layer_z),
            totals: state.extrusion_totals(),
            context: layer_change_template,
        },
    )?;
    // The BBL first-layer-inspection scan (`GCode.cpp:4780-4790`) opens
    // between the layer-change gcode and the acceleration reset, so it
    // precedes `begin_layer`'s M204.
    if layer_index == 1
        && !*second_layer_done
        && state.tags.is_bbl()
        && traversal
            .resolved
            .views
            .full
            .printer
            .gcode
            .scan_first_layer
            .0
    {
        motion::scan_first_layer_block(output, state);
    }
    motion::begin_layer(
        output,
        state,
        layer_index,
        f64::from(layer_z),
        f64::from(layer_height),
    );
    // Second-layer transition: bed temperature for the remaining
    // layers (`GCode.cpp:4777-4830`), once per slice.
    if layer_index == 1 && !*second_layer_done {
        *second_layer_done = true;
        machine::append_second_layer_transition(output, traversal, bed_cache);
    }
    // Orca `GCode.cpp:5205-5210`: core-xy BBL printers emit the eager
    // change-layer lift (the second change-layer retract is a no-op
    // for E once retracted) and the timelapse gcode inside the new
    // layer's window, after the layer-change acceleration.
    let bbl_layer_start = state.tags.is_bbl() && !state.traditional_timelapse;
    if bbl_layer_start && state.options.retract_when_changing_layer {
        motion::flush_pending_retract_eager_forced(output, state);
    }
    if bbl_layer_start {
        timelapse::append_and_track(output, state, timelapse_context)?;
    }
    Ok(BoundaryAdvance {
        layer_z,
        layer_height,
        timelapse_context,
    })
}
