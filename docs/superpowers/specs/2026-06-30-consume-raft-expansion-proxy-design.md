# Consume Raft Expansion Proxy Design

## Upstream Boundary

This slice ports the XY expansion behavior around OrcaSlicer's existing
`raft_expansion` option into Ares' current rectangular support proxy boundary.
It does not implement full raft material planning or arbitrary support polygon
generation.

Source citations:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:940` declares
  `raft_expansion` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4999-5006` defines
  `raft_expansion` as a support-category millimeter option, with minimum `0`
  and default `1.5`.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:1401` documents that
  users may increase `raft_expansion` for better first-layer adhesion.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:1575-1580` expands
  layer-0 raft contact polygons by `raft_expansion` when the value is
  positive.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport3D.cpp:1029-1030` expands tree
  support raft contact geometry by `raft_expansion`.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport3D.cpp:1049-1056` uses
  `raft_expansion` when pruning tree support elements inside the expanded raft
  contact layer.

## Current Ares State

`ares-core` already registers `raft_expansion` in
`options/registry/definitions/table/tail_raft.rs` with Orca source citations
and default `1.5`, but no runtime behavior consumes it.

Ares currently has a focused raft runtime module in `options/raft.rs` for
`raft_layers`, `raft_first_layer_expansion`, and
`raft_first_layer_density`. `raft_layers > 0` is a proxy activation gate: it
preserves support proxy artifacts when `enable_support` is absent or false,
but it does not create separate raft layers, raft contact layers, or support
generator storage.

Ares also has a rectangular support proxy transform in
`print_paths/support_interface.rs`: `support_expansion` expands closed
rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths, and
`raft_first_layer_expansion` expands the same path shapes only on layer `0`
when raft is active. Support base/interface line generation, support ironing,
toolpath moves, extrusion moves, speed moves, diagnostics, and G-code emission
consume those transformed paths later in `finalize_print_paths`.

## Design

Add a focused runtime parser for `raft_expansion` and a raft-active rectangular
support proxy expansion pass.

Parser behavior:

- default: `1.5`
- accepted input: finite numeric value or numeric string greater than or equal
  to `0.0`
- rejected input: negative values, non-finite numbers, non-numeric strings,
  bool, null, object, or array
- errors: return `SliceError::InvalidInput` with `raft_expansion` in the
  message

Print-path behavior:

- Parse `raft_expansion` during `finalize_print_paths` before disabled-support
  filtering, so invalid values are rejected even if support output would later
  be filtered.
- If `raft_layers == 0` or `raft_expansion == 0.0`, leave paths unchanged.
- If `raft_layers > 0` and `raft_expansion > 0.0`, expand closed rectangular
  `SupportMaterial` and `SupportMaterialInterface` proxy paths on existing
  Ares layers with `layer_id < raft_layers`.
- Preserve open paths, non-rectangular paths, non-support paths, metadata,
  extrusion role, closure, and layer `print_z`.
- Run the new pass after `support_expansion` and before
  `raft_first_layer_expansion`, so layer `0` composes in this order:
  `support_expansion`, `raft_expansion`, then `raft_first_layer_expansion`.
- Keep support base/interface spacing and support ironing downstream, so
  emitted G-code visibly reflects the expanded proxy geometry.

Ares does not yet materialize true Orca raft layers. The `layer_id <
raft_layers` rule is therefore a temporary proxy interpretation of Orca's
"Expand all raft layers in XY plane" option that is limited to the existing
support proxy layer stream.

## Risks And Notes

Consuming Orca's default `raft_expansion = 1.5` intentionally changes any
current raft-active closed rectangular support proxy output on targeted layers.
Existing tests that isolate `raft_first_layer_expansion` or
`raft_first_layer_density` should set `raft_expansion = 0.0` unless the test is
explicitly checking composition.

Known existing test migrations:

- `support_raft_first_layer_expansion.rs` should isolate first-layer expansion
  expectations by adding `raft_expansion = 0.0` to raft-active test inputs,
  except for one new composition assertion if needed.
- `support_raft_first_layer_expansion_gcode.rs` should add
  `raft_expansion = 0.0` to keep the G-code test scoped to
  `raft_first_layer_expansion`.
