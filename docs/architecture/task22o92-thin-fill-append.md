# Task 22O.92 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/Fill.cpp:1376-1384`, into the O91 layer entity stage.

After all grouped fill collections, move each layer-region `thin_fills` entity in
source order into the layer output. Preserve path/loop shape, 3D points, role,
flow metadata, and collection boundary; moving prevents duplicate ownership and
matches source cloning semantics without a compatibility copy.

Deferred: perimeter/fill/thin ordering for G-code islands, motion, and G-code.
No fallback or fixture branch.

Compile RED proved O91 had no thin-fill ownership. The KSR oracle now freezes
2,285 entities/paths and 5,401 points after move; three O91 tests cover exact
inventory, repeatability, disposal, and public lifecycle. Strict core Clippy,
rustfmt, diff, and sub-400-LOC gates pass. Stage/types/focused-test shards are
155, 24, and 81 LOC.
