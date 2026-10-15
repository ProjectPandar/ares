use crate::VertexBuffers;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VertexBuffers {
    /// Float32Array stride2: `[height, width]` of each vertex,
    /// `ViewerImpl::heights_widths` (ViewerImpl.cpp:954-968) at the default radii.
    #[wasm_bindgen(js_name = heightsWidths)]
    pub fn heights_widths(&self) -> Vec<f32> {
        self.viewer.heights_widths().into_flattened()
    }
}
