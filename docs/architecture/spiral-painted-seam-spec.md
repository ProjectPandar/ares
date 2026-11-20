# Seam paint in configured spiral mode

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/GCode.cpp`:

- `GCode::process_layer` (6657–6671) decides whether to enable the spiral
  G-code rewrite for each layer. Bottom shells, fills, multiple perimeter
  items, and brim/skirt constraints can keep the rewrite off.
- `GCode::extrude_loop` (8526–8549) calls `SeamPlacer::place_seam` only for
  perimeters when `m_config.spiral_mode` is false. Otherwise it splits the
  loop at the supplied start point or last nozzle position. This checks
  configured mode, not the per-layer rewrite flag: even the fixed-Z bottom
  shells bypass SeamPlacer and ignore seam paint.

## Rust boundary

Remove the spiral-only intake rejection in
`project_slice/seam_placement/paint.rs::validate_paint` and its call from
`project_slice/state.rs`. `project_slice/seam_placement/runtime.rs::placement_modes`
(used by `seam_placement::apply`) already bypasses placement for the entire
configured spiral print. Keep that dispatch and the existing loop split and
spiral rewrite unchanged. Non-spiral paint placement is unchanged, as is
selector validation at import: codes outside the supported palette remain
invalid. No other paint behavior or spiral admission change is included.

## Public behavior

Derive a painted single-part box and an unpainted control from the synthetic
seam-paint inputs, with identical spiral settings, nonzero bottom shell count,
and a wall above the bottom shells. In both Native modes, prove the painted and unpainted Native
streams are identical, with fixed-Z bottom perimeters and rising-Z vase
perimeters in the witness. Compare each production API and CLI stream against
its own repeatable Native stream, in order and byte for byte except for the
canonical generator line.
