# Adaptive PA modal-feedrate inheritance

## Upstream owner and Rust destination

Pinned libslic3r revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

This slice ports `GCode/AdaptivePAProcessor.cpp::feedrate_word` and the modal-speed portions of `AdaptivePAProcessor::process_layer`:

- Lines 70–78: identify an F word on G0/G1/G2/G3 moves, before any comment, with a numeric first character; convert mm/min to mm/s.
- Lines 121–123: track positive modal feedrates on all moves, including travels and wipes, independently of the standalone `G1 F` current-speed update.
- Lines 159–208: initialize lookahead modal speed from processor state, update it until the first XY extrusion, and inherit it into temporary/current/next feedrates when no standalone feedrate has been found.
- Lines 312–313: emit the next/max-next speeds derived from that state.
- `GCode/AdaptivePAProcessor.hpp`: `m_modal_feedrate` is processor-owned state, separate from `m_current_feedrate`.

Rust destination: `crates/ares-core/src/project_slice/gcode_emit/adaptive_pa.rs` (`Processor` state, `layer`, and `tag`) and `adaptive_pa/scan.rs` (move feedrate recognition and lookahead). The existing processor is the compatibility shell around this upstream class; fill its missing modal state rather than introduce another processing path.

## Included behavior

Maintain modal feedrate across layer chunks and tool changes just as the upstream processor does. A move F word updates modal state even during wiping. Continue updating the existing current speed only through the existing standalone-feedrate/wipe rule.

At a PA tag, scan forward with a local copy of modal state. Before the first `G1` move containing X, Y and E, consume move F words. If that first extrusion has no preceding standalone `G1 F`, and a positive modal feedrate exists, assign it to the lookahead temporary maximum, next speed and current speed. This is firmware modal-state inheritance, not a guessed default speed.

Keep existing travel-after-extrusion, WIPE, role-change and end-of-chunk stopping behavior. Do not advance persistent modal state by lookahead; normal line traversal owns that state. Preserve model interpolation, bridge overrides, pressure-advance command emission and numeric formatting.

## Root and limits

On equal producer input at the affected PA feature, Native reports a nonzero next speed while the current Rust lookahead merely marks the first extrusion and leaves next speed zero. The missing state is therefore inside the processor rather than the producer of those feature bytes.

Fan commands are allocated differently around the initial layer chunks in existing adapters. No fan or chunk-boundary behavior is part of this slice. The affected causal spans, including the preceding modal F setter and the feature lookahead, must remain byte-equal. Whole-producer equality must not be claimed when only that causal boundary is equal.

The multi-tool witness also contains later motion-coordinate differences of 0.001. Those are a separate, unresolved producer boundary. Do not repair or normalize them within this slice.

## Existing failing test

Use the existing complete-output tests without modifying their fixtures or assertions:

- `adaptive_pressure_advance::scarf_ramp::adaptive_pa_scarf_ramp_s1`
- `adaptive_pressure_advance::scarf_ramp::adaptive_pa_scarf_ramp_s2`
- `adaptive_pressure_advance::sloped_loop::adaptive_pa_sloped_loop_n1`
- `adaptive_pressure_advance::sloped_loop::adaptive_pa_sloped_loop_n2`
- `adaptive_pressure_advance::tools::adaptive_pa_tool_changes`

Record all five failures before implementation. They must first fail on the unchanged complete-output comparison at `APA Next Speed`, with Native nonzero and Rust zero. No new unit seam or duplicate algorithm is needed.

## Acceptance criteria

The four single-tool tests must change from FAIL to PASS under their existing strict complete-output assertions. The multi-tool test remains FAIL, but its first difference must advance past the APA next-speed discrepancy to the already-observed later 0.001 motion-coordinate residual. Preserve the failed multi-tool result as a failure, not a passing suite result or an excluded input.

Report fan/chunk allocation differences before implementation and retain them unchanged. Any other earlier discrepancy is out of scope. Do not modify a producer, comparator, header, M73 stream or normalizer. Retain raw input/output identities and repeatable evidence for the public tests and processor-boundary diagnosis.

Run rustfmt and strict workspace/all-target clippy on each submitted commit. Passing this slice alone does not establish whole-suite parity.

## Deferred upstream behavior

Deferred: motion generation, scarf/spiral/contour-lift coordinates, fan/chunk scheduling, tree-support retraction, filament grouping/nozzle maps, full-config membership, and unrelated APA model/interpolation or bridge-domain failures. No independent Ares pipeline, fallback, duplicate processor, option or configuration surface is introduced.
