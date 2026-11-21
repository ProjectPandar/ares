mod role;
pub(crate) use role::InternalBridgeFanSpeed;

#[derive(Clone, Copy)]
pub(crate) struct PartCoolingFanRampConfig {
    pub(crate) min_speed: f64,
    pub(crate) max_speed: f64,
    pub(crate) full_speed_layer: u32,
    pub(crate) close_fan_first_layers: u32,
    pub(crate) layer_times_s: [f64; 2],
    pub(crate) fan_kickstart_s: f64,
    pub(crate) reduce_fan_stop_start_freq: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PartCoolingFanRamp {
    min_speed: f64,
    max_speed: f64,
    full_speed_layer: u32,
    close_fan_first_layers: u32,
    slow_down_layer_time_s: f64,
    fan_cooling_layer_time_s: f64,
    fan_kickstart_s: f64,
    reduce_fan_stop_start_freq: bool,
}

impl PartCoolingFanRamp {
    pub(crate) const fn new(config: PartCoolingFanRampConfig) -> Self {
        Self {
            min_speed: config.min_speed,
            max_speed: config.max_speed,
            full_speed_layer: config.full_speed_layer,
            close_fan_first_layers: config.close_fan_first_layers,
            slow_down_layer_time_s: config.layer_times_s[0],
            fan_cooling_layer_time_s: config.layer_times_s[1],
            fan_kickstart_s: config.fan_kickstart_s,
            reduce_fan_stop_start_freq: config.reduce_fan_stop_start_freq,
        }
    }

    pub(crate) const fn close_fan_first_layers(self) -> u32 {
        self.close_fan_first_layers
    }

    pub(crate) fn speed_for_layer_time(
        self,
        layer_index: usize,
        layer_time_s: Option<f64>,
    ) -> Option<u8> {
        if self.max_speed == 0.0 {
            return None;
        }
        let layer_id = u32::try_from(layer_index).unwrap_or(u32::MAX);
        if layer_id < self.close_fan_first_layers {
            return None;
        }
        let base_speed = match layer_time_s {
            Some(layer_time_s) if layer_time_s.is_finite() => {
                self.base_speed_for_runtime_layer_time(layer_time_s)
            }
            _ => self.base_speed_for_layer(layer_index),
        };
        if layer_id.saturating_add(1) >= self.full_speed_layer
            || self.full_speed_layer <= self.close_fan_first_layers
        {
            return Some(base_speed.floor().clamp(0.0, 100.0) as u8);
        }
        let denominator = self.full_speed_layer - self.close_fan_first_layers;
        let numerator = layer_id + 1 - self.close_fan_first_layers;
        let factor = f64::from(numerator) / f64::from(denominator);
        let speed = base_speed * factor;
        Some((speed + 0.5).floor().clamp(0.0, 100.0) as u8)
    }

    pub(crate) fn role_speed_for_layer(self, layer_index: usize, raw_speed: u8) -> Option<u8> {
        let layer_id = u32::try_from(layer_index).unwrap_or(u32::MAX);
        if layer_id < self.close_fan_first_layers {
            return None;
        }
        if layer_id.saturating_add(1) >= self.full_speed_layer
            || self.full_speed_layer <= self.close_fan_first_layers
        {
            return Some(raw_speed);
        }
        let denominator = self.full_speed_layer - self.close_fan_first_layers;
        let numerator = layer_id + 1 - self.close_fan_first_layers;
        let factor = numerator as f32 / denominator as f32;
        Some(
            (f32::from(raw_speed) * factor + 0.5)
                .floor()
                .clamp(0.0, 100.0) as u8,
        )
    }

    fn base_speed_for_layer(self, layer_index: usize) -> f64 {
        if self.full_speed_layer <= 1 {
            return self.max_speed;
        }
        let layer_number = u32::try_from(layer_index)
            .ok()
            .and_then(|index| index.checked_add(1))
            .unwrap_or(u32::MAX);
        if layer_number >= self.full_speed_layer {
            return self.max_speed;
        }
        let span = f64::from(self.full_speed_layer - 1);
        let factor = f64::from(layer_number - 1) / span;
        self.min_speed + (self.max_speed - self.min_speed) * factor
    }

    fn base_speed_for_runtime_layer_time(self, layer_time_s: f64) -> f64 {
        // CoolingBuffer.cpp:740,756-768: float locals and arithmetic, then
        // widen the quotient to double. Keep the incoming fan baseline.
        let layer_time = layer_time_s as f32;
        let slow_down_layer_time = self.slow_down_layer_time_s as f32;
        let fan_cooling_layer_time = self.fan_cooling_layer_time_s as f32;
        let fan_min_speed = self.min_speed as f32;
        let fan_max_speed = self.max_speed as f32;
        let mut fan_speed_new = self.long_layer_baseline_speed() as f32;
        if layer_time < slow_down_layer_time {
            fan_speed_new = fan_max_speed;
        } else if layer_time < fan_cooling_layer_time {
            let t = f64::from(
                (layer_time - slow_down_layer_time)
                    / (fan_cooling_layer_time - slow_down_layer_time),
            );
            // The +0.5 is outside floor, before upstream's int conversion.
            fan_speed_new = ((t * f64::from(fan_min_speed) + (1.0 - t) * f64::from(fan_max_speed))
                .floor()
                + 0.5) as i32 as f32;
        }
        f64::from(fan_speed_new)
    }

    const fn long_layer_baseline_speed(self) -> f64 {
        if self.reduce_fan_stop_start_freq {
            self.min_speed
        } else {
            0.0
        }
    }
}
