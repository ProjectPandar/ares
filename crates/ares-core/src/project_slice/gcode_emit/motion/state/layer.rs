use super::super::{options::helpers::rounded_acceleration, set_layer_acceleration_and_jerk};
use super::EmitState;

/// The `begin_layer` inputs beyond the writer state: the print z/height
/// plus the per-object first/second-layer gates (`GCode.cpp:4589`,
/// `:4762`, `:4777`).
pub(in crate::project_slice::gcode_emit) struct LayerBegin {
    pub(in crate::project_slice::gcode_emit) layer_z: f64,
    pub(in crate::project_slice::gcode_emit) layer_height: f64,
    /// The per-object `first_layer` (`GCode.cpp:4589`) — under sequential
    /// printing every object's layer 0 qualifies.
    pub(in crate::project_slice::gcode_emit) first_layer: bool,
    /// The `!first_layer && !m_second_layer_things_done` gate
    /// (`GCode.cpp:4777`), reset per object under sequential printing
    /// (`GCode.cpp:3367`).
    pub(in crate::project_slice::gcode_emit) second_layer_reset: bool,
}

pub(in crate::project_slice::gcode_emit) fn begin_layer(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    begin: LayerBegin,
) {
    let LayerBegin {
        layer_z,
        layer_height,
        first_layer,
        second_layer_reset,
    } = begin;
    state.layer_change_travel_pending = true;
    state.avoid_boundary = None;
    state.last_height = Some(layer_height as f32);
    // `GCode.cpp:4629-4630`: `m_last_layer_z = float(print_z)` then
    // `m_max_layer_z = max(m_max_layer_z, m_last_layer_z)` — float-domain
    // running max over the RAW print z.
    state.max_layer_z = state.max_layer_z.max(layer_z as f32);
    // Writer-domain z (`GCode.cpp:5692` `z = print_z + z_offset`);
    // mirrors `advance_for_layer`'s offset store.
    state.layer_z = layer_z + state.options.z_offset;
    // `GCodeWriter` travel speed keys on the per-object first layer
    // (`m_writer.set_is_first_layer(first_layer)`, `GCode.cpp:4590`,
    // `GCodeWriter.cpp:699`): under sequential printing every object's
    // layer 0 travels at the initial-layer speed again.
    state.travel_feedrate = if first_layer {
        state.options.first_layer_travel_feedrate
    } else {
        state.options.travel_feedrate
    };
    // `GCode.cpp:4764-4776, 4794-4797`: the first-layer acceleration emit
    // and the second-layer reset are both gated on `default_acceleration > 0`
    // AND `initial_layer_acceleration > 0`; without both, no M204 is emitted
    // at the layer change. Both gates key on the PER-OBJECT `first_layer`
    // (`GCode.cpp:4589`), never the global chunk index: sequential
    // printing resets `m_second_layer_things_done` per object
    // (`GCode.cpp:3367`), so each object re-runs its layer-0 and
    // second-layer blocks.
    let has_default_acceleration = state.options.default_acceleration > 0.0;
    let acceleration = if has_default_acceleration && state.options.initial_layer_acceleration > 0.0
    {
        if first_layer {
            Some(state.options.initial_layer_acceleration)
        } else if second_layer_reset {
            Some(state.options.default_acceleration)
        } else {
            None
        }
    } else {
        None
    };
    let acceleration = acceleration.map(rounded_acceleration);
    // `GCode.cpp:4767-4776`: jerk emission is independent of the
    // acceleration emit; `default_jerk > 0 && initial_layer_jerk > 0` gate
    // applies on both the first-layer value and the second-layer reset.
    let has_layer_jerk = state.options.default_jerk > 0.0 && state.options.initial_layer_jerk > 0.0;
    let jerk = if has_layer_jerk {
        if first_layer {
            state.options.initial_layer_jerk
        } else if second_layer_reset {
            state.options.default_jerk
        } else {
            0.0
        }
    } else {
        0.0
    };
    if acceleration.is_some() || jerk > 0.0 {
        set_layer_acceleration_and_jerk(output, state, acceleration.unwrap_or(0), jerk);
    }
}
