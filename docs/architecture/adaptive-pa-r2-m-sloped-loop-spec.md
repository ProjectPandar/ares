# APA-R2 remaining M: classic sloped-loop PA context — SPEC

Status: **SPEC-REVIEW REQUEST, no execution/implementation authorization**.
Parent: `docs/architecture/adaptive-pa-deferred-r2-spec.md`, ARD-0104,
roadmap OPEN `2863ef3ea`; execution order **M → A → R**.
Analysis CAND: `d3f599ed5e07e3176d3cce98d8b67afbd99a46fd`, BASE
`5d79a99632fbe0157e40228f42a0adfc53467438`.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
This spec is stored outside the reviewed candidate to preserve its exact hash
while it queues full. Intended durable repository destination after approval:
`docs/architecture/adaptive-pa-r2-m-sloped-loop-spec.md`, docs-only commit
before new tests on the next slice. No current candidate code/test changes.

## 1. Non-negotiable architecture / rewrite slice

Port/verify `libslic3r::GCode::extrude_loop`'s loop PA context and
`GCode::_extrude`'s role-change override. Do not design an Ares-owned pipeline.
Existing `ares-core::project_slice::gcode_emit::motion` modules are temporary
compatibility shells around the cited upstream GCode concepts. Core consumes
platform-neutral typed state/bytes; adapters own exports, files, processes,
receipts and reports. Tier1 browser WASM/Windows/macOS/Linux boundaries stay.

A standalone `ExtrusionMultiPath` is not a sloped loop. Engineering-approved
standalone mean candidate d3f (+15/-2) does not close this residual or authorize
another production repair. Existing R1 uniform same-role slope is a control,
not heterogeneous-flow or mixed-role loop coverage.

### Exact source / Rust destination map

| Upstream boundary | Included source behavior | Rust boundary / permitted action after RED review |
| --- | --- | --- |
| `GCode.cpp:5777–5806::extrude_loop` | Ordinary non-vase outer/perimeter loop after layer0; optional conditional slope gate; slope uses clip_length0 before building slopes while ordinary loop may apply seam-gap clip. Empty clipped path set returns. | `motion/loop_paths.rs::emit`, `motion/scarf.rs::build`; **eligibility/clipping are frozen**, only verify admitted behavior. Do not repair scarf geometry. |
| `GCode.cpp:5893–5900` | Small-perimeter override applies perimeter roles, not bridge/overhang roles; leave source-observed speed/role lag unchanged. | `motion/loop_paths.rs` speed_for_path; verify only, no speed-planning repair. |
| `GCode.cpp:5902–5917` | Reset suppression false and mean0; compute sum(raw mm3_per_mm×unscaled length)/sum(length) over already clipped **pre-slope** paths excluding independent force-no-extrusion flag. | `motion/loop_paths.rs::begin_adaptive_loop`, `loop_paths/adaptive_pa.rs::mean`, `motion/state.rs` flow/emitted fields. Reuse existing mean; no ratio-scaling of entity mean or new arithmetic helper. |
| `GCode.cpp:5920–5929,5951–5966` | Ordinary or starts→flat→ends children each call `_extrude`; suppression true **after every child call**, not only when marker/positive flow emitted. | `motion/loop_paths.rs::emit`, existing materialized emission seam; a static difference is a hypothesis until reached output RED. No unconditional patch at SPEC stage. |
| `ExtrusionEntity.cpp:426–525::ExtrusionLoopSloped` | Subdivision/slope construction copies parent path attributes; `get_all_paths` returns starts→flat→ends. `526–533::clip_slope` clips end and front. | `motion/scarf.rs`, `scarf/emit.rs`; frozen geometry producer, used only to bind loop ownership. |
| `GCode.cpp:6722–6742::_extrude` | Initial adaptive+PA evaluation is suppressed after prior child, then an actual role change overrides suppression. Calibration exclusion is source-owned actual Print mode. Upstream TODO notes overhang speed/flow can precede role label; preserve ordering. | `motion/path/adaptive_pa.rs::tag`, `motion/path.rs`; bounded normal-print suppression/role context only. No invented evaluation policy. |
| `GCode.cpp:6816–6841` | Positive raw entity mean takes precedence over individual ratio-scaled `_mm3_per_mm`; marker contains used tool/ACCEL/BR/RC/OV and precedes feed/motion emission. | `motion/path/adaptive_pa.rs::{tag,marker}`; mean selection/order only, no formatter/model/rounding changes. |
| `GCode.cpp:6925–6975,7157–7200` | Constant ramp requires !evaluate_adaptive_pa; variable ramp does not. FILAMENT_CONFIG vs EXTRUDER_CONFIG differ. Preserve inverted variable debug labels and EPSILON. | `motion/path/adaptive_pa.rs::ramp`, `path/variable.rs`; **adjacent A ownership**. Observe reached interaction in M matrix but no independent ramp/activation repair before separate A RED/SPEC. |

