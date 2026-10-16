# APA-R2 M residual: entity suppression and pre-slope mean SPEC

Status: **DOCS-ONLY SPEC-REVIEW-REQUEST; no probing, RED or production approval**.
Base: `41f2c90f765d42c6620583121296bb61a2cb839c` (fresh branch).
Upstream pin: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Parent: [APA-R2](adaptive-pa-deferred-r2-spec.md), section M;
[ARD-0104](ard-0104-adaptive-pressure-advance-port.md).
This bounded request does not inherit the parent's historical M→A→R execution
permission. Current owner approval is required before each subsequent phase.

## Source-owned boundary / present baseline

All source paths below are under `OrcaSlicer/src/libslic3r/`.

| Exact upstream owner | Rust destination / residual to measure |
| --- | --- |
| `PerimeterGenerator.cpp:370::traverse_extrusions`, :555-566 | Existing Arachne open-entity producer; already materialized `motion/perimeter/multi_path.rs::emit`. No producer/geometry repair. |
| `GCode.cpp:6038-6066::extrude_multi_path`, :6091-6108::extrude_entity/extrude_path | Raw length-weighted mean excluding force-no-extrusion children; reset context at entity entry, suppression after each child even without a marker, single path resets context. Rust `motion/perimeter/multi_path.rs`, `motion/materialized.rs`, `motion/state.rs`. |
| `GCode.cpp:5777-5806,5900-5961::extrude_loop` | Mean computed on clipped ordinary paths BEFORE `ExtrusionLoopSloped` creation; suppression after every ordinary/sloped `_extrude`. Rust `motion/loop_paths.rs::begin_adaptive_loop`, `loop_paths/adaptive_pa.rs::mean`, existing scarf emission. |
| `ExtrusionEntity.cpp:426-525::ExtrusionLoopSloped` | Source starts/flat/ends order and force-no-extrusion witness only; existing Rust scarf producer unchanged. |
| `GCode.cpp:6722-6742::_extrude`, :6816-6841 | Same-role suppression; role change overrides it; positive entity mean precedes individual ratio-scaled flow. Rust `motion/path/adaptive_pa.rs::tag`, marker and existing path call sites. |
| `GCode.cpp:6925-6975,7157-7200::_extrude` | Only reached constant/variable ramp interaction with entity suppression; preserve source gates/RC/OV and inverted variable debug labels. Rust `motion/path.rs`, `path/variable.rs`, `path/adaptive_pa.rs::ramp`. Not general activation group A. |
| `GCode/AdaptivePAProcessor.cpp:66-295::process_layer` (:106-213 lookahead; :244-258 prediction) | Existing downstream consumer is retained for authoritative complete-output comparison. Rust `gcode_emit/adaptive_pa.rs`, `adaptive_pa/scan.rs`, `model.rs`; no speculative scanner rewrite. |
| `GCode.cpp:2542-2546,3717-3721,3748-3755,3817-3821,3846-3853`; `GCode/PressureEqualizer.cpp:174-202::process_layer` | Pipeline order / delayed layer and final NOP drain are adjacent owners, explicitly DEFERRED group A. No positive equalizer-slope activation in M. |

At this base standalone `multi_path.rs::emit` ALREADY computes the mean with the
existing helper and sets suppression after each child. `loop_paths.rs` already
initializes context; `path/adaptive_pa.rs::tag` already permits role override.
The earlier [mean repair SPEC](adaptive-pa-r2-m-multipath-mean-fix-spec.md) is
historical provenance, not proof of a current regression. No implementation
change is justified by static resemblance/difference alone. Existing typed
paths represent ordinary extrusion; mm3==0 must never substitute for the
independent upstream force-no-extrusion flag.

The emitter remains a compatibility shell around these upstream concepts.
ARD-0104 decisions remain unchanged: platform-neutral core, post-cooling/fan PA,
existing model/rounding/formatting and original comparator. No new architecture,
PA framework, public API, I/O/env in core or reset-event protocol is proposed.

## Bounded witness discovery after SPEC approval

Execute serially on CPU18, jobs2/threads2. Archive/hash the CURRENT canonical
`/home/indexyz/ares-coord/bin/parity-control-set.sh`; extract all reference and
oracle settings. Official AppDir/wrapper use `/home/indexyz/ares-coord/`, not old
/tmp aliases. Fresh independent caches/artifacts and Native binary identities
per official/seeded leg. No full/workspace runs; coordinator owns broad gates.

