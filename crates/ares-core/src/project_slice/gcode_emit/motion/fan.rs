#[cfg(test)]
mod tests;

use super::{EmitState, features::PathProperties, overhang::ProcessedPoint};
use crate::RawOverhangFanThreshold;

pub(super) fn update_for_constant_path(
    output: &mut Vec<u8>,
    properties: PathProperties<'_>,
    state: &mut EmitState,
) {
    let markers_enabled = state.options.enable_overhang_bridge_fan;
    let control_enabled =
        markers_enabled && state.options.overhang_fan_speed > state.part_fan_speed;
    let active = markers_enabled
        && (matches!(properties.feature, "Overhang wall" | "Bridge")
            || state.options.overhang_fan_threshold == RawOverhangFanThreshold::Percent0
                && properties.feature == "Outer wall");
    update(output, state, active, control_enabled);
}

pub(super) fn update_for_variable_segment(
    output: &mut Vec<u8>,
    properties: PathProperties<'_>,
    start: ProcessedPoint,
    end: ProcessedPoint,
    state: &mut EmitState,
) {
    let markers_enabled = state.options.enable_overhang_bridge_fan;
    let control_enabled =
        markers_enabled && state.options.overhang_fan_speed > state.part_fan_speed;
    let active = markers_enabled
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
    update(output, state, active, control_enabled);
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

fn update(output: &mut Vec<u8>, state: &mut EmitState, active: bool, control_enabled: bool) {
    let fresh_start = active
        && (!state.overhang_fan_active
            || state.overhang_fan_marker_layer != Some(state.layer_index));
    let stopping = !active && state.overhang_fan_active;
    if fresh_start && control_enabled {
        let speed = state.options.overhang_fan_speed;
        output.extend_from_slice(format!("M106 S{}\n", pwm(speed)).as_bytes());
        state.physical_fan_speed = speed;
    } else if stopping {
        output.extend_from_slice(format!("M106 S{}\n", pwm(state.part_fan_speed)).as_bytes());
        state.physical_fan_speed = state.part_fan_speed;
    }
    state.overhang_fan_active = active;
    state.overhang_fan_marker_layer = active.then_some(state.layer_index);
}

fn pwm(speed: u8) -> u32 {
    (255.5 * f64::from(speed) / 100.0).floor() as u32
}
