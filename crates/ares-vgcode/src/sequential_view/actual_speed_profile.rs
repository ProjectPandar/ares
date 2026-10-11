// Ports the `ENABLE_ACTUAL_SPEED_DEBUG` profile data of AGPL-licensed
// OrcaSlicer `src/slic3r/GUI/GCodeViewer.cpp` (`:1681-1753`), compiled in
// shipping builds (`libslic3r/Technologies.hpp:52`).

use super::SequentialView;
use crate::{MoveType, Vec3, ViewType, ViewerImpl};

/// `Slic3r::EPSILON` (`libslic3r.h:52`), a double.
const EPSILON: f64 = 1e-4;

/// Eigen `Vector3f::norm` of `a - b`: the unrolled sum `x² + (y² + z²)`.
fn distance(a: Vec3, b: Vec3) -> f32 {
    let [x, y, z] = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (x * x + (y * y + z * z)).sqrt()
}

/// `sameSpeed`: equal speeds at 0.1 precision, `roundf` rounding half away from zero.
fn same_speed(a: f32, b: f32) -> bool {
    (a * 10.0).round() as i32 == (b * 10.0).round() as i32
}

/// ORCA compression of consecutive duplicate speeds (`GCodeViewer.cpp:1730-1747`).
fn compress(data: &[ActualSpeedItem]) -> Vec<ActualSpeedItem> {
    let [first, .., last] = data else {
        return data.to_vec();
    };
    let mut compressed = vec![*first];
    for i in 1..data.len() {
        let same_as_prev = same_speed(data[i].speed, data[i - 1].speed);
        let same_as_next = i + 1 < data.len() && same_speed(data[i].speed, data[i + 1].speed);
        if !same_as_prev {
            if !same_speed(compressed[compressed.len() - 1].speed, data[i - 1].speed) {
                compressed.push(data[i - 1]);
            }
            compressed.push(data[i]);
        } else if !same_as_next {
            compressed.push(data[i]);
        }
    }
    if compressed[compressed.len() - 1].pos != last.pos {
        compressed.push(*last);
    }
    compressed
}

/// `SequentialView::ActualSpeedImguiWidget::Item` (`GCodeViewer.hpp:52-57`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActualSpeedItem {
    pub pos: f32,
    pub speed: f32,
    pub internal: bool,
}

/// The data of `SequentialView::ActualSpeedImguiWidget` (`GCodeViewer.hpp:47-59`):
/// the ActualSpeed color range, its levels as `ColorRGBA` with alpha 0.5, and
/// the compressed profile of the current G-code line.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActualSpeedProfile {
    pub y_range: [f32; 2],
    pub levels: Vec<(f32, [f32; 4])>,
    pub data: Vec<ActualSpeedItem>,
}

impl SequentialView {
    /// The actual speed block of `GCodeViewer::update_sequential_view_current`
    /// after `set_view_visible_range` and `ViewerImpl::colors`. Without an update
    /// the previous profile stays.
    pub fn update_actual_speed(&mut self, viewer: &ViewerImpl) {
        let curr_id = viewer.get_view_visible_range()[1];
        if viewer.get_view_enabled_range()[1] == curr_id {
            return;
        }
        let vertices = viewer.vertices();
        let curr_vertex = &vertices[curr_id];
        if !(curr_vertex.is_extrusion()
            || curr_vertex.is_travel()
            || curr_vertex.is_wipe()
            || curr_vertex.move_type == MoveType::Seam)
        {
            return;
        }
        let Some(color_range) = viewer.get_color_range(ViewType::ActualSpeed) else {
            return;
        };
        // Vertices sharing the gcode_id: the start is the previous line's last
        // vertex (or 0), the end is exclusive and at most the last vertex.
        let gcode_id = curr_vertex.gcode_id;
        let start_id = vertices[..curr_id]
            .iter()
            .rposition(|v| v.gcode_id != gcode_id)
            .unwrap_or(0);
        let mut end_id = vertices[curr_id + 1..]
            .iter()
            .position(|v| v.gcode_id != gcode_id)
            .map_or(vertices.len() - 1, |i| curr_id + 1 + i);
        if vertices[end_id - 1].move_type == MoveType::Seam {
            end_id -= 1;
        }

        let mut total_len = 0.0f32;
        let mut data = Vec::new();
        for i in start_id..end_id {
            let v = &vertices[i];
            let len = if i > start_id {
                distance(v.position, vertices[i - 1].position)
            } else {
                0.0
            };
            total_len += len;
            if i == start_id || f64::from(len) > EPSILON {
                data.push(ActualSpeedItem {
                    pos: total_len,
                    speed: v.actual_feedrate,
                    internal: v.times[0] == 0.0,
                });
            }
        }

        // `libvgcode::convert(Color)` (`LibVGCodeWrapper.cpp:49-53`) with alpha 0.5.
        let inv_255 = 1.0f32 / 255.0;
        self.actual_speed = ActualSpeedProfile {
            y_range: color_range.get_range(),
            levels: color_range
                .get_values()
                .into_iter()
                .map(|value| {
                    let [r, g, b] = color_range.get_color_at(value).map(f32::from);
                    (value, [r * inv_255, g * inv_255, b * inv_255, 0.5])
                })
                .collect(),
            data: compress(&data),
        };
    }

    pub fn actual_speed(&self) -> &ActualSpeedProfile {
        &self.actual_speed
    }
}