1. **Standalone admission and control.** Reuse exactly the parent's approved
   `tests/parity/validation/arachne-open-wall-order/C0-neck0.25-r0.3mf`, SHA256
   `6c5c67976071ea79e663b8a4d428f3f51202b3fb9821a49fbe6b9da1f399a07d`.
   Recheck hash/existence; preserve geometry and every non-PA setting, including
   original comments/flavor/flow ratios. First PA-off/adaptive-off whole-output
   control. Only if it passes, pair PA-on/adaptive-off and PA-on/adaptive-on using
   the existing nonconstant model from `adaptive_pressure_advance/residual_witness.rs`.
   No synthetic ExtrusionMultiPath or injected PA_CHANGE. Native trace must show
   actual standalone entity, child roles/flags/lengths/mm3_per_mm and raw weighted
   mean, plus suppression entry/exit and next single-path context.
2. **Residual sloped loop.** Reuse `tests/parity/overhang.stl` (existing
   `sloped_loop.rs` SHA `a7f79fdf8a5c136095b6ef2384de24dcaa3166c7df4e92768a434c454db55031`),
   the existing classic non-vase/sloped test settings and same nonconstant model.
   Freeze all applied/exported settings and ZIP/member hashes before comparing.
   Bounded six combinations: scarf none/all × (adaptive0/overhang0,
   adaptive1/overhang0, adaptive1/overhang1); PA enabled, bridge override0,
   comments1, no tower, equalizer slope0. Non-PA settings identical per pair.
   Existing cube R1 uniform same-role test remains a regression control, not
   residual heterogeneous/role-change proof. Native must show either multiple
   positive pre-slope child flows or an actual role transition within this loop,
   and ordered starts/flat/ends, suppression and initial/ramp tags.

Native uninstrumented runtime export/slice FIRST, then same-input validated API
and real CLI. Controls and candidate assertions use strict complete ordered
Native equality with only existing generator normalization and raw API==CLI.
Retain positive XY/E, applied settings, CSV, PA tags/feedrates/commands, first
full-output difference and input/binary identities. A read-only source trace
may establish hidden reachability, but never substitutes for uninstrumented
acceptance; scheduling perturbation or unavailable symbols is reported plainly.
No private calibration-mode mutation or test-only admission bypass.

## Failure inventory and stop rules

- Missing/changed fixture, unsupported export, Native error, absent G-code,
  compiler/wrapper/environment failure, timeout/cancel: NON-GATE, never RED.
- PA-off geometry mismatch: stop that producer and refer to its owner; do not
  repair Arachne/scarf/ordering here or call it PA suppression divergence.
- Options retained but no standalone entity / heterogeneous flow / role change:
  missing reachability; report exact unverified domain. No new mesh or geometry
  changes without an approved source-cited fixture amendment.
- Native force-no-extrusion child not representable by current typed boundary:
  STOP for representation amendment; no zero-flow heuristic or silent exclusion.
- Mean measured after slopes, E-scaled instead of raw, wrong used acceleration,
  CSV or tool binding: localization fault; preserve evidence, do not patch model
  rounding/writer precision to force a command match.
- Same-role children may suppress initial tags while role changes re-enable them;
  never assert one-tag-per-loop indiscriminately or remove ramp/debug comments.
- Earlier/later consumer, resets, equalizer timing or ND controls dominate the
  difference: stop and request A/R scope; do not hide it with comparator rules.
- Every reached input already matches: evidence-only NO-RED STOP; no passing-only
  repair submission, fabricated RED, inverse assertions or gratuitous production.

## Phase gates, included / deferred

After SPEC approval only: bounded discovery. A genuine same-input difference
must be localized to one included M source boundary before requesting public
RED authorization. Commit input-only fixtures/tests before implementation; at
runtime Native runs first. RED asserts TARGET equality and fails while the bug
exists, never passes merely because divergence is detected. Controls assert the
same strict equality. Require independent RED approval before minimal GREEN.

GREEN may replace only the demonstrated M compatibility-shell gap in the mapped
private modules. Keep approved tests/inputs/model/comparator unchanged, compare
same-input controls raw bytes RED→GREEN, preserve original failure artifacts.
Exact clean final hash, changed Rust <=400 LOC, allocated targeted config/PA
regressions, fmt/strict/WASM; broad tests only by coordinator authorization.

Included: ordinary standalone entity mean/suppression/context transitions and
reached heterogeneous or mixed-role sloped M interaction. Deferred: general
activation/downstream speed/lookahead group A, positive equalizer/drain experiments,
all reset sites/group R, calibration Line/Pattern/Tower admission, tower planning,
new force-no-extrusion representation, new geometries/producers, global scanner,
interpolator, formatter, flow ratios and unrelated option domains. No fallback.
Parent APA-R2/ARD obligations stay OPEN; this bounded M slice cannot close A/R.

This commit contains docs only; no runtime, RED or production change is claimed.
