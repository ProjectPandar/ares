# SPEC request: wipe retraction EPSILON early-return consumer

Status: **conditional SPEC-REVIEW-REQUEST; no runtime RED established**.
Baseline c251aa6166bcc0270063fc6d2e41b84ab01a637d (FULL14 merged).

## Source-owned boundary

Orca upstream GCode.cpp:313-361, Wipe::calculateWipeRetractionLengths:
- :325-330: initialize full configured retraction length; if length<=EPSILON,
  return {0,0} before computing the before-wipe fraction.
- :333-337: compute before fraction and remaining; if remaining<=EPSILON,
  return {before,0} before evaluating speed/path/during budget.
- :340-361: existing wipe-speed floor, path budget and during-budget EPSILON
  logic are context, not a new algorithm to design.
Extruder.cpp:168-172, retract_before_wipe factor clamp [0,1] supplies the fraction.
libslic3r.h:52 owns EPSILON=1e-4. GCode.cpp retract() call chain and
Wipe::wipe:426-490 consume the result; GCodeWriter.cpp:933-952 owns E omission
and comment emission. These are context, not authorization to grow the scope.

Rust destination: project_slice/gcode_emit/motion/travel/wipe_sequence.rs,
retract_and_wipe_impl's requested_before/remaining/during calculation, currently
around :99-128. Existing before calculation around :130-143 consumes the tuple.
Current compatibility shell tests during_max<=1e-4 and during_max-remaining>
1e-4 but does not directly express upstream's two earlier conditions. Merely
finding this static difference is NOT a public failure or permission to patch.

## Included / deferred

If a live Native divergence survives admission and existing complete comparison,
port ONLY the two upstream early-return conditions and their immediate tuple
consumption. Preserve existing post-wipe top-up, writer E epsilon, restart extra,
path clipping/eligibility, speed, comment forwarding, marker/reset behavior and
all unrelated math. No new threshold, validator, fallback, guard policy,
percent conversion or independently designed Ares retraction pipeline.

Deferred: wipe_distance eligibility cutoff/geometry/huge values, toolchange,
firmware retraction, role-based speed, multi-extruder identity, cooling, adaptive
PA, travel/loop selection, schema domain coverage, upstream late during-budget
EPSILON rounding and any additional residual encountered. Such a residual needs
an amendment, not a wider repair inside this slice.

## Proposed Native-first discovery and RED design after approval

Use the existing admitted input-only overhang 3MF from the merged wipe-comment
fixtures. Mesh bytes unchanged, static PA/adaptive0, Classic/no scarf/tower,
comments1, wipe1/distance2, all other options fixed. External exploratory inputs
and artifacts only; no passing-test/accounting submission.

Two directed families exercise the source conditions, with same-input controls:
1. Full length at zero / 0.00005 / 0.0001 / 0.0002, before50% (first return).
2. Length0.8, before100% / 99.995% / 99.98% / 50% (second return).
   99.995% leaves nominal0.00004, below1e-4; 99.98% leaves0.00016 above.
   These are discovery candidates, not a promise that export precision or float
   representation preserves every lexical value. Confirm actual effective config
   and real emitted WIPE XY/E first; do not infer reach from requested settings.

First live whole-output difference must be localized to the stated calculation
using actual Native movement/E and source context, not comments/rejection/timeouts.
If inputs reject, lose intended effective value, never reach wipe, or complete
outputs match, report evidence and STOP; do not invent a RED or scanner fix.
The committed Native RED fixture set is selected ONLY from genuinely divergent
admitted candidates with the unchanged controls after discovery review.

Public RED tests first, input-only fixtures with frozen SHA; use
**OrcaRunner::from_env** and existing export stages / required Native;
no committed Native output or new harness. Require Native effective values and
actual WIPE_START/END positive-XY path/E-presence witnesses before public API and
real CLI admission and the unmodified golden::compare_ordered_bytes_generator_only
complete-output comparison. Each positive must fail because of that actual
stream difference; controls must pass. Never normalize comments/E or weaken
classification/verify_value/comparator. Independent exact RED review is required
before production. If no positive survives, no test-only submission is mergeable.

GREEN only after that approval: minimum source-parity tuple change; exact frozen
inputs/assertions, complete Native/API/CLI parity and raw API=CLI; same-input raw
controls unchanged RED->GREEN. Existing wipe/retraction/comment/PA controls,
reached option coverage, fmt/strict/WASM on exact final full hash/clean status.
CPU18/jobs2/threads2, fresh serial official+seeded Native cache/artifacts, explicit
owned target reuse, actual Native/thread affinity; canonical full oracle env
from ${TMP_ROOT}/ares-parity-control-set-v2.sh (including ARES_PROCGCODE_BIN).
One START per batch; report first divergence/STOP, no per-case inbox noise.
All files<=400LOC. Coordinator owns full/workspace/push/integration. Exclusive
FULL freezes respected; no Nix GC/store deletion or independent oracle rebuild.

Only this source-cited SPEC is submitted now, with no tests/production/runtime
changes. The prior zero-E-comment slice is merged and is not being reopened.
