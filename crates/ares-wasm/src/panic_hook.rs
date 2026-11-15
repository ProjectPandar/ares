use std::panic::{self, PanicHookInfo};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// The worker's record of a panic. A panic aborts with a trap, after which nothing may call
    /// into wasm, so the hook hands the message over before the abort.
    #[wasm_bindgen(js_name = aresWasmPanic)]
    fn report_panic(message: &str);
}

/// Installs the panic hook when the module is instantiated: the panic message and its
/// `file:line:column`, as `PanicHookInfo` displays them, reach the worker.
#[wasm_bindgen(start)]
pub fn start() {
    panic::set_hook(Box::new(|info: &PanicHookInfo| {
        report_panic(&info.to_string())
    }));
}
