use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = instrumentPlugin)]
pub fn instrument_plugin(module_bytes: &[u8], max_pages: u32) -> Result<Vec<u8>, JsValue> {
    plugin_spike_host::metering::instrument(module_bytes, max_pages)
        .map(|module| module.bytes)
        .map_err(|error| JsValue::from_str(&format!("{error:#}")))
}
