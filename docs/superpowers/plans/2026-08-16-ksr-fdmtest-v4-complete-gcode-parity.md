# Plan: KSR FDM Test V4 complete G-code parity

1. Freeze the external golden seam and record structural deltas: valid finite extrusion, role/speed/acceleration tags, travel/retraction/wipe, seams/arcs, lifecycle templates, timing, and final statistics.
2. Port `GCode::_extrude` option resolution and `GCodeWriter` linear motion into small G-code emitter modules; verify the first emitted entities against fixture-derived expected lines, then commit and push.
3. Port loop start selection, clipping, travel/retraction/lift/wipe, and sortable infill chaining through the retained entity seam; verify structural counts and exact next divergence, then commit and push.
4. Port `enable_arc_fitting` output and required geometric fitting state; verify G2/G3 output and exact next divergence, then commit and push.
5. Port object/layer/end lifecycle templates, fan state, progress, and option-driven custom G-code; verify the executable body, then commit and push.
6. Port G-code processing for printing-time metadata, M73 insertion, and filament statistics; remove the golden test ignore and require normalized byte equality, then commit and push.
7. Remove obsolete encoded source-pinning/oracle tests and exports, preserve behavior tests, split any Rust source/test file approaching 400 LOC into normal modules, then commit and push.
8. Run the fixture E2E, workspace nextest, strict workspace clippy, rustfmt check, and LOC/macro checks.
9. Start an independent read-only reviewer covering requirement completeness, logic, edge cases, code quality, tests, and actual runtime output. Apply its checklist in the main thread and repeat review until approved.