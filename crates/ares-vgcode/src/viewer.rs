// Ports the view-range logic of AGPL-licensed OrcaSlicer `src/libvgcode/src/ViewerImpl.hpp` and `src/ViewerImpl.cpp`.

use crate::{
    GCodeExtrusionRole, GCodeInputData, Interval, Layers, MoveType, OptionType, PathVertex,
    ViewRange, move_type_to_option, settings::Settings,
};

/// `ViewerImpl` state behind the layer and view ranges: vertices, layers,
/// visibility settings and the full / enabled / visible `ViewRange`. Textures,
/// colors, times and markers belong to the renderer and are not ported.
#[derive(Clone, Debug, Default)]
pub struct ViewerImpl {
    settings: Settings,
    layers: Layers,
    view_range: ViewRange,
    vertices: Vec<PathVertex>,
}

/// `is_visible` (ViewerImpl.cpp:1713-1726): options by option visibility,
/// extrusions by role visibility, other moves hidden.
fn is_visible(v: &PathVertex, settings: &Settings) -> bool {
    match move_type_to_option(v.move_type) {
        Some(option) => settings.options_visibility[option.index()],
        None => {
            v.move_type == MoveType::Extrude && settings.extrusion_roles_visibility[v.role.index()]
        }
    }
}

impl ViewerImpl {
    /// `load` (ViewerImpl.cpp:985-1125): the vertex, layer and view range part.
    /// Per-vertex `layer_duration` is not populated.
    pub fn load(&mut self, data: GCodeInputData) {
        if data.vertices.is_empty() {
            return;
        }
        self.layers.reset();
        self.view_range.reset();
        self.vertices = data.vertices;
        self.settings.spiral_vase_mode = data.spiral_vase_mode;
        for (i, v) in (0..).zip(&self.vertices) {
            self.layers.update(v, i);
        }
        if !self.layers.empty() {
            self.layers.set_view_range(0, self.layers.count() - 1);
        }
        self.update_view_full_range();
        self.view_range
            .set_visible_interval(self.view_range.get_enabled());
    }

    pub fn vertices(&self) -> &[PathVertex] {
        &self.vertices
    }

    pub fn layers(&self) -> &Layers {
        &self.layers
    }

    pub fn is_option_visible(&self, option: OptionType) -> bool {
        self.settings.options_visibility[option.index()]
    }

    /// ViewerImpl.cpp:1419-1433.
    pub fn toggle_option_visibility(&mut self, option: OptionType) {
        let visible = &mut self.settings.options_visibility[option.index()];
        *visible = !*visible;
        let old_enabled_range = self.view_range.get_enabled();
        self.update_view_full_range();
        let new_enabled_range = self.view_range.get_enabled();
        if old_enabled_range != new_enabled_range {
            let visible_range = self.view_range.get_visible();
            if old_enabled_range == visible_range {
                self.view_range.set_visible_interval(new_enabled_range);
            } else if self.settings.top_layer_only_view_range
                && new_enabled_range[0] < visible_range[0]
            {
                self.view_range
                    .set_visible(new_enabled_range[0], visible_range[1]);
            }
        }
    }

    pub fn is_extrusion_role_visible(&self, role: GCodeExtrusionRole) -> bool {
        self.settings.extrusion_roles_visibility[role.index()]
    }

    /// ViewerImpl.cpp:1440-1446.
    pub fn toggle_extrusion_role_visibility(&mut self, role: GCodeExtrusionRole) {
        let visible = &mut self.settings.extrusion_roles_visibility[role.index()];
        *visible = !*visible;
        self.update_view_full_range();
    }

    /// ViewerImpl.cpp:1342-1347.
    pub fn toggle_top_layer_only_view_range(&mut self) {
        self.settings.top_layer_only_view_range = !self.settings.top_layer_only_view_range;
        self.update_view_full_range();
        self.view_range
            .set_visible_interval(self.view_range.get_enabled());
    }

    pub fn get_layers_view_range(&self) -> Interval {
        self.layers.get_view_range()
    }

