# Plan: Task 22o.111 first-layer skirt hull

1. Add a focused KSR project assertion for the exact rendered `G29` first-layer footprint; run it red against the current compensated-island-only bounds.
2. Move first-layer footprint calculation from the project emitter into `gcode_emit/footprint.rs` and port OrcaSlicer skirt/brim activation and occupied-layer selection.
3. Apply typed `skirt_distance` to the selected occupied hull bounds and keep the no-skirt/no-brim first-layer fallback.
4. Run the focused test, the KSR CLI smoke slice, rustfmt, and clippy; commit and push the completed vertical slice.
