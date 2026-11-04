# Adaptive PA residual boundary continuation — APA-R2 SPEC

Status: accepted.
Parent: [ARD-0104](ard-0104-adaptive-pressure-advance-port.md).
Upstream pin verified: OrcaSlicer 2.4.2
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Execution order is M→A→R, test-first. The producer amendment below must be
committed before its tests/runs.

## 1. Rewrite boundary and non-negotiable decisions

Port/verify remaining **GCode / AdaptivePAProcessor source behavior**, not a new
Ares pipeline. The existing `ares-core::project_slice::gcode_emit` is a temporary
compatibility shell around these upstream concepts. Keep platform-neutral core
bytes/state; CLI/test adapters own files, tools and evidence. No core I/O/env,
new orchestration framework or independent PA policy.

Terms must stay distinct: `ExtrusionMultiPath` is the standalone entity at
`GCode::extrude_multi_path`, not every segmented loop. A sloped loop average
uses the pre-slope paths; suppression state is set after each `_extrude` call,
not necessarily after emitting a marker. `resetPreviousPA` overwrites one
processor-wide last-predicted value, **not** a per-tool cache reset and not a
reset of feedrate/last-tool state. Calibration mode is `Print::calib_mode`,
not a JSON setting named by this work or the calibration-model string.

Already landed R1 `af7398b78` proves a uniform-width, same-role, no-tower sloped
loop and controls; its production delta was comment forwarding, not an invented
suppression repair. G1 `b1b958e17` closes only its contextual bridge witnesses.
Their complete-output assertions and narrow official ND rulings remain intact.
Neither proves heterogeneous-flow standalone multipaths, sloped role changes,
calibration-mode exemption or all five source reset sites. Existing fields or
static code differences are hypotheses, never implementation RED.

## 2. Source-to-Rust map (paths relative to `src/libslic3r/`)

