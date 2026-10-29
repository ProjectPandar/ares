# SPEC request: before-wipe small-positive override dispatch

Status: **conditional SPEC-REVIEW-REQUEST; no Native RED yet**.
Stacked on approved b91d9bd97b0655727242f163ac1691ed91654b0b, queued stack15;
merged base remains c251aa6166bcc0270063fc6d2e41b84ab01a637d.
Do not integrate this document as approval for its parent's runtime assumptions.
Rebase only after coordinator broadcasts merged main, without stash.

## One upstream owner / exact Rust boundary

GCodeWriter.cpp:1004-1011, GCodeWriter::retract(bool before_wipe,double
retract_length): factor=before_wipe?retract_before_wipe():1; effective length is
**retract_length>EPSILON ? retract_length : factor*retraction_length()**.
Thus a strictly positive override <=1e-4 does NOT select that explicit value.
Extruder.cpp:168-172 supplies factor clamp; libslic3r.h:52 owns EPSILON1e-4.

GCode.cpp:350-361 Wipe::calculateWipeRetractionLengths can produce a small
positive before amount when the during budget is slightly smaller than the
remaining full length. GCode.cpp:7659-7663 passes that before amount to the
writer with before_wipe=true. This producer/call chain is context only: do not
change its budget/geometry or reopen the now-approved :330/:337 early returns.
Writer _retract:1025 onward / Extruder::retract:75-84 consume effective length
and existing retracted amount; their math/state ownership is not being redesigned.

Rust destination: project_slice/gcode_emit/motion/travel/wipe_sequence.rs,
retract_and_wipe_impl's immediate before-wipe tuple consumer around :130-145.
The compatibility shell currently uses retraction_length-during directly in its
positive-during branch. Its zero-during handling already models configured
fraction selection. Port ONLY the small-positive override dispatch if a live
Native divergence proves this exact branch; preserve no-wipe consumption.
Static arithmetic difference is not a RED, and absence of an internal check is
not itself a bug. This is a bounded upstream consumer rewrite, no Ares pipeline.

## Native-first discovery proposed after approval

Use existing admitted input-only overhang 3MF, mesh/non-config bytes unchanged.
Fix length0.8, before0%, wipe1, wipe_speed30mm/s, retraction_speed60mm/s,
role_based_wipe_speed0, comments1, static PA/adaptive0; Classic/no scarf/tower.
Candidate wipe_distance values0.3999/0.39995/0.399975/0.4/0.400025mm give
nominal during budgets0.7998/0.7999/0.79995/0.8/0.80005 and before amounts
0.0002/0.0001/0.00005/0/near-zero. These are candidate arithmetic only:
actual path length, native scaled rounding, extruder-specific effective settings
and EPSILON branch must be established, never inferred from requested config.
Include same-input ordinary before50%/distance2 and before0%/distance2 controls.
If these exact candidates do not reach the branch, report; no silent range sweep
or alternative feature/producer additions. Above/below boundary names are not
assertions about floating-point equality; actual Native bytes decide evidence.

Required Native-first actual effective config, positive WIPE XY, distinct
pre-wipe E versus during/post E witness, full Native/API/CLI success and raw
API=CLI, existing generator-only complete comparator untouched. First stream
difference must be localized to effective-before dispatch, not merely movement
or E anywhere, requested settings, a rejection, comment loss or duration.
If Native matches or no reached divergence localizes here: **STOP**, evidence
only, no passing/accounting test submission or unnecessary repair.

## Included / deferred / acceptance

Included only existing non-firmware, non-toolchange before-wipe consumer's
configured-fraction versus explicit override selection and subtracting existing
retracted amount. Preserve upstream >EPSILON (not >=, abs or new threshold).
Deferred producer during budget/late EPSILON math, all path clipping/distance
eligibility, writer E/format threshold, firmware/toolchange/restart-extra/negative
schema policy/multi-tool dispatch, no-wipe paths, cooling and adaptive PA.
No generic fallback/guard abstraction, new config rejection, comparator
normalization or separate emitter. Upstream's existing parameter-selection
semantics are source behavior, not permission to add defensive fallback paths.

After discovery review: tests first with frozen pure input SHA, actual Native
witness, OrcaRunner::from_env/stages required-Native mode, public API and realCLI
complete ordered comparison; commit no Native output. Independent exact public
RED approval before code. Freeze admitted positive and passing controls; if
another difference emerges outside dispatch, stop for amendment, not scope creep.
GREEN: exact inputs/assertions, full parity both modes, raw same-input controls
unchanged plus existing wipe/retraction/comment controls, fmt/strict/WASM exact
full clean hash; <=400LOC files. CPU18/jobs2/threads2 serial official+seeded fresh
Native caches/artifacts, explicit owned target reuse, actual descendant/thread
affinity, canonical ALL oracle env including procgcode. One batch START then
first-live-divergence/STOP, no per-case inbox noise. Respect exclusive FULL freezes;
coordinator owns full/workspace/push/integration; no Nix GC/store deletion.
Only this document now; no new runtime/tests/production changes.
