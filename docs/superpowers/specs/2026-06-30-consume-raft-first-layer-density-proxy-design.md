# Consume Raft First Layer Density Proxy Design

## Upstream Boundary

This slice ports the first raft/support layer density behavior around
OrcaSlicer's existing `raft_first_layer_density` option into Ares' current
rectangular support proxy fill-line boundary. It does not implement full raft
or support polygon generation.

Source citations:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:941` declares
  `raft_first_layer_density` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5008-5016` defines
  `raft_first_layer_density` as a support-category percent option, with
  minimum `10`, maximum `100`, and default `90`.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1496-1500` applies
  `raft_first_layer_density * 0.01` as the density for classic raft support
  layer `0`.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1778-1786` applies the
  same density to the bed-contacting support base layer.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:1407-1410` applies
  `raft_first_layer_density * 0.01` to tree-support raft layer `0`.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:1453-1457` applies the
  density to raft base areas inside tree-support interface layers.
- `OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:1612-1616` applies the
  density to ordinary tree-support layer `0` base fill.

## Current Ares State

`ares-core` already registers `raft_first_layer_density` in
`options/registry/definitions/table/tail_raft.rs` with Orca source citations
and default `90`, but no runtime behavior consumes it.

Ares currently turns closed rectangular `SupportMaterial` proxy paths into
printable support base lines in `print_paths/support_base_pattern_spacing.rs`.
That transform already runs after support-interface role conversion,
`support_expansion`, and `raft_first_layer_expansion`, and before support
interface spacing, support ironing, toolpath moves, extrusion moves, speed
moves, diagnostics, and G-code. The transform currently derives its pitch only
from `support_base_pattern_spacing + support_material_width`, regardless of
layer id.

## Design

Add a focused runtime parser for `raft_first_layer_density` and route the
parsed percent into support base line generation.

Parser behavior:

- default: `90.0`
- accepted input: finite numeric value or numeric string in the inclusive
  range `10.0..=100.0`
- rejected input: values below `10`, above `100`, non-finite numbers,
  non-numeric strings, bool, null, object, or array
- errors: return `SliceError::InvalidInput` with
  `raft_first_layer_density` in the message

Print-path behavior:

- For layer `0` closed rectangular `SupportMaterial` proxy paths, use a
  density-derived pitch:
  `support_material_width / (raft_first_layer_density / 100.0)`.
- For non-layer-`0` support base proxy paths, preserve the existing
  `support_base_pattern_spacing + support_material_width` pitch.
- Keep the existing `support_base_pattern` handling: the selected first-layer
  pitch feeds either the single rectilinear family or both rectilinear-grid
  families.
- Preserve current source metadata and extrusion role through the existing
  `rebuild_path` flow.
- Validate `raft_first_layer_density` during `finalize_print_paths` before
  disabled-support filtering, so invalid values are rejected even if support
  output would later be filtered.

This intentionally applies to both current raft-active first-layer support
proxy paths and current ordinary support first-layer proxy paths. Orca names
the option under raft settings, but its tooltip and support-generation sites
cover "the first raft or support layer", and Ares' current proxy model can
express the density as layer-0 support base line pitch without needing object
trimming or full support-layer storage.

## Risks And Notes

This slice deliberately changes Ares' default layer `0` support base pitch from
the current `support_base_pattern_spacing + support_material_width` proxy
formula, about `2.9` mm with Ares' defaults, to
`support_material_width / 0.9`, about `0.44` mm with the default `90%` density
and `0.4` mm support width. That is an expected movement toward Orca parity,
not a compatibility fallback: `TreeSupport.cpp:1408-1409` uses density for
raft/support layer `0`, while non-layer-`0` raft density falls back to the
spacing-derived formula. Existing tests that use layer `0` as a baseline for
`support_base_pattern_spacing` or `raft_first_layer_expansion` need to move to
a non-first layer, keep a single-line assertion only through non-rectangular
input, or explicitly recompute expected line coordinates for
`raft_first_layer_density`.

Known existing test migrations:

- `support_base_pattern_spacing::larger_spacing_changes_support_material_gcode_coordinates_and_line_count`
  must keep testing non-first-layer spacing, because layer `0` spacing is no
  longer controlled by `support_base_pattern_spacing`.
- `support_raft_first_layer_expansion::raft_default_first_layer_expansion_expands_support_material_before_base_spacing`
  must recompute the expanded layer-`0` support-base lines with the default
  `90%` density pitch.
- `support_raft_first_layer_expansion::expanded_first_layer_support_material_preserves_source_metadata`
  must assert metadata across every generated density line instead of assuming
  one line.
- `support_raft_first_layer_expansion::zero_raft_first_layer_expansion_keeps_first_layer_support_geometry`
  and
  `support_raft_first_layer_expansion::raft_inactive_first_layer_expansion_keeps_support_geometry`
  must recompute expected layer-`0` density lines.
