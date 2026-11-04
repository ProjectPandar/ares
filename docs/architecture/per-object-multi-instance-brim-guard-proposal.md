# SLICE-PROPOSAL: retire the ByLayer per-object multi-instance brim guard

The probe base includes the ordinary owner collection and global-combine chains.
Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Native-first frozen accepted witness

Frozen target derived from `tests/parity/multi-instance-brim/target.3mf` by ONLY
skirt_type combined→perobject. Non-config ZIP members and all other settings
verified identical. Same-input combined-skirt control retained.

- Target SHA256 `6e447b091386ec11b3002fc91352e30371352bd5d2d4d26082c10bbcf5283243`.
- Control SHA256 `aac4f3c278fe16614d0694ba9fbe94db059e7bb68d75c7cf55f7ecf0f47aa07d`.

Two owners/three instances: object2 at80/140,Y110; object3 at110,Y110; Z2 and
Z scale0.4; ByLayer, one filament, outer_only brim8/gap0, combine_brims0,
skirt2/height1, no support/raft/tower. Native accepts and emits real extrusion,
TWO brim owner blocks and ONE skirt block. One skirt group spans this geometry;
this is not evidence for one group per copy, holes, or all possible placements.

Clean baseline, official and seeded independently: Native FIRST accepted target
and control. Target API returns exact
`UnsupportedProjectFeature("skirt_type per-object")`, real CLI exit1/no G-code.
Combined control passes complete ordered Native/API/CLI comparison with raw
API==CLI. This is an admission divergence, not a new XY/path failure.

## Temporary guard-only experiment

After accepted Native baseline, locally removed ONLY
`|| group.instance_indices.len() != 1` in `skirt/generate.rs` and changed the
now-unused tuple binding to `(object, _)`. No geometry/emitter/other guard change.

This was an uncommitted discovery probe, NOT claimed as a public GREEN.
Fresh probe Native was run FIRST again in each mode, then public
validated API and real CLI using the probe build and unchanged full comparator.

**Results in BOTH official/seeded modes:**

- perobject target: Native/API/CLI complete strict PASS, API==CLI raw, CLI0;
- combined control: complete strict PASS, API==CLI raw, CLI0;
- combined control API and CLI raw bytes unchanged from clean baseline;
- Native and API both retain TWO brim blocks, ONE skirt block.

No concrete residual output divergence after lifting the guard was observed.
This is not a NONE/Native-rejected result. Original production source was
restored before this proposal; probe results are never parity gate evidence.

## Owning upstream boundary and why no geometry repair is proposed

`OrcaSlicer/src/libslic3r/Print.cpp:2794-2816::_make_skirt` expands the object
hull over ALL PrintInstances and appends all owner brim-area contours.
:2847-2895 `build_grouped_points`/group-envelope merging retains PrintObject IDs,
not a singleton-instance requirement. :2897-2920 `make_group_brims` retains each
owner collection when combine_brims0. `Brim.cpp:36-45,573-581,914-956,964-977`
provides the instance-expanded owner-area/footprint and infill-once collections;
`GCode.cpp:4486-4531` consumes owner IDs once. These are the already ported
ordinary-owner prerequisites, not a request for a second independent pipeline.

Rust destination: ONLY the supported-admission predicate in
`crates/ares-core/src/project_slice/gcode_emit/skirt/generate.rs::SkirtPlan::generate`
plus public test registration/input fixtures. Existing `brim/owner.rs`,
`brim/emit.rs`, `skirt/occupied.rs`, and `layers/brim_emit.rs` are the cited source
consumers and need NO new production change based on this probe.

The remaining instance-count disjunct predates those prerequisites and still
rejects the witnessed accepted ByLayer state. It is an Ares compatibility-shell
restriction with no corresponding ByLayer upstream singleton-instance rule.
The observed local experiment shows this slice can be **guard retirement + RED**,
not another geometry, M73, estimator or bounds repair.

## Bounded retirement sequence

Scope is explicitly **ByLayer**. The experiment removed the term wholesale only
for probing this input; it does NOT justify silently admitting ByObject copies.
For production, retire this disjunct's ByLayer applicability while retaining its
existing ByObject limitation, e.g. retain the existing instance-count check only
under `print.print_sequence == ProcessPrintSequence::ByObject`. This narrows an
existing unsupported predicate, not an added defensive guard or copy-count
geometry workaround. ByObject's owner/emission behavior was deferred by the
previous SPEC and remains unproved. All other predicate terms, especially
combine_brims, non-whitelisted brim types, support/raft/tower/flush and sequential
shared-group constraints, stay unchanged. No new fallback/error/diagnostic.

Freeze these two input-only cases with SHA/member manifest in the repo, write
public runtime Native-FIRST RED asserting full ordered Native equality for target
AND passing control, raw API==CLI and source config/owner/skirt anchors. Target
must truly FAIL on the current unsupported API/CLI result; never pass because
divergence exists. No Native outputs committed.

Then source-bounded ByLayer guard retirement only, in both modes. Target and
control must both strictly PASS; control raw bytes remain stable. Preserve
owner/global-combine tests and existing perobject skirt/brim families,
fmt/strict/WASM and <=400 LOC. No comparator changes or new normalization.
Any residual whole divergence after the minimal retirement means stop and
report, not another patch. If the base already admits and fully matches the
target, there is no RED; do not fabricate one.

Deferred: ByObject copies, combine_brims1+perobject, other brim modes/holes,
new placements/overlap, support/raft/tower/multi-filament.
