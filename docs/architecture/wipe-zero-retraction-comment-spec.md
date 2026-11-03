# SPEC: zero-retraction wipe XY comment forwarding

Status: **SPEC-REVIEW-REQUEST, docs only; no public RED/code authorized yet**.
Base `caac760d464112b076821975b67232fdd0f14bf0`.
Branch worker1/wipe-zero-retraction-comment-spec; CPU18/jobs2.
Roadmap Range min/max/interior coverage remains OPEN; this is one source-owned
consumer repair, not global option/domain completion or a new wipe pipeline.

## Exactly one upstream boundary / source ownership

- GCode.cpp:426-490, **Wipe::wipe**: when length>=0, clip stored path to configured
  wipe_distance, emit wipe markers/speed and for each segment call
  writer.extrude_to_xy(point,-dE,"wipe and retract"). Zero remaining retraction
  does not cancel the existing wipe motion or its description.
- GCodeWriter.cpp:933-952, **extrude_to_xy**: abs(dE)<=double epsilon sets
  force_no_extrusion; that omits E and the filament extrusion update. Comment
  emission at951 remains unconditional with respect to force_no_extrusion,
  governed only by the writer's existing full_gcode_comment option.
- Extruder.cpp:168-172, **retract_before_wipe**: clamp requested percentage*0.01
  to [0,1]. At100%, no retraction remains for wipe; length0 also gives zero
  segment dE. Clamp is producer context already present, not a numeric repair.

Exact Rust destination: project_slice/gcode_emit/motion/travel/
wipe_sequence.rs:171-186 in the existing per-segment wipe emission. Its positive
retraction branch computes gcode_comment(" ; wipe and retract"); the zero-E
branch emits bare G1 X/Y. Forward the same option-gated source description
also in that existing zero-E branch. Keep epsilon selection, E accumulation,
coordinates/path length, feedrate, clipping, wipe markers and state untouched.
The temporary compatibility shell's dropped description is replaced, not
retained as a fallback. No new guard, regions[0], AABB helper, wrapper protocol,
config policy, parser normalization or emitter abstraction.

## Discovery and realistic RED fixture choice

Range discovery found inherited schema max first (Config.hpp:2476-2479 and
PrintConfig.cpp:5188-5193); it was a genuine whole-output diff, not admission
failure. That extreme is **not** chosen as the public RED fixture.
Read-only follow-up reuses the exact existing admitted exported overhang.stl
fixture from the prior Native-first PA witness; all meshes/ZIP non-config
members stay fixed, no synthetic G-code/expected stream. PA enabled/static0.035,
adaptive0, comments1, Classic/scarf none, no tower/raft/support, cooling20s/min10,
wipe1/distance2, with the three retraction pairs below. Model remains existing
admitted CSV but is inactive; PA/lookahead is not this repair's owner.

Runtime root: `${TMP_ROOT}/ares-wipe-realistic-discovery-BkOTyp/`.
External helper uses the **unmodified existing** golden::
compare_ordered_bytes_generator_only by absolute source include, public
slice_project_plate_validated plus real CLI. Exact main rlibs/CLI/helper/source
identities, input/source SHA, launch/cache, complete Native/API/CLI streams,
exit/stdout/stderr, effective footer and discovery-audit.json are retained.
Official and seeded serial fresh Native caches/artifacts, **6/6 Native accepted**,
API/CLI6/6admitted/raw API=CLI. Actual orca-slicer/thread CPU18 in both modes.
Repository remained clean with no tests/production/Native artifacts committed.

| Input / both modes | Native / Ares | Existing complete comparator |
|---|---|---|
| before50%, length0.8 | positive-E wipe,150 wipe blocks each | PASS |
| before100%, length0.8 | zero-E wipe,150 blocks each | DIVERGENT byte2106/line85 |
| before50%, length0 | zero-E wipe,150 blocks each | DIVERGENT byte1949/line79 |

Both realistic positives: Native G1 X112.69 Y114.659 **; wipe and retract**;
Ares identical XY but no description. Preceding G1 F1800 and WIPE_END are
retained. Runtime footer confirms requested100% or length0 survived admission.
This is source-owned text, not a geometry/PA/range rejection or a tolerance
issue. Preserve the original complete differences; do not strip descriptions
or change comments0 to hide the defect. Gate-level claim is these same-input
whole-output differences, not proof that all possible residuals are comments.

### Input-only fixtures to freeze after approval

| Case | retract_before_wipe | retraction_length | SHA256 |
|---|---|---|---|
| before-50 | 50% | 0.8 | `183a2d5cbce46aabbe99d22a39b90636326a62a20e752448e444467cb02f9993` |
| before-100 | 100% | 0.8 | `eeb6a19be3cbf4db85fea8fd31cc1128d4ee9590139e96598b68afe1c9721c64` |
| zero-retraction | 50% | 0 | `6af714a60567aa19b4ebd72e340aab3d352c118ade24b19a0e04383dbefd5ede` |

The three exact pure3MFs above are the proposed public Native-first RED set:
2realistic divergent positive cases +1ordinary nonzero-retraction control per
mode. Keep inherited-max range evidence external, no additional huge fixture.

## Included / deferred

Included only description forwarding in the already reached zero-E wipe XY
branch under the existing comment option, without altering whether E is
omitted or any movement/state. Retain positive-E descriptions and comment-off
suppression. Per-option saved/applied values and actual wipe motion are required
Native witnesses, not merely requested toggles.

Deferred: wipe geometry/distance range/clip algorithms, epsilon or retraction
rounding, negative/overflow schema policy, percent parser/clamp changes, firmware
retraction, role-based wipe speed, toolchange/final wipe ordering, multi-filament
mapping, travel-Z, cooling, adaptive PA models/scanner/reset and parent range
coverage. Discovery stopped at first diff; unrun range cases are not PASS.
No extra unsupported guards or implicit off-switch are authorized.

## Native RED -> GREEN acceptance design

1. After SPEC approval commit tests + input-only fixtures first. Use
   **OrcaRunner::from_env**, required-Native semantics and existing stages /
   public byte API + real CLI, not the external exploratory helper as a new
   committed harness. Retain fresh native result/launch/full output at runtime,
   actual effective wipe/percentage/length/comments settings and input SHA.
2. Native-first witnesses require actual WIPE_START/END, positive XY movement,
   zero-E rows plus retained description for the two positives; ordinary control
   must actually extrude/retract during wipe. Complete existing comparator then
   yields2true FAIL+1PASS per mode; no rejection-as-success or comment filtering.
   Independently review exact public RED before any production edit.
3. Only the minimum source-comment forwarding change in the existing zero-E
   branch, no surrounding cleanup or threshold/math repair. If unchanged
   fixtures still show another first difference, stop/report for amendment.
4. Same-hash directed official+seeded complete Native/API/CLI parity and raw
   API/CLI equality; control raw bytes unchanged RED->GREEN. Preserve existing
   wipe/retraction/comment-off/travel and relevant PA/cooling controls plus
   reached option_coverage. If comment-off needs a new witness, freeze it before
   code rather than add implementation-following tests afterward.
5. CPU18/jobs2/threads2, serial fresh modes/cache/artifacts/explicit owned target,
   exact full HEAD/clean status before+after legs, actual descendant/thread mask,
   one START per batch then evidence END/review (no per-case inbox noise).
   Obey exclusive FULL freezes. Same final hash strict/fmt/WASM, <=400LOC;
   full/workspace only coordinator allocation, no push or Nix store deletion.

Only this document is committed now. Await exact SPEC approval before RED.
