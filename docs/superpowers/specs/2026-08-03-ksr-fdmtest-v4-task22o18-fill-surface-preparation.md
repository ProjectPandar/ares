# Task 22O.18 — Fill-surface shell preparation Spec

## Status

Implemented after approved independent/OpenCode specification and plan reviews. The KSR O18 checksum remains `-126362407653399901571400348049652748978`; totals are `[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 5388, 519, 6, 666, 4197, 1294, 113, 6, 48, 1127, 5388, 517, 85886, 1294, 168, 46011, 0, 0]`. Seventeen focused O18 tests, 209 O10-O18 regressions, and 5,607 workspace tests with 2 skipped pass together with workspace/native check, strict Clippy, both WASM checks, formatting, diff, LOC, forbidden-pattern, dependency, pinning-removal, and staging audits. The final independent six-dimensional implementation rereview and OpenCode rereview both returned `VERDICT: APPROVE`.

## Upstream source boundary

Pinned upstream: OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This milestone rewrites exactly the slicing-state mutation in:

- the caller loop at `OrcaSlicer/src/libslic3r/PrintObject.cpp:587-592`;
- `LayerRegion::prepare_fill_surfaces` at `OrcaSlicer/src/libslic3r/LayerRegion.cpp:935-973`;
- the directly reached `SurfaceType` values and predicates in `OrcaSlicer/src/libslic3r/Surface.hpp:8-114`;
- `EPSILON = 1e-4` from `OrcaSlicer/src/libslic3r/libslic3r.h:52`.

The Rust destination is a crate-private successor after `PreparedPostSurfaceTypeDetection`. The exact stop is after every populated layer-region record has run the equivalent of `prepare_fill_surfaces`, before `PrintObject::discover_vertical_shells` at `PrintObject.cpp:595-596`.

## Option and lifecycle envelope

All decisions use each record's already typed, resolved `RegionOptions`, reached through the aligned `PostPerimeterInputPrintObject::region_options(record)` path. Do not parse raw settings again or use object-level defaults as a replacement for the record's region configuration.

Layer-config ranges remain rejected before O18. Before O18 implementation, strengthen the temporary project capability boundary to reject the typed global print option `resolved.views.full.process.print.spiral_mode` before O17. The current record-local Classic check is insufficient because bottom-shell thresholds can leave every `PerimeterInputRecord::spiral_mode` false while the global mode remains true; such a predecessor would require O17 spiral detection/corrections that O17 explicitly deferred. O18 does not widen spiral support.

The direct O18 record helper still preserves the pinned source guards exactly: pass 2 (`bottom_shell_layers`) is unconditional, while passes 1 and 3 are guarded by `!spiral_mode`. Its orchestrator reads the exact global print-option path above, never `PerimeterInputRecord::spiral_mode`; public preparation can only receive `false` after capability validation.

In the pinned source `PrintObject::infill_only_where_needed` is defined as the static value `false` in `PrintObjectSlice.cpp:22` and is never assigned elsewhere under `OrcaSlicer/src`; consequently zero top shells retag to `Internal`, not `InternalVoid`. This is pinned source behavior, not an Ares option or fallback.

The KSR 3MF resolves `top_shell_layers = 5`, `bottom_shell_layers = 3`, and `sparse_infill_density = 15%` for every reached record, so the literal KSR O18 mutation is inactive. O18 must still execute exactly once and preserve the O17 checksum/totals. Real-3MF option mutations provide active-path evidence.

## Included behavior

For each populated record, mutate only `fill_surfaces`, in its existing order, with the exact three sequential source passes:

1. If `!spiral_mode && top_shell_layers == 0`, retag every `Top` fill surface to `Internal`.
2. If `bottom_shell_layers == 0`, retag every `Bottom` and `BottomBridge` fill surface to `Internal`, including reachable spiral mode.
3. If `!spiral_mode && abs(sparse_infill_density - 100.0) < 1e-4`, retag every `Internal` fill surface to `InternalSolid`.

The order is observable: with zero shell counts and approximately 100% density, formerly top and bottom surfaces become `InternalSolid` in the third pass. The comparison is strict `<`, uses typed `f64` percent values, and preserves `NaN` behavior from the ordinary comparison (although typed project validation does not produce NaN).

Add `InternalSolid = 5` only to `RegionSurfaceKind` in `region_slices.rs`, the kind enum carried by these `fill_surfaces`, and update its exhaustive `is_bridge()` result to `false`. The separate `SurfaceType` in `surface.rs` already has this value and is unchanged. O17 cannot emit `RegionSurfaceKind::InternalSolid`; O18 may emit it only in `fill_surfaces`. `slices` remain exactly the O17 typed slices and are never retagged here. `InternalVoid` and the remaining later surface kinds stay deferred.

Retag in place. Preserve object/record/slot order, `None` slots, fill-surface vector storage, surface order, every expolygon/contour/hole/point allocation, and all thickness/layer/bridge-angle/extra-perimeter metadata. Move the exact O17 boxed predecessor, perimeters, thin fills, typed slices, fill boundaries, and no-overlap boundaries unchanged. No clipping, sorting, allocation, geometry operation, or error is introduced.

