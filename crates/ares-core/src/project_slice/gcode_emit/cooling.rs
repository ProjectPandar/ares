mod feedrate;
mod initialize;
mod markers;
mod role_fans;

use markers::strip_pressure_markers;

use crate::options::PartCoolingFanRamp;

const PART_FAN_MARKER: &[u8] = b";__ARES_PART_FAN_STATE__\n";
const ROLE_FAN_MARKER_PREFIX: &str = ";__ARES_ROLE_FAN_";

pub(super) enum DeferredRoleFan {
    Baseline,
    Conditional { speed: u8, force: bool },
    Fixed { speed: u8, force: bool },
    SupportInterface(Option<u8>),
    Ironing(Option<u8>),
}

pub(super) fn append_deferred_role_fan(output: &mut Vec<u8>, target: DeferredRoleFan) {
    let marker = match target {
        DeferredRoleFan::Baseline => format!("{ROLE_FAN_MARKER_PREFIX}BASE__\n"),
        DeferredRoleFan::Conditional { speed, force } => {
            let force = if force { "FORCE_" } else { "" };
            format!("{ROLE_FAN_MARKER_PREFIX}CONDITIONAL_{force}{speed}__\n")
        }
        DeferredRoleFan::SupportInterface(speed) => {
            let value = speed.map_or_else(|| "END".to_owned(), |speed| speed.to_string());
            format!("{ROLE_FAN_MARKER_PREFIX}SUPPORT_INTERFACE_{value}__\n")
        }
        DeferredRoleFan::Ironing(speed) => {
            let value = speed.map_or_else(|| "END".to_owned(), |speed| speed.to_string());
            format!("{ROLE_FAN_MARKER_PREFIX}IRONING_{value}__\n")
        }
        DeferredRoleFan::Fixed { speed, force } => {
            let force = if force { "FORCE_" } else { "" };
            format!("{ROLE_FAN_MARKER_PREFIX}FIXED_{force}{speed}__\n")
        }
    };
    output.extend_from_slice(marker.as_bytes());
}

pub(super) struct CoolingState {
    part_speed: Option<u8>,
    /// CoolingBuffer::m_current_fan_speed: requested percent before PWM clamping.
    current_part_speed: Option<u8>,
    provisional_part_speed: u8,
    pending_layer_index: Option<usize>,
    /// The layer the equalizer is holding back.
    equalized_layer_result: Option<(usize, bool)>,
    buffered_gcode: Vec<u8>,
    /// Per-filament fan ramps; indexed by the buffered tool id
    /// (`CoolingBuffer.cpp:739` `EXTRUDER_CONFIG` reads
    /// `get_at(m_current_extruder)`).
    part_fan_ramp: Vec<PartCoolingFanRamp>,
    additional_speed: Option<u8>,
    additional_fan_speed: Vec<u8>,
    auxiliary_fan: bool,
    part_cooling_fan_min_pwm: u8,
    gcode_comments: bool,
    equalizer: Option<crate::project_slice::pressure_equalizer::PressureEqualizerPass>,
    feedrate: feedrate::State,
}

impl CoolingState {
    pub(super) fn reset_fans(&mut self) {
        // CoolingBuffer.cpp:62–64; GCode.cpp:4678 resets between sequential instances.
        self.part_speed = None;
        self.additional_speed = None;
        self.current_part_speed = None;
    }

    pub(super) fn begin_layer(&mut self, output: &mut Vec<u8>, layer_index: usize) {
        self.pending_layer_index = Some(layer_index);
        self.provisional_part_speed = self
            .ramp(self.feedrate.current_tool())
            .speed_for_layer_time(layer_index, Some(0.0))
            .unwrap_or(0);
        output.extend_from_slice(PART_FAN_MARKER);
    }

    pub(super) const fn provisional_part_speed(&self) -> u8 {
        self.provisional_part_speed
    }

    pub(super) fn finish_layer(&mut self, output: &mut Vec<u8>, layer_start: usize, flush: bool) {
        // Upstream pipeline order (GCode.cpp:3752): generator →
        // PressureEqualizer → CoolingBuffer → fan logic. The equalizer holds
        // each layer back until the next one arrives
        // (`PressureEqualizer.cpp:174-200`), so the cooling buffer runs one
        // layer behind; `finish` drains the last layer.
        let layer_index = self.pending_layer_index.take().unwrap();
        let layer = output.split_off(layer_start);
        self.process_layer(output, layer, (layer_index, flush));
    }

    pub(super) fn empty_layer(
        &mut self,
        output: &mut Vec<u8>,
        layer_index: usize,
        last_layer: bool,
    ) {
        self.process_layer(output, Vec::new(), (layer_index, last_layer));
    }

    fn process_layer(&mut self, output: &mut Vec<u8>, layer: Vec<u8>, result: (usize, bool)) {
        let Some(pass) = self.equalizer.as_mut() else {
            self.cool_layer(output, layer, result, false);
            return;
        };
        let previous = pass.process_layer(&String::from_utf8_lossy(&layer));
        let previous_result = self.equalized_layer_result.replace(result);
        if let (Some(previous), Some((previous_index, previous_flush))) =
            (previous, previous_result)
        {
            self.cool_layer(
                output,
                previous.into_bytes(),
                (previous_index, previous_flush),
                true,
            );
        }
    }