| Source / line range | Exact included or deferred behavior / Rust owner |
| --- | --- |
| `GCode.cpp::extrude_loop`, 5777–5806,5900–5973 | Eligibility/clipping; average pre-slope non-force-no-extrusion lengths/flows; suppress after each ordinary/sloped `_extrude`. Included residual heterogeneous-flow, mixed-role and ramp interaction. Existing `gcode_emit/motion/loop_paths.rs`, `loop_paths/adaptive_pa.rs`, `motion/state.rs`; reuse scarf geometry unchanged. |
| `GCode.cpp::extrude_multi_path`, 6038–6066; `extrude_entity`,6091–6100; `extrude_path`,6103–6108 | Standalone entity average excludes force-no-extrusion paths; reset average/suppression once, set suppression after every child; subsequent single path clears both. Included only when live exported geometry actually reaches this entity. Rust `motion/perimeter/multi_path.rs`, `motion/materialized.rs`, `motion/state.rs`, existing loop mean helper if source-compatible. No new synthetic entity pipeline. |
| `GCode.cpp::_extrude`,6722–6742,6816–6841 | Initial evaluation requires adaptive+PA, excludes Line/Pattern/Tower calibration, suppresses multipath repeats then permits role change; positive multipath average takes precedence over individual flow. Normal-print residual included in `motion/path/adaptive_pa.rs::{tag,marker}`, `motion/path.rs`. Calibration exemption inventoried/deferred pending authentic mode admission below. |
| `GCode.cpp::_extrude`,6925–6975,7157–7200 | Constant ramps require `!evaluate_adaptive_pa`; variable ramps do not. Both require positive flow and PA/adaptive/overhang activation; variable uses `EXTRUDER_CONFIG`, initial/constant uses `FILAMENT_CONFIG`. Preserve source-inverted variable debug labels, RC/OV fields, tag-before-feedrate ordering and EPSILON. Included normal-print interaction in `motion/path.rs`, `path/variable.rs`, `path/adaptive_pa.rs::ramp`. No blanket calibration-off policy: these sites lack the initial branch's `!is_pa_calib` predicate. |
| `AdaptivePAProcessor.cpp`,24–42,106–213,214–295; `.hpp:55` | Per-used-tool model activation; feedrate lookahead/rewind; static fallback vs initialized-model branch, debug output, changed-tool/duplicate threshold. Reuse `gcode_emit/adaptive_pa.rs`, `adaptive_pa/scan.rs`, `model.rs`; include reached lookahead boundary semantics and reset state, not line-for-line C++ stream machinery. Do not force scanner rewrites without output RED. |
| `GCode.cpp`,2816,3717–3721,3748–3755,3817–3821,3846–3853 | Processor constructed with finalized tools. Ordinary pipeline: generator → optional equalizer → cooling → fan → PA → output; spiral branches omit PA entirely. Included activation/order controls in `gcode_emit.rs`, `adaptive_pa.rs`, existing `fan_chunks.rs` / `cooling.rs` call sites only. |
| `GCode.cpp`,2542–2546; `GCode/PressureEqualizer.cpp`,174–202 | Positive slope enables equalizer/role markers; first input produces NOP, previous layer emitted on next call, final NOP drains. Included **PA consumer timing and gate interaction**, not equalizer algorithm rewrite. Rust `project_slice/pressure_equalizer/`, `gcode_emit/cooling.rs::{process_layer,finish}`, `gcode_emit.rs`. |
| `GCode.cpp`,1088–1094,1369–1375 | `WipeTowerIntegration` first/second append sites set static PA then reset previous PA; second has valid-tool guard. Type2/second site may be included only via an already supported, actually tower-emitting witness; first/Type1 remains deferred. Rust `gcode_emit/wipe_tower/emit.rs`, `pressure_advance.rs`, `adaptive_pa.rs`. No tower activation/planning/geometry repairs. |
| `GCode.cpp`,3143–3161 | BBL initial-non-support filament restore: start G-code/VT marker, enabled static PA, reset before layers. Included reached startup case. Rust `layer_gcode/startup.rs`, `machine/first_filaments.rs`, `pressure_advance.rs`, `adaptive_pa.rs`. Not generic "reset all printers at start". |
| `GCode.cpp::set_extruder`,7710–7748,8047–8053 | Single-writer-extruder branch restores/reset before writer toolchange; multi-tool tail does so after filament-start and ooze-prevention. Included non-tower lifecycle, PA enabled/disabled and repeated-tool visits. Rust `tool_transition.rs`, `pressure_advance.rs`, `motion/state.rs`, `adaptive_pa.rs::Processor::reset`. |
| `calib.hpp`,17–30,34–49; `Print.hpp`,1087–1089; `Print.cpp`,423–426; `GCode.cpp`,2915,3072,3246,4692–4704 | Actual Print calibration mode/params and calibration output generation. Explicitly deferred API/admission/generator slice, not a model-string or option-toggle proxy. Future Rust target is `ares-core::print` typed source-owned Print calibration context, propagated to `project_slice/gcode_emit/motion/path/adaptive_pa.rs`; no invented JSON/env flag or GUI dependency. Separate slice required. |

## 3. Included first execution tranche

Three independently attributable witness groups, executed serially. Preserve all
existing PA tests; new cases use the current public exported-3MF/full-output seam
and unchanged `OrcaRunner::{export_case,slice_case}` / `compare_case`.

### M — standalone multipath / residual sloped suppression

First execution uses the existing deterministic admitted geometry and the
landed nonconstant multi-acceleration model in
`ares-cli/tests/orca_parity/adaptive_pressure_advance/residual_witness.rs`.
For **sloped/loop cases**, fixed baseline is classic, ordinary non-vase,
comments1, PA+adaptive1, no tower; bridge override0. Freeze complete numeric
settings and input hashes in the RED request before comparison, reusing
source-admitted fixture values.

### M producer amendment — standalone case only

