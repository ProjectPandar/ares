# Task 22O.27 — Region-expansion direct wave propagation Plan

Spec: `docs/superpowers/specs/2026-08-06-ksr-fdmtest-v4-task22o27-region-expansion-wave-propagation.md`

## Status

Implemented from exact O26 predecessor
`729db448a8ab784d59006a2068c282eb4fb68ced` and pinned OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. O26 exact-SHA Tier-1 run
`31097841309` is green. The revised O27 spec and plan received literal approval
from an independent reviewer and a separate default-model OpenCode reviewer
before delegated implementation. Local implementation and regression gates are
green. The final independent reviewer approved after one repair/re-review loop,
and the separate default-model OpenCode reviewer returned
`VERDICT: APPROVE`; exact pushed-SHA Tier-1 remains pending.

## Validation contract

Rewrite only the direct-seed `Algorithm::RegionExpansion` prerequisite defined
by the spec: exact ClosedLine/OpenRound offset behavior,
`RegionExpansionParameters::build`, and ordered direct
`propagate_waves(&WaveSeeds, ...)`. Reuse the single ARD-0024 kernel, explicit
`CoordinateScale`, and existing bbox prefilter. Preserve raw-path closure,
persistent offsetter configuration, contiguous group order/IDs, staged Round
offsets, Positive/Positive clipping, exact C++ arithmetic, and first error.

Do not implement ClipperZ seed discovery, source-taking overloads, expansion
merge, LayerRegion/PrintObject external-surface processing, project lifecycle,
options, public API, or KSR output changes. Public slicing remains terminal at
O26 with `ProjectSlicingIncomplete`.

Focused commands are fixed before implementation:

- end types:
  `cargo nextest run -p ares-core --lib -E 'test(/geometry::tests::clipper::offset::wave_end_types/)'`;
- region expansion:
  `cargo nextest run -p ares-core --lib -E 'test(/geometry::tests::region_expansion/)'`;
- full offset regression:
  `cargo nextest run -p ares-core --lib -E 'test(/geometry::tests::clipper::offset/)'`.

Every RED must be a compiling assertion failure, not an unresolved symbol.
Capture commands, exits, and relevant assertion output under
`/tmp/task22o27-{red,green}-*`.

## Gate 0 — Frozen predecessor, oracle, and reviewed design

1. Verify `HEAD == origin/main ==
   729db448a8ab784d59006a2068c282eb4fb68ced`, pinned Orca HEAD, no staged
   files, and only O27 spec/plan plus excluded `.pi-subagents/` differ.
2. Preserve exact O26 Tier-1 result `31097841309` and head SHA in `/tmp`.
3. Preserve the diagnostic pinned-source oracle artifacts:
   `/tmp/task22o27-clipper-oracle*.{cpp,txt}`,
   `/tmp/task22o27-propagation-oracle-linux.txt`, and
   `/tmp/task22o27-region-params-oracle-linux.txt`. The harness may adapt only
   allocator/vector scaffolding needed to compile the fixed source; it is never
   committed or invoked by Rust production/tests.
4. Require literal independent and default-model OpenCode approval of the
   complete spec, then this complete plan. Any substantive revision repeats
   both reviews.
5. Freeze the approved documents before implementation.

## Task 1 — Missing Clipper end types, test-first

Production ownership:

- `crates/ares-core/src/geometry/clipper/offset.rs`;
- `geometry/clipper/offset/input.rs`;
- `geometry/clipper/offset/generate.rs`;
- new `geometry/clipper/offset/generate/lines.rs`, at most 300 LOC.

Tests:

- declare and add
  `geometry/tests/clipper/offset/wave_end_types.rs`, at most 300 LOC.

Steps:

1. Add narrow compiling `add_closed_line` and `add_open_round_path` shells whose
   runtime behavior is deliberately wrong. Add exact assertion REDs for empty,
   one-point, straight, reverse, bent, raw-closed, repeated, and threshold paths.
2. Freeze both input predicate branches: exact equality when shortest edge is
   zero; strict squared distance when positive; near-but-unequal and exact
   threshold cases; terminal filtering for both closed types and consecutive
   filtering for all end types.
3. Freeze exact mixed ClosedPolygon/ClosedLine `FixOrientations`, raw two-sided
   ClosedLine order, OpenRound first-side/end-cap/reverse-side/start-cap order,
   one-point Round versus square behavior, and a non-default round
   `arc_tolerance` witness whose exact point sequence differs from the default.
4. Freeze the strict near-zero branch with positive sub-`1.0e-20`, exact
   `+1.0e-20`, zero, and negative delta. Add coordinate-range and positive
   cleanup output/order cases.
