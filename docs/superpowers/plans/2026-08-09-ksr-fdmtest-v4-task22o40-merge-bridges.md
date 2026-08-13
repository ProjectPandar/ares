# Task 22O.40 — Merge bridge groups implementation plan

## Status

Implementation, source-pin cleanup, documentation, and the pre-review
verification matrix are complete. The candidate has eight passing O40 tests,
69/69 passing O35-O40 focused regressions, 6,101/6,101 passing workspace tests
with two skipped, and passing warning-denying Clippy, rustfmt, native/wasm32,
diff, LOC, and include audits. The normalized KSR test remains the intentional
progress RED at the CLI `--options` contract. The initial independent review
rejected a rustfmt failure plus coverage and citation gaps; all were repaired,
all gates were rerun, and the same six-dimensional review thread returned
`VERDICT: APPROVE` with zero findings. O40 is locally complete but remains
inactive and unreleased.

## Outcome

Implement the source-cited boundary defined in
`docs/superpowers/specs/2026-08-09-ksr-fdmtest-v4-task22o40-merge-bridges.md`
without activating adjacent external-surface orchestration.

Success for this slice is behavioral parity of merged bridge surfaces at the
crate-private functional seam. Full success for the continuing goal remains the
unignored, normalized byte-exact `ksr_fdmtest_v4` 3MF-to-G-code golden test.

## Rewrite boundary

The primary upstream function is
`OrcaSlicer/src/libslic3r/LayerRegion.cpp:310-351::merge_bridges`. Its direct
dependencies are `Bridge`/`group_id` at `LayerRegion.cpp:173-190`,
`RegionExpansionEx` at `Algorithm/RegionExpansion.hpp:85-92`, contour/hole
conversion at `ExPolygon.hpp:300-307`, bottom-bridge surface defaults at
`Surface.hpp:9-47`, and Miter-3 flat closing at
`ClipperUtils.hpp:19,23-27,400-408` / `ClipperUtils.cpp:592-603`.

The Rust destination is the crate-private
`project_slice::prepare_infill::external_surfaces::merge_bridges` function in
`external_surfaces/merge_bridges.rs`. Included behavior is contiguous source-
expansion association, group-root/member collection, contour-before-hole
flattening, independent per-group closing, root-angle/default `BottomBridge`
materialization, deterministic output order, and direct Clipper errors.

Deferred behavior is `expand_bridges_detect_orientations` at
`LayerRegion.cpp:395-437`, zone trimming, active
`LayerRegion::process_external_surfaces`, lifecycle integration, Options,
adapters, and every fill/toolpath/motion/G-code stage. Existing O35-O39 Ares
records/helpers remain an inactive temporary compatibility shell until the
later upstream orchestration slice consumes them. O40 removes only the
source-shaped C++ iterator field, replacing it with owned Rust input and local
index ranges; it adds no fallback or parallel Ares-owned pipeline.

## Planned changes

Production:

- add `external_surfaces/merge_bridges.rs`;
- register the crate-private `merge_bridges` module/function;
- remove the source-shaped `bridge_expansion_begin` field and compile-time
  signature/layout pins that have no runtime behavior;
- keep O35-O39 behavior intact while adapting constructors and behavior tests.

Tests:

- add `external_surfaces/tests/merge_bridges.rs`;
- remove the one Ares implementation-text scan in
  `project/tests/effective_config/usage.rs`;
- retain explicit input/output parity tests; do not add pointer, iterator,
  signature, source-text, malformed-ID, or partial-mutation pins.

Documentation:

- record O40 in `docs/roadmap.md` and
  `docs/architecture/option-parity-v4.md` after implementation evidence exists;
- update O39's release state only from authoritative completed CI evidence.

## Red-green sequence

1. Add the empty implementation and the first observable single-bridge test.
   Run `cargo nextest run -p ares-core task22o40` and retain the compiling RED.
2. Implement only enough grouping, flattening, closing, and surface
   materialization to make that test green.
3. Add one behavior at a time for grouped members, expansion association,
   disconnected output/order, and Clipper errors; run the same focused command
   after every red and green transition.
4. Run all external-surface tests and the directly affected geometry offset
   tests. Fix only behavior introduced by this slice.

## Implementation outline

The implementation consumes `Vec<Bridge>` so Ares does not expose or preserve
Orca's iterator state. It computes bridge roots and contiguous expansion ranges
once, then moves bridge geometry into per-root polygon buffers. For each root in
ascending order it:

1. requires the precomputed root angle;
2. applies the existing Clipper positive polygon offset;
3. applies the existing negative PolyTree offset;
4. converts the tree to ExPolygons; and
5. constructs default `BottomBridge` surfaces with the root angle.

The algorithm is linear outside Clipper work. It performs no repeated scan of
all expansions per bridge, no defensive copy of bridge geometry, no scale
conversion, no global union across groups, and no fallback.

## Verification sequence

Run, in order:

```bash
cargo nextest run -p ares-core task22o40
cargo nextest run -p ares-core external_surfaces
cargo nextest run -p ares-core -E 'test(/(task22o3[5-9]|task22o40|clipper.*offset)/)'
cargo nextest run -p ares-cli --test ksr_fdmtest_v4 \
  -E 'test(project_matches_orca_242_except_generator_line)' \
  --run-ignored ignored-only
cargo nextest run --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Also require:

- `git diff --check`;
- every Rust source/test file below 400 physical lines;
- no production `include!`, `include_bytes!`, or `include_str!` used for source
  splitting;
- no fixture name/hash/reference-G-code read in production; and
- no external-surface source-text/signature/layout pin added or retained in the
  touched seam.

## Review loop

After the implementation and verification pass, start one fresh read-only
review agent. It reviews the exact diff and command evidence along the six
required dimensions: completeness, correctness, edge cases, quality, coverage,
and actual runtime results. The main thread converts findings into a concrete
repair checklist, applies accepted fixes, reruns affected and global gates, and
asks the same reviewer to revalidate. Repeat until approval or a demonstrated
external blocker.

The next source boundary after O40 is
`LayerRegion.cpp:395-437::expand_bridges_detect_orientations`.
