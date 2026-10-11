use crate::VertexBuffers;
use ares_vgcode::{
    Color, ColorRangeType, DEFAULT_EXTRUSION_ROLES_COLORS, DEFAULT_OPTIONS_COLORS, ViewType,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// View types exposed in `EViewType` order; the tool view is not ported yet.
const VIEW_TYPES: [ViewType; 16] = [
    ViewType::Summary,
    ViewType::FeatureType,
    ViewType::ColorPrint,
    ViewType::Speed,
    ViewType::ActualSpeed,
    ViewType::Height,
    ViewType::Width,
    ViewType::VolumetricFlowRate,
    ViewType::ActualVolumetricFlowRate,
    ViewType::LayerTimeLinear,
    ViewType::LayerTimeLogarithmic,
    ViewType::FanSpeed,
    ViewType::Temperature,
    ViewType::PressureAdvance,
    ViewType::Acceleration,
    ViewType::Jerk,
];

fn view_type(name: &str) -> Result<ViewType, JsError> {
    VIEW_TYPES
        .into_iter()
        .find(|view_type| format!("{view_type:?}") == name)
        .ok_or_else(|| JsError::new(&format!("unknown view type {name}")))
}

/// libvgcode `ColorRange` state (`ColorRange.hpp`). The floats are widened to
/// f64 so JSON carries the exact f32 values the legend prints.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ColorRangeView {
    range_type: &'static str,
    range: [f64; 2],
    values: Vec<f64>,
    palette: Vec<Color>,
}

#[wasm_bindgen]
impl VertexBuffers {
    /// Uint8Array RGB triples: libvgcode `ViewerImpl::get_vertex_color` of each
    /// vertex under `view_type` (`ViewerImpl::update_colors`), after the
    /// `set_visibility` the color ranges depend on, grayed as
    /// `update_colors_texture` does for the moves slider range.
    #[wasm_bindgen(js_name = vertexColors)]
    pub fn vertex_colors(
        &mut self,
        view_type_name: &str,
        visible_roles: u32,
        visible_options: u16,
    ) -> Result<Vec<u8>, JsError> {
        let view_type = view_type(view_type_name)?;
        self.set_visibility(visible_roles, visible_options);
        if self.viewer.get_view_type() != view_type {
            self.viewer.set_view_type(view_type);
        }
        Ok(self.viewer.texture_colors().as_flattened().to_vec())
    }

    /// JSON `{rangeType, range, values, palette}` of libvgcode
    /// `ViewerImpl::get_color_range(view_type)` after `vertexColors`, or `null`
    /// for a view type without a range.
    #[wasm_bindgen(js_name = colorRange)]
    pub fn color_range(&self, view_type_name: &str) -> Result<String, JsError> {
        let range = self
            .viewer
            .get_color_range(view_type(view_type_name)?)
            .map(|range| ColorRangeView {
                range_type: match range.get_type() {
                    ColorRangeType::Linear => "Linear",
                    ColorRangeType::Logarithmic => "Logarithmic",
                },
                range: range.get_range().map(f64::from),
                values: range.get_values().into_iter().map(f64::from).collect(),
                palette: range.get_palette().to_vec(),
            });
        Ok(serde_json::to_string(&range).expect("color range serializes"))
    }
}

/// libvgcode `EViewType` names `vertexColors` and `colorRange` accept, in index order.
#[wasm_bindgen(js_name = viewTypes)]
pub fn view_types() -> Vec<String> {
    VIEW_TYPES
        .iter()
        .map(|view_type| format!("{view_type:?}"))
        .collect()
}

/// Uint8Array RGB triples: libvgcode `DEFAULT_EXTRUSION_ROLES_COLORS` in `EGCodeExtrusionRole` order.
#[wasm_bindgen(js_name = roleColors)]
pub fn role_colors() -> Vec<u8> {
    DEFAULT_EXTRUSION_ROLES_COLORS.as_flattened().to_vec()
}

/// Uint8Array RGB triples: libvgcode `DEFAULT_OPTIONS_COLORS` in `EOptionType` order.
#[wasm_bindgen(js_name = optionColors)]
pub fn option_colors() -> Vec<u8> {
    DEFAULT_OPTIONS_COLORS.as_flattened().to_vec()
}
