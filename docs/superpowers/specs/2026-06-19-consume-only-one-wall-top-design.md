# Consume only_one_wall_top Design

## Goal

Consume OrcaSlicer's existing `only_one_wall_top` option as concrete perimeter and G-code behavior in Ares. This slice must reduce generated internal perimeter walls on the topmost generated layer when the option is true, instead of only preserving option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1176` declares `only_one_wall_top` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1411-1415` registers `only_one_wall_top` as a boolean quality option with default `false`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1225-1233` computes classic perimeter `loop_number`, applies `alternate_extra_wall`, applies `only_one_wall_first_layer`, then sets the topmost layer to one wall when `loop_number > 0`, `only_one_wall_top` is true, and `upper_slices == nullptr`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:2164-2176` applies the same topmost-layer condition in the Arachne perimeter path.

## Current Ares Boundary

- `crates/ares-core/src/options.rs::SliceOptions::perimeter_options()` already parses perimeter options into `PerimeterOptions`.
- `crates/ares-core/src/perimeters.rs` already generates one external rectangular loop plus internal rectangular loops from `wall_loops`, consumes `only_one_wall_first_layer`, and consumes `alternate_extra_wall`.
- Ares does not yet model Orca `upper_slices`; the closest current boundary is the last `LayerContours` item passed to `generate_perimeters()`.
- `crates/ares-core/src/pipeline/test_support.rs::rectangular_layers_pipeline()` can build deterministic multi-layer perimeter and G-code fixtures.

## Design

Add `only_one_wall_top: bool` to `PerimeterOptions`, defaulting to `false`. Parse it in `SliceOptions::perimeter_options()` with the same boundary used by existing boolean perimeter options: missing values use the Orca default and non-boolean values return `SliceError::InvalidInput` naming `only_one_wall_top`.

Existing legacy ingestion for Orca's old `top_one_wall_type` key currently rewrites string values other than `none` into `only_one_wall_top: "1"`. Because `only_one_wall_top` becomes executable boolean input in this slice, the legacy rewrite must now store `only_one_wall_top: true` for the same string values. The matching predicate stays unchanged: string `none` and non-string `top_one_wall_type` values remain under the legacy key and do not create `only_one_wall_top`.

`generate_perimeters()` will identify the topmost generated layer as `layers.last().map(LayerContours::layer_id)`. When resolving effective wall loops for a layer, Ares will preserve existing behavior in this order:

1. `wall_loops == 0` emits no perimeter paths.
2. `only_one_wall_first_layer` on layer `0` emits only the external wall.
3. `alternate_extra_wall` may add one wall on odd non-spiral layers when sparse infill density is positive, matching the current Ares predicate.
4. If the current layer id equals the topmost generated layer id, `only_one_wall_top` is true, and the effective wall loop count is greater than one, emit only the external wall.

For a one-layer object with both `only_one_wall_first_layer` and `only_one_wall_top` true, Ares emits one external wall. For `wall_loops == 1`, the top option keeps the single external wall. For `wall_loops == 0`, the top option does not synthesize a wall.

This slice deliberately treats the last generated layer as Orca's `upper_slices == nullptr` equivalent. It does not use `top_shell_layers`; upstream's condition is based on absence of upper slices, not the configured top-shell count.

## Docs Impact

Update `docs/roadmap.md` so M41 no longer reads as though `only_one_wall_top` behavior is still fully deferred after this slice. The roadmap update should state that this later source-cited perimeter slice consumes `only_one_wall_top` runtime perimeter/G-code behavior, while the original M41 registry milestone still deferred that behavior at the time and the other M41 deferred items remain deferred.

Do not rewrite retained historical milestone/spec documents such as `docs/milestones/m41-print-config-one-wall-quality-registry.md`, `docs/milestones/m175-print-config-legacy-alias-top-wall.md`, or earlier behavioral superpowers specs. Source-line-only Option documents are removed with their pinning scaffolds. This slice's spec, plan, implementation tests, and roadmap note are the forward record that the behavior is executable.

## Deferred Behavior

This slice does not port Arachne variable-width walls, true `upper_slices` geometry, top-surface polygon clipping, `min_width_top_surface`, `surface.extra_perimeters`, overhang extra perimeters, raft-layer offsets, support interactions, spiral-vase suppression beyond existing Ares normalization, or full Orca `Surface`/`ExPolygon` classification.

## Acceptance Criteria

- `only_one_wall_top` defaults to `false` in `PerimeterOptions`.
- Explicit `only_one_wall_top: true` parses into `PerimeterOptions`.
- Non-boolean `only_one_wall_top` values return `SliceError::InvalidInput` and name the option.
- Legacy `top_one_wall_type` string values other than `none` deserialize to `only_one_wall_top: true` and are accepted by `perimeter_options()`.
- Legacy `top_one_wall_type: "none"` and non-string legacy values remain under `top_one_wall_type` and do not enable `only_one_wall_top`.
- With `wall_loops = 3` and three generated rectangular layers, `only_one_wall_top: true` leaves lower layers with three perimeter paths and the topmost layer with one external perimeter path.
- With `only_one_wall_top: false` or a missing value, the topmost layer keeps the configured wall-loop count.
- With `wall_loops = 0`, `only_one_wall_top: true` still emits no perimeter paths.
- On an odd non-top layer, existing `alternate_extra_wall` behavior still adds a wall when sparse infill density is positive.
- On an odd topmost layer, `only_one_wall_top` wins after the alternate-extra-wall branch and emits one external wall.
- Pipeline/G-code regression proves the topmost layer no longer contains `;PERIMETER:internal:` or `;PRINT_PATH:internal_perimeter:` when `only_one_wall_top: true`, while a lower layer still contains internal perimeter output.
- Existing `only_one_wall_first_layer`, wall sequence, wall direction, alternate extra wall, extrusion, speed, shell-layer, and infill tests continue to pass.
- `docs/roadmap.md` records that `only_one_wall_top` runtime perimeter/G-code behavior is now consumed by this later slice.

## Verification

- Targeted RED tests before production code:
  - `cargo test -p ares-core only_one_wall_top --lib`
- Targeted GREEN tests after implementation:
  - `cargo test -p ares-core only_one_wall_top --lib`
- Final verification:
  - `cargo test -p ares-core --lib`
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `git diff --check`
  - `find crates -name '*.rs' ! -path '*/target/*' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 400 { print; found = 1 } END { exit found }'`
