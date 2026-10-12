use crate::SliceOutput;
use ares_core::{SliceError, gcode_processor::CustomGCodeItem};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// The editor's item list (`GCodeViewer.cpp:4478-4479`):
/// `model().get_curr_plate_custom_gcodes().gcodes` of the sliced plate.
pub(crate) fn plate_custom_gcodes(
    input: &[u8],
    _plate: Option<u32>,
) -> Result<Vec<CustomGCodeItem>, SliceError> {
    Ok(ares_core::load_project(input)?
        .curr_plate_custom_gcodes()
        .gcodes)
}

/// ares-vgcode `CustomGCodeOverviewRow`; the f32 time widens to f64 so the
/// JSON carries its exact value.
#[derive(Serialize)]
struct RowView {
    label: &'static str,
    layer: i32,
    time: f64,
}

impl SliceOutput {
    pub(crate) fn with_custom_gcodes(mut self, items: &[CustomGCodeItem]) -> Self {
        self.custom_gcode_overview =
            ares_vgcode::custom_gcode_overview(items, self.vertices.viewer.layers());
        self
    }
}

#[wasm_bindgen]
impl SliceOutput {
    /// Legend Custom G-code overview rows (`GCodeViewer.cpp:4477-4546`) as JSON.
    #[wasm_bindgen(js_name = customGcodeOverview)]
    pub fn custom_gcode_overview(&self) -> Result<String, JsError> {
        let rows: Vec<_> = self
            .custom_gcode_overview
            .iter()
            .map(|row| RowView {
                label: row.label,
                layer: row.layer,
                time: f64::from(row.time),
            })
            .collect();
        serde_json::to_string(&rows).map_err(|error| JsError::new(&error.to_string()))
    }
}
