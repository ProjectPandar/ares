use plugin_spike_host::{Budget, Guest, interpreter};
use wasm_bindgen::prelude::*;

fn js_err(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[wasm_bindgen]
#[derive(Default)]
pub struct SpikeInterpreter(interpreter::Host);

#[wasm_bindgen]
impl SpikeInterpreter {
    #[wasm_bindgen(constructor)]
    pub fn new(metered: bool) -> Self {
        let budget = if metered {
            Budget {
                fuel: Some(1_000_000_000),
                memory: Some(64 << 20),
            }
        } else {
            Budget::default()
        };
        Self(interpreter::Host::new(budget))
    }

    pub fn probe(module: &[u8]) -> Result<Vec<i32>, JsValue> {
        let budget = Budget {
            memory: Some(2 * 65536),
            fuel: Some(1_000),
        };
        interpreter::Host::new(budget)
            .probe(module)
            .map(|r| r.to_vec())
            .map_err(js_err)
    }

    pub fn load(&self, module: &[u8], source: Option<Vec<u8>>) -> Result<SpikeGuest, JsValue> {
        self.0
            .load(module, source.as_deref())
            .map(SpikeGuest)
            .map_err(js_err)
    }
}

#[wasm_bindgen]
pub struct SpikeGuest(interpreter::Instance);

#[wasm_bindgen]
impl SpikeGuest {
    pub fn set_source(&mut self, source: &[u8]) -> Result<(), JsValue> {
        self.0.set_source(source).map_err(js_err)
    }

    pub fn perimeters(&mut self, layer: &[u8]) -> Result<Vec<u8>, JsValue> {
        self.0.perimeters(layer).map_err(js_err)
    }

    pub fn gcode(&mut self, bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        self.0.gcode(bytes).map_err(js_err)
    }
}