    /// ViewerImpl.cpp:1329-1340.
    pub fn set_layers_view_range(&mut self, min: usize, max: usize) {
        let last = self.layers.count().saturating_sub(1);
        self.layers.set_view_range(min.min(last), max.min(last));
        self.update_view_full_range();
        self.view_range
            .set_visible_interval(self.view_range.get_enabled());
    }

    pub fn get_view_full_range(&self) -> Interval {
        self.view_range.get_full()
    }

    pub fn get_view_enabled_range(&self) -> Interval {
        self.view_range.get_enabled()
    }

    pub fn get_view_visible_range(&self) -> Interval {
        self.view_range.get_visible()
    }

    /// ViewerImpl.hpp:132.
    pub fn get_current_vertex_id(&self) -> usize {
        self.view_range.get_visible()[1]
    }

    /// `update_view_full_range` (ViewerImpl.cpp:1728-1815).
    fn update_view_full_range(&mut self) {
        let layers_range = self.layers.get_view_range();
        let Some(first) = self.full_range_start(layers_range[0]) else {
            self.view_range.set_full(0, 0);
            return;
        };
        let last = self.full_range_end(first, layers_range[1]);
        if first == last {
            self.view_range.set_full(0, 0);
        } else {
            self.view_range.set_full(first, last);
        }

        if self.settings.top_layer_only_view_range {
            let full = self.view_range.get_full();
            self.view_range
                .set_enabled(self.top_layer_start(full[0], layers_range), full[1]);
        } else {
            self.view_range
                .set_enabled_interval(self.view_range.get_full());
        }
    }

    fn extendable(&self, v: &PathVertex) -> bool {
        (self.is_option_visible(OptionType::Travels) && v.is_travel())
            || (self.is_option_visible(OptionType::Wipes) && v.is_wipe())
    }

    /// ViewerImpl.cpp:1734-1756: first visible vertex from the bottom layer.
    fn full_range_start(&self, bottom_layer: usize) -> Option<usize> {
        let mut first = self
            .vertices
            .iter()
            .position(|v| v.layer_id as usize >= bottom_layer && is_visible(v, &self.settings))?;
        // If the first vertex is an extrusion, add an extra step to properly detect the first segment
        if first > 0 && self.vertices[first].move_type == MoveType::Extrude {
            first -= 1;
        }
        // if the global range starts with a travel/wipe move, extend it to the travel/wipe start
        while first > 0 && self.extendable(&self.vertices[first]) {
            first -= 1;
        }
        Some(first)
    }

    /// ViewerImpl.cpp:1758-1789: last vertex up to the top layer, without
    /// disabled trailing options, extended over a trailing travel / wipe.
    fn full_range_end(&self, first: usize, top_layer: usize) -> usize {
        let vertices = &self.vertices;
        let mut last = first
            + vertices[first..]
                .iter()
                .take_while(|v| v.layer_id as usize <= top_layer)
                .count();
        if last != first {
            last -= 1;
        }
        // remove disabled trailing options, if any
        while last != first && !is_visible(&vertices[last], &self.settings) {
            last -= 1;
        }
        // if the global range ends with a travel/wipe move, extend it to the travel/wipe end
        while last + 1 < vertices.len()
            && self.extendable(&vertices[last])
            && vertices[last + 1].move_type == vertices[last].move_type
        {
            last += 1;
        }
        last
    }

    /// ViewerImpl.cpp:1794-1808: one step before the first visible vertex of
    /// the top layer, one more in spiral vase mode with a single layer shown.
    fn top_layer_start(&self, full_first: usize, layers_range: Interval) -> usize {
        let mut top_first = full_first;
        let mut shortened = false;
        while top_first < self.vertices.len()
            && ((self.vertices[top_first].layer_id as usize) < layers_range[1]
                || !is_visible(&self.vertices[top_first], &self.settings))
        {
            top_first += 1;
            shortened = true;
        }
        if shortened {
            top_first -= 1;
        }
        if self.settings.spiral_vase_mode
            && layers_range[0] > 0
            && layers_range[0] == layers_range[1]
        {
            top_first -= 1;
        }
        top_first
    }
}

#[cfg(test)]
mod tests;
