// Ports the Custom G-code overview of AGPL OrcaSlicer GCodeViewer::render_legend
// (GCodeViewer.cpp:4477-4546) and find_close_layer_idx (GCodeViewer.cpp:108-125).
use crate::{Layers, TimeMode};
use ares_core::gcode_processor::{CustomGCodeItem, CustomGCodeType};

/// `IMSlider.hpp:24`.
const EPSILON: f64 = 0.0011;

/// One overview row: the type label, the printed layer number (`layer + 1`,
/// so `0` for a z that matches no layer) and the Normal-mode time before it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomGCodeOverviewRow {
    pub label: &'static str,
    pub layer: i32,
    pub time: f32,
}

/// One row per item, in list order.
pub fn custom_gcode_overview(
    items: &[CustomGCodeItem],
    layers: &Layers,
) -> Vec<CustomGCodeOverviewRow> {
    let zs: Vec<f64> = layers.get_zs().into_iter().map(f64::from).collect();
    let times = layers.get_times(TimeMode::Normal);
    items
        .iter()
        .map(|item| {
            // The switch has no ColorChange case, so it falls to the default.
            let label = match item.kind {
                CustomGCodeType::PausePrint => "Pause",
                CustomGCodeType::Template => "Template",
                CustomGCodeType::ToolChange => "Tool Change",
                CustomGCodeType::Custom => "Custom",
                CustomGCodeType::ColorChange | CustomGCodeType::Unknown => "Unknown",
            };
            let layer = find_close_layer_idx(&zs, item.print_z, EPSILON);
            // Sequential f32 sum of the layers before `layer` (none for 0 or -1).
            let time = times
                .iter()
                .take(layer.max(0) as usize)
                .fold(0.0_f32, |sum, time| sum + time);
            CustomGCodeOverviewRow {
                label,
                layer: layer + 1,
                time,
            }
        })
        .collect()
}

/// The nearer neighbour of `lower_bound(z)` within `eps` (a tie goes to the
/// upper one), or `-1`.
fn find_close_layer_idx(zs: &[f64], z: f64, eps: f64) -> i32 {
    if zs.is_empty() {
        return -1;
    }
    let high = zs.partition_point(|layer| *layer < z);
    if high == zs.len() {
        if z - zs[high - 1] < eps {
            return (zs.len() - 1) as i32;
        }
    } else if high == 0 {
        if zs[0] - z < eps {
            return 0;
        }
    } else {
        let dist_l = z - zs[high - 1];
        let dist_h = zs[high] - z;
        if dist_l.min(dist_h) < eps {
            return if dist_l < dist_h { high - 1 } else { high } as i32;
        }
    }
    -1
}
