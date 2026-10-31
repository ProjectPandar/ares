use super::{EmitState, features::PathProperties, overhang::ProcessedPoint};
use crate::RawOverhangFanThreshold;

pub(super) fn update_for_constant_path(
    output: &mut Vec<u8>,
    properties: PathProperties<'_>,
    state: &mut EmitState,
) {
    let markers_enabled = state.options.enable_overhang_bridge_fan;
    let overhang_active = markers_enabled
        && (matches!(properties.feature, "Overhang wall" | "Bridge")
            || state.options.overhang_fan_threshold == RawOverhangFanThreshold::Percent0
                && properties.feature == "Outer wall");
    update_marker(output, state, FanMarker::Overhang, overhang_active);
    update_marker(
        output,
        state,
        FanMarker::InternalBridge,
        markers_enabled && properties.feature == "Internal Bridge",
    );
    update_support_interface(output, properties.feature, state);
    update_ironing(output, properties.feature, state);
}

pub(super) fn update_for_variable_segment(
    output: &mut Vec<u8>,
    properties: PathProperties<'_>,
    start: ProcessedPoint,
    end: ProcessedPoint,
    state: &mut EmitState,
) {
    let markers_enabled = state.options.enable_overhang_bridge_fan;
    let overhang_active = markers_enabled
        && fan_enabled(
            properties.feature,
            start.overlap,
            state.options.overhang_fan_threshold,
        )
        && fan_enabled(
            properties.feature,
            end.overlap,
            state.options.overhang_fan_threshold,
        );
    update_marker(output, state, FanMarker::Overhang, overhang_active);
    update_marker(
        output,
        state,
        FanMarker::InternalBridge,
        markers_enabled && properties.feature == "Internal Bridge",
    );
    update_support_interface(output, properties.feature, state);
    update_ironing(output, properties.feature, state);
}

fn update_support_interface(output: &mut Vec<u8>, feature: &str, state: &mut EmitState) {
    let speed = (feature == "Support interface")
        .then_some(state.options.support_interface_fan_speed)
        .flatten();
    let previous = state.support_interface_fan_marker_layer;
    state.support_interface_fan_marker_layer = speed.map(|_| state.layer_chunk_index);
    if (speed.is_some() && previous != Some(state.layer_chunk_index))
        || (speed.is_none() && previous.is_some())
    {
        super::super::cooling::append_deferred_role_fan(
            output,
            super::super::cooling::DeferredRoleFan::SupportInterface(speed),
        );
    }
}

fn update_ironing(output: &mut Vec<u8>, feature: &str, state: &mut EmitState) {
    let speed = (feature == "Ironing")
        .then_some(state.options.ironing_fan_speed)
        .flatten();
    let previous = state.ironing_fan_marker_layer;
    state.ironing_fan_marker_layer = speed.map(|_| state.layer_chunk_index);
    if (speed.is_some() && previous != Some(state.layer_chunk_index))
        || (speed.is_none() && previous.is_some())
    {
        super::super::cooling::append_deferred_role_fan(
            output,
            super::super::cooling::DeferredRoleFan::Ironing(speed),
        );
    }
}

fn fan_enabled(feature: &str, overlap: f64, threshold: RawOverhangFanThreshold) -> bool {
    if matches!(feature, "Overhang wall" | "Bridge") {
        return true;
    }
    match threshold {
        RawOverhangFanThreshold::Percent0 => feature == "Outer wall",
        RawOverhangFanThreshold::Percent10 => overlap <= 0.9,
        RawOverhangFanThreshold::Percent25 => overlap <= 0.75,
        RawOverhangFanThreshold::Percent50 => overlap <= 0.5,
        RawOverhangFanThreshold::Percent75 => overlap <= 0.25,
        RawOverhangFanThreshold::Percent95 => overlap <= 0.05,
    }
}

#[derive(Clone, Copy)]
enum FanMarker {
    Overhang,
    InternalBridge,
}

fn update_marker(output: &mut Vec<u8>, state: &mut EmitState, marker: FanMarker, active: bool) {
    let (was_active, marker_layer) = marker_state(state, marker);
    let fresh_start = active && (!was_active || marker_layer != Some(state.layer_chunk_index));
    let stopping = !active && was_active;
    set_marker_state(state, marker, active);
    if (stopping || fresh_start)
        && let Some(target) = deferred_target(state, stopping)
    {
        super::super::cooling::append_deferred_role_fan(output, target);
    }
}

fn marker_state(state: &EmitState, marker: FanMarker) -> (bool, Option<usize>) {
    match marker {
        FanMarker::Overhang => (state.overhang_fan_active, state.overhang_fan_marker_layer),
        FanMarker::InternalBridge => (
            state.internal_bridge_fan_active,
            state.internal_bridge_fan_marker_layer,
        ),
    }
}

fn set_marker_state(state: &mut EmitState, marker: FanMarker, active: bool) {
    let layer = active.then_some(state.layer_chunk_index);
    match marker {
        FanMarker::Overhang => {
            state.overhang_fan_active = active;
            state.overhang_fan_marker_layer = layer;
        }
        FanMarker::InternalBridge => {
            state.internal_bridge_fan_active = active;
            state.internal_bridge_fan_marker_layer = layer;
        }
    }
}

fn deferred_target(
    state: &EmitState,
    stopping: bool,
) -> Option<super::super::cooling::DeferredRoleFan> {
    use super::super::cooling::DeferredRoleFan;

    if !state.options.enable_overhang_bridge_fan {
        return None;
    }
    if !state.overhang_fan_active && !state.internal_bridge_fan_active {
        return Some(DeferredRoleFan::Baseline);
    }
    let overhang_speed = state.options.overhang_fan_speed;
    if state.overhang_fan_active {
        // `CoolingBuffer.cpp:856–869` forces emission on role END even
        // when the remaining role's START does not exceed the layer baseline.
        return Some(DeferredRoleFan::Conditional {
            speed: overhang_speed,
            force: stopping,
        });
    }
    if state.internal_bridge_fan_active {
        return Some(
            state
                .options
                .internal_bridge_fan_speed
                .role_speed(None)
                .map_or(
                    DeferredRoleFan::Conditional {
                        speed: overhang_speed,
                        force: stopping,
                    },
                    |speed| DeferredRoleFan::Fixed {
                        speed,
                        force: stopping,
                    },
                ),
        );
    }
    Some(DeferredRoleFan::Baseline)
}
