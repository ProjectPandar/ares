# Active high-temperature plate parity

## Source boundary and destination

Cover `libslic3r/PrintConfig.cpp:1061`'s first-layer hot-plate temperature,
`PrintConfig.hpp::get_bed_temp_1st_layer_key`, and `GCode.cpp:3015-3047`'s
selected-plate temperature placeholders. The Rust rewrite owners are
`options/bed_temperature.rs` and `project_slice/gcode_emit/machine/temperature.rs`.
This slice changes only the CLI parity harness's activation and public-output
tests; existing emission remains a temporary compatibility shell around those
upstream concepts, not a new pipeline.

Select High Temp Plate for these option probes, verify the exported setting,
and require the requested temperature in actual M140/M190 commands before
comparing complete official/Ares outputs. Include zero, 55 and the declared
maximum 300. These are authored probes, not a complete range: upstream declares
no minimum here. Correct misleading minimum/seeded labels.

Deferred: establishing the negative/lower accepted domain, generated random
interior coverage, multi-filament temperature formulas, other plate options,
and all-printer/default parity. Do not promote this domain to complete.

## Acceptance

Demonstrate that the old fixture fails the selected-plate assertion, then pass
all three active full-output comparisons. Run fmt, Clippy and workspace Nextest;
obtain independent six-aspect review and commit/push code and docs together.
