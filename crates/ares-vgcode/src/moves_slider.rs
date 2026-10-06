// Ports the rendering-neutral part of AGPL-licensed OrcaSlicer
// `src/slic3r/GUI/GCodeViewer.cpp` `GCodeViewer::update_moves_slider`
// (`:1765-1821`) and the thumb mapping of `GLCanvas3D::_render_gcode`
// (`src/slic3r/GUI/GLCanvas3D.cpp:8263-8266`).

use crate::ViewerImpl;

/// Moves slider stops over the viewer's enabled range.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MovesSlider {
    /// `values`: per stop, one past the last vertex id of its G-code line.
    pub values: Vec<usize>,
    /// `alternate_values`: the G-code line id of each stop.
    pub alternate_values: Vec<u32>,
    /// `SetSelectionSpan(span_min_id, span_max_id)`: the stops of the visible range.
    pub span: [usize; 2],
}

impl MovesSlider {
    /// `update_moves_slider` without the `IMSlider` calls; `None` without
    /// vertices (`GCodeViewer.cpp:1767-1768`).
    pub fn new(viewer: &ViewerImpl) -> Option<Self> {
        let vertices = viewer.vertices();
        if vertices.is_empty() {
            return None;
        }
        let range = viewer.get_view_enabled_range();
        let visible_range = viewer.get_view_visible_range();
        let gcode_id_min = vertices[visible_range[0]].gcode_id;
        let gcode_id_max = vertices[visible_range[1]].gcode_id;
        let mut slider = Self::default();
        let mut visible_range_min_id = None;
        let mut visible_range_max_id = None;
        let mut last_gcode_id = vertices[range[0]].gcode_id;
        for (i, v) in (range[0]..).zip(&vertices[range[0]..=range[1]]) {
            // skip consecutive moves with same gcode id (resulting from processing G2 and G3 lines)
            match slider.values.last_mut() {
                Some(last) if last_gcode_id == v.gcode_id => *last = i + 1,
                _ => {
                    last_gcode_id = v.gcode_id;
                    let counter = Some(slider.values.len());
                    slider.values.push(i + 1);
                    slider.alternate_values.push(v.gcode_id);
                    match v.gcode_id {
                        id if id == gcode_id_min => visible_range_min_id = counter,
                        id if id == gcode_id_max => visible_range_max_id = counter,
                        _ => (),
                    }
                }
            }
        }
        slider.span = [
            visible_range_min_id.unwrap_or(0),
            visible_range_max_id.unwrap_or(slider.values.len() - 1),
        ];
        Some(slider)
    }

    /// `GLCanvas3D.cpp:8265`: the vertex range of the lower and higher thumbs.
    pub fn vertex_range(&self, low: usize, high: usize) -> [usize; 2] {
        [self.values[low] - 1, self.values[high] - 1]
    }
}
