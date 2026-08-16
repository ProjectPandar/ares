# Spec: Task 22o.118 selected conditional template whitespace

## Observable contract

A selected single-branch `{if ...}{endif}` custom-G-code block preserves a blank line for both the opening directive and its closing `endif`. An unselected single branch preserves the opening directive blank but does not emit its closing blank. Multi-branch `if`/`elsif`/`else` behavior remains unchanged.

The KSR first-layer custom G-code therefore retains the exact blank-line sequence around its selected nested fan-control branch, including one blank line between `M106 P10 S102` and `;not reset fan`.

## Upstream boundary

Port the observable newline behavior of OrcaSlicer's `PlaceholderParser` as consumed by `GCode.cpp` layer-change template rendering. The Rust destination is `project_slice/gcode_emit/template.rs`; parsing and expression evaluation are unchanged.

## Deferred behavior

M73 time-progress post-processing and other placeholder expression forms remain separate source-cited slices.
