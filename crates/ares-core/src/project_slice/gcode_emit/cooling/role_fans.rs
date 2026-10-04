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
        let mut role_speed = None;
        let mut support_speed = None;
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
                let replacement = self.resume_fan_commands(role_speed, support_speed);
                cursor = start + replacement.len();
                output.splice(start..end, replacement);
                continue;
            }
            let support_value = marker
                .strip_prefix(";__ARES_ROLE_FAN_SUPPORT_INTERFACE_")
                .and_then(|value| value.strip_suffix("__\n"));
            if support_value == Some("END") && support_speed.is_none() {
                // CoolingBuffer.cpp:875 leaves an inactive END as an ordinary line.
                let retained = b";_SUPP_INTERFACE_FAN_END\n";
                cursor = start + retained.len();
                output.splice(start..end, retained.iter().copied());
                continue;
            }
            let force = if let Some(value) = support_value {
                let was_active = support_speed.is_some();
                support_speed =
                    (value != "END" && layer_open).then(|| value.parse::<u8>().unwrap());
                was_active != support_speed.is_some()
            } else if marker.contains("_BASE__") {
                role_speed = None;
                true
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
                role_speed = (requested > baseline).then_some(requested);
                force || role_speed.is_some()
            } else {
                let value = marker
                    .strip_prefix(";__ARES_ROLE_FAN_FIXED_")
                    .and_then(|value| value.strip_suffix("__\n"))
                    .unwrap();
                let value = value.strip_prefix("FORCE_").unwrap_or(value);
                role_speed = layer_open.then(|| value.parse::<u8>().unwrap());
                true
            };
            let target = role_speed.or(support_speed).unwrap_or(baseline);
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

    fn resume_fan_commands(&self, role_speed: Option<u8>, support_speed: Option<u8>) -> Vec<u8> {
        let mut replacement = Vec::new();
        // CoolingBuffer.cpp:890–898 sends auxiliary before the part fan.
        if self.auxiliary_fan
            && let Some(speed) = self.additional_speed
        {
            let comment = match (self.gcode_comments, speed == 0) {
                (false, _) => "",
                (true, true) => " ; disable additional fan ",
                (true, false) => " ; enable additional fan ",
            };
            replacement.extend_from_slice(
                format!("M106 P2 S{}{comment}\n", additional_fan_pwm(speed)).as_bytes(),
            );
        }
        if role_speed.is_none() && support_speed.is_none() {
            replacement.extend_from_slice(&self.part_fan_command(self.physical_part_speed));
        }
        replacement
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
