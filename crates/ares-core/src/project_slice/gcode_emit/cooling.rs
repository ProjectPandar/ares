mod feedrate;
mod markers;
mod role_fans;

use markers::strip_pressure_markers;

#[cfg(test)]
mod tests;

use crate::{
    options::{PartCoolingFanRamp, PartCoolingFanRampConfig},
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

const PART_FAN_MARKER: &[u8] = b";__ARES_PART_FAN_STATE__\n";
const ROLE_FAN_MARKER_PREFIX: &str = ";__ARES_ROLE_FAN_";

pub(super) enum DeferredRoleFan {
    Baseline,
    Conditional { speed: u8, force: bool },
    Fixed { speed: u8, force: bool },
    SupportInterface(Option<u8>),
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
        DeferredRoleFan::Fixed { speed, force } => {
            let force = if force { "FORCE_" } else { "" };
            format!("{ROLE_FAN_MARKER_PREFIX}FIXED_{force}{speed}__\n")
        }
    };
    output.extend_from_slice(marker.as_bytes());
}

pub(super) struct CoolingState {
    part_speed: Option<u8>,
    physical_part_speed: u8,
    provisional_part_speed: u8,
    pending_layer_index: Option<usize>,
    /// The layer the equalizer is holding back.
    equalized_layer_result: Option<(usize, bool)>,
    buffered_gcode: Vec<u8>,
    part_fan_ramp: PartCoolingFanRamp,
    additional_speed: Option<u8>,
    additional_fan_speed: u8,
    auxiliary_fan: bool,
    part_cooling_fan_min_pwm: u8,
    gcode_comments: bool,
    equalizer: Option<crate::project_slice::pressure_equalizer::PressureEqualizerPass>,
    feedrate: feedrate::State,
}

impl CoolingState {
    pub(super) fn from_traversal(
        traversal: &PreparedPostClassicTraversal,
        writer: &super::motion::EmitState,
    ) -> Self {
        let full = &traversal.resolved.views.full;
        let filament = &full.filament.print;
        let runtime = &traversal.resolved.views.runtime_gcode;
        let first_bool = |values: &[crate::OrcaBool]| values.first().is_some_and(|value| value.0);
        let max_speed = first_percent(&filament.fan_max_speed.0);
        let slow_down_layer_time = filament
            .slow_down_layer_time
            .0
            .first()
            .map_or(0.0, |value| value.0);
        let part_fan_ramp = PartCoolingFanRamp::new(PartCoolingFanRampConfig {
            min_speed: first_percent(&filament.fan_min_speed.0),
            max_speed,
            full_speed_layer: first_non_negative(&filament.full_fan_speed_layer.0) as u32,
            close_fan_first_layers: first_non_negative(&filament.close_fan_the_first_x_layers.0)
                as u32,
            layer_times_s: [
                slow_down_layer_time,
                filament
                    .fan_cooling_layer_time
                    .0
                    .first()
                    .map_or(0.0, |value| value.0),
            ],
            fan_kickstart_s: runtime.fan_kickstart.0,
            reduce_fan_stop_start_freq: first_bool(&filament.reduce_fan_stop_start_freq.0),
        });
        let gcode = &traversal.resolved.views.runtime_gcode;
        let slope = gcode.max_volumetric_extrusion_rate_slope.0;
        let equalizer = (slope > 0.0).then(|| {
            crate::project_slice::pressure_equalizer::PressureEqualizerPass::new(
                slope,
                gcode.max_volumetric_extrusion_rate_slope_segment_length.0,
                gcode.extrusion_rate_smoothing_external_perimeter_only.0,
                &gcode
                    .filament_diameter
                    .0
                    .iter()
                    .map(|value| value.0)
                    .collect::<Vec<f64>>(),
                gcode.use_relative_e_distances.0,
            )
        });
        Self {
            equalizer,
            part_speed: None,
            physical_part_speed: 0,
            provisional_part_speed: 0,
            pending_layer_index: None,
            equalized_layer_result: None,
            buffered_gcode: Vec::new(),
            part_fan_ramp,
            additional_speed: None,
            additional_fan_speed: first_percent_int(&filament.additional_cooling_fan_speed.0),
            auxiliary_fan: runtime.auxiliary_fan.0,
            part_cooling_fan_min_pwm: runtime.part_cooling_fan_min_pwm.0.clamp(0, 100) as u8,
            gcode_comments: full.process.print.gcode_comments.0,
            feedrate: feedrate::State::new(
                writer
                    .extruders
                    .ids()
                    .iter()
                    .map(|&id| {
                        (
                            id,
                            feedrate::Config {
                                enabled: filament
                                    .slow_down_for_layer_cooling
                                    .0
                                    .get(id)
                                    .unwrap_or(&filament.slow_down_for_layer_cooling.0[0])
                                    .0,
                                target_time: filament
                                    .slow_down_layer_time
                                    .0
                                    .get(id)
                                    .unwrap_or(&filament.slow_down_layer_time.0[0])
                                    .0 as f32,
                                minimum_speed: filament
                                    .slow_down_min_speed
                                    .0
                                    .get(id)
                                    .unwrap_or(&filament.slow_down_min_speed.0[0])
                                    .0 as f32,
                                keep_outer_wall_speed: filament
                                    .dont_slow_down_outer_wall
                                    .0
                                    .get(id)
                                    .unwrap_or(&filament.dont_slow_down_outer_wall.0[0])
                                    .0,
                                relative_e: runtime.use_relative_e_distances.0,
                                keep_markers: slope > 0.0,
                            },
                        )
                    })
                    .collect(),
                runtime.travel_speed.0,
                super::motion::tool_selection::prefix(traversal),
                writer.initial_filaments.initial(),
            ),
        }
    }

