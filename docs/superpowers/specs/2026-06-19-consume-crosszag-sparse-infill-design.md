# Consume CrossZag Sparse Infill Design

## Goal

Consume the existing OrcaSlicer `sparse_infill_pattern = "crosszag"` and `infill_shift_step` option boundary in concrete Ares sparse-infill generation. This slice must generate sparse infill paths and downstream G-code for `crosszag` instead of adding more option metadata or treating the value as a parser-only milestone.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:87-98` declares `InfillPattern`, including `ipZigZag`, `ipCrossZag`, and `ipLockedZag`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:225-254` maps `"crosszag"` to `ipCrossZag` and `"lockedzag"` to `ipLockedZag`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2928-2985` registers `sparse_infill_pattern`, includes `crosszag` and `lockedzag`, and defaults sparse infill to `ipCrossHatch`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1099` declares `ConfigOptionFloat infill_shift_step` on `PrintRegionConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3862-3870` registers `infill_shift_step`, constrains it to `0..10`, and defaults it to `0.4` mm.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:40-76` dispatches `ipCrossZag` to `FillCrossZag` and `ipLockedZag` to `FillLockedZag`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:877-885` forwards `symmetric_infill_y_axis` for `ipCrossZag`, `ipLockedZag`, and `ipZigZag`; it forwards `infill_lock_depth` and `skin_infill_depth` only for `ipLockedZag`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1298-1314` marks `ipLockedZag` as locked, applies `infill_shift_step` to `params.horiz_move` for `ipCrossZag` and `ipLockedZag`, and forwards `symmetric_infill_y_axis` for `ipCrossZag`, `ipLockedZag`, and `ipZigZag`.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.hpp:192-208` defines `FillZigZag` and `FillCrossZag` as `FillRectilinear` subclasses with `has_consistent_pattern() == true`.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:2751-2817` generates rectilinear-family line fill and applies `params.horiz_move` by shifting the scanline origin modulo line spacing.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:3391-3393` sends `ipCrossZag`, `ipZigZag`, and `ipLockedZag` through the single-line `fill_surface_by_lines` path rather than multiline fill.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:3866-3944` shows `FillLockedZag` has separate skin/skeleton lock-region behavior driven by `skin_infill_depth`, `infill_lock_depth`, and multi-width flow partitioning.

## Current Ares State

- `crates/ares-core/src/options/infill/patterns.rs` currently recognizes `crosszag` and `lockedzag` only to reject them as known unimplemented sparse infill patterns.
- `crates/ares-core/src/options/infill.rs` has no `InfillPattern::CrossZag` variant and no runtime `infill_shift_step` storage.
- `crates/ares-core/src/infills/rotation.rs` already models pattern-dependent single-pass vs grid-pass behavior, fixed layer angle, and zigzag segment alternation for supported Ares patterns.
- `crates/ares-core/src/infills/symmetry.rs` currently applies `symmetric_infill_y_axis` only to `InfillPattern::ZigZag`.
- `crates/ares-core/src/infills.rs` owns scanline clipping and is already at the repository split threshold, so implementation must move scanline candidate generation to a focused child module before adding new behavior.

## Ares Destination Boundary

Implement the smallest source-cited Rust slice that makes `crosszag` a real sparse-infill behavior:

- Add `InfillPattern::CrossZag` to `ares-core`.
- Parse `"crosszag"` as `InfillPattern::CrossZag` for `sparse_infill_pattern`.
- Keep `"lockedzag"` rejected as known unimplemented until the lock-region source boundary is implemented.
- Parse `infill_shift_step` as a finite float in millimeters with Orca's default `0.4` and range `0.0..=10.0`.
- Store `infill_shift_step` on `InfillOptions` with crate-visible access for infill generation and focused tests.
- Treat `CrossZag` as a rectilinear-family single-pass line pattern with fixed layer angle and alternating segment direction, matching the upstream `FillCrossZag` inheritance shape and single-line dispatch.
- Apply `infill_shift_step` only to `CrossZag` sparse infill scanline placement. The Ares shift must affect the scanline origin before clipping, not post-translate finished paths.
- Use the same layer parity shape as upstream: layer 0 has no shift, layer 1 shifts by `+infill_shift_step * (layer_id / 2)`, layer 2 shifts by `-infill_shift_step * (layer_id / 2)`, layer 3 shifts by `+infill_shift_step * (layer_id / 2)`, using integer division on the Ares layer id.
- Continue to apply `symmetric_infill_y_axis` for `ZigZag`; extend the existing temporary Ares mirror boundary to `CrossZag`, because upstream forwards the same option for `ipCrossZag`.
- Preserve all existing `rectilinear`, `alignedrectilinear`, `line`, `grid`, `zigzag`, and `crosshatch` sparse infill output unless the new `crosszag` value is selected.
- Preserve platform neutrality in `ares-core`: no filesystem, terminal, UI, OpenGL, native-only APIs, or new dependencies.

## Explicitly Deferred

- `lockedzag` parser acceptance and generated behavior.
- `FillLockedZag::fill_surface_locked_zag`, including `skin_infill_depth`, `infill_lock_depth`, skeleton/skin density partitioning, flow partitioning, and multi-width output.
- Full upstream connected-polyline graph traversal, monotonic region chaining, object-level `extended_object_bounding_box()` plumbing, overlap/spacing adjustment parity, bridge handling, multi-surface region behavior, travel optimization, and gap-fill fallback.
- Advanced `sparse_infill_rotate_template` syntax. This slice preserves Ares' current plain comma-separated template support.
- Runtime consumption of `infill_shift_step` for any pattern other than `crosszag`.
- Any new option milestone, new crate, dependency, UI behavior, terminal behavior, filesystem behavior, OpenGL/viewer behavior, or independent Ares-owned slicing pipeline design.

## Design

Split scanline candidate generation out of `crates/ares-core/src/infills.rs` into a focused child module before implementation work. The child module should own transformed points, candidate records, point/segment ordering, contour clipping, and anchored segment construction. The public behavior remains internal to `infills`; this is a size-control split, not a new abstraction layer for future patterns.

Extend `InfillPasses` so each sparse fill pass carries a scanline shift in millimeters. For non-`CrossZag` patterns the shift is `0.0`. For `CrossZag`, compute the shift from the actual Ares layer id and `infill_shift_step`, then pass it into scanline clipping. The clipping loop should start from the existing spacing-aligned scanline plus a normalized modulo shift, so changed paths are clipped from the shifted scanline lattice.

Use existing Ares zigzag-compatible segment alternation for `CrossZag`. This is not claiming full Orca graph traversal parity; it consumes the source boundary Ares already models for rectilinear-family single-line fill and the unique `crosszag` shift option.

## Tests

Use TDD with focused RED/GREEN tests before implementation:

- Option/runtime tests:
  - `"crosszag"` parses to `InfillPattern::CrossZag`.
  - `"lockedzag"` remains rejected with the existing known-unimplemented message.
  - default `infill_shift_step` is `0.4`.
  - explicit `infill_shift_step` values inside `0.0..=10.0` parse.
  - non-numeric, `NaN`, negative, and greater-than-10 `infill_shift_step` values fail through `SliceOptions::infill_options()`.
- Infill unit tests:
  - `CrossZag` layer 0 generates zigzag-compatible alternating sparse scanlines with no shift.
  - `CrossZag` layer 2 with `infill_shift_step = 0.25` shifts the scanline lattice relative to `ZigZag`.
  - `symmetric_infill_y_axis = true` affects `CrossZag` paths through the existing mirror boundary.
  - non-`CrossZag` sparse patterns keep their previous path coordinates when `infill_shift_step` is configured.
- Pipeline/G-code tests:
  - `sparse_infill_pattern = "crosszag"` reaches `LayerInfills`, `PrintPathRole::SparseInfill`, and emitted `;INFILL:sparse:` / `;PRINT_PATH:sparse_infill:` / sparse extrusion comments.
  - `crosszag` with `infill_shift_step` changes the expected layer-2 sparse infill G-code comments.

## Acceptance Criteria

1. Ares no longer rejects `sparse_infill_pattern = "crosszag"`.
2. Ares still rejects `sparse_infill_pattern = "lockedzag"` as known unimplemented, with scope clearly documented.
3. `crosszag` produces concrete sparse infill paths, print paths, extrusion moves, and G-code comments.
4. `infill_shift_step` is consumed by `crosszag` sparse infill scanline generation and does not affect other currently supported sparse patterns.
5. `symmetric_infill_y_axis` applies to both `zigzag` and `crosszag`.
6. Existing sparse infill pattern behavior and tests remain stable for all previously supported values.
7. All touched Rust source files stay at or below 400 LOC.
8. No new dependencies, crates, platform-specific behavior, or option-only milestones are introduced.

## Verification

- Targeted RED/GREEN tests for parser, infill unit behavior, and pipeline/G-code behavior.
- `cargo test -p ares-core --lib sparse_infill_pattern`
- `cargo test -p ares-core --lib infills`
- `cargo test -p ares-core --lib symmetric_infill_y_axis`
- `cargo test -p ares-core --lib`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- Rust LOC gate for files under `crates/`.

## SDD Gates

- Do not write implementation code until this spec/design and the implementation plan both receive independent reviewer `VERDICT: APPROVE`.
- After implementation, dispatch an independent implementation reviewer with the spec, reviewed plan, diff, and verification output. Commit and push only after that reviewer returns `VERDICT: APPROVE`.

## Documentation Impact

This spec and the implementation plan are the primary documentation artifacts for the slice. Update `docs/roadmap.md` after implementation to record that `crosszag` and `infill_shift_step` now have narrow sparse-infill runtime consumption while `lockedzag` and lock-region behavior remain deferred.
