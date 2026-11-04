# Per-object skirt Painted brim admission SPEC

## Scope and guard-off result

This slice is based on InnerOnly admission `2a37ae4a3dd6f7c40950681b93fdae2f1ef1509f`, with the source boundary at `4351a644d6683555ae48e44ea7f898a66f75d585`.
The frozen two-object input has two separated
objects, `skirt_type=perobject`, `brim_type=painted`, `combine_brims=0`, one
filament and one instance per object, ByLayer, no tower/support/raft.

A guard-off probe temporarily added `ProcessBrimType::Painted` to the existing
`generate.rs:48-56` match, without any other code change. Native and Ares were
then run on the same input in both official and seeded modes with fresh roots; Native returned rc 0 in both.

Native asserted active geometry: each mode contains four `;TYPE:Skirt` blocks
(lines 74, 242, 547, 698) and a `;TYPE:Brim` block (line 310, byte offset
7572). The temporary Ares output has the same block counts. Raw files differ
only in generated header bytes (Native 99190 bytes, Ares 99158); after removing
those generated metadata lines, both modes compare equal at all 4038 lines with
no first difference. The divergence is therefore admission-only: the current
Ares guard returns `UnsupportedProjectFeature("skirt_type per-object")`, while
Native defines a successful painted-brim/per-object output.

## Upstream and Ares boundaries

`OrcaSlicer/src/libslic3r/Print.cpp::_make_skirt` (`2780-2977`) groups each
object's skirt using its occupied footprint; object/brim footprint collection
and fixed-point grouping are `2847-2971`. Painted brim points are selected by
`Brim.cpp:354-412` and replace the ordinary ear points at `456-458`; the shared
support/no-brim mask path is `590-634`.

The exact Ares admission boundary is
`crates/ares-core/src/project_slice/gcode_emit/skirt/generate.rs:48-56`, where
`plan.is_some()` currently admits only `OuterOnly`, `InnerOnly`, and
`OuterAndInner`. `skirt/occupied.rs:61-80` already consumes each plan's
`BrimPlan::covered_hull()`. Since guard-off whole-output parity is exact after
normalization, no painted-ear geometry port is included here.

## Included, deferred, and RED plan

The GREEN changes only the brim-type match by adding
`ProcessBrimType::Painted`; `combine_brims`, instance-count, and tower/flush
conditions remain unchanged. BrimEars and Auto remain separate deferred slices.

The public RED is test-only with a committed frozen copy of this input. It runs
Native through `OrcaRunner::from_env` and asserts Native rc 0, at least two
Skirt blocks, and a Brim block before the unchanged strict whole-output
comparison. API and CLI must both compare strictly to Native; the test must
fail at the current guard while the corresponding OuterOnly/InnerOnly controls
pass. Official and seeded modes use fresh roots. No Native artifacts are
committed.
