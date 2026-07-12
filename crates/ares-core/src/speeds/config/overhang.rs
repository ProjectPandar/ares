const MIN_DYNAMIC_OVERHANG_SPEED_MM_S: f64 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverhangSpeedBands {
    external_line_width_mm: f64,
    speeds_mm_s: [Option<f64>; 4],
    final_severe_speed_mm_s: Option<f64>,
}

impl OverhangSpeedBands {
    pub const fn new(
        external_line_width_mm: f64,
        speeds_mm_s: [Option<f64>; 4],
        final_severe_speed_mm_s: Option<f64>,
    ) -> Self {
        Self {
            external_line_width_mm,
            speeds_mm_s,
            final_severe_speed_mm_s,
        }
    }

    pub const fn disabled(external_line_width_mm: f64) -> Self {
        Self {
            external_line_width_mm,
            speeds_mm_s: [None; 4],
            final_severe_speed_mm_s: None,
        }
    }

    pub(super) fn speed_for_unsupported_span_mm(self, unsupported_span_mm: f64) -> Option<f64> {
        if self.external_line_width_mm <= 0.0 || unsupported_span_mm <= 0.0 {
            return None;
        }
        let ratio = unsupported_span_mm / self.external_line_width_mm;
        let speed = if ratio > 1.0 {
            self.final_severe_speed_mm_s
        } else {
            self.speeds_mm_s[band_index(ratio)]
        };
        speed.filter(|speed| speed.is_finite() && *speed >= MIN_DYNAMIC_OVERHANG_SPEED_MM_S)
    }
}

fn band_index(ratio: f64) -> usize {
    if ratio <= 0.25 {
        0
    } else if ratio <= 0.5 {
        1
    } else if ratio <= 0.75 {
        2
    } else {
        3
    }
}
