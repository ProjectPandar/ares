# SAFC coupled option-family bounded source slice

Status: Proposed. This slice adds coverage for the coupled active-model SAFC options.

## Selection and ownership

Select only `small_area_infill_flow_compensation` (coBool) and `small_area_infill_flow_compensation_model` (coStrings) from the 650-key plan. Existing `option_coverage/domains.rs` generates generic false/true for the switch; `domains/probes.rs:89-92` supplies only `probe-empty` for the model. This is **not** a claim that the keys are absent from the inventory: the missing coverage is coupled nonempty model initialization, role/pattern admission and measurable E modification. A free-form empty probe cannot prove an active PCHIP consumer. No nonempty-model CLI parity witness was found under `crates/ares-cli/tests`.

A historical web UI metadata reference is not a logic-port claim. Baseline for this specification is G4 `e051ee15863ceffc4b7599632e08ca48ab7f5f9e`.

## Non-negotiable source boundary

Paths below are under `OrcaSlicer/src/libslic3r`; this is a rewrite/admission slice, not an Ares-designed extrusion correction pipeline.

| Upstream definition / consumer | Included boundary | Rust destination |
| --- | --- | --- |
| `PrintConfig.cpp:4472-4477` | coBool, default false | existing process-region typed option; no new key |
| `PrintConfig.cpp:4479-4491` | serialized coStrings length,factor model; exact ten-point stock default | existing process-gcode typed model option |
| `GCode.cpp:2548-2549` | construct only when enabled AND vector nonempty | `project_slice/gcode_emit/small_area.rs::from_traversal` |
| `GCode.cpp:6304-6320` `_needSAFC` | initialization/enable gate; supported rectilinear/aligned/monotonic/monotonic-line pattern gate | existing small-area consumer and emit call sites, bounded single-region context |
| `GCode/SmallAreaInfillFlowCompensator.cpp:27-114` | lawful model parsing, monotone point constraints, zero/beyond-last length bypass, solid/top/bottom role filter and dE multiplication | `extrusions/small_area.rs::{parse,multiplier}` / existing PchipModel |
| `GCode/PchipInterpolatorHelper.cpp:26-109` | coefficient/interpolation owner for admitted stock/custom lawful curves | existing `extrusions/small_area.rs::PchipModel`, only if real RED establishes a mismatch |
| `GCode.cpp:7004-7006,7064-7066,7087-7089,7212-7214` | emitted segment-length dE correction; finite slice below exercises ordinary linear segments only | `project_slice/gcode_emit/motion/{extrusion,path/constant,path/variable}.rs` as actual reached ownership requires |

Existing Ares scaffolding is not automatically credited as parity: `small_area.rs::from_traversal` currently passes three pattern booleans as true; `motion/extrusion.rs:81-83` uses feature-name-based scaling. These are temporary compatibility shells around the cited upstream gates. Inspect the actual selected emit route before proposing a repair; do not redesign all extrusion paths. Source inspection raises possible gate differences, **not a demonstrated production bug or permission to fix them now**.

## Finite admitted context and controls

One stock Ender-3 .4 nozzle / one logical PLA, one region, existing 10mm cube geometry, layer .2mm, constant fixed role speeds, relative E, no support/raft/skirt/prime tower/arc fitting/scarf/spiral vase. Keep startup/end templates and Native defaults unless a fixture field is explicitly admitted. Rectilinear bottom/internal/top patterns for activated witnesses; one paired concentric-pattern control. Native exports the full stock profiles and model; Ares consumes those exact byte inputs, not separately synthesized settings.

Propose seven coherent cases, not a Cartesian sweep:
1. OFF + exact stock ten-point model: baseline gate-off witness.
2. ON + exact stock model: nonempty active PCHIP witness.
3. ON + custom lawful model `0,0;5,0.5;20,1`: different interpolated multiplier and 20mm cutoff.
4. OFF + the same custom model: initialization/enable control, geometry/E channel must match case1 except preserved config echoes.
5. ON + genuinely empty coStrings vector `[]`: upstream initialization bypass, not a one-element malformed empty model string; require requested/effective vector identity and lawful successful Native admission before credit.
6. ON + custom model with bottom/internal/top all concentric: unsupported-pattern control.
7. OFF + the same custom model and all-concentric patterns: case6's geometry-identical disabled companion; no blanket claim that all patterns are covered.

The custom curve is legal by source: first length0, strictly increasing lengths/factors, final factor1. Model strings are unbounded; these are **finite lawful consumer samples**, not global schema-domain completeness. Empty-vector Native export/input errors remain harness/admission failures, not production RED. No workaround to force malformed inputs through upstream guards. Exact serialized array representation must be audited before runtime.

## Test-first proof obligations

Before production edits, author the coupled E2E harness and strong consumer assertions, then run real Native/Ares whole-output comparisons. Preserve all initial failures. Require actual positive-E solid/top/bottom extrusion, at least one segment in the active interior, observable E reduction on an admitted matching segment, and wall/sparse roles unaffected. An output footer, enable flag, model token or PASS on inactive stock geometry alone is insufficient.

Observer must preserve TYPE and E addressing state across layers. Use actual XY length and positive extrusion, bind comparable segment endpoints/roles for paired enable controls, and respect Native emission quantization instead of fitting a loose epsilon to output. Whole comparator remains unchanged (generated banner policy only). If no matching eligible segment exists, admission fails; don't weaken the witness. For unsupported-pattern control, verify actual concentric role extrusion and no correction against its own disabled companion, not against a different geometry pattern.

Tests precede code. A real RED must come from successfully admitted identical project bytes and actual consumer/output mismatch, not compile/input failure. Any production repair requires a separate source-cited bounded repair request after RED. Existing unit scaffolding stays intact; no post-hoc assertion edits to manufacture GREEN.

## Included / deferred / exit

Included: single-region one-nozzle initialization, stock/custom lawful model interpolation in supported linear solid extrusion, explicit disabled/empty and concentric gate controls, key-local coupled parity metadata only after all witnesses pass. Preserve the other648 plan rows and global OptionOutcome; keep harness modules below400LOC. Any promotion must label this finite context, not claim arbitrary coStrings completeness.

Deferred: malformed parser/error parity, nonfinite/numeric-prefix acceptance, near-equal validation edge cases, all-pattern exhaustion, object/region overrides, multi-filament/multi-nozzle, arc/scarf variable-Z routes, external support/bridge behavior beyond role nonapplication controls, GUI editing and config-stack/CLI adapters. Existing code may already support parts; this slice doesn't credit unobserved behavior or add legacy fallback.

Exit: SPEC → exact retained Native admission/RED → separate source-owned repair if required → all finite consumer/whole witnesses GREEN → fresh workspace/strict fmt/WASM and four independent standard full legs on latest production baseline → raw input/member/channel and unchanged648-plan audit. Existing Native ND exemptions do not automatically apply to a new failure. Core remains filesystem/UI/terminal-free on WASM/Windows/macOS/Linux; runtime E2E is the CLI adapter only.
