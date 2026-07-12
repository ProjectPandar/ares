# M12: Ordered print path artifacts

## Goal
Combine perimeter and sparse infill artifacts into deterministic layer-level print path artifacts using Orca-compatible wall/infill ordering, including layer-0 wall-first behavior.

## Exit checklist
- `ares-core` exposes `generate_print_paths`, `LayerPrintPaths`, `PrintPath`, and `PrintPathRole`.
- `SliceOptions` exposes typed `is_infill_first()` with Orca default `false`.
- Default print path order is external perimeters before sparse infill.
- `is_infill_first = true` keeps layer 0 perimeter-first and orders later layers sparse infill before external perimeters.
- Layer metadata mismatches between perimeter and infill artifacts are rejected.
- `SlicingPipeline` includes print path artifacts and diagnostics.
- `slice` and `ares slice` output include total and per-layer print path metadata.
- Existing segment, contour, perimeter, and infill metadata remains unchanged except for appending print path stage/metadata.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No support, bridges, skirt/brim, extrusion values, speeds, accelerations, travel optimization, island/extruder grouping, or Orca G-code parity.
- No new workspace crates.
