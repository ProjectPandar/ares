# CrossZag resolved-surface runtime controls

## Upstream and Rust boundaries

`Fill.cpp:865–879` retains sticky raw SurfaceFillParams during grouping;
`Fill.cpp:1282,1305–1326` constructs fresh FillParams for emission and enables
phase/symmetry according to the resolved filler. Preserve the existing
raw-group contract in `docs/architecture/task22o73-base-fill-grouping.md`.
Rust destination is the conversion in `fill_entities/monotonic.rs`, not
`group_fills/params/projection.rs`.

Included: copy horizontal phase only for CrossZag in this dispatch; enable
mirroring only for ZigZag/CrossZag with the stored flag enabled. LockedZag
has a separate emitter and its depth/sidecar metadata remains unchanged.
Deferred: raw group ordering/metadata, filler geometry, phase arithmetic,
solid-spacing adjustment, bridge scoring and full option/printer coverage.
No new pipeline or tolerance acceptance.

## Evidence and plan

Official CrossZag differs in layer-3 internal solid fill phase, then top
surface phase. Raw sticky sparse-pattern controls were copied directly into
Monotonic runtime params.

Plan: red test at the actual raw-to-runtime control conversion covering all
six callers, signed phase and enabled/disabled symmetry; preserve group
metadata and its original behavioral test. Gate the conversion, run official
full-output comparisons, workspace/KSR/fmt/clippy, then commit code and
docs together.

## Raw-group correction

The initial approach reset raw group fields. That approach was
rejected: it broke the existing ARD contract and its test. Projection,
sticky assignment and the original grouped sidecar test are restored
exactly. The implementation now changes only the runtime
conversion boundary. Earlier phase-helper tests and validation of that
rejected approach are not acceptance evidence for the final patch; final
runtime tests and gates are rerun. The raw group is never mutated by this
conversion.