Rust inspection: current scarf branch invokes begin_adaptive_loop with original
paths and clip0, clears context after loop; ordinary branch passes seam-gap clip.
`tag` currently sets emitted based on mean presence when it emits. Source sets
suppression after every child. These facts nominate a verification boundary;
they are not runtime RED and do not justify adding suppression assignments now.

## 2. Offline fixture inventory and requested producer scope

Artifact: `${TMP_ROOT}/ares-adaptive-pa-r2/M-sloped-offline-fixture-inventory.json`
contains original ZIP member hashes/settings. No Native/export/compile executed.

- R1 `tests/parity/cube10.stl` +
  `adaptive_pressure_advance/residual_witness.rs` is already admitted uniform
  same-role slope. Keep unchanged.
- `seam-overhang-nonuniform-height.3mf`, SHA
  `699e9effa47a72c0de5292532aed8b1966813705f55fd0b4b22fdca661b04ee6`, is
  **arachne**, scarf none/comments0/PAoff. Its prior seam-height parity evidence
  does not admit a classic mixed-role sloped loop. Excluded from this tranche;
  changing wall generator would silently change its geometry context.
- `overhang-jerk/project.3mf`, SHA
  `c46dc04b20a64772a75a292de0b03d3e292b7ac42192466eeb9a57e53b79186c`, is
  classic but tower enable1, wall_loops2/seam_gap10%/comments0. Not an admitted
  no-tower PA fixture. Do not toggle unrelated settings to call it admitted.
- **Requested bounded producer:** existing `tests/parity/overhang.stl`, SHA
  `a7f79fdf8a5c136095b6ef2384de24dcaa3166c7df4e92768a434c454db55031`,
  unmodified vertices/mesh. Export using exactly the existing R1
  `settings()` + process/filament override table in `residual_witness.rs`,
  replacing only the mesh argument cube10→overhang for the new test. This is
  a proposed reuse/admission scope, **not runtime reachability evidence**.

Approval requested covers this one existing mesh and R1 option context, no
manual mesh creation, shears, mesh rescaling, generator substitution, new
printer context, unsupported tower producer or synthetic entity injection.
If export or whole-output negative control fails, preserve and refer to owner;
stop M. If Native cannot prove heterogeneous flow or real mixed roles in a
single classic sloped loop, stop as missing witness, not PA RED. A new manual
mesh/producer requires a separate amendment; no unbounded probe sweep.

## 3. Test-first, frozen option/input contract

After approval and docs commit, commit a new separate E2E module before any
production change. Do not edit R1 assertions or current standalone off/active
tests. Use established private runner export/slice/compare seam; no public field
visibility expansion or isolated implementation-following unit tests.

Copy R1's exact option values, including classic/wall_loops1, layer/initial0.2,
outer_width0.4, scarf conditional0/inner0/entire0/start0%/min10/steps4/speed100%,
seam_gap0/nearest, comments1/non-vase, infill0/top0/bottom0/support0/tower0,
arc-fitting0/brim-none/skirt0. Static PA0.035, PA enabled1, bridge override0,
landed nonconstant multi-acceleration CSV unchanged:

