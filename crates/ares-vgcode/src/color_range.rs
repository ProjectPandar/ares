// Ports AGPL-licensed OrcaSlicer `src/libvgcode/include/ColorRange.hpp`,
// `src/libvgcode/src/ColorRange.cpp` and `round_to_bin` (`src/ViewerImpl.cpp:47-63`).

use crate::{Color, ColorRangeType, Palette, lerp_color};

/// `DEFAULT_RANGES_COLORS` (`ColorRange.hpp`): bluish to reddish.
pub const DEFAULT_RANGES_COLORS: [Color; 11] = [
    [11, 44, 122],
    [19, 89, 133],
    [28, 136, 145],
    [4, 214, 15],
    [170, 242, 0],
    [252, 249, 3],
    [245, 206, 10],
    [227, 136, 32],
    [209, 104, 48],
    [194, 82, 60],
    [148, 38, 22],
];

/// `libvgcode::ColorRange`: a value range detected while loading, mapped onto a palette.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorRange {
    range_type: ColorRangeType,
    palette: Palette,
    /// `[min, max]`.
    range: [f32; 2],
    /// Count of values passed to `update` that differed from the bounds.
    count: usize,
}

impl ColorRange {
    pub fn new(range_type: ColorRangeType) -> Self {
        Self {
            range_type,
            palette: DEFAULT_RANGES_COLORS.to_vec(),
            range: [f32::MAX, -f32::MAX],
            count: 0,
        }
    }

    pub fn get_type(&self) -> ColorRangeType {
        self.range_type
    }

    pub fn get_palette(&self) -> &[Color] {
        &self.palette
    }

    pub fn set_palette(&mut self, palette: Palette) {
        if palette.len() > 1 {
            self.palette = palette;
        }
    }

    pub fn get_range(&self) -> [f32; 2] {
        self.range
    }

    /// `get_step_size` (`ColorRange.cpp:17-33`).
    fn step_size(&self) -> f32 {
        let [min, max] = self.range;
        let bins = self.palette.len() as f32 - 1.0;
        match self.range_type {
            ColorRangeType::Linear => (max - min) / bins,
            ColorRangeType::Logarithmic if min != 0.0 => (max / min).ln() / bins,
            ColorRangeType::Logarithmic => 0.0,
        }
    }

    /// `get_color_at` (`ColorRange.cpp:57-80`).
    pub fn get_color_at(&self, value: f32) -> Color {
        let [min, max] = self.range;
        // `std::clamp`, which keeps the low bound of an unset (inverted) range.
        let value = if value < min {
            min
        } else if max < value {
            max
        } else {
            value
        };
        let step = self.step_size();
        let mut global_t = 0.0;
        if step > 0.0 {
            match self.range_type {
                ColorRangeType::Logarithmic if min != 0.0 => global_t = (value / min).ln() / step,
                ColorRangeType::Logarithmic => {}
                ColorRangeType::Linear => global_t = (value - min) / step,
            }
        }
        let max_idx = self.palette.len() - 1;
        let low = (global_t as usize).min(max_idx);
        let high = (low + 1).min(max_idx);
        lerp_color(self.palette[low], self.palette[high], global_t - low as f32)
    }

    /// `get_values` (`ColorRange.cpp:87-115`): one or two detected values, else one per palette color.
    pub fn get_values(&self) -> Vec<f32> {
        let [min, max] = self.range;
        match self.count {
            1 => vec![min],
            2 => vec![min, max],
            _ => {
                let step = self.step_size();
                (0..self.palette.len())
                    .map(|i| match self.range_type {
                        ColorRangeType::Linear => min + i as f32 * step,
                        // `::exp(::log(float))` runs in double.
                        ColorRangeType::Logarithmic => {
                            (f64::from(min).ln() + f64::from(i as f32 * step)).exp() as f32
                        }
                    })
                    .collect()
            }
        }
    }

    /// `update` (`ColorRange.cpp:123-130`).
    pub(crate) fn update(&mut self, value: f32) {
        if value != self.range[0] && value != self.range[1] {
            self.count += 1;
        }
        self.range = [self.range[0].min(value), self.range[1].max(value)];
    }

    pub(crate) fn reset(&mut self) {
        self.range = [f32::MAX, -f32::MAX];
        self.count = 0;
    }
}

/// `round_to_bin` (`ViewerImpl.cpp:47-63`): at least two significant digits.
pub(crate) fn round_to_bin(value: f32) -> f32 {
    const SCALE: [f32; 5] = [100.0, 1000.0, 10000.0, 100000.0, 1000000.0];
    const INVSCALE: [f32; 5] = [0.01, 0.001, 0.0001, 0.00001, 0.000001];
    const THRESHOLD: [f32; 5] = [0.095, 0.0095, 0.00095, 0.000095, 0.0000095];
    let mut i = 0;
    while value < THRESHOLD[i] && i < 4 {
        i += 1;
    }
    let a = f64::from(value * SCALE[i]);
    // `fast_round_up` (`ViewerImpl.cpp:37-43`).
    let rounded = if a == 0.499_999_999_999_999_94 {
        0
    } else {
        (a + 0.5).floor() as i64
    };
    rounded as f32 * INVSCALE[i]
}
