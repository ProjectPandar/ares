# ARD: FanMover mixed-precision timing rewrite

Status: authorized docs-only SPEC, then input-only formal RED; STOP before
GREEN without review. Independent base3fbad615e334a283596d0e6acff921f1d02804c6.
This eighth W5 chain follows the seven frozen approved chains at integration;
none is stacked, rebased or modified here. No FULL/integration authorization.

## Upstream owner and exact Rust boundary

OrcaSlicer2.4.2 libslic3r/GCode/FanMover.hpp:
- BufferData.time: **float**, line19; constructor float time at24.
- FanMover.m_current_speed: **double**,46, initial1000/60.0.
- m_buffer_time_size: **double**,57; delay/kickstart remain float.
- put_in_buffer adds stored float item time into double accumulation,78–80.

FanMover.cpp::_process_gcode_line279–305:
- local double time at284;
- line.f()/60.0f computes float feed conversion then promotes into double
  m_current_speed at288;
- float reader dist_X/Y/Z results first promote into double distx/y/z
  at297–299; squared sum/sqrt/division occur in double at300–305.
- BufferData(line.raw(),time,fan_speed) at442 narrows double local time
  to float storage. Do not globally promote BufferData.time.
- _print_in_middle_G1 at171–212 takes float nb_sec, computes float percent
  and float split coordinate/E products, preserving fixed precision output.
  Caller double buffer subtraction narrows at this float argument boundary.

Rust destination ONLY
`crates/ares-core/src/project_slice/gcode_emit/fan_mover.rs` and
`fan_mover/motion.rs`: timing field/local types, arithmetic order, and
explicit promotion/narrowing at the source's boundaries. Current fields
buffer_time_size66/current_speed71 and process_line local time137 are f32;
motion.rs19–54 computes f32 norm/division too early.

Non-negotiable future GREEN boundary: mixed precision, NOT blanket f64.
Keep reader subtraction/position/deltas float, then promote deltas before
squares/sum/sqrt and divide by double speed. Float feed conversion precedes
promotion. Accumulate stored float item times into double buffer duration;
narrow at the upstream BufferData.time and float split-call boundaries.
Preserve float split percentage/products and existing E5/XY3 output.
No downstream formatter/tolerance/special-case E correction.

All other files/behavior DEFERRED. Existing private float timing consumers
live in fan.rs/helpers.rs; this phase DOES NOT authorize editing them.
If a later two-file type port needs coordinated changes outside these
files to preserve upstream narrowing, STOP/report for explicit boundary
clarification, not silent scope expansion or blanket widening.

Defer epsilon/threshold policy, scheduling/control-flow/order, role gates,
arc handling, relative-E state/sentinels, kickstart behavior, synthesized
fan comment/flavor/PWM-floor context, whitespace/raw rewriting, config,
processor/viewer, global GCodeReader/GCodeWriter, seam and travel logic.
No dependency/helper/public API/new pipeline/fallback. W6 value.rs and
W4 GCode.cpp::needs_retraction remain untouched; all seven prior W5
chains frozen. Any future strict residual requires STOP/report.

## Ordinary GUI sole-key frozen actual INPUT pair

PLA220C,0.4nozzle,0.2layers,Classic2walls,15%crosshatch,3top/3bottom,
ordinary32-sided cylinder12mm diameter/6mm high/30layers. Original ordinary
prime-line machine-start macro retained; no custom injected probe. Fan
range30–70%,kickstart0,VerboseOFF(default),only-overhangsOFF so lead time
applies to ordinary transitions. No support/brim/skirt/spiral/fuzzy/scarf.
Sole GUI key fan_speedup_time **0(control)→1second(target)**.

Actual official-exported INPUT3MFs copied without post-export edits:
- control b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298
- target98149dd837288535426ca78195dce3451f24fc3dd0906e783e8526d9ae18e084

Same ZIP inventory; only Metadata/project_settings.config payload differs,
only fan_speedup_time setting differs. Freeze whole/per-entry hashes and
input provenance/README; no expected G-code snapshot/Native result constants.

## Causal discovery and acceptance

Native-first official1/official2/seeded stable: whole controlPASS all3,
target2 strict non-M73 E differences at3242/3244, same XY. The two split E
amounts redistribute one E5 quantum and retain equal sum; no exaggerated
material/physical-error claim. Full ordered-byte strict parity is the goal.

Live read-only captures:39 buffered raw records/dx/dy/de bits identical;
15 stored durations differ1ULP. Native double buffer1.0275561378803104 vs
Rustf32 1.0275554656982422. Actual Native float nb_sec0.02755613811314106;
Rust source-derived0.027555465698242188. Derived float E products straddle
E5 rounding threshold. Arithmetic replay reproduces27/27 first XY stored
durations with respective source precision. This diagnostic is NOT a
patched-production counterfactual or proof of whole-GREEN sufficiency.
Unused relative-E starting values/sentinels differ and remain deferred.
Both successful full debug outputs match CLI (Native only canonical
identity/date normalization; Rust raw equality); no inferior writes/calls.
Discovery evidence
`/home/indexyz/ares-coord/evidence/w5-3fb-fan-mixed-timing/`.

Formal RED: two DIRECT native_stream_case::check calls reuse existing
NativeCase double replay then complete validated API/bare CLI comparison.
CPU11/thread1/job1, official then seeded. Require stable Native references,
controlPASS/targetactualFAIL with both public attempts retained; no setup
failure or environment-disabled case counted. No duplicated harness,
Native output constants/assertions/snapshots, ignored/expected-failure flags.
Only existing generated-by identity/date normalization; NO M73/header/time/
fan/E/XY/comment/config/trailing-line mask, sorting or numeric tolerance.

Freeze RED tests/inputs/helpers/docs before later GREEN. Send SPEC+RED RR
then STOP. No production implementation, FULL, rebase or main update here.
