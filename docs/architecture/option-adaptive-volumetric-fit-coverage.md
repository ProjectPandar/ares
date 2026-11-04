# Adaptive volumetric fit: coupled option-family SPEC

## Upstream ownership and hard boundary

This is a finite Rust rewrite witness of OrcaSlicer `libslic3r`, not an independently designed speed pipeline.
`PrintConfig.cpp:2645-2657` defines `filament_adaptive_volumetric_speed` as nullable bool vector, default false, and `volumetric_speed_coefficients` as string vector, default empty. Lines 2650-2657 retain the experimental/profile-saved limitation.
`GCode.cpp:6323-6342`, `GCode::calc_max_volumetric_speed`, splits the string on spaces, attempts `stod` per token, requires six parsed coefficients, treats all-zero coefficients as no fitted limit, and evaluates
`c0*h*h + c1*w*w + c2*h*w + c3*h + c4*w + c5`.
`GCode.cpp:6555-6559`, within `_extrude`, reads the current filament's enable/model and takes `min(filament_max_volumetric_speed, fitted)` when enabled. **This local fitted value is consumed only by the `speed == 0` autospeed branch (6561-6562).** The subsequent explicit-speed cap (6615-6618) reads `FILAMENT_CONFIG(filament_max_volumetric_speed)` again, not that fitted local variable. First-layer overrides and slow-down interpolation (6565-6597) follow autospeed; they must not be bypassed to obtain a witness.

Rust destination: existing `ares-core/src/options/registry/definitions/{table/tail_terminal_suffix.rs,metadata.rs}` owns declared option types; existing profile/project option resolution owns the selected filament values. Actual project G-code emission belongs to `ares-core/src/project_slice/gcode_emit/motion/{options,extrusion}.rs` and its path callers. Test admission and whole comparison belong to new key-local modules under `ares-cli/tests/orca_parity/option_coverage/`.
Existing `ares-core/src/speeds/{config.rs,volumetric.rs}` has typed six-coefficient algebra, but is only a compatibility scaffold until its actual CLI/project emission reachability is established. Its isolated unit tests are not proof of the `GCode::_extrude` consumer. A repair, if a real admitted RED demands one, must wire or replace that scaffold around this cited source boundary, never create another speed pipeline.

## Included finite context and proposed cases

One stock Ender3 0.4/PLA filament, one region, cube10, 0.2 mm layer height, rectilinear 40% infill; no support, raft, skirt, tower, spiral or arcs. Use the existing stock profile admission helpers. Keep geometry and all other settings identical; disable cooling slowdown using admitted upstream settings only, and set a common user maximum of 12 mm3/s with requested role speeds high enough for a 1 mm3/s fit to visibly bind. If those controls are not admitted or yield no Native speed effect, report that failure and revise SPEC; do not invent an expected stream.
Vectors have one selected filament slot. Coefficients below are space-separated strings inside a string vector, not comma-separated points.

### Fixture revision request after actual no-effect Native admission

The original explicit-100 mm/s fixture is a valid **negative control**, not an active-fit witness. An offline fit audit records both modes' actual project values and raw Native footer: OFF-low enable `0`, ON-low `1`, model `0 0 0 0 0 1`, user maximum `12`, cooling slowdown `0`; no nil or lost slot. Both modes have identical ON/OFF positive-XYE F distributions on every layer: layer1 F1800×8/F2100×57; layers2/3 F6000×63. All eight whole comparisons PASS, but the Native active-fit assertion fails. This is witness admission failure, not production RED.

Rerun the same eight models with the five scoped wall/infill/top role speeds set to **0** (upstream autospeed) rather than 100; verify exported speed values and preserve first-layer defaults. Preserve explicit-100 OFF/ON-low as a separate no-effect pair, not delete the failed run or weaken active assertions. Autospeed is now included only for these supported roles; no promise that every role or first-layer feed is fitted. Require at least one non-first-layer reduction in each active model on Native before assessing Ares. If zero-speed profile validation refuses admission, stop and revise to an upstream-supported context rather than bypass validation.

The fitted algebra runs regardless of `speed == -1` role lookup, but its only speed consumer is `speed == 0`. Later ordinary volumetric capping uses the original user limit, explaining explicit-speed no-effect. Cooling is explicitly disabled in both exported input and footer; it is not necessary to hypothesize cooling masking to explain these observations. No runtime tracing claim is made: control flow plus identical distributions establishes the fixture error.

| Case | Enable | Coefficients | Intended upstream control |
|---|---|---|---|
| OFF-empty | false | empty string | user maximum only |
| ON-empty | true | empty string | invalid count, no fitted cap |
| OFF-low | false | `0 0 0 0 0 1` | inactive fitted model |
| ON-low | true | `0 0 0 0 0 1` | positive constant fitted cap binds |
| ON-high | true | `0 0 0 0 0 100` | user maximum wins |
| ON-zero | true | `0 0 0 0 0 0` | all-zero sentinel, no fitted cap |
| ON-short | true | `0 0 0 0 1` | five parsed coefficients, no fitted cap |
| ON-polynomial | true | `1 1 1 1 1 0.5` | ordered height/width polynomial binds |