Source tracing established that `PerimeterGenerator.cpp:370::traverse_extrusions`
is the Arachne producer and its open entities become `ExtrusionMultiPath` at
`555–566`, dispatched through `GCode.cpp:6092–6095` to `6038::extrude_multi_path`.
Classic thin walls instead call `variable_width` at `PerimeterGenerator.cpp:
231–233`; `VariableWidth.cpp:216–233` emits open children as individual paths.
**Only the standalone M producer context** changes from classic
to this already-admitted Arachne fixture:
`tests/parity/validation/arachne-open-wall-order/C0-neck0.25-r0.3mf`, SHA256
`6c5c67976071ea79e663b8a4d428f3f51202b3fb9821a49fbe6b9da1f399a07d`.
No new mesh, geometry edits, seam/open-wall production changes or cached-entity
JSON producer. Keep the original geometry/options and modify only PA-related
settings at runtime, reopen/hash/verify the actual repacked input and unchanged
non-config ZIP members. The sloped/loop producer remains classic.

Before the active case, commit and run **PA-off baseline control** with
`enable_pressure_advance=0` and `adaptive_pressure_advance=0` (preserving all
other original settings). Its official Native/Ares complete-output comparison
must PASS; if it differs, retain raw input/channels/products/first-difference
and report the Arachne geometry baseline mismatch separately, **stop this subcase**.
Do not repair geometry here or call that mismatch PA RED. Only after that
control admits the producer, commit the active PA tracer before running it.
Both variants must keep identical geometry and non-PA settings. A subsequent
implementation RED must be attributable to the PA activation delta and cited
PA sites, not an incidental geometric difference. Native runtime must still
prove standalone multipath and heterogeneous positive flow; Arachne option
retention alone does not admit this branch.

Request Native branch evidence for standalone `extrude_multi_path`: actual
entity/children, roles, force-no-extrusion flags, lengths/mm3_per_mm, weighted
average and suppression entry/exit, plus ordered tags/feeds/moves. Pair PA-on
with adaptive-off on identical geometry. Separately pair scarf-none/all with
adaptive-overhang0/1 on geometry Native proves has multiple flows or a role
change inside the sloped loop. The ordinary R1 uniform same-role cube is a
regression control, not sufficient residual activation. Require positive real
extrusion and role/flow observations; merely changing options is not coverage.
If no existing public geometry reaches a required entity/role, stop as missing
witness and request a fixed manual-mesh fixture scope; do not inject artificial
ExtrusionMultiPath/PA_CHANGE objects into isolated tests or alter slicer geometry
production to manufacture reachability.

### A — remaining activation / downstream-speed consumer

Retain old PA-off/adaptive-off/invalid-model cases. Add only an admitted
ordinary positive-overhang/ramp case and its option-off control, a real spiral
control (source pipeline omits PA), and a positive equalizer-slope case with its
zero-slope control using the same valid nonconstant model. Bind actual finalized
logical tools and saved/applied option values. Native must show which constant
or variable trigger was reached, current/next/max speed after equalizer/cooling,
and complete last-layer output including the delayed equalizer drain. Positive
slope retained in JSON alone is not activation; show modified feedrates and
buffer/drain execution. Diagnose lookahead across feed changes, RC1, wipe,
post-extrusion travel and final-layer EOF only where real output reaches them.
No invented expected prediction, blanket zero-PA assertion or equalizer policy.
Mixed logical tools preserve FILAMENT_CONFIG versus EXTRUDER_CONFIG ownership;
if the admitted mapping cannot expose the distinction, report it unverified.

### R — source-specific reset sequence, not layer-only equivalence

Use existing J1 non-tower two-tool exported witness as baseline, replace its
constant models only in a new case with the already-admitted nonconstant model
so prediction differs from static PA. Require repeated A→B→A visits within a
layer, distinct static values using existing admitted 0.035/0.055, and mixed
adaptive1/0 plus PA-off controls. Native source trace must bind each reached
reset site, logical filament, value, call order and subsequent processor
previous/tool/speed observations; whole G-code remains authoritative. Include
an admitted BBL initial-non-support restore witness; do not assume J1 startup
exercises `3160`. Existing Type2 tower positive-control geometry may be reused
unchanged for `1374` only if live Native/Ares products actually emit the tower
and reach that call. An enable1/no-tower product cannot certify tower resets.