    /// The trailing NOP layer result that drains the equalizer.
    pub(super) fn finish(&mut self, output: &mut Vec<u8>) {
        if let Some(last) = self.equalizer.as_mut().and_then(|pass| pass.flush()) {
            let (layer_index, flush) = self.equalized_layer_result.take().unwrap();
            self.cool_layer(output, last.into_bytes(), (layer_index, flush), true);
        }
    }

    /// `CoolingBuffer::process_layer` plus the fan resolution for one layer.
    /// The equalizer re-emits the `;_EXTRUDE_SET_SPEED`/`;_EXTRUDE_END`
    /// markers for the slowdown; they are stripped after the cooling.
    fn cool_layer(
        &mut self,
        output: &mut Vec<u8>,
        mut layer: Vec<u8>,
        (layer_index, flush): (usize, bool),
        equalized: bool,
    ) {
        if self.buffered_gcode.is_empty() {
            self.buffered_gcode = layer;
        } else {
            self.buffered_gcode.append(&mut layer);
        }
        if !flush {
            return;
        }
        let mut layer = std::mem::take(&mut self.buffered_gcode);
        let layer_start = output.len();
        // `CoolingBuffer.cpp:824` calls `change_extruder_set_fan(true)`
        // before iterating the layer's lines: the layer baseline uses
        // `m_current_extruder` at layer entry (the previous layer's last
        // tool), not the tool this layer's text ends on.
        let entry_tool = self.feedrate.current_tool();
        let layer_time = feedrate::rewrite_layer(&mut layer, 0, &mut self.feedrate);
        if equalized {
            let stripped = strip_pressure_markers(&String::from_utf8_lossy(&layer));
            layer = stripped.into_bytes();
        }
        output.extend_from_slice(&layer);
        let tool = entry_tool;
        let part_speed = self
            .ramp(tool)
            .speed_for_layer_time(layer_index, Some(f64::from(layer_time)))
            .unwrap_or(0);
        let mut replacement =
            if self.part_speed != Some(part_speed) || self.current_part_speed != Some(part_speed) {
                let speed = clamped_part_speed(part_speed, self.part_cooling_fan_min_pwm);
                let comment = match (self.gcode_comments, speed == 0) {
                    (false, _) => "",
                    (true, true) => " ; disable fan",
                    (true, false) => " ; enable fan",
                };
                format!("M106 S{}{comment}\n", part_fan_pwm(speed)).into_bytes()
            } else {
                Vec::new()
            };
        // `CoolingBuffer.cpp:834`: even empty final input updates the fan.
        let marker_len = if layer.is_empty() {
            0
        } else {
            PART_FAN_MARKER.len()
        };
        let marker_start = if layer.is_empty() {
            layer_start
        } else {
            layer_start
                + output[layer_start..]
                    .windows(PART_FAN_MARKER.len())
                    .position(|window| window == PART_FAN_MARKER)
                    .unwrap()
        };
        if !replacement.is_empty() {
            self.current_part_speed = Some(part_speed);
        }
        let additional_speed = if layer_index < self.ramp(tool).close_fan_first_layers() as usize {
            0
        } else {
            self.additional_fan_speed
                .get(tool)
                .or_else(|| self.additional_fan_speed.first())
                .copied()
                .unwrap_or(0)
        };
        if self.auxiliary_fan && self.additional_speed != Some(additional_speed) {
            self.additional_speed = Some(additional_speed);
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
        output.splice(marker_start..marker_start + marker_len, replacement);
        while let Some(offset) = output[layer_start..]
            .windows(PART_FAN_MARKER.len())
            .position(|window| window == PART_FAN_MARKER)
        {
            let start = layer_start + offset;
            output.drain(start..start + PART_FAN_MARKER.len());
        }
        self.resolve_role_fans(
            output,
            layer_start,
            role_fans::RoleFanContext {
                baseline: part_speed,
                layer_index,
                tool: entry_tool,
                layer_time: f64::from(layer_time),
            },
        );
        // `CoolingBuffer.cpp:840-850`: after TYPE_SET_TOOL, the retained
        // baseline follows the last buffered tool of this layer.
        let end_tool = self.feedrate.current_tool();
        self.part_speed = Some(
            self.ramp(end_tool)
                .speed_for_layer_time(layer_index, Some(f64::from(layer_time)))
                .unwrap_or(0),
        );
        self.provisional_part_speed = part_speed;
    }

    /// The buffered tool's ramp (`EXTRUDER_CONFIG` in
    /// `CoolingBuffer.cpp:739`), falling back to slot 0 like `get_at`.
    fn ramp(&self, tool: usize) -> &PartCoolingFanRamp {
        self.part_fan_ramp
            .get(tool)
            .or_else(|| self.part_fan_ramp.first())
            .expect("fan ramps are built for at least one filament")
    }
}

fn clamped_part_speed(speed: u8, minimum: u8) -> u8 {
    if speed > 0 && speed < minimum {
        minimum
    } else {
        speed
    }
}

fn part_fan_pwm(speed: u8) -> u32 {
    (255.5 * f64::from(speed) / 100.0).floor() as u32
}

fn additional_fan_pwm(speed: u8) -> u32 {
    (255.0 * f64::from(speed) / 100.0).floor() as u32
}