```
0.015,1,100
0.045,5,100
0.09,20,100
0.02,1,1000
0.08,5,1000
0.12,20,1000
0.035,1,5000
0.1,5,5000
0.2,20,5000
```

Pin source mesh bytes in test. First export freezes **complete** effective
machine/process/filament settings, 3MF bytes and every member hash in evidence;
verify exported saved/applied values. Different matrix inputs may change only
`seam_slope_type`, `adaptive_pressure_advance`,
`adaptive_pressure_advance_overhangs`. Prefer rerunning repacked frozen export
rather than allowing exporter-generated IDs/member content to vary between
controls. Reopen each runtime ZIP and assert every nonconfig member and every
other setting unchanged; do not invent unknown input SHA before actual export.

### Bounded six-context matrix (each in official and standard seeded modes)

| Context | scarf | adaptive | adaptive-overhang | Purpose |
| --- | --- | --- | --- | --- |
| N0 | none | 0 | 0 | PA-enabled static, no slope negative geometry/control admission first |
| S0 | all | 0 | 0 | Sloped geometry control; must have rising-Z start + flat/end extrusion |
| N1 | none | 1 | 0 | Ordinary heterogeneous/mixed-role loop mean/override context |
| S1 | all | 1 | 0 | Sloped initial mean/same-role suppression/role override without extra ramps |
| N2 | none | 1 | 1 | Ordinary ramp-interaction comparison; not independent A closure |
| S2 | all | 1 | 1 | Sloped role/suppression vs ramp interaction; preserve source ramp ownership |

All controls compare whole outputs, not PA-only streams. N0/S0 must pass before
classifying any active divergence as M RED. If an active context already
matches, report verification GREEN, do not invent RED or apply an unnecessary
production fix. R1 three existing uniform cube witnesses remain unchanged
regression controls. No preset/domain/sweep registration changes in this slice.

## 4. Admission evidence: source-owned, observable and honest

Use actual uninstrumented required-Native outputs and executed Slice receipts.
Capture `PA_CHANGE`, `APA` debug and actual flavor-owned PA commands without
stripping any from acceptance. Bind actual finalized tool, used ACCEL, BR/RC/OV,
flow/feed sequence, layer, XY/E extrusion and slope rising-Z/flat/end blocks.

Distinct requested proofs:

1. **Sloped branch:** source eligibility + layer>0 + measured rising-Z positive
   extrusion and subsequent flat/end blocks, not JSON scarf=all alone.
2. **Heterogeneous mean:** source-admitted single loop with >1 positive raw
   flow, not different speeds/flows across unrelated loops and not inference
   from E/distance alone (slope E ratio, filament/print ratio can confound it).
   Mean is pre-slope raw value; slope-varying E does not itself prove different
   original path mm3_per_mm.
3. **Mixed role/suppression:** real source role transition within the same loop,
   not dummy RC1 ramp tag. Initial tag first, later same-role children
   suppressed, actual role-change tag permitted; map starts→flat→ends and the
   BR/OV/TYPE/feed ordering using the cited copying/dispatch boundary.

Full final output may not expose original raw path count/length/flow/ownership
unambiguously. If comments/debug plus known source producer cannot distinguish
one loop from several, mark that proof **UNVERIFIED**, stop admission and request
an approved evidence seam or fixture amendment. Do not use GDB/private mode
injection, instrumented substitutes for final output, hidden-register release
requirements, invented assertions or synthetic caches. Owner discontinued GDB;
that ruling remains in force. Frozen geometry identity is necessary but not
sufficient runtime path admission.

## 5. RED localization and permitted repair envelope

Preserve first complete same-input Native/Ares mismatch with raw inputs,
nonconfig-member/settings hashes, request/producer receipt, binary hashes,
argv/env/CPU, captured stdout/stderr/status, raw streams, run IDs and exact first
difference. Native actual exit0/output is required. Export rejection, compile
failure, unreachable branch, missing product, timeout or SIGTERM is infrastructure
or admission blocker, never parity RED.