Crucial uncertainty: resets execute in generator/startup code while PA is a
later serial filter within an upstream pipeline (12 tokens), with optional
one-layer equalizer delay. Rust currently queues `(layer,pa)` and applies all
eligible resets before processing a layer (`adaptive_pa.rs:87–110,130–138`).
This is not proof of equivalent source scheduling. Nor does source call-site
text justify relocating resets to each final-byte toolchange. Measure source
lifecycle first; do not preselect a generic ordered-event framework, inline reset
marker protocol, per-tool previous cache or arbitrary byte-offset replay.
Debugger scheduling is diagnostic and may perturb ordering; retain uninstrumented
official outputs as acceptance evidence. If reset effects are inherently
nondeterministic, stop and report rather than choose favorable schedules.

## 4. Deferred / stop barriers (parent OPEN remains)

- Calibration Line/Pattern/Tower exemption and generation are an **admission
  blocker**, not closed by normal mode controls. Source GUI callers set actual
  Print params (`slic3r/GUI/Plater.cpp:12644,13215`); inspection of
  `OrcaSlicer.cpp` found no corresponding calib_mode/set_calib entry.
  Current Ares public project request likewise carries no Print calibration
mode. Need an authentic Native public/API producer and a separate
  source-owned Rust request/context spec before those E2Es. Never set private
  mode through GDB, add a test-only bypass or fake a 3MF calibration option.
- Type1/first append reset `1093`, unsupported tower integration and any tower
  activation/geometry delta; Type2 source-specific PA consumer may be measured,
  but an unrelated tower failure is retained/referred, not fixed in APA-R2.
- PressureEqualizer constructor/rate/segment algorithms beyond reached PA
  interaction; calibration hardware diagnostics; exhaustive models, printers,
  multi-hotend remapping and all Option/vector domains. No full650 option scan.
- Any force-no-extrusion/role/stream boundary not actually reached remains
  enumerated unverified. Do not close all of `2863ef3ea` after one group passes.
  ARD-0104's older "not ported" prose must be reconciled to measured residuals
  in the eventual completion record, without deleting open obligations.

## 5. RED → GREEN admission / acceptance contract

Commit test/input changes first. For each group,
record Native admission/reachability and preserve the first successful fresh
Native output and original public Ares complete-output comparison. Only a real
same-input, source-localized difference is RED; export rejection, unreachable
branch, infrastructure/compiler failure, cancellation, timeout or absent product
is not parity RED. If already equal, report verification GREEN/test-only coverage
without inventing RED or unnecessary production edits.

Retain runtime input ZIP/member/applied-setting hashes, binary/source identities,
argv/env, stdout/stderr/results/G-code, exits, timeout/cancellation status,
and diagnostic source-site/tag offsets outside Git. Official and seed-only
Native are independently identified with fresh per-leg caches and separate roots;
no frozen Native substitution. Use centralized existing permitted normalization
only; never strip PA/debug/ramp/movement lines or relax expected/tolerances.
Optional Native absence visibly SKIP; required absent/invalid/nonzero/timeout/
missing product FAIL. Existing ND rulings apply only to their original cases.

Only after source-localized RED, implement minimal upstream deltas in the
mapped private Rust modules. Keep <=400 LOC touched modules; no isolated
implementation-following tests after code. Verification requires active directed
Native old+new assertions, separate BASE/CAND CLI and test binaries, seeded and
official same-input legs, named failure-subset audit. Production change requires
unfiltered standard `orca_parity` full controls, workspace Nextest, strict Clippy,
rustfmt and WASM.
Do not edit test trees mid-run, retry until PASS, ignore failures or force push.

Execution: **APA-R2 bounded M→A→R witness-first execution**.
Calibration-mode/API and first/Type1 tower reset remain
deferred; production repair still requires genuine RED localization. The M
producer amendment requires PA-off admission before its active tracer. Parent
The parent remains open until its separate deferred obligations have admitted
evidence.