5. Implement only reached `ClosedLine` and `OpenRound`; leave OpenSquare absent.
   Generalize raw generation to zero, one, or two paths per input and split line
   generation into the real child module before `generate.rs` reaches 400 LOC.
6. Preserve all existing OpenButt and ClosedPolygon tests unchanged. Kill
   OpenRound→OpenButt, ClosedLine→ClosedPolygon, non-strict threshold, ignored
   or reset arc tolerance, mixed ClosedPolygon/ClosedLine orientation reversal,
   and wrong near-zero mutations. Restore production exactly after each
   compiling mutation, then run end-type and full-offset GREEN.

## Task 2 — RegionExpansion types and parameter arithmetic, test-first

Production ownership:

- declare `geometry::region_expansion` from `geometry.rs`;
- new `geometry/region_expansion.rs`, at most 100 LOC;
- new `geometry/region_expansion/types.rs`, at most 220 LOC.

Tests:

- declare `geometry/tests/region_expansion.rs`;
- add `geometry/tests/region_expansion/parameters.rs`, at most 300 LOC.

Steps:

1. Add compiling types and a deliberately wrong build result. Add assertion REDs
   freezing `to_bits()` for every `f32`/`f64` field and exact integer equality
   for `num_other_steps`, using the fixed Normal/LargeBed oracle cases: capped
   steps, one-step fallback, and multi-step reduction.
2. Add trusted assertion-precondition tests for zero full expansion, zero step,
   and zero maximum steps; do not add a recoverable public error.
3. Implement exact source expression types: f32 tiny/first division, double
   `0.25`/`4.` reduction, f32 fallback, f64 arc/shortest, and f32-sum→f64
   `*1.1`→f32 max inflation. Pass `CoordinateScale` explicitly.
4. Define crate-private `WaveSeed { src, boundary, path }` and
   `RegionExpansion { polygon, src_id, boundary_id }` with `u32` IDs and ordered
   owned geometry. Add no serialization/public API.
5. Kill all-f64, all-f32, scale substitution, reassociation, and wrong step
   count mutations. Run parameter GREEN.

## Task 3 — Direct supplied-seed wave propagation, test-first

Production ownership:

- add the cited ordered group-extent seam `BoundingBox::from_polygons` to
  `geometry/bounding_box.rs`;
- new `geometry/region_expansion/propagate.rs`, at most 300 LOC;
- split one real child module before exceeding 300 LOC rather than weakening
  source order.

Tests:

- add `geometry/tests/region_expansion/helpers.rs` and `propagate.rs`, each at
  most 300 LOC; extend ordinary bbox tests only for the new narrow constructor.

Steps:

1. Add a compiling propagation shell returning empty. Add complete ordered-path
   and annotation REDs from the diagnostic C++ outputs for empty, open, closed,
   single-step, multi-step, boundary-hole, and multiple-seed cases.
2. Add grouping REDs for multiple contiguous paths with one `(boundary, src)`,
   adjacent groups with different IDs, and separated equal IDs proving no
   hidden sorting or regrouping.
3. Add topology REDs distinguishing Positive/Positive from NonZero and staged
   wavefronts from a single total offset. Compare complete ordered integer paths
   and IDs, not area/count/bounds alone.
4. Add bbox REDs for contour-before-holes prefilter order, positive f32→i64
   truncation near an edge, and a distant out-of-range boundary contour removed
   before Clipper input.
5. Add first-error REDs for an initial offset range failure, a later wave-step
   failure, and clipping failure. Empty seeds must return before boundary access.
6. Implement one `ClipperOffset` configured once before the outer group loop
   with `params.arc_tolerance` and `params.shortest_edge_length`. Its `clear()`
   must preserve both across every path, step, and group.
7. For each contiguous group, preserve seed path order; select ClosedLine versus
   OpenRound from raw `front() == back()` before input normalization; use Round
   and `initial_step`; trim only the selected boundary ExPolygon by the inflated
   group bbox; clip with Positive/Positive.
8. Repeat exactly `num_other_steps`: each input path is Round+ClosedPolygon,
   sign comes from original orientation, and outputs are reversed only for a
   clockwise input before Positive/Positive clipping.
9. Append final paths with group IDs in Clipper order. Propagate `ClipperError`
   without a project-level `SliceError` mapping.
10. Kill hidden sort, ID swap, per-group offsetter defaults, OpenButt,
    ClosedPolygon initial open path, NonZero, one-shot offset, wrong sign,
    missing reversal, changed step count, missing bbox trim, and error-reorder
    mutations. Run direct and complete geometry GREEN.

## Task 4 — Parent audit and regression gates

1. Inspect every diff against the approved spec and pinned source. Verify no
   deferred symbol or lifecycle wiring entered the patch.
2. Run focused end-type, RegionExpansion, complete offset, Clipper Boolean, and
   bounding-box tests.
