# ARD-0048: Exercise exhaust-fan gates independently of support geometry

## Upstream boundary

`PrintConfig.cpp:1886-1921` defines three filament Boolean gates and two
0..100 exhaust speeds; `PrintConfig.cpp:3899` defines the printer capability.
`GCode.cpp::_do_export` (3182-3196, 3502-3515) requires the capability, master
filament gate and respective phase gate, then selects the maximum speed of
used extruders. `GCodeWriter.cpp::set_exhaust_fan` emits `M106 P3` with
truncated percentage-to-255 conversion.

## Decision

The Rust destination is the CLI option coverage activation and public
project-to-complete-G-code tests, validating the existing
`ares-core::project_slice::gcode_emit::machine::exhaust_fan` rewrite.
`support_air_filtration` means hardware capability, not support material:
do not enable model supports, select overhang geometry, or excuse a fan
divergence as an unimplemented support generator.

For each of the six option domains, enable the other gates and use distinct
start/end speeds (40/80). Apply the tested value last. Verify exported
settings and the original's exact ordered P3 commands before comparing the
entire output. Exercise both Boolean values and numeric min/max/seeded
interior, with no new normalization or oracle retries in these tests.

This includes single-filament phase gating and speed conversion. Used-tool
maximum selection, custom-template fan commands, full-comment mode and
multi-material interactions are deferred; no all-context claim follows.
The current CLI harness is a temporary compatibility shell around those
upstream behaviors, not an independent Ares pipeline design.
