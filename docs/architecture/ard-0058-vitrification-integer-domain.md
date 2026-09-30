# ARD-0058: Vitrification temperature integer-domain parity

Upstream ownership: `libslic3r/PrintConfig.cpp:2917-2924` declares
`temperature_vitrification` as `coInts` without narrower bounds;
`Config.hpp:995-1085` stores signed `int` elements, while
`Config.hpp:2475-2479` supplies default schema limits beyond that storage
domain. However, `Config.hpp:1069-1077` reserves INT_MAX as nil and throws
`Serializing NaN` for that value in non-nullable vectors. The serializable
domain is therefore INT_MIN through INT_MAX-1. Test both legal endpoints and
retain an explicit original-binary rejection test for INT_MAX, not a parity pass.
`GCode.cpp:3017-3040` computes the minimum over used extruders and supplies
`min_vitrification_temperature` to the start template.

Rust destinations are the existing project option wire representation and
`project_slice/gcode_emit/placeholders.rs`. The native coverage adapter in
`ares-cli/tests/orca_parity/option_coverage` is a temporary fixture construction
shell around those upstream concepts, not an independent slicing pipeline.

Include one-filament endpoint and seeded-interior configuration round trips,
start-template expansion, and complete ordered first-reference G-code parity.
Retain the existing 110-degree sample. Do not reinterpret negative values as
physically recommended temperatures: this is serialization and slicing parity,
not authorization to run generated G-code on hardware.

Defer used-extruder selection across multiple filaments, arrangement warnings,
viewer temperature storage and the separate generic SliceOptions API. Change
production only if the original-binary comparison exposes a mismatch. Preserve
strict application proof and full-stream comparison; rejected endpoints cannot
be reported as parity passes.
