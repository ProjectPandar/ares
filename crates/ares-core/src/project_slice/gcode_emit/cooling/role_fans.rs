//! Role requests resolve against the flushing LayerResult's id and baseline.

use super::{
    CoolingState, PART_FAN_MARKER, ROLE_FAN_MARKER_PREFIX, additional_fan_pwm, clamped_part_speed,
    part_fan_pwm,
};

const FORCE_RESUME_FAN: &str = ";_FORCE_RESUME_FAN_SPEED\n";
const BELT_BAND_START: &str = ";__ARES_ROLE_FAN_BELT_BAND_START__\n";
const BELT_BAND_END: &str = ";__ARES_ROLE_FAN_BELT_BAND_END__\n";
const BELT_FAN_CARRY: &str = ";__ARES_ROLE_FAN_BELT_CARRY__\n";

pub(in crate::project_slice::gcode_emit) fn mark_belt_extrusion(
    output: &mut Vec<u8>,
    in_band: bool,
) {
    output.extend_from_slice(if in_band {
        BELT_BAND_START.as_bytes()
    } else {
        BELT_BAND_END.as_bytes()
    });
}

pub(super) fn mark_belt_fan_carry(layer: &mut Vec<u8>) {
    // CoolingBuffer.cpp reapplies the retained gate after processing the
    // first parsed command, even when the layer's new baseline is nonzero.
    let mut end = 0;
    for line in layer.split_inclusive(|&byte| byte == b'\n') {
        end += line.len();
        if [b"G0 ".as_slice(), b"G1 ", b"G2 ", b"G3 ", b"G92 "]
            .iter()
            .any(|prefix| line.starts_with(prefix))
        {
            layer.splice(end..end, BELT_FAN_CARRY.bytes());
            break;
        }
    }
}

/// The flushing layer's fan context: computed baseline speed, layer id and
/// the buffered tool id (`CoolingBuffer`'s `m_current_extruder`).
pub(super) struct RoleFanContext {
    pub(super) baseline: u8,
    pub(super) layer_index: usize,
    /// Tool active at the start of the flushed span
    /// (`CoolingBuffer.cpp:824` initial `change_extruder_set_fan`).
    pub(super) tool: usize,
    /// Adjusted layer time used to evaluate a ramp for a mid-layer
    /// tool change (`CoolingBuffer.cpp:840-850`).
    pub(super) layer_time: f64,
}

