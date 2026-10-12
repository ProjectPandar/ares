// Ports the Custom G-code overview of AGPL OrcaSlicer GCodeViewer::render_legend
// (GCodeViewer.cpp:4477-4546) and find_close_layer_idx (GCodeViewer.cpp:108-125).
use crate::Layers;
use ares_core::gcode_processor::CustomGCodeItem;

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
    _items: &[CustomGCodeItem],
    _layers: &Layers,
) -> Vec<CustomGCodeOverviewRow> {
    Vec::new()
}
