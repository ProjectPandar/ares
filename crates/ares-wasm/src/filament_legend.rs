use crate::VertexBuffers;
use ares_vgcode::{Color, FilamentLegend};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// ares-vgcode `FilamentLegend`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FilamentLegendView<'a> {
    color_print: crate::color_print::ColorPrintView,
    columns: Vec<String>,
    rows: Vec<FilamentRowView<'a>>,
    filament_diameters: &'a [f32],
    filament_densities: &'a [f32],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FilamentRowView<'a> {
    extruder_id: u8,
    color: Color,
    volumes: &'a [f64],
}

impl<'a> From<&'a FilamentLegend> for FilamentLegendView<'a> {
    fn from(legend: &'a FilamentLegend) -> Self {
        Self {
            color_print: crate::color_print::ColorPrintView::from(&legend.color_print),
            columns: legend
                .columns
                .iter()
                .map(|column| format!("{column:?}"))
                .collect(),
            rows: legend
                .rows
                .iter()
                .map(|row| FilamentRowView {
                    extruder_id: row.extruder_id,
                    color: row.color,
                    volumes: &row.volumes,
                })
                .collect(),
            filament_diameters: &legend.filament_diameters,
            filament_densities: &legend.filament_densities,
        }
    }
}

#[wasm_bindgen]
impl VertexBuffers {
    /// ColorPrint legend columns, used extruders, mm³ volumes and filament
    /// metadata from ares-vgcode, without unit conversion or formatting.
    #[wasm_bindgen(js_name = filamentLegend)]
    pub fn filament_legend(&self) -> Result<String, JsError> {
        serde_json::to_string(&FilamentLegendView::from(&self.filament_legend))
            .map_err(|error| JsError::new(&error.to_string()))
    }
}
