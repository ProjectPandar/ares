mod feedrate;

#[cfg(test)]
mod tests;

use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) struct CoolingState {
    part_speed: u8,
    additional_speed: u8,
    max_speed: u8,
    close_fan_first_layers: usize,
    full_fan_speed_layer: usize,
    additional_fan_speed: u8,
    auxiliary_fan: bool,
    part_cooling_fan_min_pwm: u8,
    /// Non-BBL printers emit the initial fan state at the first layer
    /// boundary; BBL machines carry it inside their start sequence.
    emit_initial_fan: bool,
    fan_mover_enabled: bool,
    feedrate: feedrate::State,
}

impl CoolingState {
    pub(super) fn from_traversal(traversal: &PreparedPostClassicTraversal) -> Self {
        let full = &traversal.resolved.views.full;
        let filament = &full.filament.print;
        let runtime = &traversal.resolved.views.runtime_gcode;
        let first_bool = |values: &[crate::OrcaBool]| values.first().is_some_and(|value| value.0);
        Self {
            part_speed: 0,
            additional_speed: 0,
            max_speed: first_percent(&filament.fan_max_speed.0),
            close_fan_first_layers: first_non_negative(&filament.close_fan_the_first_x_layers.0),
            full_fan_speed_layer: first_non_negative(&filament.full_fan_speed_layer.0),
            additional_fan_speed: first_percent_int(&filament.additional_cooling_fan_speed.0),
            auxiliary_fan: runtime.auxiliary_fan.0,
            part_cooling_fan_min_pwm: runtime.part_cooling_fan_min_pwm.0.clamp(0, 100) as u8,
            emit_initial_fan: !super::tags::Tags::of(traversal).is_bbl(),
            fan_mover_enabled: runtime.fan_speedup_time.0 != 0.0 || runtime.fan_kickstart.0 > 0.0,
            feedrate: feedrate::State::new(
                feedrate::Config {
                    enabled: first_bool(&filament.slow_down_for_layer_cooling.0),
                    target_time: filament
                        .slow_down_layer_time
                        .0
                        .first()
                        .map_or(0.0, |value| value.0 as f32),
                    minimum_speed: filament
                        .slow_down_min_speed
                        .0
                        .first()
                        .map_or(0.0, |value| value.0 as f32),
                    keep_outer_wall_speed: first_bool(&filament.dont_slow_down_outer_wall.0),
                    relative_e: runtime.use_relative_e_distances.0,
                },
                runtime.travel_speed.0,
            ),
        }
    }

    pub(super) fn begin_layer(&mut self, output: &mut Vec<u8>, layer_index: usize) {
        let part_speed = self.part_speed_for_layer(layer_index);
        let initial = should_emit_initial_part_fan(
            layer_index,
            self.emit_initial_fan,
            self.fan_mover_enabled,
            part_speed,
        );
        if part_speed != self.part_speed || initial {
            self.part_speed = part_speed;
            let speed = if part_speed > 0 && part_speed < self.part_cooling_fan_min_pwm {
                self.part_cooling_fan_min_pwm
            } else {
                part_speed
            };
            output.extend_from_slice(format!("M106 S{}\n", part_fan_pwm(speed)).as_bytes());
        }

        let additional_speed = if layer_index < self.close_fan_first_layers {
            0
        } else {
            self.additional_fan_speed
        };
        // Orca emits the initial auxiliary-fan state unconditionally at the
        // first layer boundary (print-start block, before LAYER_CHANGE), not
        // only when the speed changes.
        if self.auxiliary_fan
            && (additional_speed != self.additional_speed
                || (layer_index == 0 && self.emit_initial_fan))
        {
            self.additional_speed = additional_speed;
            output.extend_from_slice(
                format!("M106 P2 S{}\n", additional_fan_pwm(additional_speed)).as_bytes(),
            );
        }
    }

    pub(super) const fn part_speed(&self) -> u8 {
        self.part_speed
    }

    pub(super) fn finish_layer(&mut self, output: &mut Vec<u8>, layer_start: usize) {
        feedrate::rewrite_layer(output, layer_start, &mut self.feedrate);
    }

    fn part_speed_for_layer(&self, layer_index: usize) -> u8 {
        if layer_index < self.close_fan_first_layers {
            return 0;
        }
        if self.full_fan_speed_layer <= self.close_fan_first_layers
            || layer_index + 1 >= self.full_fan_speed_layer
        {
            return self.max_speed;
        }
        let numerator = layer_index + 1 - self.close_fan_first_layers;
        let denominator = self.full_fan_speed_layer - self.close_fan_first_layers;
        let speed = f64::from(self.max_speed) * numerator as f64 / denominator as f64;
        (speed + 0.5).floor().clamp(0.0, 100.0) as u8
    }
}

fn should_emit_initial_part_fan(
    layer_index: usize,
    emit_initial_fan: bool,
    fan_mover_enabled: bool,
    speed: u8,
) -> bool {
    layer_index == 0 && emit_initial_fan && !(fan_mover_enabled && speed == 0)
}

fn first_percent(values: &[crate::OrcaFloat]) -> u8 {
    values
        .first()
        .map_or(0, |value| value.0.round().clamp(0.0, 100.0) as u8)
}

fn first_percent_int(values: &[crate::OrcaInt]) -> u8 {
    values
        .first()
        .map_or(0, |value| value.0.clamp(0, 100) as u8)
}

fn first_non_negative(values: &[crate::OrcaInt]) -> usize {
    values.first().map_or(0, |value| value.0.max(0) as usize)
}

fn part_fan_pwm(speed: u8) -> u32 {
    (255.5 * f64::from(speed) / 100.0).floor() as u32
}

fn additional_fan_pwm(speed: u8) -> u32 {
    (255.0 * f64::from(speed) / 100.0).floor() as u32
}
