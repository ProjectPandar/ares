use super::{PlaterSession, js_error};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl PlaterSession {
    /// Changes the filament for the named object or volume ids in selection order.
    #[wasm_bindgen(js_name = setFilament)]
    pub fn set_filament(&mut self, targets: &str, slot: i32, count: i32) -> Result<(), JsValue> {
        let targets: Vec<ares_core::FilamentTarget> =
            serde_json::from_str(targets).map_err(js_error)?;
        self.session
            .set_filament(&targets, (slot, count))
            .map_err(js_error)
    }
}
