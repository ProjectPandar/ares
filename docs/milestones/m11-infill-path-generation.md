# M11: Infill path generation

## Goal
Generate first deterministic sparse infill path artifacts by clipping rectilinear scanlines to existing simple contours and expose them through the core pipeline and CLI metadata.

## Exit checklist
- `ares-core` exposes `generate_infills`, `LayerInfills`, `InfillPath`, `InfillRole`, and `InfillOptions`.
- `SliceOptions` exposes typed `sparse_infill_density`, `infill_direction`, and `sparse_infill_line_width` accessors.
- Density `0` preserves represented layers with empty infill lists.
- Valid simple contours produce deterministic sparse infill path artifacts clipped inside contour boundaries.
- Malformed contours with fewer than three points are rejected.
- `SlicingPipeline` includes infill artifacts and diagnostics.
- `slice` and `ares slice` output include total and per-layer infill metadata.
- Existing segment, contour, and perimeter metadata remains unchanged except for appending the infill stage/metadata.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No solid infill classification, top/bottom shells, bridge infill, gap fill, support infill, hole-aware clipping, polygon offsets, perimeter overlap, path connection optimization, extrusion values, speeds, accelerations, or Orca G-code parity.
- No new workspace crates.
