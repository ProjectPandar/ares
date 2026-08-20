# Plan: KSR FDM Test V4 task232 module and oracle cleanup

1. Extract machine limits and machine-start template rendering into a normal `gcode_emit::machine` module without behavior changes.
2. Remove compile-only region interface pins so `region_slices.rs` remains below 400 lines.
3. Delete obsolete encoded task22J/task22K intermediate-stream tests and retain region behavior checks using order-independent polygon membership.
4. Run focused G-code emitter and region composition tests, verify all Rust source LOC, then run rustfmt and focused Clippy.
5. Record the cleanup boundary in the roadmap; commit and push independently.
