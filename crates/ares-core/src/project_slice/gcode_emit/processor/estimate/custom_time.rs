//! `process_custom_gcode_time`, `finalize`, `get_custom_gcode_times`.

use crate::gcode_processor::{CustomGCodeTime, CustomGCodeType};

#[derive(Default)]
pub(super) struct CustomTimes {
    records: Vec<(CustomGCodeType, f32)>,
    cache: f32,
    registered: bool,
}

impl CustomTimes {
    pub(super) fn add(&mut self, time: f64) {
        self.cache = (f64::from(self.cache) + time) as f32;
    }

    pub(super) fn register(&mut self, kind: CustomGCodeType) {
        self.registered = true;
        if self.cache != 0.0 {
            self.records.push((kind, self.cache));
        }
        self.cache = 0.0;
    }

    pub(super) fn finish(mut self, trailing_delay: f64, total: f64) -> Vec<CustomGCodeTime> {
        if !self.registered {
            return Vec::new();
        }
        self.cache += trailing_delay as f32;
        self.register(CustomGCodeType::ColorChange);
        let mut elapsed = 0.0_f32;
        self.records
            .into_iter()
            .map(|(kind, duration)| {
                let remaining = (total - f64::from(elapsed)) as f32;
                elapsed += duration;
                CustomGCodeTime {
                    kind,
                    duration,
                    remaining,
                }
            })
            .collect()
    }
}
