# APA-R2 M residual — NO-RED STOP

Status: **NO-RED STOP following owner rejection of SPEC
`c2c7a1af9f417f114dac3c1dd1cabedf209c5c94`**. No discovery, tests or production
repair authorized or executed for this request. This record supersedes that
commit's proposed discovery plan; it is not a renewed SPEC request.

Base: `41f2c90f765d42c6620583121296bb61a2cb839c`.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Parents: [APA-R2 section M](adaptive-pa-deferred-r2-spec.md),
[ARD-0104](ard-0104-adaptive-pressure-advance-port.md).

## Why the proposed work is stopped

Standalone ordinary multipath mean/suppression is already implemented by the
landed `4d7d69fae` boundary in
`crates/ares-core/src/project_slice/gcode_emit/motion/perimeter/multi_path.rs:23-44`.
Loop initialization/suppression is already present in
`crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths.rs:46,95-103,147-158`;
role override is present in
`crates/ares-core/src/project_slice/gcode_emit/motion/path/adaptive_pa.rs:16-34`.
The rejected discovery duplicated existing public tests:
`crates/ares-cli/tests/orca_parity/runner/adaptive_pa_activation_edits_hold_arachne_parity.rs`, `apa_m_pa_off.rs`,
`adaptive_pressure_advance/sloped_loop.rs` n0-n2 and `scarf_ramp.rs` s0-s2.
Neither missing accounting nor a speculative code difference is an authentic RED.
No new run is necessary to repeat that already-covered scope.

Corrections to the rejected source inventory: the sloped branch sets clip_length
 to zero (`OrcaSlicer/src/libslic3r/GCode.cpp:5799`); the fallback flow includes
role/scarf ratios (`GCode.cpp:6469-6482`). Any later proposal must use these
actual source inputs, not treat fallback as only print×filament scaling.

## Sole candidate examined: force-no-extrusion

`GCode.cpp:5867-5888` tests whether more than one region perimeter touches the
de-retraction point, then constructs `fake_path_wipe` with mm3_per_mm=0 and
force_no_extrusion=true, description `move inwards before retraction/seam`.
It calls `extrude_path`, not `extrude_multi_path`; `GCode.cpp:6103-6108` clears
the entity mean/suppression before `_extrude`. The initial PA marker requires
positive mean or positive fallback (`GCode.cpp:6816-6841`), and intra-feature
ramps likewise require positive flow (:6932-6936,7162 onward).

Rust's corresponding inward move is a specialized travel/pending-target path
in `crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths.rs::
append_wipe_before_external`, not a standalone mean child. The helper
`crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths/adaptive_pa.rs::mean`
does not model an independent force-no-extrusion flag. That absence alone does
not establish an incorrect mean: the cited fake path is dispatched independently
after the loop, not inserted into that loop's mean collection.

I cannot name a Native-admitted input reaching this producer for which a
source-localized PA/M900 divergence is predicted from the actual Rust code.
The specialized path may warrant later source-specific investigation, but no
measured reachability or differing command is claimed here. Inventing a flag,
using zero flow as a flag proxy, injecting a child, changing geometry or adding
a passing-only test would not supply the missing public RED.

## Disposition and remaining limits

Stop M work under this assignment, evidence-only and docs-only. No claim that
the force-no-extrusion domain is proven equivalent, and no claim of complete
APA-R2 parity. No Native probing, cargo/nextest, production edits, test additions,
comparator relaxation or fixture changes occurred in this request.

Activation/downstream-speed A, source-specific reset R, calibration-mode
admission, tower integration and equalizer algorithms remain deferred under the
parent records. ARD-0104 architectural decisions are unchanged. A future repair
requires a named authentic input, predicted source-localized output difference,
new scope approval and a public RED that asserts Native target equality and
fails until GREEN. Await owner reassignment rather than repeat accounting work.
