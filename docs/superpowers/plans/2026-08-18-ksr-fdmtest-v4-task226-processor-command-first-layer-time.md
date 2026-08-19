# Plan: KSR FDM Test V4 task226 processor command and first-layer timing semantics

1. Add failing processor tests proving an unsupported command cannot mutate modal feedrate and first-layer time ends at the first `; CHANGE_LAYER` marker.
2. Move `F` parsing behind supported-motion dispatch and select the first layer transition without changing other processor state.
3. Run focused processor tests, strict `ares-core` Clippy, rustfmt check, and the KSR project slice smoke test; record the remaining timing delta.
4. Commit and push this source-cited processor slice independently.
