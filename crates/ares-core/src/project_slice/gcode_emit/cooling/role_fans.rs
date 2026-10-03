//! Role requests resolve against the flushing LayerResult's id and baseline.

use super::{CoolingState, ROLE_FAN_MARKER_PREFIX, clamped_part_speed, part_fan_pwm};

impl CoolingState {
    pub(super) fn resolve_role_fans(
        &mut self,
        output: &mut Vec<u8>,
        layer_start: usize,
        baseline: u8,
        layer_index: usize,
    ) {
        let layer_open = layer_index >= self.part_fan_ramp.close_fan_first_layers() as usize;
        while let Some(offset) = output[layer_start..]
            .windows(ROLE_FAN_MARKER_PREFIX.len())
            .position(|window| window == ROLE_FAN_MARKER_PREFIX.as_bytes())
        {
            let start = layer_start + offset;
            let end = output[start..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(output.len(), |offset| start + offset + 1);
            let marker = std::str::from_utf8(&output[start..end]).unwrap();
            let (target, force) = if marker.contains("_BASE__") {
                (baseline, true)
            } else if let Some(value) = marker
                .strip_prefix(";__ARES_ROLE_FAN_CONDITIONAL_")
                .and_then(|value| value.strip_suffix("__\n"))
            {
                let (value, force) = value
                    .strip_prefix("FORCE_")
                    .map_or((value, false), |value| (value, true));
                let requested = self
                    .part_fan_ramp
                    .role_speed_for_layer(layer_index, value.parse::<u8>().unwrap())
                    .unwrap_or(0);
                (requested.max(baseline), force || requested > baseline)
            } else {
                let value = marker
                    .strip_prefix(";__ARES_ROLE_FAN_FIXED_")
                    .and_then(|value| value.strip_suffix("__\n"))
                    .unwrap();
                let value = value.strip_prefix("FORCE_").unwrap_or(value);
                (
                    if layer_open {
                        value.parse::<u8>().unwrap()
                    } else {
                        baseline
                    },
                    true,
                )
            };
            // An END requests emission even if the matching START was inactive.
            let replacement = if force
                && (layer_open || marker.contains("_BASE__") || marker.contains("_FORCE_"))
            {
                self.physical_part_speed = target;
                let emitted = clamped_part_speed(target, self.part_cooling_fan_min_pwm);
                let comment = if self.gcode_comments {
                    if emitted == 0 {
                        " ; disable fan"
                    } else {
                        " ; enable fan"
                    }
                } else {
                    ""
                };
                format!("M106 S{}{comment}\n", part_fan_pwm(emitted)).into_bytes()
            } else {
                Vec::new()
            };
            output.splice(start..end, replacement);
        }
    }
}