    pub(super) fn begin_layer(&mut self, output: &mut Vec<u8>, layer_index: usize) {
        self.pending_layer_index = Some(layer_index);
        self.provisional_part_speed = self
            .part_fan_ramp
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
        let layer_time = feedrate::rewrite_layer(&mut layer, 0, &mut self.feedrate);
        if equalized {
            let stripped = strip_pressure_markers(&String::from_utf8_lossy(&layer));
            layer = stripped.into_bytes();
        }
        output.extend_from_slice(&layer);
        let part_speed = self
            .part_fan_ramp
            .speed_for_layer_time(layer_index, Some(f64::from(layer_time)))
            .unwrap_or(0);
        let mut replacement = if self.part_speed != Some(part_speed) {
            let speed = clamped_part_speed(part_speed, self.part_cooling_fan_min_pwm);
            let comment = if self.gcode_comments {
                if speed == 0 {
                    " ; disable fan"
                } else {
                    " ; enable fan"
                }
            } else {
                ""
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
            self.physical_part_speed = part_speed;
        }
        let additional_speed = if layer_index < self.part_fan_ramp.close_fan_first_layers() as usize
        {
            0
        } else {
            self.additional_fan_speed
        };
        if self.auxiliary_fan && self.additional_speed != Some(additional_speed) {
            self.additional_speed = Some(additional_speed);
            let comment = if self.gcode_comments {
                if additional_speed == 0 {
                    " ; disable additional fan "
                } else {
                    " ; enable additional fan "
                }
            } else {
                ""
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
        self.resolve_role_fans(output, layer_start, part_speed, layer_index);
        self.part_speed = Some(part_speed);
        self.provisional_part_speed = part_speed;
    }
}

fn first_percent(values: &[crate::OrcaFloat]) -> f64 {
    values
        .first()
        .map_or(0.0, |value| value.0.clamp(0.0, 100.0))
}

fn first_percent_int(values: &[crate::OrcaInt]) -> u8 {
    values
        .first()
        .map_or(0, |value| value.0.clamp(0, 100) as u8)
}

fn first_non_negative(values: &[crate::OrcaInt]) -> usize {
    values.first().map_or(0, |value| value.0.max(0) as usize)
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
