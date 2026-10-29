# B proposal: retire ByObject perobject-skirt owner multiplicity guard

Status: **DOCS-ONLY REVIEW-REQUEST / PROPOSAL ONLY**. Base exact approved A GREEN
`344320817d78b96cbc7005fac426e913d127c8cd`, queued stack26, not locally integrated
or rebased. Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
B production/test changes are NOT authorized or committed by this proposal.

## Fresh Native-first probe after A

Evidence: `/tmp/ares-byobject-perobject-B-proposal-344320817/discovery-report.json`.
Fresh baseline AND guard-lift Native runs precede public API/real CLI in BOTH
modes, serial CPU18 with independent roots/caches, canonical reference settings
passed explicitly to Native AND public launches, actual descendant/thread masks
sampled CPU18. Do not reuse the pre-A failed probe as current evidence.

Reuse the frozen accepted input-only family from the earlier A/B proposal:
2 owners / 3 instances X50,125,200,Y110,Z2, scale0.4, spacing75, ByObject,
outer_only brim8/gap0, combine0, skirt2/height1, one filament, no support/raft/tower.
All three frozen ZIP SHA/member manifests are in `inputs/manifest.json`:

| Case | SHA256 | Only config difference vs target |
| --- | --- | --- |
| perobject-brim TARGET | `12f8a0ad17a51166aa603b01f49e4c8bd7afb5dc4df56b5717c67ebd6452713c` | none |
| combined-brim CONTROL | `5467219291d3d177824fc4b7677605eb855707962ee9fd5d773db96dc15d9922` | skirt_type=combined |
| perobject-no-brim CONTROL | `fd0e711e298cfab71ded112b722dbb49ca2383a825b96d890a13a1e5b52871d4` | brim_width=0 |

Combined control is byte-identical to committed A `combined.3mf`. All other ZIP
members match; config diff is exactly the one named field. No G-code ZIP member.
Native CLI0 for ALL cases: positive model extrusion at ALL three centers;
active-brim cases have TWO owner brim blocks and positive brim extrusion at
ALL three centers; width0 has NONE. Perobject cases have TWO real positive skirt
blocks, combined has ONE. Two sequential origin transitions in all cases.

| Clean A baseline, both modes | Public validated API / real CLI |
| --- | --- |
| perobject-brim target | UnsupportedProjectFeature("skirt_type per-object"), CLI1 / NO G-code |
| combined-brim control | COMPLETE Native/API/CLI strict PASS, raw API==CLI |
| perobject-no-brim control | COMPLETE Native/API/CLI strict PASS, raw API==CLI |

This is a Native-accepted unsupported witness, not Native rejection, NONE, or
shared-group SlicingError. Unlike the pre-A proposal, combined-brim now genuinely
qualifies as a PASS prerequisite control.

## Exact uncommitted guard-only experiment

Removed ONLY the residual `ByObject && instance_indices.len()!=1` disjunct in
`skirt/generate.rs::SkirtPlan::generate`, its obsolete ownership comment and
now-unused tuple binding. No planning, geometry, ordering, cursor, M73, estimator,
other guard or comparator edits. Patch `guard-only.patch`, SHA256
`979bc7d7bd98fc4d416e4ea43bf792e1c0e407fdfef2e6aea2e992365a1199ec` — the SAME
patch that failed complete parity on pre-A0577970d4.

On A344320817, BOTH modes now give **3/3 COMPLETE unchanged ordered Native/API/CLI
PASS**, raw API==CLI. Target has Native/API TWO owner brim blocks / TWO skirt
blocks. Both controls retain IDENTICAL raw API AND CLI bytes baseline→probe.
No M73 filtering, snaps, tolerance, fallback or equality inversion. This is DIRTY
probe evidence, **not a clean B GREEN**, and does not replace authentic reviewed RED.

Source restored byte-for-byte. Restoration build/fmt/strict clippy exit0; restored
CLI SHA256 `6d399d05ed02f1ff70f8fd7ff3762f2415e9e1a3e15bc88e3214f466190c85f8`
is BIT-IDENTICAL to the clean baseline executable. `restoration-audit.json`
records this; no lifted guard remains in the reusable cargo target. No B family
or full/workspace runs, new WASM run, rebase, merge or push during FULL25 freeze.

## Owning upstream rewrite slice and exact Rust destination

- `Print.cpp:2520-2530` de-duplicates owner IDs; `_make_skirt:2646/2794-2816`
  expands instance hulls and consumes the COMPLETE owner brim footprint;
  :2847-2920 forms perobject skirt groups. Instance count is NOT an unsupported
  policy for a separated single-owner skirt group.
- `Brim.cpp:36-45,573-581,914-977` prepares ordinary once-per-owner areas even
  for ByObject; global combine is separately disabled. A already supplies this
  owner collection, plate-frame footprint and emitted-owner lifetime in Rust.
- `GCode.cpp:4465-4484::generate_object_skirt_group` addresses the owner's group,
  calls `generate_skirt` at plate origin and uses `m_skirt_group_done[group_idx]`.
  :4486-4531 emits brim once per outstanding owner at plate origin; :5334-5335
  invokes skirt/brim before model extrusion. A's :3298/3322→:7506 travel cursor
  lifecycle is already ported; B does not change it.
- `Print.cpp:2551-2556` is the DISTINCT multiple-owner shared-group rejection,
  NOT a multiple-instance owner rejection. It remains deferred/guarded.

Destination is ONLY the temporary admission shell in
`crates/ares-core/src/project_slice/gcode_emit/skirt/generate.rs::SkirtPlan::generate`:
retire the remaining ByObject instance-count restriction after the cited producer
and consumers have been ported by A. No new Ares pipeline or skirt producer.
Preserve existing brim-kind predicate (OuterOnly / OuterAndInner), combine1+brim,
tower/flush and ByObject shared-group guards. Preserve `skirt/occupied.rs`,
`skirt/member_groups.rs`, `layers/object_skirt.rs`, ordinary owner expansion,
plate-frame emission and cursor consumers unchanged. No API/parser widening.

## Requested next barrier

Approve this B source boundary/frozen family for a test-only public RED commit:
three named cases, Native FIRST, Native positive activity above, target accepted
output expected (therefore genuine Unsupported failure on clean A), controls
complete PASS, unmodified `golden::compare_ordered_bytes_generator_only`, raw
API==CLI for accepted outputs. Commit input-only fixtures/SHA/member manifests,
never Native outputs. Independent RED approval before minimal production guard
retirement; probe PASS must not be substituted for either review barrier.

After that authorization, GREEN must first pass all three cases BOTH modes,
then targeted brim/skirt/per_object/sequence/ByObject/wipe families BOTH modes,
fmt/strict/WASM, exact clean hashes, CPU18/jobs2/threads2 and files<=400 LOC.
Controls' raw API/CLI must remain RED→GREEN stable. ANY residual is STOP/report,
not permission for estimator/exporter work or adjacent geometry changes.
Deferred: new brim-kind/holes-specific parity claims (witness here is outer_only),
shared-group error parity, combine1+perobject admission, new overlap/touching
geometries, support/raft/tower/multifilament. Existing supported kind policy is
not expanded. A remains approved; B remains guarded pending the above process.
