# Plan: KSR FDM Test V4 task22o226 grouped offset PolyTree execution

1. Add a focused regression proving that a negative PolyTree offset over disconnected holed paths retains grouped root order.
2. Replace per-path raw offset and follow-up union in `offset_paths_tree` with one configured `ClipperOffset` containing every path, matching Orca's grouped `offset_paths<PolyTree>` call.
3. Re-run the offset regression and focused first-layer project slice; regenerate KSR G-code and record the next normalized motion divergence and structural counts.
4. Run formatting, focused Clippy, and file-size checks; update the roadmap and commit/push this source-cited slice independently.
