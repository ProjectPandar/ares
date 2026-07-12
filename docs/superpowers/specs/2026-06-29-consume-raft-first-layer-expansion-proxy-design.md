# Consume Raft First Layer Expansion Proxy Design

## Upstream Boundary

This slice ports the raft-active first-layer XY expansion behavior around
OrcaSlicer's existing `raft_first_layer_expansion` option into Ares' current
support proxy path boundary. It does not implement the full Orca raft/support
layer generator.

Source citations:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:942` declares
  `raft_first_layer_expansion` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5018-5026` defines
  `raft_first_layer_expansion` as a support-category `coFloat` in millimeters,
  with minimum `0` and default `2.0`.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:286-288` computes a
  classic-support compensated first-layer expansion by subtracting
  `inflate_factor_fine`; Ares does not yet have the separate fine interface
  polygon model that makes that compensation exact.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:317-349` applies that
  compensated first-layer inflation in the full classic raft layer generator,
  which remains deferred.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:1394-1400` expands
  tree-support raft areas and applies the configured offset on layer `0`.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:2352-2364` applies hybrid
  tree first-layer expansion when no raft layers are active; Ares does not yet
  have the support-layer model needed for that branch.

## Current Ares State

`ares-core` already registers `raft_first_layer_expansion` in
`options/registry/definitions/table/tail_raft.rs` with Orca source citations
and default `2.0`, but no runtime behavior consumes it.

Ares currently has a rectangular support proxy transform in
`print_paths/support_interface.rs` for the existing `support_expansion` option.
`finalize_print_paths` applies this transform before support base/interface
spacing, ironing, and disabled-support filtering. The previous raft slice also
made `raft_layers > 0` preserve current support proxy paths, matching Orca's
support-material state predicate without generating real raft layers.

## Design

Add a focused parser for `raft_first_layer_expansion` and apply it only to
first-layer, raft-active, closed rectangular support proxy paths:

- default: `2.0`, matching Orca's option definition
- accepted input: finite numeric value or numeric string greater than or equal
  to `0`
- rejected input: negative, non-finite, non-numeric, bool, null, object, or
  array, returning `SliceError::InvalidInput` with `raft_first_layer_expansion`
  in the message

The first-layer expansion transform must run in the current support path
finalization pipeline after `support_expansion` and before base/interface
spacing, so the expanded first-layer rectangle is what downstream spacing,
ironing, toolpath, extrusion, speed, diagnostics, and G-code stages observe.
On the current rectangular proxy, `support_expansion` and
`raft_first_layer_expansion` compose additively on layer `0` because there is no
separate Orca-style fine interface polygon pass to compensate.

The transform applies only when all of these are true:

- `raft_layers > 0`
- `layer_id == 0`
- the path role is `PrintPathRole::SupportMaterial` or
  `PrintPathRole::SupportMaterialInterface`
- the path is closed
- the path is recognized as a rectangle by the existing rectangle helper
- `raft_first_layer_expansion > 0`

If the expansion value is `0`, the layer is not layer `0`, raft is inactive,
the path is not a support proxy path, the path is open, or the path is not a
recognized rectangle, the path remains unchanged. If a future negative
shrink/collapse behavior is needed, it belongs to another source-cited slice;
Orca's option minimum is `0`.

This proxy follows the raw configured offset form used by
`TreeSupport.cpp:1394-1400`, because Ares currently has one rectangular support
proxy footprint rather than the separate `interface_polygons`, `base`, and
`columns` polygons used by `SupportCommon.cpp:306-349`. Reuse the existing
rectangular rebuild behavior so metadata and extrusion role handling stay
consistent with `support_expansion`.

## Included Behavior

- Positive `raft_layers` plus omitted `raft_first_layer_expansion` expands
  current first-layer support proxy rectangles by Orca's default `2.0` mm.
- Positive `raft_layers` plus explicit positive `raft_first_layer_expansion`
  expands current first-layer support proxy rectangles by the configured
  millimeter value.
- Explicit `raft_first_layer_expansion = 0` keeps first-layer support proxy
  rectangles unchanged.
- Non-first support proxy layers remain unchanged even when raft is active.
- Raft-inactive support proxy paths remain unchanged by this option, including
  ordinary support-only first-layer proxies.
- Non-rectangular, open, and non-support paths remain unchanged.
- Expanded first-layer support proxy geometry affects downstream spacing,
  toolpath moves, extrusion moves, speed moves, diagnostics, and emitted
  support G-code coordinates.
- On layer `0`, existing `support_expansion` and
  `raft_first_layer_expansion` compose additively because
  `support_expansion` runs first in Ares' current proxy pipeline.
- Invalid `raft_first_layer_expansion` values fail during finalization before
  disabled-support filtering.

## Deferred Behavior

- Full raft layer generation from `SupportCommon.cpp:317-349`.
- The classic-support `inflate_factor_fine` split from
  `SupportCommon.cpp:286-312`, including the `0.5` mm fine interface expansion
  when multiple raft layers are active and its matching subtraction from the
  first-layer expansion. That compensation requires separate interface/base
  polygons rather than Ares' current single rectangular proxy footprint.
- Orca's no-raft normal-support first-layer expansion branch from
  `SupportCommon.cpp:286-288` and `TreeSupport.cpp:2352-2364`, because Ares
  does not yet model the support-layer storage, `bottom_z`, raft interface
  height, or object-trimming data those branches require.
- `raft_expansion`, `raft_first_layer_density`, and `raft_contact_distance`.
- Arbitrary polygon offsets, stepped trimming against object slices, organic
  support, tree support, and support blockers/enforcers.
- UI, CLI, WASM bindings, and Orca binary E2E parity.

## Acceptance Criteria

1. With `raft_layers > 0` and omitted `raft_first_layer_expansion`, a layer `0`
   closed `SupportMaterial` rectangle expands by `2.0` mm before support base
   spacing converts it to printable lines.
2. With `raft_layers > 0` and explicit `raft_first_layer_expansion`, a layer
   `0` closed `SupportMaterialInterface` rectangle expands by that value before
   interface spacing, support ironing, and G-code emission.
3. With `raft_layers > 0` and `raft_first_layer_expansion = 0`, layer `0`
   support proxy geometry remains unchanged.
4. With `raft_layers = 0` or omitted, layer `0` support proxy geometry remains
   unchanged by `raft_first_layer_expansion`.
5. Non-first support proxy layers remain unchanged by
   `raft_first_layer_expansion`.
6. Non-rectangular, open, and non-support paths remain unchanged by
   `raft_first_layer_expansion`.
7. Invalid `raft_first_layer_expansion` values return
   `SliceError::InvalidInput` during finalization and include the option key in
   the error text.
8. `docs/roadmap.md` records this source-cited raft-active first-layer
   expansion proxy slice and names the deferred full raft/support generator
   behavior.

## Verification Plan

- Add RED pipeline tests under
  `crates/ares-core/src/pipeline/tests/support_raft_first_layer_expansion.rs`
  for default expansion, explicit expansion, zero expansion, raft-inactive
  behavior, non-first-layer behavior, unchanged unsupported path shapes, invalid
  values, and G-code coordinate changes.
- Add direct parser tests if the implementation introduces a public
  `SliceOptions` accessor.
- Run targeted tests:
  - `cargo nextest run -p ares-core support_raft_first_layer_expansion`
  - `cargo nextest run -p ares-core support_expansion support_enable`
- Run final repo checks:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run --workspace`
  - Rust touched-file LOC guard for files over 400 lines.
