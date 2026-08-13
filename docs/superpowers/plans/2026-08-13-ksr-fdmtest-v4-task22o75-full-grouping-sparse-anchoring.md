# Task 22O.75 implementation plan

## Validation contract

The source caller at `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1394-1407` is
complete when sparse anchoring calls O74 `group_fills`, filters the owned result
for `Internal`, and the old reduced grouping is deleted. Verification is the
real prepared-graph seam, the fixed-MSVC KSR anchoring oracle, focused behavior,
workspace Nextest, strict Clippy, rustfmt, static/LOC checks, and an independent
review after the larger user goal reaches complete G-code parity.

## TDD sequence

1. Change the graph-native KSR anchoring test to call a prepared-graph entry by
   object/layer index. Run it RED while production still exposes only
   `SparseAnchoringLayer`.
2. Replace `SparseAnchoringLayer` and `grouping::group_and_prioritize` with one
   call to `group_fills`. Feed returned Internal CrossHatch groups to the
   existing filler from grouped params. Run focused tests GREEN.
3. Change bridge transaction `candidate_expansion` to pass
   `&PreparedPostExternalSurfaces`, object index, and lower-layer index. Keep
   zero-density skipping and existing capability/error ownership.
4. Remove caller-built focused tests that directly assemble private option
   fields; add graph-native priority/narrow/error/immutability witnesses where
   they discriminate the full caller.
5. Delete `sparse_anchoring/grouping.rs`, remove its module and every obsolete
   symbol, and run static scans.
6. Run `cargo nextest run -p ares-core` for sparse anchoring, grouping, and
   bridge transaction; then `cargo nextest run --workspace`, strict Clippy, and
   rustfmt. Check every changed/new Rust file is below 400 LOC.
7. Update ADR, option parity, and roadmap with exact results. Commit using
   Conventional Commits and push `main` before starting the next source-cited
   slice.

## Completed evidence

- RED: the graph-native KSR test failed to compile against the old one-argument
  `SparseAnchoringLayer` entry.
- GREEN: anchoring 1/1, full grouping 35/35, and bridge transaction 17/17.
- Workspace Nextest: 6,516/6,516 passed, 27 slow, two configured skips.
- Core all-target/all-feature Clippy with `-D warnings`, rustfmt, diff, static
  symbol deletion, and sub-400-LOC checks passed.
- KSR output retained 186 paths, 5,941 points, and aggregate digest
  `917adc6ea02ad7cd7af79e45d90db6f4c1497bf5c8716d7f2f49b7de4b2070ef`.

## Non-goals

Do not activate a lifecycle stage, add public output, implement unsupported
fill generators, touch G-code, or preserve a compatibility path. Do not read
options outside the prepared 3MF-derived graph.
