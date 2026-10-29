# Textured-PEI filament-start elsif rewrite slice

## Source boundary

This slice ports the `elsif` keyword boundary of
`OrcaSlicer/src/libslic3r/PlaceholderParser.cpp`: `kw` uses
`distinct(alnum | '_')` at line 2110, and `if_else_output` consumes
`kw["elsif"] > bool_expr_eval` at lines 2178–2186. An opening parenthesis
is a legal immediate successor; whitespace is not required.

Rust destination:
`ares-core/src/project_slice/gcode_emit/template/parser.rs::Parser::next_directive`.
The existing recursive branch parser remains the compatibility shell for this
upstream grammar; this is not a new slicing or template pipeline.

## Diagnostic result

The X1C stock filament-start script uses `{elsif(...)}`. On Textured PEI
at 55 C it must emit `M106 P3 S150`. Both Native and Ares already bind
`bed_temperature[0]` and `bed_temperature_initial_layer[0]` to 55:
`GCode.cpp:3023–3033,3043` selects plate vectors using
`PrintConfig.hpp:466–510 get_bed_temp_key/get_bed_temp_1st_layer_key`.
Diagnostic expressions also evaluate `bed_temperature[0] > 50` to true on
both implementations. Changing temperature bindings would therefore be the
wrong fix.

Ares recognizes `if(` but recognizes `elsif` only before space/tab/`}`.
The stock elsif arms are consequently swallowed as text in the false first
arm and never selected. Port the missing `(` boundary, matching the existing
`if` handling. Leave the bed-vector bindings unchanged.

## Included / deferred

Included: the immediate-parenthesis elsif spelling and the stock X1C
filament-start behavior it controls. `else` and `endif` use the same upstream
`kw` distinct rule at `PlaceholderParser.cpp:2184–2186`; Ares currently recognizes
their closing-brace forms and rejects identifier continuations. This slice does
not expand those directives without a separate Native-first witness. Deferred:
other keyword-boundary or extended statement-grammar differences, unrelated
parser refactoring, bed-temperature formula changes, and UI behavior.

## Regression contract

Freeze `tests/parity/adaptive-octree-spacing/baseline.3mf` with only the
project-setting `curr_bed_type` changed to `Textured PEI Plate`. Keep all
other ZIP entry payloads, all temperatures, and the stock filament script.
Commit the input fixture only, never Native output.

The E2E test first qualifies two ordered Native streams, asserts that the
55 C heat commands and exactly one `M106 P3 S150` are present, then compares
the complete public API and CLI output with the canonical comparator.
The same test runs against official and seed0 replay Native, CPU17/thread1.
The RED must fail because Ares omits the fan command; GREEN must preserve
whole-stream equality, not merely satisfy a substring check.
