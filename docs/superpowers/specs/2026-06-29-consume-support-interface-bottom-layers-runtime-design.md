# Consume Support Interface Bottom Layers Runtime Design

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:965`: `support_interface_bottom_layers` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6090-6102`: option definition, `min = -1`, default `0`, and `-1` enum value labeled "Same as top".
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:27-32`: `num_top_interface_layers = max(0, support_interface_top_layers)`, `num_bottom_interface_layers = support_interface_bottom_layers < 0 ? num_top_interface_layers : support_interface_bottom_layers`, and bottom contacts exist when the resolved bottom count is positive.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1648-1741`: top and bottom interface enablement controls whether contact/interface layers are extruded as `erSupportMaterialInterface` or `erSupportMaterial`.

## Rust Destination

- `crates/ares-core/src/print_paths/support_interface.rs`
- `crates/ares-core/src/pipeline/tests/support_interface_top_layers_runtime.rs`, extending the existing top-layer runtime test file because this slice changes the same generic interface reclassification shell.

## Current Ares Boundary

Ares currently has only a generic `PrintPathRole::SupportMaterialInterface` path role in the print-path finalization shell. It does not yet generate or classify Orca top-contact, bottom-contact, raft-contact, or generic interface layers.

The implemented compatibility behavior must therefore be limited to existing generic support-interface paths. Full bottom-contact geometry generation, bottom-interface spacing/density, support region merging, raft contact behavior, organic tree behavior, and separate top/bottom support layer classification remain deferred until their upstream support-generation slices are ported.

The `top = 0, bottom > 0` preservation rule below applies only to this generic Ares shell and is not a full port of Orca's `bottom_interfaces = top_interfaces && config.support_interface_bottom_layers != 0` contact-layer extrusion gate.

## Required Behavior

1. Parse `support_interface_bottom_layers` from `SliceOptions` in `finalize_print_paths`.
   - Missing value uses Orca default `0`.
   - Accepted values are decimal integers `>= -1`, from JSON numbers or numeric strings.
   - Reject non-integral, less-than-`-1`, non-finite, boolean, null, array, and object values with `SliceError::InvalidInput` mentioning `support_interface_bottom_layers`.
2. Continue parsing `support_interface_top_layers` as a non-negative decimal integer with default `3`.
3. Resolve bottom layer count using Orca semantics:
   - `support_interface_bottom_layers < 0` resolves to the parsed top layer count.
   - Otherwise it resolves to the parsed bottom layer count.
4. Reclassify existing generic `SupportMaterialInterface` paths to `SupportMaterial` only when both resolved interface counts are zero.
5. Preserve current behavior when `support_interface_bottom_layers` is omitted: `support_interface_top_layers = 0` still reclassifies the generic interface path to support material because the bottom default is `0`.
6. Preserve generic interface paths when either count is positive:
   - `support_interface_top_layers = 0`, `support_interface_bottom_layers = -1` preserves nothing through bottom layers because `-1` resolves to top `0`.
   - `support_interface_top_layers = 0`, `support_interface_bottom_layers = 1` preserves the generic interface path as interface material.
   - `support_interface_top_layers = 2`, `support_interface_bottom_layers = 0` preserves the generic interface path as interface material.
7. Keep path geometry and metadata preservation unchanged when a path is reclassified.

## Deferred Behavior

- New support polygons, contact-layer generation, bottom-contact detection, top/bottom interface path role split, raft contact behavior, `support_bottom_interface_spacing`, `bottom_interface_density`, bridge flow for bottom contacts, support material region merging, tree/organic support behavior, object contact analysis, Z-distance support generation, UI behavior, and Orca binary E2E are out of scope.

## Acceptance Criteria

- Focused tests cover default omitted behavior, explicit positive bottom preserving generic interfaces when top is zero, `-1` resolving to top, positive top with zero bottom preserving generic interfaces, invalid bottom values, and metadata-preserving reclassification when both counts are zero.
- Existing support interface top-layer, spacing, pattern, ironing, speed/flow, and support expansion tests continue to pass.
- `cargo fmt --check`, `git diff --check`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted nextest, and `cargo nextest run --workspace` pass before commit.
