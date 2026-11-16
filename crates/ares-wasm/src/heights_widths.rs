use crate::VertexBuffers;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VertexBuffers {
    /// Float32Array stride2: `[height, width]` of each vertex,
    /// `ViewerImpl::heights_widths` (ViewerImpl.cpp:954-968) at the default radii.
    #[wasm_bindgen(js_name = heightsWidthsRange)]
    pub fn heights_widths_range(&self, first: u32, count: u32) -> Result<Vec<f32>, JsError> {
        let range = self.vertex_range(first, count)?;
        Ok(self.viewer.heights_widths_range(range).into_flattened())
    }

    /// Float32Array stride3: `ViewerImpl::positions` (ViewerImpl.cpp:945-952).
    #[wasm_bindgen(js_name = segmentPositionsRange)]
    pub fn segment_positions_range(&self, first: u32, count: u32) -> Result<Vec<f32>, JsError> {
        let range = self.vertex_range(first, count)?;
        Ok(self.viewer.positions_range(range).into_flattened())
    }

    /// Float32Array: join angles using global neighbors (ViewerImpl.cpp:928-934,979-980).
    #[wasm_bindgen(js_name = segmentAnglesRange)]
    pub fn segment_angles_range(&self, first: u32, count: u32) -> Result<Vec<f32>, JsError> {
        let range = self.vertex_range(first, count)?;
        Ok(self.viewer.angles_range(range))
    }
}
