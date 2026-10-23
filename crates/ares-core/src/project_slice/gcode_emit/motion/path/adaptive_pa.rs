//! `GCode::_extrude`'s PA_CHANGE marker, before the extrusion feedrate.

use super::{EmitState, PathProperties, SOURCE_EPSILON_MM, role_id};
use crate::project_slice::gcode_emit::format_processor_float;

pub(super) fn tag(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    properties: PathProperties<'_>,
    acceleration: u32,
    role_changed: bool,
) -> bool {
    if !state.options.adaptive_pa.enabled {
        return false;
    }
    if state.adaptive_pa_emitted && !role_changed {
        return false;
    }
    let flow = state
        .adaptive_pa_flow
        .unwrap_or_else(|| super::extrusion::effective_flow_at_emission(properties, state));
    if flow <= 0.0 {
        return true;
    }
    let role = role_id(properties.feature);
    marker(
        output,
        state.extruders.active_id(),
        flow,
        acceleration,
        role,
        role_changed,
        role == 3,
    );
    state.adaptive_pa_emitted = state.adaptive_pa_flow.is_some();
    true
}

pub(super) enum Ramp {
    Constant { previous: f64, next: f64 },
    Variable { previous: f64, next: f64 },
}

// `GCode::_extrude` updates m_last_mm3_mm only when an intra-feature
// marker is emitted, not when the initial path marker is emitted.
pub(super) fn ramp(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    properties: PathProperties<'_>,
    acceleration: u32,
    kind: Ramp,
) {
    if !state.options.adaptive_pa.enabled || !state.options.adaptive_pa.overhangs {
        return;
    }
    let flow = super::extrusion::effective_flow_at_emission(properties, state);
    let (Ramp::Constant { previous, next } | Ramp::Variable { previous, next }) = kind;
    if flow <= 0.0
        || ((previous - next).abs() <= SOURCE_EPSILON_MM
            && (flow - state.adaptive_pa_last_flow).abs() <= SOURCE_EPSILON_MM)
    {
        return;
    }
    let down = previous > next;
    if state.options.gcode_comments {
        // The variable-path debug labels are inverted in upstream.
        output.extend_from_slice(match (kind, down) {
            (Ramp::Constant { .. }, true) => b"; Ramp down-non-variable\n",
            (Ramp::Constant { .. }, false) => b"; Ramp up-non-variable\n",
            (Ramp::Variable { .. }, true) => b"; Ramp up-variable\n",
            (Ramp::Variable { .. }, false) => b"; Ramp down-variable\n",
        });
    }
    marker(
        output,
        state.extruders.active_id(),
        flow,
        acceleration,
        role_id(properties.feature),
        true,
        down,
    );
    state.adaptive_pa_last_flow = flow;
}

#[expect(
    clippy::too_many_arguments,
    reason = "PA_CHANGE carries six independent upstream fields plus its output buffer"
)]
fn marker(
    output: &mut Vec<u8>,
    tool: usize,
    flow: f64,
    acceleration: u32,
    role: u32,
    role_changed: bool,
    overhang: bool,
) {
    output.extend_from_slice(
        format!(
            "; PA_CHANGE:T{tool} MM3MM:{} ACCEL:{acceleration} BR:{} RC:{} OV:{}\n",
            format_processor_float(flow),
            u8::from(matches!(role, 3 | 9)),
            u8::from(role_changed),
            u8::from(overhang),
        )
        .as_bytes(),
    );
}