impl CoolingState {
    pub(super) fn stream_layer_with_fans(
        &mut self,
        layer: &[u8],
        output: &mut Vec<u8>,
        part_fan_replacement: &[u8],
        context: RoleFanContext,
    ) {
        let RoleFanContext {
            mut baseline,
            layer_index,
            mut tool,
            layer_time,
        } = context;
        let prefix = self.feedrate.toolchange_prefix().as_bytes();
        let tool_bound = self.feedrate.tool_bound();
        let mut layer_open = layer_index >= self.ramp(tool).close_fan_first_layers() as usize;
        let mut role_speed = None;
        let mut support_speed = None;
        let mut ironing_speed = None;
        let mut part_fan_replacement = Some(part_fan_replacement);
        output.reserve(layer.len() + part_fan_replacement.map_or(0, <[u8]>::len));

        for line in layer.split_inclusive(|&byte| byte == b'\n') {
            if line == PART_FAN_MARKER {
                output.extend_from_slice(part_fan_replacement.take().unwrap_or_default());
                continue;
            }
            if let Some(new_tool) = match_set_tool(line, prefix, tool, tool_bound) {
                tool = new_tool;
                layer_open = layer_index >= self.ramp(tool).close_fan_first_layers() as usize;
                // CoolingBuffer.cpp:840-850: TYPE_SET_TOOL runs
                // change_extruder_set_fan(true), emitting part then
                // auxiliary fan at the tool change's own position.
                let (replacement, part_speed) = self.set_tool_fan(
                    SetToolContext {
                        layer_index,
                        layer_time,
                        baseline,
                        layer_open,
                    },
                    tool,
                );
                baseline = part_speed;
                output.extend_from_slice(&replacement);
                output.extend_from_slice(line);
                continue;
            }
            let Some(marker_pos) = find_marker_start(line) else {
                output.extend_from_slice(line);
                continue;
            };
            output.extend_from_slice(&line[..marker_pos]);
            let marker_bytes = &line[marker_pos..];
            if marker_bytes == PART_FAN_MARKER {
                output.extend_from_slice(part_fan_replacement.take().unwrap_or_default());
                continue;
            }
            let marker = std::str::from_utf8(marker_bytes).unwrap();
            if matches!(marker, BELT_BAND_START | BELT_BAND_END | BELT_FAN_CARRY) {
                self.belt_fan_off = match marker {
                    BELT_BAND_START => true,
                    BELT_BAND_END => false,
                    _ => self.belt_fan_off,
                };
                let target = role_speed
                    .or(support_speed)
                    .or(ironing_speed)
                    .unwrap_or(baseline);
                output.extend_from_slice(&self.set_gated_part_fan(target));
                continue;
            }
            if marker == FORCE_RESUME_FAN {
                output.extend_from_slice(&self.resume_fan_commands(
                    role_speed,
                    support_speed,
                    ironing_speed,
                ));
                continue;
            }
            let support_value = marker
                .strip_prefix(";__ARES_ROLE_FAN_SUPPORT_INTERFACE_")
                .and_then(|value| value.strip_suffix("__\n"));
            if support_value == Some("END") && support_speed.is_none() {
                // CoolingBuffer.cpp:875 leaves an inactive END as an ordinary line.
                output.extend_from_slice(b";_SUPP_INTERFACE_FAN_END\n");
                continue;
            }
            let ironing_value = marker
                .strip_prefix(";__ARES_ROLE_FAN_IRONING_")
                .and_then(|value| value.strip_suffix("__\n"));
            // CoolingBuffer.cpp:885 consumes even an inactive ironing END.
            let ironing_end = ironing_value == Some("END");
            let force = if let Some(value) = ironing_value {
                let was_active = ironing_speed.is_some();
                ironing_speed = (!ironing_end && layer_open).then(|| value.parse::<u8>().unwrap());
                ironing_end || (!was_active && ironing_speed.is_some())
            } else if let Some(value) = support_value {
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
                    .ramp(tool)
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
            let target = role_speed
                .or(support_speed)
                .or(ironing_speed)
                .unwrap_or(baseline);
            // An END requests emission even if the matching START was inactive.
            let emit = force
                && (layer_open
                    || ironing_end
                    || marker.contains("_BASE__")
                    || marker.contains("_FORCE_"));
            if emit {
                output.extend_from_slice(&self.set_gated_part_fan(target));
            }
        }
    }
}

/// Inputs for `set_tool_fan`: layer coordinates shared by every ramp
/// evaluation plus the running baseline of the current tool.
struct SetToolContext {
    layer_index: usize,
    layer_time: f64,
    baseline: u8,
    layer_open: bool,
}

/// Matches a `T<tool>`-style toolchange line whose parsed tool differs from
/// `current` — the `TYPE_SET_TOOL` gate in `CoolingBuffer.cpp:506-510` /
/// `840-850` (`tool < tool_map.len()`, `tool != current`).
fn match_set_tool(line: &[u8], prefix: &[u8], current: usize, tool_bound: usize) -> Option<usize> {
    let digits = line.strip_prefix(prefix)?;
    let length = digits.iter().take_while(|b| b.is_ascii_digit()).count();
    if length == 0 {
        return None;
    }
    let tool = std::str::from_utf8(&digits[..length])
        .unwrap()
        .parse::<usize>()
        .ok()?;
    (tool < tool_bound && tool != current).then_some(tool)
}

/// Finds the first cooling marker (`PART_FAN_MARKER`, `ROLE_FAN_MARKER_PREFIX`,
/// or `FORCE_RESUME_FAN`) within a single `\n`-terminated line.
fn find_marker_start(line: &[u8]) -> Option<usize> {
    let mut offset = 0;
    while let Some(rel) = line[offset..].iter().position(|&byte| byte == b';') {
        let pos = offset + rel;
        let tail = &line[pos..];
        if tail.get(1) == Some(&b'_')
            && (tail == PART_FAN_MARKER
                || tail.starts_with(ROLE_FAN_MARKER_PREFIX.as_bytes())
                || tail == FORCE_RESUME_FAN.as_bytes())
        {
            return Some(pos);
        }
        offset = pos + 1;
    }
    None
}

impl CoolingState {
    /// `change_extruder_set_fan` for a mid-layer `TYPE_SET_TOOL`
    /// (`CoolingBuffer.cpp:733-822`): part then auxiliary commands, part
    /// speed first like `set_fan` then `set_additional_fan`. Returns the
    /// bytes to insert at the `T` line and the tool's baseline speed.
    fn set_tool_fan(&mut self, context: SetToolContext, tool: usize) -> (Vec<u8>, u8) {
        let SetToolContext {
            layer_index,
            layer_time,
            baseline,
            layer_open,
        } = context;
        let part_speed = if layer_open {
            self.ramp(tool)
                .speed_for_layer_time(layer_index, Some(layer_time))
                .unwrap_or(0)
        } else {
            0
        };
        let additional_speed = if layer_open {
            self.additional_fan_speed
                .get(tool)
                .or_else(|| self.additional_fan_speed.first())
                .copied()
                .unwrap_or(0)
        } else {
            0
        };
        let mut replacement = Vec::new();
        if baseline != part_speed || self.current_part_speed != Some(part_speed) {
            replacement.extend_from_slice(&self.part_fan_command(part_speed));
            self.current_part_speed = Some(part_speed);
        }
        if self.auxiliary_fan && self.additional_speed != Some(additional_speed) {
            let comment = match (self.gcode_comments, additional_speed == 0) {
                (false, _) => "",
                (true, true) => " ; disable additional fan ",
                (true, false) => " ; enable additional fan ",
            };
            replacement.extend_from_slice(
                format!(
                    "M106 P2 S{}{comment}\n",
                    additional_fan_pwm(additional_speed)
                )
                .as_bytes(),
            );
        }
        self.additional_speed = Some(additional_speed);
        (replacement, part_speed)
    }

    fn resume_fan_commands(
        &mut self,
        role_speed: Option<u8>,
        support_speed: Option<u8>,
        ironing_speed: Option<u8>,
    ) -> Vec<u8> {
        if self.belt_fan_off {
            return self.set_part_fan(0);
        }
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
        if role_speed.is_none()
            && support_speed.is_none()
            && ironing_speed.is_none()
            && let Some(speed) = self.current_part_speed
        {
            replacement.extend_from_slice(&self.part_fan_command(speed));
        }
        replacement
    }

    fn set_gated_part_fan(&mut self, target: u8) -> Vec<u8> {
        self.set_part_fan(if self.belt_fan_off { 0 } else { target })
    }

    fn set_part_fan(&mut self, target: u8) -> Vec<u8> {
        // CoolingBuffer.cpp:1066–1071 compares the request, not clamped PWM bytes.
        if self.current_part_speed == Some(target) {
            return Vec::new();
        }
        let command = self.part_fan_command(target);
        self.current_part_speed = Some(target);
        command
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