- `support_raft_first_layer_density::density_composes_after_raft_first_layer_expansion`
  should add `raft_expansion = 0.0` so it remains a density plus
  first-layer-expansion test.

## Included Behavior

- Omitted `raft_expansion` returns Orca's default `1.5` mm.
- Explicit non-negative finite numeric and numeric string values are accepted.
- Invalid values fail during finalization and include the option key in the
  error text.
- `raft_layers = 1` expands layer `0` closed rectangular support proxy paths by
  the default or configured expansion. On layer `0`, this composes with the
  existing default `raft_first_layer_expansion = 2.0` unless a test or caller
  sets `raft_first_layer_expansion = 0.0` to isolate `raft_expansion`.
- `raft_layers = 2` expands current proxy layers `0` and `1`, while layer `2`
  remains outside the raft-expansion proxy scope.
- `SupportMaterialInterface` paths are expanded before support interface
  spacing and support ironing.
- `SupportMaterial` paths are expanded before support base pattern spacing and
  support base density conversion.
- `support_expansion`, `raft_expansion`, and `raft_first_layer_expansion`
  compose in the stated order.
- G-code emitted from expanded support proxy paths changes coordinates compared
  with `raft_expansion = 0.0`.

## Deferred Behavior

- Full Orca raft layer generation and raft contact/base/interface planning.
- Exact Orca support-layer storage, support generator invalidation, and
  support-layer synchronization.
- Arbitrary ExPolygon offsetting, clipping, hole handling, support area
  simplification, path sorting, linking, and fill-engine behavior.
- Tree support element pruning beyond the current rectangular proxy path
  stream.
- `raft_contact_distance` Z-gap behavior.
- UI, CLI, WASM bindings, and Orca binary E2E parity.

## Acceptance Criteria

1. `SliceOptions::raft_expansion_mm()` returns `1.5` when the option is
   omitted.
2. The parser accepts finite non-negative numeric and numeric string values,
   and rejects invalid values with `SliceError::InvalidInput` containing
   `raft_expansion`.
3. `finalize_print_paths` validates `raft_expansion` before disabled-support
   filtering.
4. With `raft_layers = 1` and `raft_first_layer_expansion = 0.0`, a layer `0`
   closed rectangular support proxy path is expanded by default `1.5` mm
   unless `raft_expansion = 0.0`.
5. With `raft_layers = 2`, layers `0` and `1` are expanded by the configured
   raft expansion, while layer `2` is unchanged.
6. `SupportMaterialInterface` expansion is visible before support ironing and
   interface spacing.
7. Non-rectangular, open, non-support, and raft-inactive paths remain unchanged
   by this option.
8. `support_expansion`, `raft_expansion`, and
   `raft_first_layer_expansion` compose in order on layer `0`.
9. Expanded raft proxy geometry changes emitted support G-code coordinates.
10. `docs/roadmap.md` records this source-cited proxy slice and names the
    deferred full raft/support generator behavior.

## Verification Plan

- Add parser tests under `crates/ares-core/src/options/tests/raft.rs`.
- Add a new pipeline test module under
  `crates/ares-core/src/pipeline/tests/support_raft_expansion.rs` for default
  expansion, configured multi-layer proxy scope, inactive/zero behavior,
  invalid values, support-interface spacing/ironing visibility, non-target
  paths, and composition after `support_expansion` plus before
  `raft_first_layer_expansion`.
- Add a G-code visibility test under
  `crates/ares-core/src/pipeline/tests/support_raft_expansion_gcode.rs`.
- Register the new test modules in `crates/ares-core/src/pipeline/tests.rs`.
- Update existing first-layer expansion/density tests only where needed to set
  `raft_expansion = 0.0` for isolation.
- Update `docs/roadmap.md`.
- Run targeted tests:
  - `cargo nextest run -p ares-core raft_expansion`
  - `cargo nextest run -p ares-core support_raft_expansion support_raft_first_layer_expansion support_raft_first_layer_density`
  - `cargo nextest run -p ares-core support_enable support_expansion support_base_pattern_spacing`
- Run final repo checks:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run --workspace`
  - Rust touched-file LOC guard for files over 400 lines
