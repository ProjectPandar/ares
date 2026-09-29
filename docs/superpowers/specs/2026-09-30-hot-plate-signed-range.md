# Signed first-layer hot-plate range

Port coverage of `PrintConfig.cpp:1061` and its numeric validation loop,
`Config.hpp::ConfigOptionInts`, and `GCode.cpp::get_bed_temperature` /
`get_highest_bed_temperature`. Rust destinations are CLI option-domain
generation and public full-output tests, with any demonstrated emission
discrepancy confined to the corresponding existing temperature rewrite.
Existing emission is a temporary compatibility shell around these upstream
boundaries; this is not an independent slicing pipeline feature.

Investigate the representable signed minimum (no explicit schema minimum),
negative one, maximum 300 and deterministic random interior using the active
High Temp Plate fixture. Verify effective values and actual G-code. Never
call zero the minimum without upstream evidence. Rejections and divergences
must remain visible rather than excused as coverage.

Include both formulas for one filament and both vendor-script/automatic startup.
Fix demonstrated `get_highest_bed_temperature` zero initialization in
`project_slice/gcode_emit/placeholders.rs` and `machine/temperature.rs`, plus
the latter's zero-temperature startup emission to match `GCodeWriter.hpp`'s
initial reached-temperature cache and `GCodeWriter.cpp:168-175`.

Deferred: other plate domains, multi-filament formulas, and remaining global
printer/option coverage. Accept only after full-output evidence, workspace
Nextest, fmt, Clippy, independent six-aspect review and commit/push.
