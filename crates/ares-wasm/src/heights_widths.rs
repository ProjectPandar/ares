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

    /// Float32Array stride 3: segment position of each vertex,
    /// `ViewerImpl::positions` (ViewerImpl.cpp:945-952).
    #[wasm_bindgen(js_name = segmentPositions)]
    pub fn segment_positions(&self) -> Vec<f32> {
        self.viewer.positions().into_flattened()
    }

    /// Float32Array: join angle of each vertex, `ViewerImpl::angles`
    /// (ViewerImpl.cpp:928-934,979-980).
    #[wasm_bindgen(js_name = segmentAngles)]
    pub fn segment_angles(&self) -> Vec<f32> {
        self.viewer.angles()
    }
}
