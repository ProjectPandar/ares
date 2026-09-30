# ARD-0060: Filament-printable integer domain and single-extruder behavior

Upstream boundary: `libslic3r/PrintConfig.cpp:2905-2914` declares
`filament_printable` as coInts and documents the left/right printable bits.
`Config.hpp:995-1085,2475-2479` supplies signed integer storage, the reserved
INT_MAX nil value and default numeric schema bounds. The legal serializable
integer interval is INT_MIN through INT_MAX-1, as for ARD-0058.

Behavior owner: `Print.cpp:3215-3248`,
`Print::get_physical_unprintable_filaments`, returns empty constraints for a
single extruder. Multi-extruder behavior additionally belongs to
`GCode/ToolOrdering.cpp:67-82` and the CLI mapping check at
`OrcaSlicer.cpp:5944-5968`; changing a bitmask is not proof those branches ran.

Rust destination is the existing project option wire/config output boundary,
with native `option_coverage` fixture-domain generation as a temporary evidence
shell around those upstream concepts. No independent Ares pipeline is added.

Include exact single-extruder min/max/full-interval seeded round trips and
complete ordered output comparisons, plus masks 0, 1, 2 and 3. Retain INT_MAX
as an explicit serialization-rejection negative, not successful G-code parity.
Do not interpret arbitrary signed masks as recommended machine settings or
execute generated G-code on hardware.

Defer two-extruder grouping, automatic/manual material mapping and conflict
diagnostics. Single-extruder sample completion must be described as such; it
does not complete the global printer/option cross-product or those behaviors.
Any actual production mismatch must first have original-binary output evidence.