3. Run `cargo nextest run --workspace --no-fail-fast`.
4. Run:
   - `cargo fmt --all -- --check`;
   - `cargo check --workspace --all-targets`;
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
   - all four wasm32 checks from `.github/workflows/tier1.yml`;
   - optimized default/feature WASM builds, wasm-bindgen export audit, and both
     Playwright repetitions;
   - `git diff --check`.
5. Audit every Rust file `<400 LOC`, every new source/test shard `<=300 LOC`, no
   dependency/manifest diff, and no staged `.pi-subagents/` or `target/parity/`.
6. Audit changed code for no `unsafe`, `include!`, `include_bytes!`, broad lint
   allowance, source-text/hash/line tests, binary oracle, fixture branch,
   reference-G-code read, Orca command/FFI, or fallback.
7. Prove project lifecycle and KSR checkpoints are unchanged: O26 remains the
   terminal consumer and successful traversal still yields
   `ProjectSlicingIncomplete`.

## Task 5 — Documentation, six-dimensional review, ship

1. Update `docs/roadmap.md` and `docs/architecture/option-parity-v4.md` with O27
   source boundary, implementation outcome, exact tests/mutations, limitations,
   rollback, next source boundary, reviews, and ship status. Keep ARD-0024
   unchanged. Finalize this spec and plan with the same evidence.
2. Rerun both spec reviewers and both plan reviewers after substantive evidence
   changes.
3. Dispatch one fresh independent review-only thread over requirement
   completeness, logic correctness, boundary cases, code quality, test coverage,
   and actual execution. Dispatch a separate default-model OpenCode review over
   the same final diff and evidence. Require literal `VERDICT: APPROVE` from
   both.
4. The parent is sole fix writer. Convert every finding to a repair list, search
   for sibling defects, fix and rerun affected/full gates, then return the same
   revised state to both reviewers until approval. Reviewers never edit code.
5. Use Conventional Commits; separate implementation and final evidence where
   practical. Never stage `.pi-subagents/` or generated parity artifacts.
6. Push `main`, verify exact `HEAD == origin/main`, then wait for the complete
   pushed-SHA Tier-1 matrix. Any failure repeats repair, verification, and final
   reviews; pending CI blocks shipping.

## Expected files

Modified:

- `crates/ares-core/src/geometry.rs`;
- `geometry/bounding_box.rs`;
- `geometry/clipper/offset.rs`, `offset/input.rs`, `offset/generate.rs`;
- ordinary test roots;
- O27 spec/plan, roadmap, and option-parity architecture record.

New:

- `geometry/clipper/offset/generate/lines.rs`;
- `geometry/region_expansion.rs`;
- `geometry/region_expansion/types.rs`;
- `geometry/region_expansion/propagate.rs`;
- focused ordinary test shards listed above.

Explicitly unchanged:

- `Cargo.toml`, `Cargo.lock`;
- `project_slice.rs` and `project_slice/prepare_infill.rs`;
- public APIs, WASM exports, project options, and persisted formats.

## Completion evidence

Tasks 1-3 are implemented in ordinary source/test modules. The delegated worker
had no command tool, so it could not provide chronological assertion RED logs;
the parent recovered compiling REDs and records them honestly as recurrence
evidence. Twenty-eight compiling mutations are killed after adding pinned-source
clockwise and precision-sensitive witnesses; production is restored before
every GREEN. The final additions cover f64/f32 substitution, scale,
reassociation, hidden sorting, open-as-closed, one-shot/extra steps, and eager
later-boundary access before the first error.

The final focused geometry run passes 77 tests. The full workspace passes 5,929
tests with 2 skipped. Native all-target check, strict workspace all-feature
Clippy, four wasm32 checks, optimized default/feature WASM builds and export
audit, two 11-test Playwright runs, rustfmt, diff, LOC, dependency,
forbidden-pattern, lifecycle, and rollback audits pass. Twenty-one focused O27
tests and the mutation set freeze end types, parameter bits, contiguous groups,
ordered paths/IDs, bbox trimming, Positive/Positive clipping, staged offsets,
Clipper orientation, clockwise sign/reversal, and error propagation. Task 5's
independent repair/re-review loop and separate default-model OpenCode review
both end in literal `VERDICT: APPROVE`; exact pushed-SHA Tier-1 evidence remains
to be appended after that release gate completes.

## Rollback

Restore the exact O26 predecessor by removing only the O27 RegionExpansion
module/tests/docs and the added ClosedLine/OpenRound offset paths, then restoring
the prior private offset generator/input shape. Existing ClosedPolygon,
OpenButt, Clipper Boolean/offset APIs, O26 lifecycle/state, KSR checks, and
public incomplete result remain intact.
