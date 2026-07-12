# Consume Brim EFC Outline Design

## Goal

Consume the already-registered `brim_use_efc_outline` option into concrete first-layer brim geometry so it no longer remains metadata-only.

## Upstream Boundary

This is a source-cited Rust rewrite slice of OrcaSlicer brim outline selection:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:922` declares `PrintObjectConfig::brim_use_efc_outline`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:929-930` declares the `elefant_foot_compensation` and `elefant_foot_compensation_layers` fields used by the runtime gate.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:943` declares `raft_layers`, which disables the EFC brim outline path when nonzero.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1648-1656` defines the `brim_use_efc_outline` option metadata and default.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:717-735` defines the elephant-foot compensation values used by the gate.
- `OrcaSlicer/src/libslic3r/Brim.cpp:55-62` implements `use_brim_efc_outline(...)`: the option is active only when `brim_use_efc_outline` is true, elephant-foot compensation is positive, elephant-foot compensation layers are positive, and `raft_layers` is zero.
- `OrcaSlicer/src/libslic3r/Brim.cpp:461-468` selects the post-EFC bottom outline for brim base slices when that gate is active.
- `OrcaSlicer/src/libslic3r/Brim.cpp:532-545` generates the outer brim area from the selected base outline.
- `OrcaSlicer/src/libslic3r/Brim.cpp:896-899` uses the same EFC outline selection for brim bounding-box accounting.

Historical Ares M44 already registered `brim_use_efc_outline` and explicitly deferred runtime behavior. This slice moves one bounded part of that deferred behavior into the current Ares brim scaffold.

## Current Ares State

Ares already:

- Registers `brim_use_efc_outline`, `elefant_foot_compensation`, `elefant_foot_compensation_layers`, and `raft_layers`.
- Parses `brim_width`, `brim_object_gap`, `brim_type`, `combine_brims`, and brim-ear tuning into `BrimOptions`.
- Generates first-layer outer rectangular brim loops from contour bounds in `crates/ares-core/src/brims.rs`.
- Generates combined outer brim envelopes in `crates/ares-core/src/brims/combine.rs`.
- Routes brim paths through print paths, extrusion, speeds, diagnostics, and G-code.

Ares does not yet parse `brim_use_efc_outline` into `BrimOptions`, and current brim base geometry is always the raw first-layer contour bounds.

## Ares Destination Boundary

Implement a conservative rectangle-scaffold consumption of `brim_use_efc_outline` inside the existing `ares-core` brim path generation:

- `SliceOptions::brim_options()` reads `brim_use_efc_outline`, `elefant_foot_compensation`, `elefant_foot_compensation_layers`, and `raft_layers`.
- `BrimOptions` stores the active EFC outline offset as `Some(elefant_foot_compensation)` only when the Orca gate from `Brim.cpp:55-62` is satisfied; otherwise it stores no EFC outline offset.
- Outer brim generation uses the EFC outline offset by shrinking the current rectangular outer contour bounds before applying `brim_object_gap` and brim loop offsets.
- Combined outer brim generation uses the same active offset before computing the combined envelope.
- If the active offset collapses a rectangular bounds, that bounds contributes no outer brim path.
- The behavior stays first-layer-only through the existing `generate_brims(...)` layer gate.

For example, a rectangular contour `(0,0)..(4,4)` with `line_width = 0.4`, `brim_width = 0.4`, `brim_object_gap = 0`, `brim_use_efc_outline = true`, `elefant_foot_compensation = 0.2`, `elefant_foot_compensation_layers = 1`, and `raft_layers = 0` emits the first outer brim loop around the compensated base `(.2,.2)..(3.8,3.8)`: `(-0.2,-0.2)..(4.2,4.2)` instead of the raw-outline loop `(-0.4,-0.4)..(4.4,4.4)`.

## Included Behavior

- `brim_use_efc_outline = true` changes generated outer brim geometry only when the Orca gate is satisfied.
- `brim_use_efc_outline = false`, missing, or gated off by zero compensation, zero compensation layers, or nonzero raft layers preserves current raw-outline brim geometry.
- The changed geometry reaches pipeline diagnostics, print paths, extrusion moves, speed moves, and G-code through existing paths.
- Existing `brim_width`, `brim_object_gap`, `brim_type`, `combine_brims`, and brim flow behavior remains intact.
- No new option metadata milestone files are added.

## Deferred Behavior

This slice does not implement:

- Full Orca first-layer EFC surface generation or layer-region surface mutation.
- General polygon offset, clipping, ExPolygon hole handling, or support brim EFC behavior.
- Inner-hole EFC outline behavior; current inner brim paths stay based on raw hole bounds.
- Brim-ear EFC snapping and painted-ear matching from `Brim.cpp:354-420`.
- Auto-brim adhesion heuristics, per-volume-group intersection, support brims, multi-object object maps, or by-object scheduling.
- `elefant_foot_compensation` effects outside brim outline selection.
- New crates, dependencies, filesystem, UI, OpenGL, terminal, or native-only behavior.

## File Size Constraints

- Keep all touched Rust files at or below 400 LOC.
- `crates/ares-core/src/options.rs`, `crates/ares-core/src/options/tests.rs`, `crates/ares-core/src/options/tests/core.rs`, `crates/ares-core/src/brims.rs`, and `crates/ares-core/src/brims/tests.rs` are already close to the limit. Put new logic and tests in focused child modules where needed.
- Parent module changes should be limited to module registration and small call-site wiring.

## Test Strategy

- Add a focused options test proving the Orca gate parses into active/inactive `BrimOptions` states.
- Add a focused brim geometry test proving active EFC outline shrinks the outer brim base before loop expansion.
- Add a focused brim geometry test proving gated-off EFC inputs preserve raw-outline geometry.
- Add a combined-brim test proving combined outer envelopes use the same active EFC outline bounds.
- Add a pipeline/G-code regression proving the changed brim geometry reaches downstream output.
- Run focused RED/GREEN tests, then full `ares-core` and workspace verification.

## Acceptance Criteria

- `brim_use_efc_outline` affects concrete generated brim paths under the same gate as Orca `Brim.cpp:55-62`.
- Disabled or gated-off cases preserve existing brim geometry.
- The behavior is source-cited to `PrintConfig.hpp`, `PrintConfig.cpp`, and `Brim.cpp`.
- No additional option metadata milestone is added.
- All touched Rust source files stay at or below 400 LOC.
- `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass before commit.
