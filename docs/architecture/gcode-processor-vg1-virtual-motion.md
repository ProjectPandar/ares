# VG1 virtual-motion processor rewrite

## Decision and source boundary

Port OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`, not an Ares timing heuristic. The owning operation is `src/libslic3r/GCode/GCodeProcessor.cpp:4237–4546::process_VG1`. Required adjacent ownership is `process_gcode_line` (2881–2927), command registration (1838), `process_tags` virtual-flush toggles (3181–3189), `process_filament_change` remaining-volume initialization, `UsedFilaments::update_flush_per_filament` (1606–1619), `TimeMachine::calculate_time` (419 onward), and `GCodeProcessor.hpp::TimeBlock` (447 onward).

Rust destination is exclusively `crates/ares-core/src/project_slice/gcode_emit/processor/`: estimate dispatch, motion state/block admission and planner, moves/schedule/export-cache attribution, and required filament/volume bookkeeping. New processor submodules may separate virtual-motion and flush accounting to remain at most 400 LOC. Existing processor and emitter-supplied statistics are temporary compatibility shells around those upstream concepts, not authority for a newly invented accounting model.

The boundary also includes `GCodeProcessor::post_process:993–1010` (material quantities from authoritative total-volume maps) and `process_used_filament:1147–1174` (canonical material-statistic export). Processor `UsedFilaments` is the sole volume source for all five public maps and material footer fields. Delete the superseded finish accounting and snapshot handoff; do not retain parallel accounting, comparison or a fallback. This source-required export is not an arbitrary footer correction. All non-VG1 reference footer bytes must remain unchanged, while VG1 changes must equal Native. Web ColorPrint legend and presetSlice E2E remain mandatory gates.

Do not modify `gcode_emit/{layers/boundary,motion/state,motion/travel}` or any emitter ordering, templates, bridge geometry, object labels, travel/lift order, or profile data. Those BBL ordering areas belong to a separate emitter ordering slice. A source-required volume prerequisite must stay in the processor; if that cannot be achieved, stop for a boundary decision rather than expanding the emitter scope.

## Admitted data before this spec

After the SYNC slice, processing the same original Native X2D stream gives Native `23298.746094` seconds versus Ares `21930.91064350919`. The Native producer is the supplied procgcode image; its parsed f32 total is used, not a rounded footer.

The source input's complete CONFIG_BLOCK is at lines 13–656. The oracle first applies CLI defaults, then `process_file` recognizes OrcaSlicer, loads this block, sets the BBL printer identity, and reapplies its configuration (`GCodeProcessor.cpp:2591–2641`). Prefixes retain that block; unknown Ares producer text is renamed only in forensic oracle copies, preserving command/configuration text and line ids.

Same-input prefixes end on complete lines. At 8209 both totals narrowed to f32 are `618.8775024414062`. At 8210 (`;VG1 E4 F523`) they remain equal because Native does not finalize the isolated remaining block at EOF. At 8211 (`;VG1 E4 F261`) Native is `620.2560424804688`; Ares remains `618.8775024414062`. After 8217 (`;VG1 E34.9231 F523`) the discrepancy is `5.38519287109375`. At 8219, SYNC T5 adds five seconds to both, preserving that discrepancy. These are equal-input prefix-finalization results, not invented full-stream per-block timestamps.

The complete stream has 762 VG1 lines, three per each of 254 transitions. A negative control replaces only those lines with blank lines, preserving every line id, other command, configuration and SYNC. Native then reports `21930.910156`, bit-identical after f32 parsing to Ares's original-stream total narrowed to f32. This closes the remaining full-stream timing discrepancy. Move-vertex `time0` first differs at 8271 by the opposite-sign five seconds; that attribution observation alone is not the missing-time cause.

External X2D files remain forensic evidence, never repository fixtures or expected outputs.

## Dispatch: comments are not ordinary G1

`GCodeReader::parse_line_internal` retains the raw comment and produces no ordinary command for `;VG1`. `GCodeProcessor::process_gcode_line:2881–2927` advances the source line id and copies end position to start position before dispatch, including comment lines. For a commandless raw line starting with `;`, it removes exactly that semicolon. If the next character is `V` or `v`, it reparses that content through a fresh `GCodeReader` and invokes the normal command table; `VG1` is registered at 1838. Otherwise it processes tags.

Preserve this upstream distinction, including exact prefix placement, parser case behavior and command token recognition. Do not strip arbitrary whitespace after `;`, reinterpret ordinary commented G1 as motion, or introduce a BBL-only gate absent from this dispatch. Direct `VG1` command lines use the same handler. Recognizing VG1 does not authorize new VM104/VM109 behavior; their independently owned command handlers are deferred.

## Line-by-line side-effect ledger

All side effects of `process_VG1` below are included. No VG1 side effect is deferred to make the timing test pass.

| Upstream operation | Included Rust contract |
| --- | --- |
| 4239–4244: current/previous logical filament, diameter and f32 cross-sectional area | Use the upstream logical-to-physical mapping, previous filament and diameter fallback already required by processor configuration. Match f32 operation order, not a new length-to-time formula. |
| 4246–4259: absolute-position lambda | Compute each XYZE endpoint from global-relative or E-local-relative modes, origin and inch scale. This function does **not** reuse G1's volumetric-E conversion. Preserve absent-axis start positions. |
| 4261–4282: move classification | Preserve wipe, retract/unretract, travel and extrusion classification for all axis combinations, even though no visible motion vertex is appended. |
| 4284–4304: G1 ordinal, envelope enable, endpoints, F, delta and zero-distance return | Advance internal G1 id and enable envelope processing before the zero-displacement return. Update endpoint and F in the upstream order. Both subsequent ordinary lines and virtual lines see the endpoint via `process_gcode_line`'s start-state copy. Preserve source-line and internal-G1 identity separately. |
| 4306–4324: virtual-flush volume split | Only positive E-only Unretract while virtual flushing is active updates flush volumes. Read physical extruder remaining nozzle volume; attribute its consumed portion to the previous logical filament, overflow to the active logical filament, then subtract or zero the remainder. Preserve the strict `remaining > flushed` branch and f32 volume arithmetic. |
| `UsedFilaments::update_flush_per_filament:1606–1619` | Nonzero virtual volume updates both `flush_per_filament` and `total_volumes_per_filament`, with upstream f64 map accumulation of the f32 quantity. Do not add it to model, support, wipe-tower or color-change buckets unless another cited upstream operation does so. Required virtual-flush toggles and remaining-volume initialization/reset are included. |
| 4326–4356: F, distance, enabled time modes | Preserve the repeated F assignment, XYZ length versus absolute E-only length, enabled-machine filtering and minimum feedrate selection. Normal mode is mandatory; do not silently construct a second independent silent-mode planner. |
| 4358–4409: directions, role, prepare/layer flags, centripetal and axis-feedrate caps | Build the upstream block with its role/travel exception, layer minimum and prepare flag. Reuse the existing upstream planner only where its arithmetic and state transitions are equivalent; port any demonstrated VG1-specific difference. Respect extrude-factor override and f32 axis scaling. |
| 4411–4537: acceleration, safe exit, jerk/junction, allowable entry, flags and trapezoid | Preserve the full algorithm, including retract acceleration for E-only moves, axis caps, the junction thresholds and previous state. Do not replace this with `E/F` or a timing multiplier. |
| 4539–4546: previous state and queued block, no saved move | Advance machine previous state and enqueue the block. **Do not append a VG1 MoveVertex**. Preserve default `TimeBlock.move_id=0` and its consequence for dummy-vertex time assignment; do not associate virtual blocks with the preceding physical move. Respect upstream cache eligibility and additional-time/flush matching despite G1-id advancement. |

Volume exit checks cover both branches of the split, repeated virtual moves, inactive virtual flushing and zero E. Endpoint checks cover M82/M83, G90/G91, G92, G20/G21, absent F, zero distance and a subsequent real move. Recording enabled/disabled must produce identical exported bytes.

The emitted `config.print_statistics` snapshot is not a license to discard or double-count processor virtual-flush updates. Replace the required snapshot boundary with source-owned processor flush/total accounting. Prewritten public statistics checks must discriminate fresh template-injected VG1 from preset-generated virtual flush already represented by the compatibility shell. Physical volume cache prerequisites may be ported only as the cited `UsedFilaments` dependency needed for that result; no unrelated statistics redesign.

Deferred adjacent behavior: VM104/VM109, independent firmware features, general silent-mode exposure, ColorChange/Pause/Custom extensions not required by VG1, unrelated G2/G3 or planner defects, all emitter/order/bridge behavior and empty-plate CLI exit parity. Existing enabled modes and recording guarantees must not regress; encountering a required missing prerequisite blocks acceptance rather than silently deferring a VG1 side effect.

## RED and neighboring controls, before production edits

Primary public seam: existing Orca parity runner through the byte-oriented slice API, with exact requested/applied settings verification, Native success, complete artifacts and full ordered-byte equality. Repository-owned small cube geometry and admitted BBL X1C/X2D profiles supply a minimal two-logical-filament transition. Keep bridge absent and M73 enabled. Supply only the minimum virtual exchange segment through existing user-template overrides when natural preset transitions bring unrelated emitter ownership into the case; require the exact VG1 bracket to be present in both complete outputs.

The RED must fail specifically on processor timing/progress/statistics, not fixture application, Unsupported, Native setup, bridge or emitter movement ordering. A dual-filament request must demonstrably select/apply both filament identities. Do not call a single-material cube with two unused settings a dual-material RED. If a natural dual-material cube fails solely in the emitter ordering area, retain that failure separately and use an admitted template-based transition until the ordering chain is rebased, without relaxing bytes.

Prewrite no-VG1 and zero-displacement VG1 controls, ordinary `;G1` and spaced `; VG1` non-command controls, direct VG1 dispatch, and same transition with SYNC T0/T5. Add public processed-result/oracle checks for absence of VG1 vertices, dummy-vertex attribution, following real-move endpoint/F state, both nozzle-volume split branches and flush/total statistics. Keep the existing move-comparator tolerances unchanged; compare copied f32 fields as bits and upstream f64 accumulated volume exactly. The oracle must receive the same full embedded settings rather than its bootstrap defaults. Any needed test-only statistics dump must be declared in the oracle provenance; no production probes.

Tests must assert the complete stream including every changed M73 and estimated-time line. Neither M73 suppression, time-comment rewriting, comparator normalization nor retrying a genuine failure is permitted. Native seeded output remains complete-byte authority; official differences require the existing two-condition ND rule and retained original evidence.

X2D is supplemental targeted validation: run the unchanged original inputs when current main can admit them without the separate bridge branch; additionally replay their complete original Native/Ares streams with identical configurations and require the VG1 missing-time discrepancy to disappear. Remaining emitter differences are reported independently, never hidden by the timing result. Do not commit original external models or generated G-code.

## Implementation sequence

1. Prewrite the admitted minimal public RED and neighboring controls, record failing current-base evidence and effective settings, and preserve all artifacts before processor edits.
2. Port the ledger in source order with the minimum required state/volume dependencies. Keep modules at most 400 LOC and no direct I/O or host-only behavior in core. Production changes remain inside processor.
3. Verify the minimal complete-byte cases, processed-result/oracle side effects, unchanged control streams, and X2D targeted accounting. No next speculative timing correction is bundled.
4. Run workspace nextest, strict workspace/all-target/all-feature Clippy, fmt, core and adapter WASM checks; run complete current-base seeded and official controls, raw common-stream comparisons and explicit reference-enabled profile_entry GREEN.

The BBL emitter ordering slice and this processor slice do not share production code. Whichever lands second must rerun the **full** gates and complete reference controls on top of the first. Source disjointness does not waive validation because the emitted command sequence drives the processor.

Exit requires all included VG1 side effects, not merely matching total time, plus admitted full-byte RED/control parity. If a side-effect prerequisite cannot be implemented within this boundary, revise the spec instead of shipping a time-only approximation.
