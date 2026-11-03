# Adaptive PA residual slice APA-R1 — sloped-loop suppression + active bridge domain

Status: **SPEC approved / witness stage authorized; no production repair without Native-proven RED**. Coordinator approved APA-R1 scope and witness stage: commit this document independently before tests/fixes; first retain runtime Native sloped-loop/suppression reachability evidence, stop if insufficient without expanding witness scope; if all outputs are GREEN submit test-only coverage honestly; any production repair must cite AdaptivePAProcessor.cpp/GCode.cpp lines and pass full production gates. No tests, production implementation, Native patches or coverage classification changed at spec commit. Analysis baseline `bc78d50bd12885130283d6d7b222499d0e56681b`; upstream OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1` (HEAD verified). This is a bounded source-cited continuation of [ARD-0104](ard-0104-adaptive-pressure-advance-port.md), not a new Ares pipeline. The existing emitter is a temporary compatibility shell around upstream GCode/AdaptivePAProcessor semantics.

## 1. Ownership / smallest first slice

Select **ordinary single-logical-filament, no-tower sloped-loop PA average and first-subpath suppression**. Add the directly related residual G1 `adaptive_pressure_advance_bridges` active-context controls, rather than declaring existing inactive .5/2 sweeps domain-complete. This keeps geometry construction, calibration generation, tool/tower reset sequencing and arbitrary stream-parser rewrites out of the first slice. Existing upstream-matching behavior is retained; change only a runtime Native-proven mismatch.

Sources below are relative to `OrcaSlicer/src/libslic3r/` at the pinned HEAD:

| Upstream boundary | Behavior / Rust destination |
| --- | --- |
| `GCode.cpp::extrude_loop`, 5777–5806 | Sloped activation: eligible external/inner-wall role, enabled scarf type, non-vase, layer_id>0; conditional smoothness/overhang guards. Loop clipping is zero for slope, seam-gap clipping otherwise. Existing scarf construction is reused, not repaired or redesigned. |
| `GCode.cpp::extrude_loop`, 5900–5924, 5926–5973 | Reset average/suppression once per loop; ordered length-weighted average of pre-slope paths excluding force-no-extrusion; ordinary and sloped emit loops set suppression **after each `_extrude` call**. Existing `gcode_emit/motion/loop_paths.rs`, `loop_paths/adaptive_pa.rs`, `motion/path/adaptive_pa.rs`, `motion/state.rs` are the only contemplated production destinations. |
| `GCode.cpp::_extrude`, 6720–6745, 6816–6841 | PA+adaptive enabled and non-calibration initial marker gate; suppression can be overridden by a role change; positive average takes precedence over single-path flow. Marker precedes extrusion feedrate. Preserve exact source ordering, not a blanket “one marker per loop” rule. |
| `GCode.cpp::_extrude`, 6925–6975 and 7155–7205 | Independent intra-feature constant/variable ramp tags, gated by adaptive-overhang option. First slice disables this option to isolate initial/suppression behavior. These sites are cited adjacent/deferred, not silently merged into loop suppression. |
| `GCode/AdaptivePAProcessor.cpp::AdaptivePAProcessor`, 24–42; `process_layer`, 214–295 | Only create models when both PA toggles are on. Only initialized-model branch uses bridge value when `BR!=0 && value>EPSILON`; invalid/no-model path keeps static PA. Debug bridge-override comment has its own predicate outside prediction branch. Existing `gcode_emit/adaptive_pa.rs::{Options,enabled,process,Processor::tag}`. |
| `GCode/AdaptivePAInterpolator.cpp::parseAndSetData`, 17–75; `operator()`, 83–118 | Group by acceleration; retain groups with >1 flow sample; initialized state vs no usable model; one acceleration group is valid; round prediction to three decimals. Existing `adaptive_pa/model.rs`, `pchip.rs`, `number.rs`: reuse without arithmetic changes. |
| `PrintConfig.cpp`, 2401–2408; `Config.hpp`, 2475–2479; `libslic3r.h`, 52 | Bridge option coFloats, default0, explicit max2, inherited min `-FLT_MAX`; `EPSILON=1e-4`. Distinguish schema, application, effective threshold, printer hardware constraints. Test-domain destination: `ares-cli/tests/orca_parity/adaptive_pressure_advance/` and a narrowly scoped `option_coverage` context contract only after live evidence. |

Existing Rust inspection finds average and suppression fields already present, including sloped-loop setup and `tag` role-change exception. This is **not proof of parity or proof of a bug**. Native branch reachability and an actual mismatch are the RED admission barrier; do not force a code patch if all source-owned behavior already matches.

## 2. Included

1. One real public exported-3MF/sliced-full-output witness containing sloped perimeter paths. Compare a scarf/sloped case with its non-sloped control using identical saved geometry and other settings. Weighted PA flow must be computed before sloping, not per generated slope fragment. Verify the first marker and subsequent same-role suppression in ordered output; role changes remain source-defined exceptions.
2. Static/adaptive enable gates and valid/invalid model controls **only as required to establish APA-R1/G1 bridge activation**, not exhaustive calibration/activation closure.
3. G1 `adaptive_pressure_advance_bridges`: active `BR:1` witness plus non-bridge `BR:0` control, dependency verification, disable/threshold/upper bound in a documented single-filament context. Preserve complete ordered Native/Ares byte comparison, including PA debug comments and every non-PA byte. Do not compare only extracted M900 commands.
4. Artifacts: actual exported input, verified applied config, full Native/Ares output, input/binary SHA, argv, exit/channels/deadline receipts, ordered PA tag/command extraction with byte offsets for diagnosis, unchanged comparator result. All generated results outside Git.

## 3. Explicit deferred work and residual domain accounting

| Deferred boundary | Why not APA-R1 / future entry |
| --- | --- |
| Standalone `ExtrusionMultiPath` in `GCode.cpp:6038–6066`, force-no-extrusion traversal not reached by R1 witness | Same upstream mechanism, but a different public geometry/traversal witness. Do not silently assume loop proof covers it; preserve the exclusion rule when relevant. |
| Mixed roles and enabled adaptive-overhang ramps on sloped loops, variable-speed trigger/order sites above | Separate source-owned ramp/suppression interaction with dynamic overhang segmentation. APA-R1 sets overhang adaptive option0; a plain BR bridge-control case does not close these sites. |
| `Calib_PA_Line/Pattern/Tower` generation and `is_pa_calib` exemption | Requires public calibration API/admission boundary and calibration fixture; no invented calibration flag or internal test-only bypass. Ordinary PA toggle controls do not close calibration. |
| Tool-change/reset restore at `GCode.cpp:1093,1374,3160`, also current-file `7743,8051`; `AdaptivePAProcessor.hpp::resetPreviousPA` | Distinct source-specific sequencing and logical-tool/tower lifecycle, not layer-only reset equivalence. Inventory remains open; no tower producer/activation edits. |
| `AdaptivePAProcessor.cpp:106–213` lookahead stream save/clear/seek, feature/wipe/travel boundaries | Existing Rust slice scanner is not replaced just to resemble C++; requires focused evidence of source stream-position behavior. Retain current lookahead and all existing assertions in R1. |
| G1 `bridge_flow`, `internal_bridge_flow` | Flow/LayerRegion ownership, not Adaptive PA option domain. They remain INCOMPLETE; use fixed already-admitted flow solely to reach bridge. Do not claim PA tests close these keys. |
| G1 `filament_diameter`, `filament_flow_ratio` | Extruder positive/vector/effective-domain ownership, independent of PA. Fixed context only; preserve INCOMPLETE. |
| Exhaustive PA bridge schema/vector domain | Negative inherited raw bound, huge-magnitude rejection/materialization, multiple logical slots and schema-vs-effective contract remain explicit. R1 proves contextual effective behavior, **not global domain PASS**. |
| Type2 `enable_prime_tower=1` activation, tower geometry/PA integration, timelapse/wrapping | **worker2 ownership; excluded**. R1 explicitly verifies `enable_prime_tower=0` and no prime-tower feature output. |

Only `adaptive_pressure_advance_bridges` in `/tmp/worker6-option-residual-audit.md` G1 has AdaptivePAProcessor as owner. Include it here; keep the other four G1 keys visible as deferred adjacent owners, not omitted or reclassified as complete. The report's older baseline is provenance, not fresh current-main coverage. Do not change `provably_legal=true` simply because additional tracers PASS.

## 4. Runtime Native RED design — tests first, then gate

### A. Witness and existing regression admission

After approval, create only a test/input changeset first. Reuse `OrcaRunner::{export_case,slice_case}`, applied-value verification, and unchanged `compare_case` at the public exported-3MF/full-G-code seam. First retain/run the existing `adaptive_pressure_advance` constant/models/tool-change controls without deleting/assertion tuning. Use Ender-3 single logical filament and classic perimeter presets from existing helper; initial candidate inputs are existing cube/overhang fixtures, not a fabricated PA_CHANGE stream. Select the smallest input whose **Native** output proves the requested branch; if current overhang geometry cannot create a real sloped witness, define a minimal deterministic manual mesh input and request fixture approval before changing it.

Set no-tower, adaptive+PA enabled, comments1, nonconstant valid model already covered by the landed port, ordinary print mode and adaptive-overhang0. For the initial sloped witness request scarf All on external walls, inner-wall scarf0, conditional0, non-entire-loop with fixed positive start-height/min-length/steps from the existing source-owned scarf fixture; compare against scarf None, otherwise identical. Verify at least one eligible perimeter above layer0 and actual sloped Z/extrusion sequence. Cite `GCode.cpp:5777–5806` for these gates; do not confuse first-layer inactivity with suppression. Freeze the exact numeric fixture settings in the RED request before running it; no new scarf math/settings API. Verify saved and applied values (including all seam-slope activation parameters and both PA toggles). Do not lower epsilon, select a constant model that hides average errors, or alter seam/flow geometry after a failure just to get PASS.

The three initial traversal cases are non-sloped control, sloped same-role witness, and the matched sloped witness with adaptive disabled. Native must show actual sloped moves and PA marker/command activity; mere option retention or a count assertion is insufficient. Record the Native PA_CHANGE flow and its ordering relative to feedrates, roles and moves. Branch extraction is diagnostic only; complete-output equality remains authoritative. A case failing Native activation/admission is **NOT** implementation RED.

### B. Bridge G1 dependency matrix in a fixed reachable context

Use the existing real overhang/bridge exported case after Native proves at least one `BR:1` PA_CHANGE and actual bridge/overhang extrusion; also require non-bridge PA activity. Fix all non-PA options and model before recording comparisons.

- Valid model + PA/adaptive on, bridge values `0`, `0.0001` (exact source threshold), `0.000101` (above threshold, materializable through Native export), `0.06` (interior), `2` (explicit upper bound).
- Same context/bridge `0.06` with PA off; with adaptive off; with invalid model. Assert source output (static prediction/no adaptive emission as applicable). Do **not** assert the bridge debug comment is absent for invalid-model: source comment guard is separate from prediction fallback.
- For valid models, require override comment/ordered predicted PA at reachable `BR:1` only above EPSILON. Writer significant-digit formatting can collapse nearby numeric commands; do not use command presence alone to decide the threshold. Use Native full comments plus actual comparison; no test-selected tolerance.
- `0` is a disable sentinel, **not proven schema minimum**. Negative/-FLT_MAX and >2 are not blindly sliced in R1; future precise validation/materialization receipt tests must establish admission/rejection before making global Option completion claims.

Native export admission observation (first run `886ccc1d-3064-4b1b-94a2-761b843117dc`, `${TMP_ROOT}/ares-adaptive-pa-r1/bridge-first-d491`): the initially proposed `0.0001000001` was exported as `0.0001`, and unchanged requested/effective verification correctly failed before slicing that case. This is NOT Adaptive PA implementation RED. `Config.hpp::ConfigOptionFloatsTempl::{serialize,vserialize}` at 846–865 and `serialize_single_value` at 910–912 use fresh ostringstream default precision (six significant digits); `libslic3r.h:52` keeps EPSILON=1e-4. Coordinator explicitly approved the materializable above-threshold request `0.000101`, retaining the original failed admission log as an observation only, with no extra collapse-control test. No epsilon, comparison, application checker or geometry changes.

### C. Honest RED criterion and stop policy

Actual implementation RED requires successful fresh Native output/schema/activation, retained identical input/config, then original public Ares full-output mismatch attributable to cited loop/suppression/bridge behavior. Preserve first failure and byte offsets; no ignore, unsupported guard, filtered comparator, frozen Native expectation, retry-until-success, or unrelated pipeline repair. Official and separately identified seeded references must have separate artifact/cache namespaces; never normalize movements/PA fields. Existing approved generator-identity/wall-clock adaptation only, centralized comparator unchanged.

If all first-slice full outputs PASS, record **already-matching coverage evidence**, not a fabricated RED or unnecessary production rewrite; notify coordinator for a test-only slice ruling or approval of the next deferred boundary. If Native branch never activates, stop as missing witness, not PASS. If differences are unrelated, retain failure and ask owner before expanding scope.

Missing optional Native must visibly skip; `ARES_REQUIRE_ORCA=1` must fail missing Native. Configured bad paths, nonzero/timeout/missing output are failures, never skip/fallback. Use existing runner ownership/timeouts and external receipts; no new runtime oracle harness/production probes.

## 5. GREEN / review / exit

After actual RED review/approval, implement only source-proven deltas in the private destinations above; <=400 LOC touched Rust modules, no core I/O/new pipeline/CLI dependency. Preserve all existing Adaptive PA assertions and other integrations. Run directed required-Native cases (zero skips) and old controls with official CPU0/seeded identities separately. Any official nondeterminism is retained as raw failure pending a narrowly evidenced owner ruling, not erased by a seeded PASS.

Production change requires latest-main paired baseline/candidate controls + `cargo nextest run --workspace`, strict `cargo clippy --workspace --all-targets --all-features -- -D warnings`, fmt and WASM gates, scheduling one heavy stage at a time. Exact-hash combined review before authorized integration; no RED-only merge. If test-only, coordinator carry rules apply but active Native evidence is still required. Retain signed/source-cited completion record: R1 closes only the witnessed loop/suppression and contextual bridge behaviors; ARD-0104 overall, calibration/reset/lookahead and global Option domain remain open.

Approval requested: **APA-R1 scope and runtime Native witness/test-first stage only**. No implementation begins before approval; no Type2 tower work.
