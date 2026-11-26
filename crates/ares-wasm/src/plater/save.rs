use ares_core::{GuiProfileSliceRequest, ProfileSliceRequest};
use wasm_bindgen::prelude::*;

use super::{PlaterSession, PresetCatalog, ProfileSliceOptions, js_error};

#[wasm_bindgen]
impl PlaterSession {
    /// Saves the edited project using its own presets and the current option edits.
    #[wasm_bindgen(js_name = saveProject)]
    pub fn save_project(&self, overrides_json: Option<String>) -> Result<Vec<u8>, JsValue> {
        let overrides = crate::parse_overrides(overrides_json.as_deref()).map_err(js_error)?;
        self.session.save_project(&overrides).map_err(js_error)
    }

    /// Saves the edited project with the same preset selection as the plater slice.
    #[wasm_bindgen(js_name = saveProjectWithProfiles)]
    pub fn save_project_with_profiles(
        &self,
        catalog: &PresetCatalog,
        selection_json: String,
        options_json: String,
    ) -> Result<Vec<u8>, JsValue> {
        let options = ProfileSliceOptions::parse(&options_json).map_err(js_error)?;
        let overrides = options.overrides();
        self.session
            .save_project_with_profiles(GuiProfileSliceRequest(ProfileSliceRequest {
                filename: &self.filename,
                catalog_json: catalog.catalog_json(),
                selection_json: &selection_json,
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::None,
            }))
            .map_err(js_error)
    }
}
