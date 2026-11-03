# ARD-0082: Round seam weight and blend math consistently

Initial source boundary under investigation was `libslic3r/GCode.cpp:426-500`
(`Wipe::wipe`), `Polyline.cpp:52-72` (`clip_end`), and
`GCodeWriter.cpp:1224-1265` (`GCodeFormatter::emit_axis`). Rust destinations
were existing motion travel/wipe and formatting modules, not a new pipeline.
Captured inputs now prove the actual owner is
`GCode/SeamPlacer.cpp:42-59::gauss/compute_angle_penalty` and
`:1237-1392::SeamPlacer::align_seam_points` (curve fitting and float blend).
Production destinations are existing `project_slice/seam_placement.rs`
(`angle_penalty`) and `seam_placement/alignment.rs::finalize_string`.

ARD-0081's complete ordered browser gate is reproducibly RED at line8134:
native `E-.02097`, WASM `E-.02098`. Two actual Chromium outputs replay
identically except their generator line. No tolerance/fallback is permitted.

First minimise the real input or replay captured calculation inputs. Ranked
hypotheses: formatting scale/rounding, different integer wipe endpoints,
different length/clip arithmetic. A throwaway harness compiling the actual
format module for native/WASM gives identical scales and rounding for the
three adjacent doubles around -0.020975; this excludes formatting at those
inputs, not every upstream calculation.

Temporary tagged diagnostics at the first divergent wipe segment may record
raw calculation bits through generated output. These diagnostics are not
acceptance results and must be removed before final production verification.
Preserve independent reference bytes,
actual outputs and modified-baseline global scan identities. Deferred adjacent
seam/arc/M73 differences must not be claimed solved by a first-line repair.

Exit: causal minimal reproduction, regression RED/GREEN at the public complete
output seam, original KSR strict native/browser comparisons, fmt/Clippy,
workspace Nextest and proportional Tier-1 builds; independent six-axis
review/fix/re-review before code/docs commit and push. A diagnostic checkpoint
alone is not a completed repair.

## Causal evidence and selected repair

Native diagnostic output `${TMP_ROOT}/ares-ksr-ordered-CeO4MG/ares.gcode` and actual
WASM diagnostic output captured12 real layer-index2 spline/blend inputs.
`${TMP_ROOT}/ares-ard82-spline-actual-replay.log` shows identical observations and
sample points but different weights. Replaying each captured input through
the same actual spline/QR sources for native/WASM gives identical fits;
feeding the other producer's weights reproduces that producer's changed fit.
This eliminates spline/QR arithmetic as the first divergence's owner.
The temporary Node diagnostic output path was reused for later clean output;
the original WASM spline stream is not retained at that path. Its field and
cross-replay comparisons remain in the contemporaneous replay log, while
the full native capture and original public Chromium RED outputs are retained.
The wipe start differs BEFORE clipping: native(-312680,-20412517),
WASM(-312678,-20412519), while both paths have no retained arc projection.

Scalar replay `${TMP_ROOT}/ares-ard82-transcendental-probe.log` identifies
`exp(0.9162646532058716f32)`: native bits401ffeef, WASM401ffeee.
Real target seam weights differ as3f4132ef versus3f4132f1. Also real cubic
blend ratios1043670509 and1043999504 produce WASM results one ulp below
the native capture. These are math-library discrepancies, not RNG or formatting.

Evaluate the two dynamic exponential arguments in f64 and cast the result
once to the upstream f32; evaluate the integer cube in f64 then cast once.
Keep all surrounding upstream f32 arithmetic, candidate selection and spline
operations unchanged. The scalar probes produce matching native results on
both targets. This is a portable numerical implementation of the cited
upstream float-returning library operations, not an epsilon bias or target
fallback. It does not prove identical results for every possible float input.

The existing actual browser project/complete-output gate is the witnessed
public regression RED from ARD-0081. No private structure/source pin test is
added. All tagged diagnostics and temporary TLS storage are removed from
production sources before the GREEN attempt. Native complete KSR passes
35098/18.492s. Clean WASM output under Node passes the unchanged strict
comparator71791/0.271s. Actual Chromium runs7685 and52983 both pass2/2
tests (1.4m including Cargo lock, then25.1s); full outputs are retained at
`${TMP_ROOT}/ares-wasm-gcode-dYmH9A/browser.gcode` and
`${TMP_ROOT}/ares-wasm-gcode-LIFkkO/browser.gcode` and replay identically except
the generator line. Initialized and untouched official reference hashes
are unchanged. This establishes this project's identified native/browser
contract, not arbitrary libm equivalence or all-printer/default/domain parity.

Workspace84870 passes7459 tests with9 skips in258.351s. All-target Clippy
2472 exits0 with existing warnings; fmt/diff checks pass. True core cargo builds
Windows/macOS/Linux67240 all exit0; release WASM52515 exits0 (3m12s).
Changed Rust files are238/306LOC; no include-based split or private pin test
is added. Independent initial six-axis review finds no required code fix,
requests first-owner and final-status wording corrections (now applied), and
recommends shared live consumer regressions. Initial consumer invocation
fails10/10 before slicing because the artifact environment key was misspelled;
it is not a producer RED. Corrected60159 with ARES_PARITY_ARTIFACT_ROOT
passes10/10 in33.320s: nine live Ares full-output consumers and native KSR
self-replay. Both newly generated KSR native streams also match actual clean
Chromium output in complete ordered generator-line-only comparisons. Full
fresh inputs/native outputs remain under `${TMP_ROOT}/ares-ard82-consumers`.
Final immutable patch re-review remains pending.

Immutable009d printer scan5535 terminates exit100 in2150.617s:978/1001
modified-classic casesPASS,13 reference-side errors and10 missing compatible
presets, no recorded byte divergences. Keep generated full summary and
external artifacts. This is not untouched-profile or all-printer completion.
Option scan89280 now terminates exit100 in3254.113s:386/650 bounded domains
complete,1661 generated cases,1519 strictly compared,1 rejected.264 domains
remain incomplete or failed. Full summary/artifacts are retained under
`${TMP_ROOT}/ares-options-009d3b49-initialized`. This is the old immutable009d
producer, not verification of the pending seam math repair.
