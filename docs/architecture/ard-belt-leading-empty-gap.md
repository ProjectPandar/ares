# Belt lead-in does not count as an empty-layer gap

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/GCode.cpp:2431–2443`: `GCode::collect_layers_to_print`
uses `belt_leading_gap` to exclude a gap before an object's first extrusion
when `belt_printer` is enabled. The object rests on the conveyor even when its
first extrusion is well above Z=0 in the slicing frame. The first extrusion
still becomes the reference for later gap checks (`:2445–2447`).

## Rust boundary

Port this condition at the range insertion in
`crates/ares-core/src/project_slice/gcode_emit/layers/schedule/empty_layers.rs`.
The existing collector retains its per-object last-extrusion state. If the
configured printer is a belt printer and the object has no previous extrusion
layer, omit the leading range, but still update the last extrusion. Retain all
later gap checks, Cartesian warnings, and checked CLI rejection behavior.

This change does not alter layer selection, brim scheduling, geometry, axis
remapping, or output order. The existing strict CLI parity cases for tilted
belt printing and translated belt placement exercise the lead-in. The
Cartesian empty-layer and ordered-warning cases retain their rejection checks.
