use wasm_bindgen::prelude::*;

/// The web E2E probe's deterministic panic; the plater session's `sliceWithProfilesProcessed`
/// calls it first.
#[wasm_bindgen(js_name = panicProbe)]
pub fn panic_probe() {
    panic!("panic probe");
}
