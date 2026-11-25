use super::{PlaterSession, js_error};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl PlaterSession {
    /// Adds, deletes or merges a project slot in the current palette.
    #[wasm_bindgen(js_name = editFilamentSlots)]
    pub fn edit_filament_slots(&mut self, json: &str) -> Result<(), JsValue> {
        let edit = serde_json::from_str(json).map_err(super::js_error)?;
        self.session
            .edit_filament_slots(edit)
            .map(|_| ())
            .map_err(super::js_error)
    }

    /// Changes the filament for the named object or volume ids in selection order.
    #[wasm_bindgen(js_name = setFilament)]
    pub fn set_filament(&mut self, targets: &str, slot: i32, count: i32) -> Result<(), JsValue> {
        let targets: Vec<ares_core::FilamentTarget> =
            serde_json::from_str(targets).map_err(js_error)?;
        self.session
            .set_filament(&targets, (slot, count))
            .map_err(js_error)
    }

    /// Changes the filament for one object-list row.
    #[wasm_bindgen(js_name = setFilamentCell)]
    pub fn set_filament_cell(
        &mut self,
        target: &str,
        slot: i32,
        count: i32,
    ) -> Result<(), JsValue> {
        let target = serde_json::from_str(target).map_err(js_error)?;
        self.session
            .set_filament_cell(target, (slot, count))
            .map_err(js_error)
    }
}