All eight cases must first pass real Native admission in both seeded and official modes with fresh caches. No predeclared RED: production RED exists only after successful Native execution and an actual strict Ares mismatch or API failure.

## Required observations / failure checklist before implementation

- Bool/vector decoding or slot resolution may disable the active model; inspect admitted project/export and actual selected filament, not footer alone.
- Strings may be split incorrectly, padded with zero, or read from the wrong slot. Preserve input ZIP member hashes and emitted settings as evidence, without treating them as consumer proof.
- Existing isolated speed algebra may never reach the project emitter. Require positive-XYE extrusion segments with persistent role and effective F tracking; comments/settings-only tests are insufficient.
- Coefficient order or effective height/width may be wrong. Native polynomial witness must be distinct from constant-low where quantized F allows it; derive analytical bounds from the admitted effective geometry, not nominal width alone. Do not demand equality to an approximate reconstructed volume.
- Disabled, empty, zero, short and above-user controls may accidentally change commanded motion. Compare Native and Ares paired command streams independently; strict whole-file comparator remains unchanged, including settings footer.
- Active fit may be applied to travel or only to the first region snapshot. Observe unchanged travel geometry/E and supported extrusion effects; this single-region context does not prove multi-region/tool switching.
- Cooling, first-layer overrides or configured role speed may hide a lawful effect. Require at least one non-first-layer supported-role F reduction for ON-low versus OFF-low and ON-polynomial versus OFF-empty on Native and Ares; preserve XY/E correspondence. A no-effect Native outcome is failed witness admission, not Ares GREEN.
- Native path-order nondeterminism may spoil paired alignment. Preserve raw streams and audit hashes/input identity; report it explicitly, never strip motion or relax tolerances.
- Compiler/harness/admission failures are not production RED. No unit tests added after production changes; no repair before accepted real RED and a source-cited repair plan.

## Deferred / honest completeness

Defer malformed numeric tokens, repeated-space/partial-`stod` parsing parity, nonfinite and negative fits, JSON null/nullable inheritance, arbitrary strings, multiple filaments/regions, autospeed outside the five scoped role settings, first-layer-only, raft, bridge/overhang, scarf/variable-width/arcs, cooling and pressure equalization, and performance claims. Existing finite-positive filtering in `speeds/volumetric.rs` is not claimed equivalent to all upstream inputs.
No promotion of an unbounded string domain or generic bool completeness from eight controls; no changes to the other 648 plan rows, comparator, Native golden output, APA, load-settings, dataset/SEMM or matrix DIVERGENT ownership.

## Test-first exit and artifacts

Commit bounded E2E tracer first; persist eight whole manifests, inputs, Native commands, both raw streams and paired role/F/XYE observations. Admit both modes; submit actual RED or test-only GREEN evidence. Any production repair is a separate step, remains platform-neutral/no I/O in core, and stays under 400 LOC per key-local module. Run fresh directed two-mode checks, workspace nextest, workspace all-target/all-feature strict Clippy, fmt and WASM. Production changes require fresh standard four-leg full with candidate FAIL subset of base FAIL and raw audit. RED precedes one GREEN.

## Bounded reachability audit and test-only closure

Audit completed within the 30-minute bound, without Native edits or validation bypass. `GCode.hpp:408-418,663` defaults extrusion speeds to -1; GCode.cpp:6144/6169-6171 perimeters/infill delegate that sentinel, 6213-6224 support also passes -1. `_extrude` maps it to configured role speeds at 6516-6550; unrecognized roles throw rather than supply zero. `extrude_multi_path` (6038-6061) forwards its speed. `extrude_loop` 5808-5814 small-perimeter zero special case becomes positive half outer-wall speed, and 5895-5896 selects only a positive small-perimeter override. Skirt/brim callers4438-4441/4503/5395 pass support_speed. No literal zero-speed extrusion call was located in the bounded libslic3r caller search.

Adjacent source options also have min1: PrintConfig1658/1670 bridge/internal bridge,2137 small perimeter,3707 gap,4342 ironing,5570 scarf,6275 support interface,6346 support. Consequently no CLI-admitted zero source was found for the audited mapped roles/callers. This is a bounded reachability conclusion, not a mathematical proof across arbitrary future configurations or all internal APIs. Fit consumption is unreachable under the CLI validation of this fixture and audited ordinary role paths; no global unbounded-domain completeness claim.

Close as test-only: retain all eight explicit-100 model controls plus the named explicit OFF/ON-low pair; assert unchanged positive-XYE geometry/E/F per pair, not active fit reduction. This assertion change is a scope correction, not a production RED repair. Add zero-speed loaded-project CLI E2E: run live Native and real Ares CLI on identical 3MF bytes, compare captured exit code and stderr byte-for-byte, and compare the validated API rejection message with that same live Native stderr. Preserve both channels, API outcome and Native commands; no hardcoded expected stderr golden. Inventory/plan/sweep unchanged; run both fresh Native modes and the complete option_coverage group including tests::*, plus strict/fmt/WASM. Any group failure stays visible. No production change or full exemption is assumed.
