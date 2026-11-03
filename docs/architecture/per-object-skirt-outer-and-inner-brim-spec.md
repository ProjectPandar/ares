# SPEC request: per-object skirt with OuterAndInner brim

Status: **docs-only SPEC-REVIEW-REQUEST following authorized discovery**.
Base is approved ByObject Slice2 GREEN
`ee270b46a3ebf5d66f8f6838b06b6557b1c0ce00` plus docs-only roadmap
`e4ff7be92dd0b1590432336c2e60cd95427f541b`.
Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
No public RED or production edit yet; wait for owner approval.

## Native source owner / Rust destination

`OrcaSlicer/src/libslic3r/Print.cpp::_make_skirt:2780-2977` builds occupied
object hulls including brim/support-brim footprints before nominal-reach grouping.
`:2847-2897::build_grouped_points` carries source object IDs through grouping;
`:2898-2971::make_group_brims` attaches existing brim entities to their owners,
combining only under its explicit configured/contact predicates. `:2972+` builds
the group's skirt from the occupied outline without adding the distance twice.
Brim type OuterAndInner does not imply a prohibition on perobject skirt.

`OrcaSlicer/src/libslic3r/Brim.cpp:448-460` determines inner/outer brim capability:
OuterAndInner participates in both; holes are actual geometry, not guaranteed
by the enum. `Brim.cpp:890::make_brim`, :948-962 preserves object footprint maps
and permits global combined brims only when combine_brims is set, no active
perobject skirt/shield, and sequence is not ByObject. `:964-977` emits per-object
brim entities when global combination is off. This slice uses combine_brims0,
so does NOT port the global/group combination algorithm.

Exact Rust destination:
`crates/ares-core/src/project_slice/gcode_emit/skirt/generate.rs::SkirtPlan::generate`
brim guard predicate. After independently approved public RED, proposed minimal
GREEN retains all existing predicates but permits `ProcessBrimType::OuterAndInner`
alongside already admitted `OuterOnly`, with combine_brims0 and exactly one
instance per print-object group. Reuse existing installed brim plans and occupied
footprint/member grouping consumers unchanged. If admission alone cannot achieve
whole-output equality, stop for a source-cited amendment; no geometry repair.

Do not remove all `brim_type != OuterOnly` filtering: Auto/InnerOnly/Ear/Painted
etc remain guarded when a plan exists, absent their own qualifying witness.
Keep combine_brims and multiple-instance subconditions unchanged; keep W2
support/raft and W5 tower/multi-filament predicates and W1 ByObject shared-group
scope guard unchanged. Existing Ares geometry/emission remains a compatibility
shell around Print/Brim/GCode ownership, not a new brim/skirt pipeline.

## Completed bounded discovery (reported before this SPEC)

Evidence `${TMP_ROOT}/ares-skirt-brim-disjunct-discovery/discovery-report.json`.
Exact head e4ff7be92; CPU18 jobs2, all sampled Native/public threads mask18,
current coordinator canonical archived/extracted, Native FIRST, fresh independent
official and seeded roots/cache. Five combined-skirt controls were run FIRST;
a perobject target ran only if its identical-input combined control strictly
matched complete Native/API/CLI and raw API==CLI in BOTH modes.

All candidates: ByLayer, 2 objects, 1 filament, no support/raft/tower; width8,
gap0; existing far-brim source geometry and unchanged non-config ZIP members.
Copies candidate uses existing copies family: one of two print objects has two
instances (three instances total). Results identical in both modes:

| Candidate | Combined control | Perobject target / disposition |
| --- | --- | --- |
| auto_brim, combine0 | whole PASS | Native exit0 but no Brim blocks; Ares whole PASS. NO RED; Auto stays guarded for active plans. |
| outer_and_inner, combine0 | whole PASS, active brim | Native exit0 with 2 Brim blocks; Ares UnsupportedProjectFeature("skirt_type per-object"), CLI1/no output. QUALIFIES. |
| inner_only, combine0 | whole PASS | No holes, no Brim blocks; Ares whole PASS. NO RED; InnerOnly stays guarded. |
| outer_only, combine1 | whole FAIL | STOP: target not run. |
| outer_only, one object with 2 instances | whole FAIL | STOP: target not run. |

OuterAndInner witness proves its active OUTER brim footprint/group interaction
on this existing no-hole geometry. It does not claim active inner-hole brim,
Auto adhesion, or InnerOnly admission. No new hole geometry is proposed; keep
InnerOnly guarded per owner ruling rather than manufacture reachability.

### Later prerequisite candidates: retain exact first divergence

Owner classifies these as pre-existing combined-skirt divergences, not this
perobject admission bug. Keep their subconditions guarded and do not change
comparator or tolerances:

- combine1 outer_only, official+seeded: byte5513/line184, Native
  `G1 X127.841 Y103.322 Z.6 ; move to first brim point`, Ares Y103.321.
  Artifacts `{official,seeded}/combine-control/public-result.json` under root.
- copies outer_only, official+seeded: byte1608/line67, Native
  `G1 X74.265 Y90.954 ; move to first skirt point`, Ares
  `G1 X73.627 Y90.998 ; move to first skirt point`.
  Artifacts `{official,seeded}/copies-control/public-result.json` under root.

These are candidates for separately approved prerequisite localization slices,
not authorized repairs here. No corresponding perobject target runs occurred.

## Planned public Native-oracle RED / controls

After SPEC approval only: commit input-only OuterAndInner target and its
same-input combined control with exact SHA/member manifest; no Native output.
Runtime Native runs FIRST through OrcaRunner::from_env; required Native acceptance,
actual effective OuterAndInner/combine0/ByLayer/single-filament settings, two
active object-owned brim blocks, positive XY/E and perobject skirt groups.
Assert strict complete ordered Native/API/CLI equality, API==CLI, for BOTH cases.
Target fails on current real unsupported API rejection, never passes by detecting
divergence. Controls assert target equality too. Do not borrow control G-code as
target oracle or weaken comments/coordinate comparison. Existing OuterOnly,
perobject grouping and ByObject/Slice1 controls remain unchanged.

Native acceptance and output anchors, not option retention alone, admit the
branch. Record actual inputs, non-config ZIP hashes, applied settings, source/
binary identities, commands, products, errors, first differences and affinity.
No Native outputs in Git; keep input-only artifacts frozen for RED→GREEN.

## Failure inventory / boundaries / exit

Combined-control mismatch: STOP that case, preserve original failure, no
perobject repair or favorable retry. No brim generated: NO RED, no guarded domain
claim. Native rejects/absent product: admission/infrastructure fault, not output
RED. Env/compiler/timeout/cancel/affinity faults: NON-GATE. If target gets past
admission but footprint/order differs, seek new source boundary approval before
production. Never relax comparator, rounding or error classification.

Only independently approved RED permits minimal enum-allowlist GREEN. Preserve
combine/copies/Auto/InnerOnly and W2/W5 guards; no fallback, duplication, new
pipeline or API. Same inputs/tests/model/comparator, control raw bytes unchanged.
Fresh serial official+seeded CPU18 jobs2/threads2, relevant existing brim/skirt
controls and allocated fmt/strict/WASM on exact clean hash; changed Rust <=400
LOC. No full/workspace/push/merge; coordinator owns integration.

Included: active OuterAndInner outer brim + perobject skirt, separated singleton
objects in ByLayer, combine0. Deferred: active holes/InnerOnly, Auto, combine1,
multiple instances, touching/merged brim groups beyond this witness, ears,
painting, support/raft/tower/multi-filament, geometry and clearance changes.
Existing architecture stands; no new ARD needed. This commit changes docs only.