O18 is infallible after O17 succeeds. Trusted positional alignment is checked before the mutation; O18 itself adds no user-facing error or preflight. The prerequisite global spiral capability repair returns the existing `UnsupportedProjectFeature("spiral_mode")` before O17. Public slicing runs O18 exactly once after O17 and continues to return `SliceError::ProjectSlicingIncomplete` after iterative disposal. Earlier capability/Classic errors and every O17 preflight or geometry error must leave the O18 invocation count at zero.

## Explicitly deferred

- Public global spiral slicing, rejected by the strengthened early capability boundary because O17 spiral detection/corrections remain deferred. Exact spiral guards are covered only by the direct O18 transformation test.
- Layer-config ranges, already rejected earlier.
- Caller cancellation checks, status/logging, and debug SVG exports: these are native/UI infrastructure outside the platform-neutral byte API, and no cancellation token or debug-export sink exists in `ares-core`.
- `InternalVoid`, because the pinned static `infill_only_where_needed` remains false.
- Tiny-surface solidification mentioned by the stale caller comment but absent from the executable `LayerRegion::prepare_fill_surfaces` body.
- `discover_vertical_shells`, `discover_horizontal_shells`, `process_external_surfaces`, fill clipping/combination/generation, thin-fill transfer, seams, ordering, motion, G-code, and post-processing.
- Any fixture identity branch, reference-G-code replay/read, source-text/hash/line pin, Orca runtime/FFI, or legacy fallback.

## State and ownership

Add `prepare_infill::fill_surfaces` with a named `PreparedPostFillSurfacePreparation` successor. It owns the exact boxed traversal predecessor and O17 object records after in-place fill-kind mutation; it must not keep a stale nested O17 wrapper around already-mutated data. Existing O17 record structs may be moved unchanged because only their private fill-surface payload is retagged.

No public API, persisted format, dependency, migration, or compatibility layer is added.

## Tests and acceptance criteria

1. Direct tests pin `InternalSolid = 5`, bridge classification, all three individual passes, unaffected kinds, strict epsilon behavior, and sequential composition.
2. Direct ownership tests use surfaces with distinct geometry and metadata to prove only kinds change and all vector/surface geometry allocations, order, and metadata remain identical.
3. Real KSR-derived 3MF mutations independently set top shells to zero, bottom shells to zero, density to 100%, and all three together. Every active case must freeze nonzero literal transition counts and prove only the expected fill kinds change; typed slices, coordinates, metadata, predecessor identity, and all unrelated O17 fields remain unchanged. Use concrete density values `99.99995%` (inside) and `99.9998%` (outside) to distinguish strict epsilon behavior.
4. Option-provenance tests must not mutate only the global process value. A normal-part/model-settings region override leaves the global option unchanged but changes the reached record. A synthetic aligned two-object state combines records whose embedded region options differ, proving orchestration calls `region_options(record)` instead of using a global, first-object, or first-record shortcut.
5. The KSR characterization parses independently twice, first guards the literal O17 checksum/totals, then pins an O18 checksum/totals. Because the active KSR options are 5/3/15%, O18 equals O17 structurally; exact O18 invocation and state type prevent a no-op lifecycle bypass from satisfying the test.
6. Lifecycle tests prove O18 runs once on public KSR slicing, remains incomplete before vertical shells, and is not reached for global spiral mode (including bottom-shell thresholds that make every record-local spiral flag false), earlier counterbore, or O17 interface-shell, active-extra-bridge, or instrumented geometry failures. Direct tests, not public lifecycle, prove the helper skips passes 1 and 3 but executes pass 2 when its spiral operand is true. The existing 64-KiB/10,000-node incomplete cleanup witness is moved through the O18 consumer.
7. Focused O18, O17-O10 regressions, full workspace Nextest, strict all-target/all-feature Clippy, workspace/native check, both WASM checks, rustfmt, diff, LOC, forbidden-pattern, source-pinning, dependency, and staging audits pass. Every Rust source/test file remains below 400 LOC.
8. No `unsafe`, `include!`, `include_bytes!`, binary oracle payload, broad lint allowance, source pinning test, reference-G-code read, fixture identity branch, or new dependency is introduced.
9. Independent specification and plan reviewers approve before implementation. After implementation, an independent read-only reviewer validates requirement completeness, logic, edge cases, code quality, coverage, and actual execution; a separate OpenCode reviewer checks the same final diff. Findings return to the main thread and both reviewers rerun until approval.

## Documentation and rollback

Update `docs/architecture/option-parity-v4.md` and `docs/roadmap.md` with the exact three-pass retag boundary, typed option source, identity guarantees, KSR evidence, and next source boundary `PrintObject::discover_vertical_shells` beginning at `PrintObject.cpp:595`.

Rollback restores the O17 terminal and removes only O18 state, wiring, tests, and documentation while preserving O1-O17 behavior.
