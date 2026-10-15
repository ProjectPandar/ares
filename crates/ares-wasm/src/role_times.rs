use crate::VertexBuffers;
use wasm_bindgen::prelude::*;

/// Feature Type legend time inputs (`GCodeViewer.cpp:3151, 3410-3418,
/// 3638-3669`) in the Normal time mode of the loaded viewer.
#[wasm_bindgen]
impl VertexBuffers {
    /// `ViewerImpl::get_extrusion_roles` (ViewerImpl.hpp:141), `EGCodeExtrusionRole` ids.
    #[wasm_bindgen(js_name = extrusionRoles)]
    pub fn extrusion_roles(&self) -> Vec<u8> {
        self.viewer
            .get_extrusion_roles()
            .map(|role| role.index() as u8)
            .collect()
    }

    /// `get_extrusion_role_estimated_time` (ViewerImpl.hpp:142) of each `extrusionRoles` id.
    #[wasm_bindgen(js_name = extrusionRoleTimes)]
    pub fn extrusion_role_times(&self) -> Vec<f32> {
        self.viewer
            .get_extrusion_roles()
            .map(|role| self.viewer.get_extrusion_role_estimated_time(role))
            .collect()
    }

    /// `get_travels_estimated_time` (ViewerImpl.hpp:147).
    #[wasm_bindgen(getter, js_name = travelsTime)]
    pub fn travels_time(&self) -> f32 {
        self.viewer.get_travels_estimated_time()
    }

    /// `get_estimated_time` (ViewerImpl.hpp:136), the legend total when the
    /// print statistics time is 0.
    #[wasm_bindgen(getter, js_name = estimatedTime)]
    pub fn estimated_time(&self) -> f32 {
        self.viewer.get_estimated_time()
    }
}
