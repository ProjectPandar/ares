# Task 22O.22 — Single-region vertical-shell morphology regularization Spec

## Status

Implemented and locally validated from Ares baseline `7d607b4bcda5ede5d5eb1d5c513148ecf1ab25d4` against pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`. Frozen O22 evidence: checksum `134936948052282121922360252649864225707`, totals `[1, 460, 0, 460, 632, 632, 128, 34557]`, ordered events `[259, 259, 259, 259]`, radii digest `-119839535044106185061007902266478724784`, 11 direct tests, 22 integration tests, 346 O10-O22 regressions, and 5,750 workspace passes with 2 skipped. Both design gates approved before implementation; final implementation reviews and exact-commit Tier-1 remain ship gates.

## Upstream source boundary

This milestone rewrites only the next release-observable block of `PrintObject::discover_vertical_shells`:

- the already-wired caller at `OrcaSlicer/src/libslic3r/PrintObject.cpp:595-596`;
- initial `regularized_shell` construction at `PrintObject.cpp:2344-2367`;
- the aligned `solid_infill_flow`, `infill_line_spacing`, and `min_perimeter_infill_spacing` source established at `PrintObject.cpp:2173-2182` and the `spacing()` / `scaled_spacing()` accessors at `Flow.hpp:60-70`;
- `union_ex`, `offset2_ex`, and `shrink_ex` overload/defaults at `ClipperUtils.hpp:19-34,344-347,384-392,548-553`, including negative-offset shrinking at `ClipperUtils.cpp:368-405`, first-stage `expolygons_offset` and PolyTree offset grouping at `540-589`, two-pass PolyTree union ordering at `634-668`, `_clipper_ex` conversion at `737-739`, and the flat-Paths `union_ex` entry at `813-814`;
- `coord_t` as signed `int64_t` at `libslic3r.h:40-43`;
- Bambu-vendored Clipper 6 Paths/PolyTree execution, Square joins, coordinate validation, and output ordering already rewritten by the current Ares geometry layer.

The exact source order is:

1. `infill_line_spacing = solid_infill_flow.scaled_spacing()`;
2. `min_perimeter_infill_spacing = float(infill_line_spacing) * 1.05f`;
3. compute `min_width_narrow_ensure = 0.5f * 0.65f * min_perimeter_infill_spacing`;
4. compute `min_width_narrow_sparse_infill = 0.5f * 1.2f * min_perimeter_infill_spacing`;
5. compute `min_width_tiny_overlap = 0.2f * min_perimeter_infill_spacing`;
6. `union_ex(shell)` using NonZero fill and PolyTree ExPolygon grouping;
7. `offset2_ex(regularized_shell, -narrow_ensure, narrow_ensure + narrow_sparse, jtSquare)`;
8. `shrink_ex(..., narrow_sparse - tiny_overlap, jtSquare)`.

The Rust destination is a crate-private successor after `PreparedPostVerticalShellTrim`, with an aligned fresh regularization sidecar while retaining the exact O21 predecessor, O18 objects, O19 caches, O20 projections, O21 trims, and all nested allocations. O19-O22 sidecars remain temporary compatibility representations of `PrintObject::discover_vertical_shells`, not an Ares-owned pipeline.

The exact stop is after assignment of the shrunken `regularized_shell` at `PrintObject.cpp:2367`. Stop before `object_volume = ...` and neighbor-volume/tiny-area filtering beginning at `PrintObject.cpp:2369`.

## Active envelope and provenance

O22 retains the reviewed O17-O21 envelope: global spiral is rejected before O17; each object has exactly one compatible region; `interface_shells = false`; active extra-bridge modes remain rejected; only `ensure_vertical_shell_thickness = EnsureAll` can produce a nonempty O21 trim. `None` slots remain aligned `None`. An inactive mode or an O21 empty gate produces an empty O22 regularization without geometry calls.

For every populated record, derive `solid_infill_flow.scaled_spacing()` exclusively from the retained aligned `ClassicPreludeRecord::solid_infill_spacing`, which itself is resolved from typed 3MF region/nozzle/layer flow inputs. Convert the signed scaled coordinate to `f32` exactly once, multiply by `1.05_f32` to form `min_perimeter_infill_spacing`, then preserve the source expression order and `f32` rounding in all three radii. No fixture identity, dimensions, layer count, geometry identity, or reference G-code may select values or behavior.

The later area thresholds and scaled constants are deliberately outside this milestone. No area conversion, `object_volume` selection, neighboring-layer inspection, or `intersection_ex` is included.

## Included behavior

For each populated record, stage a fresh `VerticalShellRegularization { regularized_shell: Vec<ExPolygon> }` in object/slot order:

1. If the aligned O21 trim shell is empty, retain an empty regularization and invoke no O22 geometry.
2. Cast the aligned scaled solid-infill spacing from `i64` to `f32`, then compute `min_perimeter_infill_spacing = spacing * 1.05_f32`. Compute the three source radii with literal left-associated `f32` arithmetic: `0.5_f32 * 0.65_f32 * min_perimeter_infill_spacing`, `0.5_f32 * 1.2_f32 * min_perimeter_infill_spacing`, and `0.2_f32 * min_perimeter_infill_spacing`.
3. Execute `union_ex` on the flat O21 shell with NonZero fill. Preserve input path/point order and existing PolyTree ExPolygon grouping/output order. If this union is empty, still invoke both source offset stages and shrink on the empty intermediate; only the earlier O21 empty-shell `continue` skips the full regularization expression.
4. Execute the source `offset2_ex` with first delta `-min_width_narrow_ensure`, second delta `min_width_narrow_ensure + min_width_narrow_sparse_infill`, `JoinType::Square`, and default miter limit `3.0`. Preserve exact `f32` negation/addition and the two existing offset stages.
5. Execute source `shrink_ex` through Ares's existing `offset_expolygons` with signed delta `-(min_width_narrow_sparse_infill - min_width_tiny_overlap)`, `JoinType::Square`, and default miter limit `3.0`, exactly matching the one-line upstream wrapper. Do not add a second named wrapper. Preserve exact subtraction then negation and the existing offset/PolyTree order.
6. Store the resulting ExPolygons without sorting, canonicalization, union, deduplication, area filtering, or intersection.

Validate complete O21/object/cache/projection/trim/input/prelude/plan/lslice alignment before the first O22 geometry event. Stage the whole project while borrowing O21. Only after all objects/slots succeed may the implementation move the exact O21 state beside fresh regularizations. Any union, either `offset2_ex` stage, or shrink failure returns `SliceError::InvalidInput("vertical-shell regularization geometry is outside the supported Clipper range")`, exposes no successor, and iteratively disposes O21. Earlier capability/O17/O19/O20/O21 errors retain precedence.

Wire public slicing through O22 exactly once and continue returning `ProjectSlicingIncomplete`.

## Explicitly deferred

- `object_volume`, neighbor-volume accumulation, tiny-island/area filtering, and `intersection_ex` at `PrintObject.cpp:2369-2415`;
- `InternalVoid` production, multi-region/all-material projection, `interface_shells = true`, and spiral shortened layer count;
- mutation/rebuilding of `fill_surfaces` at `PrintObject.cpp:2417-2432`;
- cancellation, TBB scheduling, logging, profiling, debug SVG, and disabled debug/no-op blocks;
- horizontal shells, external surfaces, fill generation, seams, ordering, motion, G-code, and post-processing;
- reference-G-code reads/replay, fixture identity/name/hash/layer-count/geometry branches, Orca runtime/FFI, legacy fallback, or hard-coded fixture output.

## Tests and acceptance

1. Direct geometry tests freeze NonZero `union_ex`, Square `offset2_ex`, and Square shrink composition with literal disjoint, touching, holed, narrow, and near-gap witnesses. Tests freeze exact path/point/ExPolygon order, empty-intermediate propagation through all remaining calls, and independent coordinate failures at union, each offset2 stage, and shrink.
2. Direct record tests freeze exact `f32` radius bits for ordinary, odd, and large-but-supported positive spacings; exact source operation order; the O21 empty-trim continue and empty-union propagation through the remaining calls; Square rather than Miter/Round behavior; morphology that removes too-narrow material, closes a near gap, then erodes the outer overlap; and fresh nonaliasing output.
3. Test-only hooks independently fail union, offset2 first, offset2 second, and shrink. Whole-project tests prove stable error text, no partial successor, later-object/slot failure after earlier successful geometry, whole-project stage-before-move, and iterative success/error/public-incomplete cleanup with both predecessor tree families at depth 10,000 on the shared constrained-stack baseline.
4. Alignment and recursive ownership tests cover every O21 outer/object/record/slot/count/source/transform/region/compatibility/layer/current identity and retain exact predecessor allocation identity/content. New regularization ExPolygons/paths/points must be allocation-distinct from every O21 and earlier geometry buffer.
5. Real-3MF tests prove active and inactive ensure behavior, model-part precedence, ZIP repack/non-slicing rename invariance, and typed 3MF mutations that change solid-infill spacing and therefore exact radius/output evidence without fixture-specific branching.
6. KSR parses independently twice, guards O19/O20/O21 frozen parent evidence, then freezes parent-bound O22 checksum/totals/events over objects, slots/`None`, regularized ExPolygons, contours, holes, points, coordinates, and exact radii. Tests never read reference G-code.
7. Focused O22, O10-O22 regressions, workspace Nextest, strict Clippy, native all-target, default and feature-enabled Tier-1 WASM checks, formatting/diff, all Rust files `<400 LOC`, every new O22 shard `<=300 LOC`, dependency, source-pinning, and staging audits pass. Every new O22 Rust file must contain no `unsafe`, `include!`, or `include_bytes!`; broad lint allowances, reference-G-code access, fixture identity/hash/layer/geometry branches, Orca runtime/FFI, and fallback are also forbidden. Every test root and shard is a real Rust file connected through ordinary `mod` declarations.
8. Independent spec and plan reviewers plus separate default-model OpenCode reviews must return literal `VERDICT: APPROVE` before implementation. After implementation, an independent six-dimensional reviewer and OpenCode reviewer inspect the same final diff/evidence. The main thread fixes findings and repeats both reviews until approval.

## Frozen implementation evidence

Two independent KSR parses first reassert O19 checksum/totals, O20 checksum/totals/events, and O21 checksum/totals/events before freezing O22 checksum `134936948052282121922360252649864225707`, totals `[1, 460, 0, 460, 632, 632, 128, 34557]`, ordered events `[259, 259, 259, 259]`, and exact-radii digest `-119839535044106185061007902266478724784`. The final direct filter passes 11 tests, the integration filter passes 22 tests, the explicit O10-O22 regression filter passes 346 tests, and the workspace passes 5,750 tests with 2 skipped. Strict all-target/all-feature Clippy passes.

Because the initial implementation worker could not execute commands, the preserved compiling RED evidence is explicitly post-implementation mutation evidence, not falsely presented as chronological pre-implementation RED. Removing only the production `* 1.05_f32` factor fails 4 of the same 11 direct tests at `/tmp/task22o22-red-direct.txt` and 2 of the same 22 integration tests at `/tmp/task22o22-red-integration.txt`. Separate compiling production mutations prove the remaining integration contracts: removing pre-geometry alignment fails all 5 mismatch tests at `/tmp/task22o22-red-integration-alignment.txt`; restoring O21 terminal consumption fails the O22 public lifecycle witness at `/tmp/task22o22-red-integration-lifecycle.txt`; and truncating staged records before the genuine second active slot fails the later-slot transaction witness at `/tmp/task22o22-red-integration-transaction.txt`. Current tuple-signature production artifacts are preserved at `/tmp/task22o22-green-production-regularize.rs`, `/tmp/task22o22-green-production-stage.rs`, and `/tmp/task22o22-green-production-project-slice.rs`; each mutation was restored byte-for-byte before identical GREEN and full-workspace validation.

## Documentation and rollback

After implementation evidence is frozen, update `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, this spec, and the plan with exact checksum/totals/events and gate results. O22 adds no public API, persisted format, dependency, migration, fallback, or independently designed pipeline. Rollback restores O21 as the terminal consumer and removes only the O22 module, state/wiring/tests/docs, the O22-only inter-stage observer entry, its re-exports, and its geometry tests while restoring ordinary `offset2_ex` to its original two-line body; all O21 geometry and behavior remain unchanged.
