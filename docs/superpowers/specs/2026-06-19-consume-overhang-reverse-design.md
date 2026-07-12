# Consume Overhang Reverse Design

## Goal

Consume the existing `overhang_reverse` option in concrete Ares perimeter/G-code behavior. This is a source-cited Rust rewrite slice of OrcaSlicer overhang perimeter reversal, not another option metadata milestone.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1205` declares `ConfigOptionBool overhang_reverse`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1446-1453` defines the option metadata and default `false`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:108-109` and `374-375` gate reversal detection on `config->overhang_reverse && layer_id % 2 == 1`, described there as "even (from GUI POV) layers".
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1445-1455` and `2498-2506` call `reorient_perimeters` only when `overhang_reverse` is enabled.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1117-1141` reverses eligible perimeter loops.

## Ares Destination Boundary

- `crates/ares-core/src/options.rs` must parse the existing `overhang_reverse` boolean into `PerimeterOptions`.
- `crates/ares-core/src/perimeters.rs` and a new focused `crates/ares-core/src/perimeters/overhang_reverse.rs` module must apply that option to generated perimeter path orientation.
- Existing pipeline/G-code stages must observe the changed path point order through the already-generated `LayerPerimeters`, `LayerPrintPaths`, toolpath moves, and G-code output.
- Existing option registry metadata remains the source for key/default lookup; this slice does not add new option definitions.

## Included Behavior

1. Default behavior remains unchanged: missing `overhang_reverse` defaults to `false`.
2. `SliceOptions::perimeter_options()` rejects non-boolean `overhang_reverse` values with the existing boolean option error style.
3. `PerimeterOptions` exposes `overhang_reverse()`.
4. When `overhang_reverse == true`, Ares reverses the point order for a perimeter path only when all of these are true:
   - the generated path role is `PerimeterRole::Overhang`;
   - the layer id satisfies `layer_id % 2 == 1`, matching the Orca "even from GUI POV" gate for zero-based layer ids;
   - the path is already classified as an overhang by Ares' existing `detect_overhang_wall` rectangular support check.
5. Reversal composes after the configured base `wall_direction`: `wall_direction` first orients the path, then `overhang_reverse` flips eligible overhang paths on gated layers.
6. Wall sequence ordering remains unchanged. Only the eligible path's point order changes.
7. G-code emitted from the pipeline changes movement order for the gated unsupported overhang perimeter when the option is enabled.

## Deferred Behavior

- `overhang_reverse_internal_only` remains registry metadata only in this slice. Ares' current rectangular perimeter scaffold does not yet model Orca loop roles well enough to distinguish internal contour/hole behavior for this option.
- `overhang_reverse_threshold` remains registry metadata only. Ares does not yet implement Orca's `detect_steep_overhang` distance-threshold logic from `PerimeterGenerator.cpp:58-95`.
- `counterbore_hole_bridging` remains registry metadata only.
- The Orca branch that reverses every eligible loop when `detect_overhang_wall` is disabled is deferred. This slice is tied to Ares' existing overhang classification and must not invent a second unclassified reversal mode.
- Fuzzy skin special reversal, Arachne extrusion reversal, thin walls, holes, support, raft-layer gates, partial polyline clipping, and full `reorient_perimeters` parity remain deferred.

## Tests

- Options test: default `overhang_reverse` is false, true parses to `PerimeterOptions`, and non-boolean values fail.
- Perimeter test: an unsupported second layer rectangle with `overhang_reverse=false` keeps the existing point order; with `overhang_reverse=true` on layer id 1 it is reversed.
- Perimeter test: a supported second layer rectangle is not reversed even when `overhang_reverse=true`.
- Perimeter test: an unsupported layer id 2 rectangle is not reversed because it does not satisfy the odd zero-based layer gate.
- Pipeline/G-code test: enabling `overhang_reverse` changes the first overhang perimeter movement coordinate order in formatted G-code for the existing unsupported second-layer fixture.

## Acceptance Criteria

- This change consumes existing metadata into concrete slicing behavior, not new option-only registry work.
- Rust files under `crates/` remain at or below 400 LOC.
- The implementation is platform-neutral and does not add file I/O, terminal, UI, OpenGL, native viewer runtime, or non-WASM-safe behavior to `ares-core`.
- The M42 milestone documentation and `docs/roadmap.md` are updated to state that the first concrete `overhang_reverse` runtime slice is implemented while the deferred behaviors above remain deferred.
- Verification passes with `cargo fmt --check`, focused tests, `cargo test -p ares-core --lib`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the Rust LOC gate.
