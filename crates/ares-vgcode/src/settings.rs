// Ports the view-range subset of AGPL-licensed OrcaSlicer `src/libvgcode/src/Settings.hpp`.

use crate::{GCodeExtrusionRole, OptionType};

/// `Settings` visibility fields used by the view range. View type, time mode
/// and the update flags belong to the renderer and are not ported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub top_layer_only_view_range: bool,
    pub spiral_vase_mode: bool,
    pub options_visibility: [bool; OptionType::COUNT],
    pub extrusion_roles_visibility: [bool; GCodeExtrusionRole::COUNT],
}

impl Default for Settings {
    fn default() -> Self {
        let mut options_visibility = [false; OptionType::COUNT];
        options_visibility[OptionType::Seams.index()] = true;
        Self {
            top_layer_only_view_range: false,
            spiral_vase_mode: false,
            options_visibility,
            extrusion_roles_visibility: [true; GCodeExtrusionRole::COUNT],
        }
    }
}
