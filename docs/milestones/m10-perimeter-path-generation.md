# M10: Perimeter path generation

## Goal
Generate first external perimeter path artifacts from existing closed contours and expose them through the core pipeline and CLI metadata.

## Exit checklist
- `ares-core` exposes `generate_perimeters`, `LayerPerimeters`, `PerimeterPath`, and `PerimeterRole`.
- Each valid contour becomes one external perimeter path without duplicating the closing point.
- Empty contour layers remain represented with empty perimeter lists.
- Malformed contours with fewer than three points are rejected.
- `SlicingPipeline` includes perimeter artifacts and diagnostics.
- `slice` and `ares slice` output include total and per-layer perimeter metadata.
- Existing segment and contour metadata remains unchanged.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No polygon offsetting, internal wall loops, gap fill, fill surfaces, extrusion E values, seam placement, overhang detection, Arachne, spiral vase, or Orca perimeter parity.
- No new workspace crates.
