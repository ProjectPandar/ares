# Task 22O.95 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/GCode.cpp:5434-5470,6131-6148`, limited to single-region island
perimeter/infill phase dispatch.

Flatten each O94 island into owned print entities. First layers always place
perimeter collections before generated/retained infills. Later layers use the
3MF-derived `is_infill_first` region option; KSR resolves it to false. Preserve
within-phase source order and fallback-island position.

Deferred: source infill greedy chaining/reversal (`GCode.cpp:6150-6174` and
`ShortestPath.cpp`), multi-region/tool/wiping, motion, and G-code. No legacy
pipeline or fixture branch.

Focused option coverage proves wall-first on layer zero and both later-layer
branches. KSR freezes 3,350 islands, 2,881 nonempty/perimeter-first islands,
and exact 2,881 perimeter / 1,658 generated fill / 2,285 thin inventories.
Four tests and strict core Clippy, rustfmt, diff, and sub-400-LOC gates pass;
implementation/test shards are 132/101 LOC.
