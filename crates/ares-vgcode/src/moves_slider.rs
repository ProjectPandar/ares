// Ports the rendering-neutral part of AGPL-licensed OrcaSlicer
// `src/slic3r/GUI/GCodeViewer.cpp` `GCodeViewer::update_moves_slider`
// (`:1765-1821`), the horizontal `IMSlider` thumbs it drives
// (`src/slic3r/GUI/IMSlider.cpp`) and the thumb mapping of
// `GLCanvas3D::_render_gcode` (`src/slic3r/GUI/GLCanvas3D.cpp:8263-8266`).

use crate::{Interval, ViewerImpl};

/// Moves slider stops over the viewer's enabled range and its thumbs. The
/// minimum stop is 0: GCodeViewer creates the slider as
/// `IMSlider(0, 0, 0, 100, wxSL_HORIZONTAL)` (`GCodeViewer.cpp:977`) and never
/// sets another minimum; the higher thumb is the selected one
/// (`IMSlider.cpp:143`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MovesSlider {
    /// `values`: per stop, one past the last vertex id of its G-code line.
    pub values: Vec<usize>,
    /// `alternate_values`: the G-code line id of each stop.
    pub alternate_values: Vec<u32>,
    /// `m_lower_value`.
    pub lower: usize,
    /// `m_higher_value`.
    pub higher: usize,
    /// The layers view range the stops were made for.
    layers: Interval,
}

impl MovesSlider {
    /// The slider of a loaded G-code: `GCodeViewer::load_as_gcode`, then the
    /// higher thumb at the max (`GLCanvas3D.cpp:3098`).
    pub fn new(viewer: &ViewerImpl) -> Self {
        let mut slider = Self {
            layers: viewer.get_layers_view_range(),
            ..Self::default()
        };
        slider.update(viewer, false);
        slider.set_higher_value(isize::MAX);
        slider
    }

    /// `GCodeViewer::set_layers_z_range` (`GCodeViewer.cpp:1864-1868`) when
    /// `layers` differs from the slider's last layers view range, otherwise
    /// the `update_moves_slider()` refresh after a legend visibility toggle
    /// (`GCodeViewer.cpp:3886-3974`).
    pub fn set_layers_z_range(&mut self, viewer: &mut ViewerImpl, layers: Interval) {
        viewer.set_layers_view_range(layers[0], layers[1]);
        let set_to_max = self.layers != layers;
        self.layers = layers;
        self.update(viewer, set_to_max);
    }

    /// `update_moves_slider(set_to_max)`: unchanged without vertices
    /// (`GCodeViewer.cpp:1767-1768`).
    fn update(&mut self, viewer: &ViewerImpl, set_to_max: bool) {
        let vertices = viewer.vertices();
        if vertices.is_empty() {
            return;
        }
        let range = viewer.get_view_enabled_range();
        let visible_range = viewer.get_view_visible_range();
        let gcode_id_min = vertices[visible_range[0]].gcode_id;
        let gcode_id_max = vertices[visible_range[1]].gcode_id;
        let mut values = Vec::with_capacity(range[1] - range[0] + 1);
        let mut alternate_values = Vec::with_capacity(values.capacity());
        let mut visible_range_min_id = None;
        let mut visible_range_max_id = None;
        let mut last_gcode_id = vertices[range[0]].gcode_id;
        for (i, v) in (range[0]..).zip(&vertices[range[0]..=range[1]]) {
            // skip consecutive moves with same gcode id (resulting from processing G2 and G3 lines)
            match values.last_mut() {
                Some(last) if last_gcode_id == v.gcode_id => *last = i + 1,
                _ => {
                    last_gcode_id = v.gcode_id;
                    let counter = Some(values.len());
                    values.push(i + 1);
                    alternate_values.push(v.gcode_id);
                    match v.gcode_id {
                        id if id == gcode_id_min => visible_range_min_id = counter,
                        id if id == gcode_id_max => visible_range_max_id = counter,
                        _ => (),
                    }
                }
            }
        }
        let span_min_id = visible_range_min_id.unwrap_or(0);
        let span_max_id = visible_range_max_id.unwrap_or(values.len() - 1);
        let keep_min = self.higher == 0;
        self.values = values;
        self.alternate_values = alternate_values;
        // IMSlider::SetSelectionSpan (`IMSlider.cpp:193-204`).
        self.lower = span_min_id;
        self.higher = span_max_id.min(self.values.len() - 1).max(self.lower);
        if set_to_max {
            self.set_higher_value(if keep_min { 0 } else { isize::MAX });
        }
    }

    /// `IMSlider::SetHigherValue` and `correct_higher_value`
    /// (`IMSlider.cpp:186-191`, `:1675-1683`): clamped to the stops, the lower
    /// thumb follows when crossed.
    pub fn set_higher_value(&mut self, value: isize) {
        let max = self.values.len().saturating_sub(1);
        self.higher = value.max(0).unsigned_abs().min(max);
        self.lower = self.lower.min(self.higher);
    }

    /// `GLCanvas3D.cpp:8265`: the vertex range between the thumbs; `None`
    /// without stops.
    pub fn vertex_range(&self) -> Option<Interval> {
        Some([
            self.values.get(self.lower)? - 1,
            self.values.get(self.higher)? - 1,
        ])
    }
}
