use plugin_spike_host::{Guest, direct};
use wasm_bindgen::prelude::*;

fn js_err(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[wasm_bindgen]
pub struct SpikeDirect(Box<dyn Guest>);

#[wasm_bindgen]
impl SpikeDirect {
    #[wasm_bindgen(constructor)]
    pub fn new(engine: &str, subdivision: bool, source: &[u8]) -> Result<SpikeDirect, JsValue> {
        let _ = (subdivision, source);
        let guest: Box<dyn Guest> = match engine {
            #[cfg(feature = "spike-plugin-compiled-rust")]
            "rust" => Box::new(direct::Rust(direct::sample(subdivision))),
            #[cfg(feature = "spike-plugin-quickjs")]
            "quickjs" => Box::new(direct::QuickJs::new(source)),
            #[cfg(feature = "spike-plugin-boa")]
            "boa" => Box::new(direct::Boa::new(source)),
            _ => return Err(JsValue::from_str("engine is not compiled in")),
        };
        Ok(Self(guest))
    }

    #[cfg(any(feature = "spike-plugin-quickjs", feature = "spike-plugin-boa"))]
    pub fn probe(engine: &str) -> Result<Vec<i32>, JsValue> {
        let result = match engine {
            #[cfg(feature = "spike-plugin-quickjs")]
            "quickjs" => direct::probe_quickjs(),
            #[cfg(feature = "spike-plugin-boa")]
            "boa" => direct::probe_boa(),
            _ => return Err(JsValue::from_str("engine limit probe is not compiled in")),
        };
        Ok(result.to_vec())
    }

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
