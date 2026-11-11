//! Undo/redo snapshots of the plater (`UndoRedo::Stack`, `slic3r/Utils/UndoRedo.cpp`); the web
//! keeps the stack.

use wasm_bindgen::prelude::*;

use super::PlaterSession;

/// A saved state of a plater session's model and plates.
#[wasm_bindgen]
pub struct PlaterSnapshot(ares_core::PlaterSnapshot);

#[wasm_bindgen]
impl PlaterSession {
    /// The model and plates as they are (`UndoRedo::StackImpl::take_snapshot`).
    #[wasm_bindgen(js_name = takeSnapshot)]
    pub fn take_snapshot(&self) -> PlaterSnapshot {
        PlaterSnapshot(self.session.snapshot())
    }

    /// Puts back a snapshot of this session (`UndoRedo::StackImpl::load_snapshot`).
    #[wasm_bindgen(js_name = loadSnapshot)]
    pub fn load_snapshot(&mut self, snapshot: &PlaterSnapshot) {
        self.session.load_snapshot(&snapshot.0);
    }
}
