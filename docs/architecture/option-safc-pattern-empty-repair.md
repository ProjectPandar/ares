# SAFC empty-model initialization and pattern-gate repair request

Parent scope: [coupled-model SPEC](option-safc-coupled-model-coverage.md).

## Test-first RED

One test failed. All seven cases were successfully exported/sliced by Native and their whole-output results persisted in `artifacts/safc-cases.json` before the consumer assertion. Five PASS; `on-empty` ARES_ERROR; `on-concentric` DIVERGENT.

`on-concentric` Native line93 has `G1 X109.798 Y110.202 E.01336`; Ares counterpart has `E.00056`. Both `off-concentric` streams have `.01336`. Native paired consumer passed; Ares paired consumer then failed at `safc_observation.rs:105`. Thus the RED is not Native-versus-observer disagreement. Whole comparison already reported byte1932 / line93 difference before that assertion.

Lawful custom curve `(0,0),(5,0.5),(20,1)` has first-interval PCHIP slopes0.1 and1/30, endpoint derivative0.1 and internal derivative `60/(35/0.1+25/(1/30))=0.05454545`. At quantized segment length0.404, `t=0.0808` and factor≈0.041763892; multiplying displayed baseline `.01336` gives `.000557966`, emitted `.00056`. This explains the erroneous active model application; it is not an exact reconstruction of unrounded internal dE from formatted G-code. Concentric must bypass correction entirely by source gate. No tolerance or assertion correction proposed.

`on-empty` retains Ares error `small_area_infill_flow_compensation_model needs at least two points`; Native actual role extrusion equals disabled baseline. Input verification required actual requested/effective coStrings `[]`, not `['']` or suppressed configuration.
No production change preceded RED.

## Upstream ownership and minimal Rust repair

All upstream paths are `OrcaSlicer/src/libslic3r`:

1. **Initialization**: `GCode.cpp:2548-2549` constructs `SmallAreaInfillFlowCompensator` only when enabled AND `small_area_infill_flow_compensation_model` vector nonempty. In `crates/ares-core/src/project_slice/gcode_emit/small_area.rs::from_traversal`, return the existing disabled value for an empty model vector as well as disabled switch. Do not make `parse` accept malformed empty input, substitute a default curve, change parser errors, or treat a one-element empty string vector as genuine vector emptiness.
2. **Pattern/layer gate**: `GCode.cpp:6304-6320::_needSAFC` requires initialized compensator and enabled switch, then admits rectilinear, aligned rectilinear, monotonic and monotonic-line when `(on_first_layer && bottom_pattern_supported) || (role==solid && internal_pattern_supported) || (role==top && top_pattern_supported)`. `SmallAreaInfillFlowCompensator.cpp:106-114::modify_flow` separately restricts corrected roles to solid/top/bottom. Port these two intersecting predicates, not a rule that all solid-like feature strings always scale.
3. **Destination**: replace hard-coded `true,true,true` pattern flags in `gcode_emit/small_area.rs::from_traversal` with the actual resolved bounded single-region pattern choices. Use the existing `SmallAreaInfillFlowCompensation` pattern flags and role-aware `multiplier(role,is_first_layer,length)` in `crates/ares-core/src/extrusions/small_area.rs`, retaining its existing PCHIP coefficients and cutoff behavior. The emitter already carries `EmitState::on_first_layer`; do not infer layer context from a TYPE comment or global layer counter.
4. **Actual emit callers**: `project_slice/gcode_emit/motion/extrusion.rs::linear_segment` currently calls `multiplier_for_feature` without first-layer or pattern gates. Route its existing feature vocabulary through the source-owned role gate and pass `state.on_first_layer`. Audit the same helper callers in `motion/path/constant.rs`, `motion/path/variable.rs`, `motion/scarf/emit.rs`; keep their gate API consistent if the helper signature must change, but add no independent arc/scarf behavior and claim no coverage for deferred routes. Do not refactor unrelated extrusion math, travel, cooling or APA state.

The exact resolved pattern type/location is an implementation detail to confirm before editing; use existing typed enums and exhaustive matching rather than introducing string-based configuration parsing. Existing feature-to-role mapping should be reused if available; do not invent a general-purpose new extrusion abstraction. Existing Ares scalar parser/PCHIP math remains untouched unless a separate real RED and SPEC demands otherwise.

## Included, deferred and no-go changes

Included only: enabled+empty model bypass and source-faithful pattern/first-layer gate for the already admitted seven single-region linear cases. Stock/custom active rectilinear cases continue to reduce E identically on both sides; disabled/empty/concentric cases bypass. Existing feature-only scaling is a temporary compatibility shell and is replaced only at this source seam.

Deferred unchanged: malformed model/error-domain parity, region/object switch overrides, multi-filament/multi-nozzle, all-pattern runtime exhaustion, arc/scarf and variable-Z active proof, external support/bridge changes, GUI and settings-stack/CLI adapters. Do not change `Config.hpp` nil semantics, footer comparator, Native producer, fixtures, role observer, consumer assertions or the other648 plans. No fallback curve or guards to swallow invalid models. Core remains byte-oriented/platform-neutral for WASM/Windows/macOS/Linux.

## Verification order

Make only the source-owned production gate change. Keep the RED commit and unchanged tests. Run cargo fmt and strict Clippy/build; actual Native/Ares seven-case GREEN must show seven whole-output PASS plus every paired role/E consumer, including empty/concentric and retained stock/custom controls. On any failure preserve raw receipts and stop for diagnosis, not assertion weakening.

Promote coupled finite metadata/key-only sweep only after consumer GREEN, separately from production repair; don't turn unbounded model samples into a global coStrings completeness claim. Then fresh workspace/strict/fmt/WASM and full parity runs plus input/member/channel and648-row identity audit. Existing G4 Native ND adjudication is not a waiver for any new SAFC failure.
