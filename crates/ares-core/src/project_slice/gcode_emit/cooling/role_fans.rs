//! Role requests resolve against the flushing LayerResult's id and baseline.

use super::{
    CoolingState, ROLE_FAN_MARKER_PREFIX, additional_fan_pwm, clamped_part_speed, part_fan_pwm,
};

const FORCE_RESUME_FAN: &str = ";_FORCE_RESUME_FAN_SPEED\n";

impl CoolingState {
    pub(super) fn resolve_role_fans(
        &mut self,
        output: &mut Vec<u8>,
        layer_start: usize,
        baseline: u8,
        layer_index: usize,
    ) {
        let layer_open = layer_index >= self.part_fan_ramp.close_fan_first_layers() as usize;
        let mut request_active = false;
        let mut cursor = layer_start;
        loop {
            let role = output[cursor..]
                .windows(ROLE_FAN_MARKER_PREFIX.len())
                .position(|window| window == ROLE_FAN_MARKER_PREFIX.as_bytes());
            let resume = output[cursor..]
                .windows(FORCE_RESUME_FAN.len())
                .position(|window| window == FORCE_RESUME_FAN.as_bytes());
            let Some(offset) = role.into_iter().chain(resume).min() else {
                break;
            };
            let start = cursor + offset;
            let end = output[start..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(output.len(), |offset| start + offset + 1);
            let marker = std::str::from_utf8(&output[start..end]).unwrap();
            if marker == FORCE_RESUME_FAN {
                let mut replacement = Vec::new();
                // CoolingBuffer.cpp:890–898 sends auxiliary before the part fan.
                if self.auxiliary_fan
                    && let Some(speed) = self.additional_speed
                {
                    let comment = if self.gcode_comments {
                        if speed == 0 {
                            " ; disable additional fan "
                        } else {
                            " ; enable additional fan "
                        }
                    } else {
                        ""
                    };
                    replacement.extend_from_slice(
                        format!("M106 P2 S{}{comment}\n", additional_fan_pwm(speed)).as_bytes(),
                    );
                }
                if !request_active {
                    replacement.extend_from_slice(&self.part_fan_command(self.physical_part_speed));
                }
                cursor = start + replacement.len();
                output.splice(start..end, replacement);
                continue;
            }
            let (target, force) = if marker.contains("_BASE__") {
                request_active = false;
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
                request_active = requested > baseline;
                (requested.max(baseline), force || requested > baseline)
            } else {
                let value = marker
                    .strip_prefix(";__ARES_ROLE_FAN_FIXED_")
                    .and_then(|value| value.strip_suffix("__\n"))
                    .unwrap();
                let value = value.strip_prefix("FORCE_").unwrap_or(value);
                request_active = layer_open;
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
                self.part_fan_command(target)
            } else {
                Vec::new()
            };
            cursor = start + replacement.len();
            output.splice(start..end, replacement);
        }
    }

    fn part_fan_command(&self, target: u8) -> Vec<u8> {
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
    }
}
