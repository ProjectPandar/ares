// Ports AGPL-licensed OrcaSlicer `src/libvgcode/src/Settings.hpp`.

use crate::{GCodeExtrusionRole, OptionType, TimeMode, ViewType};

/// `Settings` fields used by the view range, the enabled entities and the
/// vertex colors. The textures update flag belongs to the renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub view_type: ViewType,
    pub time_mode: TimeMode,
    pub update_colors: bool,
    pub update_enabled_entities: bool,
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
            view_type: ViewType::FeatureType,
            time_mode: TimeMode::Normal,
            update_colors: true,
            update_enabled_entities: true,
            top_layer_only_view_range: false,
            spiral_vase_mode: false,
            options_visibility,
            extrusion_roles_visibility: [true; GCodeExtrusionRole::COUNT],
        }
    }
}
