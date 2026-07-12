# M277: PrintApply PrintObjectStatus state

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintObjectStatus` state declaration in `OrcaSlicer/src/libslic3r/PrintApply.cpp:473-498`, with follow-on `PrintObjectStatusDB` context from `PrintApply.cpp:500-540`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add private `ares-core` staged data structures for `PrintObjectStatus::Status` values: `Unknown`, `Deleted`, `Reused`, `New`.
- Add a private staged print-object status record keyed by model-object/print-object id.
- Preserve constructor defaults equivalent to `PrintObjectStatus(ObjectID id)`: null/deferred print object, identity/deferred transform, and `Unknown` status.
- Preserve source-order enum vocabulary and id-based ordering/equality semantics for later `PrintObjectStatusDB` work.
- Keep `PrintObject *` and `Transform3d` as documented deferred upstream payloads only; do not add Rust fields, tokens, or placeholder print-object/transform pipelines for them in M277.
- Defer `PrintObjectStatusDB`, real `PrintObject` pointers, concrete `Transform3d`, model-object apply wiring, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
