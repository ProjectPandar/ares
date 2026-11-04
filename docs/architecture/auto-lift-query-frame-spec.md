# Auto-lift current-plate query frame

Status: proposed. Docs-first SPEC precedes tests-only RED; GREEN is a separate phase. Stack on the frozen sequential label-flush tip without modifying or requalifying it.

## Source-owned boundary

Pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/GCode.cpp::GCode::needs_retraction:7593–7604` clips the travel and translates it into current-plate coordinates using `m_origin - m_writer.get_xy_offset()`. Line7604 is the owning query-frame operation.
- `GCodeWriter.hpp:98–99` exposes the writer's plate XY offset. It is not the extruder offset used for nozzle/output coordinates (`GCode.cpp:8124–8127`).
- `GCode.cpp:7607–7615,7635–7647` consumes the overhang query when selecting Auto Spiral/Slope. Existing overhang/cache ownership stays with `PrintObject.cpp::detect_overhangs_for_lift` and `Print.cpp::layers_sorted_for_object`.

Rust destination: existing `crates/ares-core/src/project_slice/gcode_emit/motion/auto_lift.rs::AutoLift::crosses_overhang`, specifically its translation of the clipped query. `motion/origin.rs::set_origin` defines `state.offset` as origin minus extruder offset; that nozzle-coordinate translation is not the geometric query frame. `motion/path/start_travel/approach.rs` already invokes the predicate and selects Spiral on true; no duplicate predicate is needed.

The existing AutoLift module is a temporary compatibility shell around the cited upstream concept. Correct its query translation in place, not an Ares-owned travel pipeline. Preserve existing scaling, clipping/truncation order, protect-z lookup, cache geometry, caller timing, retraction and writer semantics.

## Included and deferred behavior

Included only: restore the upstream current-plate query translation, distinct from nozzle coordinates, and verify its existing Spiral and complete-output consequences. The original witness has zero writer plate offset, so upstream origin-minus-plate-offset is the existing plate-relative origin. Do not introduce new plate-offset plumbing or silently broaden this fix into a multi-plate/cache-domain port; a conflicting frame on the destination is a STOP requiring scope revision.

Deferred/unchanged: overhang detection, holes/cache geometry, layer-window arithmetic, fallback Auto behavior, explicit lift modes, wipe/retract, output-coordinate conversion, writer formatting, processor timing/cache/progress insertion, configuration and other vendors. The early M73 discrepancy on this witness is a downstream consequence of the missing spiral's total time, not an independent B implementation slice. Fix no M73 bytes directly.

The tokenizer (`processor/motion_util.rs`), verbose fan (`fan_mover` and `layers/fan_setup.rs`), and zero-E (`motion/extrusion.rs`) stay untouched. Report unexpected overlap rather than folding it into this change. Core remains platform-neutral for WASM/browser, Windows, macOS and Linux; no new file I/O, terminal, viewer or adapter behavior.

## Public witness and genuine RED

Use the original stock Bambu Lab P1S 0.4-nozzle two-object **ByLayer** project, without changing its exported settings, geometry, placement or extruder offset to manufacture isolation. Native export and two fresh stable Native slices/replay precede any public API/CLI slice in each official/seeded mode, run serially.

Use the existing `runner::native_stream_case` public whole-output seam and unchanged comparator. At the exact tests-only commit require successful Native/API/CLI calls, raw API==CLI, and a real complete-output mismatch. Production remains byte-identical to the SPEC parent. Admission errors, unstable Native replay, or an unrelated first difference are STOPs, not RED.

Native-activity and behavioral assertions must establish:

1. The original two-object ByLayer/Auto input is actually exercised.
2. Native contains the target `G17\nG3 Z4 I.058 J-1.216 P1  F30000\n` spiral.
3. Native has the early sequence `G0 X125 Y260.0\nG0 Z-1.01\nG0 X131 F211\nG0 X124\nM73 P29 R13\n`, without the extra `M73 P28 R13` inserted before `G0 X131 F211` by the pre-fix public outputs.
4. The public result must match the Native spiral and early progress bytes, and pass the complete comparator. RED must fail behaviorally on the unmodified production implementation, not through a synthetic failure or expected-error wrapper.

Assertions diagnose the witness, never replace full-output acceptance. Do not filter progress, compare only motion, recompute geometry in tests, alter comparator normalization or add a writer/M73 workaround. Store complete Native/API/CLI streams, replay/comparison records, exact executable provenance and repeatable artifacts outside the architecture decision document.

## Sequence and exit criteria

1. Commit this SPEC, its concise source/scope ARD and roadmap entry only. No test or production changes in SPEC.
2. Then add the genuine public tests-only RED described above, before any production change.
3. GREEN changes only the query-coordinate translation in `AutoLift::crosses_overhang` to upstream origin minus writer plate XY offset. Keep tests and comparator unchanged. Require the original public witness to pass completely in the same modes, with raw API==CLI, correct spiral and no extra early P28R13. Any residual mismatch is STOP, not acceptable GREEN.
4. Run rustfmt/clippy after edits and selected `cargo nextest run` gates after test/production changes. No `cargo test` default. Report platform compile checks separately from runtime qualification; claim no unexecuted platform coverage.
5. Each phase is a separate conventional commit. Keep the earlier sequential label-flush stack and localization/search seals unchanged.
