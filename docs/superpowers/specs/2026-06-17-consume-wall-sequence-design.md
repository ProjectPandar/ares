# Consume Wall Sequence Design

## Goal

Consume Orca `wall_sequence` as concrete perimeter print-order behavior in Ares instead of leaving it as registry/metadata-only option coverage.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:131-137` defines `enum class WallSequence` with `InnerOuter`, `OuterInner`, `InnerOuterInner`, and `Count`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1209` declares the `PrintRegionConfig` `wall_sequence` option tuple.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:279-285` maps enum keys `"inner wall/outer wall"`, `"outer wall/inner wall"`, and `"inner-outer-inner wall"` to `WallSequence`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2070-2091` defines `wall_sequence` as `coEnum`, exposes the three values, and defaults to `InnerOuter`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7946-7958` migrates legacy `wall_infill_order` values into `wall_sequence`; Ares already ports this ingestion behavior.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1458-1468` reverses perimeter entity order for `OuterInner`.
- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1469-1549` implements `InnerOuterInner` sandwich ordering after the first layer.

## Current Ares State

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs` registers `wall_sequence` metadata with default `"inner wall/outer wall"`.
- `crates/ares-core/src/options/legacy.rs` normalizes legacy `wall_infill_order` into `wall_sequence`.
- `SliceOptions::perimeter_options()` does not parse `wall_sequence`.
- `PerimeterOptions` carries wall-loop count, perimeter widths, and wall direction, but no wall sequence.
- `generate_perimeters()` currently appends each contour's external path first, then rectangular internal paths from outside to inside.

## Design

Add a runtime `WallSequence` enum in the perimeter domain with `InnerOuter`, `OuterInner`, and `InnerOuterInner` variants. Parse Orca option values from `SliceOptions::perimeter_options()`, defaulting to `InnerOuter`.

`PerimeterOptions` will carry the parsed wall sequence. `generate_perimeters()` will apply wall sequence to the current Ares per-contour perimeter path group before returning paths:

- `InnerOuter`: emit internal walls first and the external wall last. This matches Orca's default intent and fixes the current Ares order for multi-wall rectangular contours.
- `OuterInner`: emit the external wall first, then internal walls from outside to inside.
- `InnerOuterInner`: on layer 0, use `InnerOuter`; on later layers with at least three walls, emit second-and-deeper internal walls from inside to outside, then the external wall, then the first internal wall. On later layers with fewer than three walls, fall back to `OuterInner`, matching the upstream branch that requires at least three walls to form the sandwich.

This ports the wall-order behavior that Ares can represent today with flat contour path groups and role metadata. It does not invent new island, hole, or Arachne behavior beyond the current model.

## Deferred Upstream Behavior

- Per-island grouping across multiple external loops remains deferred because current Ares perimeter output is a flat path list without island identifiers.
- Hole-aware ordering remains deferred because current `LayerContours` does not distinguish contour loops from hole loops.
- Arachne extrusion-line ordering remains deferred because Ares does not yet model Orca Arachne perimeter generation.
- Brim-driven first-layer outer-wall-first behavior remains deferred because current Ares brim generation is separate from perimeter generation and does not carry object-config context into `generate_perimeters()`.
- `precise_outer_wall` interactions remain deferred, including the upstream note that precise-wall behavior is ignored for `OuterInner` / `InnerOuterInner` wall sequences and the corresponding perimeter-generation spacing behavior.
- Exact Orca nested-entity traversal, `inset_idx`-based multi-island sandwich reordering, and thin-wall-hole special ordering remain deferred until the corresponding upstream data structures are ported.

## Acceptance Criteria

- `SliceOptions::perimeter_options()` returns default `WallSequence::InnerOuter` when `wall_sequence` is absent.
- `wall_sequence: "inner wall/outer wall"` emits internal perimeter paths before the external path for rectangular multi-wall contours.
- `wall_sequence: "outer wall/inner wall"` emits the external path before internal paths.
- `wall_sequence: "inner-outer-inner wall"` uses `InnerOuter` behavior on layer 0.
- `wall_sequence: "inner-outer-inner wall"` on later layers with three walls emits the deepest internal path first, then the external path, then the first internal path.
- Existing legacy `wall_infill_order` migrations still parse into the expected runtime wall sequence.
- Invalid non-legacy `wall_sequence` values return `SliceError::InvalidInput` at the perimeter option boundary.
- A pipeline/G-code regression proves JSON `wall_sequence` changes emitted `;PERIMETER:` / `;PRINT_PATH:` path order for a rectangular fixture.
- Rust source files touched by this slice stay below the repository 400 LOC split threshold.

## Files

- Modify `crates/ares-core/src/perimeters.rs` for `WallSequence`, `PerimeterOptions`, and path ordering.
- Modify `crates/ares-core/src/perimeters/tests.rs` for perimeter-level sequence tests.
- Add `crates/ares-core/src/options/wall_sequence.rs` for option parsing.
- Add `crates/ares-core/src/options/tests/wall_sequence.rs` for parsing tests.
- Modify `crates/ares-core/src/options.rs` only to include the new module and call the parser from `perimeter_options()`.
- Modify `crates/ares-core/src/lib.rs` only to re-export `WallSequence` alongside the existing perimeter API.
- Modify `crates/ares-core/src/options/tests.rs` only to declare the new focused test module and preserve the 400 LOC limit by moving the existing `wall_direction` focused test declaration into the existing `#[rustfmt::skip] option_test_modules!(...)` macro before adding a standalone `mod wall_sequence;` line near the focused option-consumption tests.
- Add `crates/ares-core/src/pipeline/tests/wall_sequence.rs` for the G-code regression, and add only the module declaration to `crates/ares-core/src/pipeline/tests.rs`.

## Docs Impact

No architecture or roadmap documentation update is required for this narrow runtime consumption slice. The source-cited design document, implementation plan, and regression tests are the durable documentation for the behavior added here.

## Verification

- `cargo fmt --check`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `wc -l` on touched Rust files to confirm none exceed 400 LOC
