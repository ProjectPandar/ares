// Ports rendering-neutral data from AGPL-licensed OrcaSlicer `src/libvgcode/include/GCodeInputData.hpp` and `include/ColorPrint.hpp`.

use crate::{Palette, PathVertex, TimeMode};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GCodeInputData {
    pub spiral_vase_mode: bool,
    pub vertices: Vec<PathVertex>,
    pub tools_colors: Palette,
    pub color_print_colors: Palette,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorPrint {
    pub extruder_id: u8,
    pub color_id: u8,
    pub layer_id: u32,
    pub times: [f32; TimeMode::COUNT],
}

impl Default for ColorPrint {
    fn default() -> Self {
        Self {
            extruder_id: 0,
            color_id: 0,
            layer_id: 0,
            times: [0.0; TimeMode::COUNT],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_data_default_is_empty_and_not_spiral_vase() {
        let data = GCodeInputData::default();
        assert!(!data.spiral_vase_mode);
        assert!(data.vertices.is_empty());
        assert!(data.tools_colors.is_empty());
        assert!(data.color_print_colors.is_empty());
    }

    #[test]
    fn color_print_default_ids_and_times_are_zero() {
        let color_print = ColorPrint::default();
        assert_eq!(color_print.extruder_id, 0);
        assert_eq!(color_print.color_id, 0);
        assert_eq!(color_print.layer_id, 0);
        assert_eq!(color_print.times, [0.0, 0.0]);
    }
}