Before repair review, show the whole-output delta is causally PA activation:
negative geometry controls exact; geometry/XY/E/feed differences must be
separated and referred, not patched in this slice. Compare role/flow tags
against upstream initial mean/suppression/role semantics, not expected numeric
prediction picked to force GREEN. If divergence belongs to scarf geometry,
small-perimeter speed, processor, formatter, ramp A or reset R, stop and re-scope
with owner, do not grow this M repair boundary.

After genuine localized M RED approval, the only potential production owners
are loop PA context initialization/after-child suppression and initial marker
mean/role override in the existing mapped motion files. Preserve raw mean/E
ratio distinction, geometry, tool/role order, wipe and context teardown. No
blanket “disable PA on slopes”, no per-tool previous cache, no zero-flow
force-no-extrusion heuristic, no calibration JSON/env flag, no new framework,
API/options, model/PCHIP/rounding/formatting patch or comparator relaxation.
Touched Rust modules <=400LOC; split only if required, no unrelated cleanup.

## 6. Adjacent behavior explicitly deferred

- Independent force_no_extrusion representation/exclusion: Rust path has no
  equivalent flag. Zero mm3_per_mm is not the flag. If admitted witness needs
  such a path, stop and request separate source-owned representation spec.
- Conditional scarf/smoothness thresholds, spiral producer/PA omission,
  variable-ramp FILAMENT vs EXTRUDER ownership, cooling/equalizer/delayed drain
  are A or adjacent geometry, not closed by this matrix.
- Actual Print calibration Line/Pattern/Tower mode and authentic public/API
  admission remain deferred; normal mode/model-string is not proxy coverage.
- R source reset timing, repeated-tool cache semantics, Type1 tower and tower
  activation/geometry remain deferred. Do not infer closure from tag equality.
- Existing ND rulings remain limited to original domains. No new broad ignore,
  comparator/tolerance or parity-semantics changes.

## 7. Exit gates and stop conditions

SPEC review may approve **bounded producer admission/test-first execution** only;
there is no claimed runtime RED or implementation authorization now.

- Admitted producer and same-loop heterogeneous/mixed-role/slope proof, or
  explicit missing-witness report with no fake coverage.
- Six contexts official + `${TMP_ROOT}/ares-parity-control-set-v2.sh seeded` standard
  mode with fresh caches/roots and actual Slice receipt/binary policies. Correct
  notification mode labels and verify mode independently from receipts. Native
  optional absence SKIP is not proof; required absence/nonzero/timeout FAIL.
- Preserve original `golden::compare_ordered_bytes_generator_only`; entire
  output assertion + permitted generator normalization only. Source-admission
  extraction is supplemental, never an acceptance comparator.
- If production changes: authentic test-first RED review→localized GREEN,
  workspace Nextest/strict Clippy/fmt/WASM, four-leg standard BASE/CAND full and
  independent bounded group REVIEW. If already equal: test-only verification
  review and explicit owner full-exemption ruling if applicable.
- If option inventory/domain plan/sweep registration is unexpectedly required,
  stop for scope approval; any approved registration delta additionally requires
  entire option_coverage group including nonNative `option_coverage::tests::*`.
- No heavy during offline SPEC work; no editing tests during runs. Directed
  load<60, engineering load<30/jobs2threads2/nonpinned; pinned only seeded Native
  replay directed. Honor HOLD/browser and exact exclusive-leg protocol/current
  coordinator rulings. No retry-until-PASS, stash/force push or premature full.
- Current d3f standalone slice remains queued W3brim→W5inherits→W1; separate
  next-slice docs/tests do not mutate its approved candidate. Production main
  advancement VOID gates and requires fresh rebase/gates; test/CI-only advances
  carry only under owner ruling with range-diff. Exact approval before FF.

Completing admission or a bounded repair here does not close parent APA-R2,
ARD-0104 or roadmap OPEN2863. M residuals close only to the actually measured
included domain; A/R/calibration/unsupported flag obligations remain explicit.
