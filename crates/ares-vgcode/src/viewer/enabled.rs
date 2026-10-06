// Ports `update_enabled_entities` (`:1130-1215`) and the `valid_lines_bitset`
// part of `extract_pos_and_or_hwa` (`:911-943`) of AGPL-licensed OrcaSlicer
// `src/libvgcode/src/ViewerImpl.cpp`.

use super::ViewerImpl;
use crate::{MoveType, OptionType, PathVertex, move_type_to_option};

/// `valid_lines_bitset` (`ViewerImpl.cpp:925-943`): the line `i -> i + 1` is
/// drawn when the next vertex moved and has the same type, and `i` is no seam.
pub(super) fn valid_lines(vertices: &[PathVertex]) -> Vec<bool> {
    let mut valid: Vec<bool> = vertices
        .windows(2)
        .map(|pair| {
            pair[1].position != pair[0].position
                && pair[1].move_type == pair[0].move_type
                && pair[0].move_type != MoveType::Seam
        })
        .collect();
    valid.push(false);
    valid
}

impl ViewerImpl {
    /// Vertex ids of the enabled segments (`i -> i + 1`), refreshed as
    /// `render` does when `update_enabled_entities` is set (`ViewerImpl.cpp:1294-1295`).
    pub fn enabled_segments(&mut self) -> &[u32] {
        if self.settings.update_enabled_entities {
            self.update_enabled_entities();
        }
        &self.enabled_segments
    }

    /// Vertex ids of the enabled option markers.
    pub fn enabled_options(&mut self) -> &[u32] {
        if self.settings.update_enabled_entities {
            self.update_enabled_entities();
        }
        &self.enabled_options
    }

    /// `update_enabled_entities` (`ViewerImpl.cpp:1130-1184, 1215`).
    pub(super) fn update_enabled_entities(&mut self) {
        self.settings.update_enabled_entities = false;
        self.enabled_segments.clear();
        self.enabled_options.clear();
        if self.vertices.is_empty() {
            return;
        }
        let mut range = self.view_range.get_visible();
        // when top layer only visualization is enabled, we need to render
        // all the toolpaths in the other layers as grayed, so extend the range
        // to contain them
        if self.settings.top_layer_only_view_range {
            range[0] = self.view_range.get_full()[0];
        }
        // to show the options at the current tool marker position we need to extend the range by one extra step
        if self.vertices[range[1]].is_option() && range[1] < self.vertices.len() - 1 {
            range[1] += 1;
        }
        if self.settings.spiral_vase_mode {
            // when spiral vase mode is enabled and only one layer is shown, extend the range by one step
            let layers_range = self.layers.get_view_range();
            if layers_range[0] > 0 && layers_range[0] == layers_range[1] {
                range[0] -= 1;
            }
        }
        for i in range[0]..range[1] {
            let v = &self.vertices[i];
            if !self.valid_lines[i] && !v.is_option() {
                continue;
            }
            let visible = if v.is_travel() {
                self.is_option_visible(OptionType::Travels)
            } else if v.is_wipe() {
                self.is_option_visible(OptionType::Wipes)
            } else if v.is_option() {
                move_type_to_option(v.move_type)
                    .is_some_and(|option| self.is_option_visible(option))
            } else {
                v.is_extrusion() && self.is_extrusion_role_visible(v.role)
            };
            if !visible {
                continue;
            }
            if v.is_option() {
                self.enabled_options.push(i as u32);
            } else {
                self.enabled_segments.push(i as u32);
            }
        }
    }
}
