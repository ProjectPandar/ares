use plugin_spike_host::files::FileProbe;
use wasm_bindgen::prelude::*;

fn js_err(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[wasm_bindgen]
pub struct SpikeFiles(FileProbe);

#[wasm_bindgen]
impl SpikeFiles {
    #[wasm_bindgen(constructor)]
    pub fn new(module: &[u8], wasi: bool, prior: &[u8]) -> Result<SpikeFiles, JsValue> {
        FileProbe::new(module, wasi, prior)
            .map(Self)
            .map_err(js_err)
    }

    pub fn run(&mut self, iterations: usize) -> Result<(), JsValue> {
        self.0.run(iterations).map_err(js_err)
    }

    pub fn finish(&self, prior: &[u8]) -> Result<Vec<u8>, JsValue> {
        self.0.finish(prior).map_err(js_err)
    }
}
