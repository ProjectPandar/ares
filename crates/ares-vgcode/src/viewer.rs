// Ports the view-range and color logic of AGPL-licensed OrcaSlicer `src/libvgcode/src/ViewerImpl.hpp` and `src/ViewerImpl.cpp`.

use std::collections::BTreeMap;

use crate::extrusion_roles::add_times;
use crate::{
    Color, ColorPrint, DUMMY_COLOR, ExtrusionRoles, GCodeExtrusionRole, GCodeInputData, Interval,
    Layers, MoveType, OptionType, Palette, PathVertex, TimeMode, ViewRange, move_type_to_option,
    settings::Settings,
};

mod color_print;
mod colors;
mod custom_gcode_overview;
pub use color_print::{
    ColorPrintLegend, ColorPrintRange, ExtruderColorPrints, MissingColorVolume, PartialTime,
    PartialTimeKind, PrintedColor, color_print_legend,
};
pub use custom_gcode_overview::{CustomGCodeOverviewRow, custom_gcode_overview};
mod enabled;
mod heights_widths;
pub use colors::{DEFAULT_EXTRUSION_ROLES_COLORS, DEFAULT_OPTIONS_COLORS};

/// `ViewerImpl` state behind the layer and view ranges: vertices, layers,
/// visibility settings, the full / enabled / visible `ViewRange`, enabled
/// segments and options, color ranges and vertex colors. Textures and drawing
/// belong to the renderer.
#[derive(Clone, Debug, Default)]
pub struct ViewerImpl {
    settings: Settings,
    layers: Layers,
    view_range: ViewRange,
    vertices: Vec<PathVertex>,
    total_time: [f32; TimeMode::COUNT],
    // ViewerImpl.hpp:270-281: for each layer, the index of its first vertex
    // and the running time at that vertex, so `get_estimated_time_at` only
    // accumulates the vertices of one layer.
    layer_first_vertex: Vec<u32>,
    layer_start_times: Vec<[f32; TimeMode::COUNT]>,
    travels_time: [f32; TimeMode::COUNT],
    extrusion_roles: ExtrusionRoles,
    tool_colors: Palette,
    color_print_colors: Palette,
    /// `m_used_extruders`: ordered per-extruder ColorPrint transitions.
    used_extruders: BTreeMap<u8, Vec<ColorPrint>>,
    vertices_colors: Vec<Color>,
    settings_used_for_ranges: Option<Settings>,
    ranges: colors::ColorRanges,
    valid_lines: Vec<bool>,
    enabled_segments: Vec<u32>,
    enabled_options: Vec<u32>,
    radii: heights_widths::Radii,
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
        self.tool_colors = data.tools_colors;
        self.color_print_colors = data.color_print_colors;
        self.vertices_colors.clear();
        self.settings_used_for_ranges = None;
        self.settings.spiral_vase_mode = data.spiral_vase_mode;
        self.used_extruders.clear();
        self.total_time = [0.0; TimeMode::COUNT];
        self.layer_first_vertex.clear();
        self.layer_start_times.clear();
        self.travels_time = [0.0; TimeMode::COUNT];
        self.extrusion_roles.reset();
        let mut total_time = [0.0; TimeMode::COUNT];
        for (i, v) in (0..).zip(&self.vertices) {
            self.layers.update(v, i);
            // ViewerImpl.cpp:1007-1026: timestamp includes this vertex.
            add_times(&mut total_time, v.times);
            if v.move_type == MoveType::Travel {
                add_times(&mut self.travels_time, v.times);
            }
            if v.move_type != MoveType::Extrude {
                continue;
            }
            self.extrusion_roles.add(v.role, v.times);
            let colors = self.used_extruders.entry(v.extruder_id).or_default();
            if colors.last().is_none_or(|last| last.color_id != v.color_id) {
                colors.push(ColorPrint {
                    extruder_id: v.extruder_id,
                    color_id: v.color_id,
                    layer_id: v.layer_id,
                    times: total_time,
                });
            }
        }
        // `update_colors` (ViewerImpl.cpp:1265-1273): a tool color up to the
        // max used extruder id.
        if let Some((&max, _)) = self.used_extruders.last_key_value() {
            let len = self.tool_colors.len().max(usize::from(max) + 1);
            self.tool_colors.resize(len, DUMMY_COLOR);
        }
        self.total_time = total_time;
        self.build_layer_time_caches();
        if !self.layers.empty() {
            self.layers.set_view_range(0, self.layers.count() - 1);
        }
        self.valid_lines = enabled::valid_lines(&self.vertices);
        self.update_view_full_range();
        self.view_range
            .set_visible_interval(self.view_range.get_enabled());
        self.settings.update_enabled_entities = true;
    }

    /// `load` (ViewerImpl.cpp:1189-1213): index of the first vertex of each
    /// layer, walked back to front so that a layer with no vertex of its own
    /// inherits the next layer's index and the array stays non-decreasing;
    /// then the running time at each layer's first vertex, summed in vertex
    /// order so that `get_estimated_time_at` matches a full accumulation
    /// exactly.
    fn build_layer_time_caches(&mut self) {
        if self.layers.empty() {
            return;
        }
        let vertices_count = self.vertices.len() as u32;
        self.layer_first_vertex = vec![vertices_count; self.layers.count()];
        for i in (1..=vertices_count).rev() {
            let layer_id = self.vertices[(i - 1) as usize].layer_id as usize;
            if layer_id < self.layer_first_vertex.len() {
                self.layer_first_vertex[layer_id] = i - 1;
            }
        }
        for i in (1..self.layer_first_vertex.len()).rev() {
            self.layer_first_vertex[i - 1] =
                self.layer_first_vertex[i - 1].min(self.layer_first_vertex[i]);
        }
        let mut running = [0.0; TimeMode::COUNT];
        self.layer_start_times = vec![[0.0; TimeMode::COUNT]; self.layer_first_vertex.len()];
        let mut layer = 0;
        for i in 0..=self.vertices.len() {
            while layer < self.layer_first_vertex.len()
                && self.layer_first_vertex[layer] as usize == i
            {
                self.layer_start_times[layer] = running;
                layer += 1;
            }
            if i < self.vertices.len() {
                add_times(&mut running, self.vertices[i].times);
            }
        }
    }

    pub fn vertices(&self) -> &[PathVertex] {
        &self.vertices
    }

    pub fn layers(&self) -> &Layers {
        &self.layers
    }

    /// ViewerImpl.cpp:1717-1732: layer-seeded partial sum; `id >= size`
    /// (and an out-of-range mode, impossible with the typed index) is 0.0.
    pub fn get_estimated_time_at(&self, id: usize) -> f32 {
        if id >= self.vertices.len() {
            return 0.0;
        }
        let mode = self.settings.time_mode.index();
        let mut first = 0;
        let mut time = 0.0;
        let layer = self.vertices[id].layer_id as usize;
        if layer < self.layer_first_vertex.len() && self.layer_first_vertex[layer] as usize <= id {
            first = self.layer_first_vertex[layer] as usize;
            time = self.layer_start_times[layer][mode];
        }
        for v in &self.vertices[first..=id] {
            time += v.times[mode];
        }
        time
    }

    /// ViewerImpl.hpp:179.
    pub fn get_estimated_time(&self) -> f32 {
        self.total_time[self.settings.time_mode.index()]
    }

    /// ViewerImpl.hpp:141.
    pub fn get_extrusion_roles(&self) -> impl Iterator<Item = GCodeExtrusionRole> + '_ {
        self.extrusion_roles.get_roles()
    }

    /// ViewerImpl.hpp:142.
    pub fn get_extrusion_role_estimated_time(&self, role: GCodeExtrusionRole) -> f32 {
        self.extrusion_roles.get_time(role, self.settings.time_mode)
    }

    /// ViewerImpl.hpp:147.
    pub fn get_travels_estimated_time(&self) -> f32 {
        self.travels_time[self.settings.time_mode.index()]
    }

    /// GCodeViewer.cpp:452-454, for the engine-selected property-window vertex.
    pub fn get_marker_time(&self, window_vertex: usize) -> crate::MarkerTime {
        crate::MarkerTime {
            elapsed_seconds: self.get_estimated_time_at(window_vertex),
            vertex_seconds: self.vertices[window_vertex].times[self.settings.time_mode.index()],
        }
    }

    /// ViewerImpl.cpp:1363-1370.
    pub fn get_used_extruders_ids(&self) -> Vec<u8> {
        self.used_extruders.keys().copied().collect()
    }

    /// ViewerImpl.cpp:1373-1376.
    pub fn get_color_prints_count(&self, extruder_id: u8) -> usize {
        self.used_extruders.get(&extruder_id).map_or(0, Vec::len)
    }

    /// ViewerImpl.cpp:1379-1382, borrowing the neutral records without copying.
    pub fn get_color_prints(&self, extruder_id: u8) -> &[ColorPrint] {
        self.used_extruders
            .get(&extruder_id)
            .map_or(&[], Vec::as_slice)
    }

    pub fn get_tool_colors(&self) -> &Palette {
        &self.tool_colors
    }

    pub fn is_option_visible(&self, option: OptionType) -> bool {
        self.settings.options_visibility[option.index()]
    }

    /// ViewerImpl.cpp:1419-1435.
    pub fn toggle_option_visibility(&mut self, option: OptionType) {
        let visible = &mut self.settings.options_visibility[option.index()];
        *visible = !*visible;
        self.settings.update_colors = true;
        self.settings.update_enabled_entities = true;
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

    /// ViewerImpl.cpp:1442-1448.
    pub fn toggle_extrusion_role_visibility(&mut self, role: GCodeExtrusionRole) {
        let visible = &mut self.settings.extrusion_roles_visibility[role.index()];
        *visible = !*visible;
        self.settings.update_colors = true;
        self.settings.update_enabled_entities = true;
        self.update_view_full_range();
    }

    pub fn is_top_layer_only_view_range(&self) -> bool {
        self.settings.top_layer_only_view_range
    }

    /// ViewerImpl.cpp:1342-1347.
    pub fn toggle_top_layer_only_view_range(&mut self) {
        self.settings.top_layer_only_view_range = !self.settings.top_layer_only_view_range;
        self.update_view_full_range();
        self.view_range
            .set_visible_interval(self.view_range.get_enabled());
        self.settings.update_enabled_entities = true;
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
        self.settings.update_enabled_entities = true;
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

    /// ViewerImpl.cpp:1450-1459; the colors texture is `texture_colors`.
    pub fn set_view_visible_range(&mut self, min: usize, max: usize) {
        // force update of the full range, to avoid clamping the visible range with full old values
        // when calling m_view_range.set_visible()
        self.update_view_full_range();
        self.view_range.set_visible(min, max);
        self.update_enabled_entities();
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