- `support_raft_first_layer_expansion::raft_first_layer_expansion_composes_after_support_expansion`
  must recompute expected post-expansion layer-`0` density lines.
- `support_expansion::support_expansion_changes_emitted_support_gcode_span`
  is an audited layer-`0` rectangular `SupportMaterial` G-code test. Its
  current span substring assertions survive the density change, but the module
  stays in the targeted verification set because it exercises the same
  expansion-before-support-base-line path.

## Included Behavior

- Omitted `raft_first_layer_density` makes layer `0` `SupportMaterial` proxy
  lines use Orca's default `90%` density.
- Explicit valid densities in `10..=100` change layer `0`
  `SupportMaterial` proxy line pitch and emitted support G-code coordinates.
- `raft_first_layer_density = 100` uses the support material width as the
  first-layer support base pitch.
- `raft_first_layer_density = 50` uses twice the support material width as the
  first-layer support base pitch.
- Non-first-layer `SupportMaterial` proxy paths continue to use
  `support_base_pattern_spacing + support_material_width`.
- `SupportMaterialInterface`, open paths, non-rectangular support paths, and
  non-support paths remain outside support base density conversion, except
  when the existing `support_interface_top_layers = 0` conversion first
  rewrites interface paths to `SupportMaterial`.
- `support_base_pattern = rectilinear-grid` uses the same density-derived
  first-layer pitch for both line families.
- The density composes with prior `support_expansion` and
  `raft_first_layer_expansion`, because those transforms still run before
  support base line generation.
- Invalid values fail during finalization and include the option key in the
  error text.

## Deferred Behavior

- Full raft layer generation from `SupportCommon.cpp`.
- Full tree-support raft/support area generation from `TreeSupport.cpp` and
  `TreeSupport3D.cpp`.
- Raft base areas inside later tree-support interface layers, because Ares
  does not yet distinguish raft interface areas from ordinary
  `SupportMaterialInterface` proxy paths.
- Orca sheath/perimeter generation around first-layer support fill.
- Exact Orca fill engine behavior, including arbitrary ExPolygon offsets,
  clipping, path sorting, linking, `FillParams::dont_adjust`, and
  `fill_expolygons_with_sheath_generate_paths`.
- `raft_expansion`, `raft_contact_distance`, and full `raft_layers`
  material/height planning beyond Ares' current proxy activation.
- UI, CLI, WASM bindings, and Orca binary E2E parity.

## Acceptance Criteria

1. `SliceOptions::raft_first_layer_density_percent()` returns the Orca default
   `90.0` when the option is omitted.
2. The parser accepts finite numeric and numeric string values in
   `10..=100`, and rejects invalid values with `SliceError::InvalidInput`
   containing `raft_first_layer_density`.
3. A layer `0` closed rectangular `SupportMaterial` proxy path uses
   `support_material_width / density` as its support base line pitch.
4. Non-first-layer `SupportMaterial` proxy paths preserve the existing
   `support_base_pattern_spacing + support_material_width` pitch.
5. Layer `0` density changes emitted support G-code coordinates and line
   count.
6. `rectilinear-grid` support base pattern uses the density-derived layer `0`
   pitch for both line families.
7. Non-rectangular, open, interface, and non-support paths remain unchanged by
   this option unless existing interface-to-base conversion runs first.
8. Existing `support_expansion` and `raft_first_layer_expansion` geometry is
   consumed before the density-derived support base lines are generated.
9. `docs/roadmap.md` records this source-cited density proxy slice and names
   the deferred full raft/support generator behavior.

## Verification Plan

- Add RED parser tests under `crates/ares-core/src/options/tests/raft.rs`.
- Add RED pipeline and G-code tests under
  `crates/ares-core/src/pipeline/tests/support_raft_first_layer_density.rs`
  for default/explicit density, first-layer-only behavior, G-code visibility,
  rectilinear-grid composition, unchanged non-target paths, invalid values,
  and composition after first-layer raft expansion.
- Update existing support-base-spacing tests that currently use layer `0`
  G-code output so they keep testing non-first-layer spacing behavior after
  the first-layer density default becomes active, and ensure the roadmap entry
  records the intentional default pitch change.
- Update existing raft-first-layer-expansion tests that use layer `0`
  rectangular `SupportMaterial` paths so they either assert the new
  density-derived line families or continue isolating expansion behavior
  through non-target paths where a single path is still expected.
- Run targeted tests:
  - `cargo nextest run -p ares-core raft_first_layer_density`
  - `cargo nextest run -p ares-core support_base_pattern_spacing support_base_pattern support_expansion support_raft_first_layer_expansion`
- Run final repo checks:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check -p ares-core --target wasm32-unknown-unknown`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo nextest run --workspace`
  - Rust touched-file LOC guard for files over 400 lines
