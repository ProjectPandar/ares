# Non-tower flush volume precision

## Upstream boundary

Port the numeric conversions in OrcaSlicer v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/GCode.cpp`, `GCode::set_extruder` (10990), to
`project_slice/gcode_emit/tool_transition/change/dynamic/flush.rs::insert_flush`.
The existing placeholder builder remains the mapping from prepared options to
change-filament G-code; it is not a new purge algorithm.

| Upstream expression | Type and conversion |
| --- | --- |
| `get_flush_volumes_matrix`, `GCode.cpp:11159`; `PrintConfig.hpp:2115,2691–2699` | The option and selected block contain `double`; `cast<float>` makes the selected matrix cells `float` before lookup. |
| `wipe_volume *= flush_multiplier.get_at(...)`, `GCode.cpp:11151,11178–11193`; `PrintConfig.hpp:2137` | The selected `float` cell is promoted to `double` for multiplication with the `double` option, then stored back as `float`. This applies both to the outgoing filament and to the filament remembered on a different physical extruder. |
| `grab_purge_volume`, `GCode.cpp:11177,11194`; `PrintConfig.hpp:2162` | Multiply the `double` option by the `double` literal `2.4`, then store as `float`; subtract and clamp in `float`. |
| `filament_area`, `wipe_length`, `GCode.cpp:11154,11207`; `PrintConfig.hpp:1732` | Compute area from the `double` diameter in `double`, then cast to `float`. Divide the `float` volume by the `float` area in `float`. |
| Placeholder lengths, `GCode.cpp:209–210,11263–11264,11327–11341` | Halves use `2.f`. Volume divided by the `float` constant `135.f` gives a `float` for `round`; the rounded count is `int`, capped at four. The split divides the `float` length by the count converted to `float`; the scalar option then widens each written length to `double`. Unused slots stay zero. |
| `filament_tower_interface_purge_volume`, `GCode.cpp:11276`; `PrintConfig.hpp:1902` | Despite its name, this placeholder is also bound on the non-tower path. It copies a `double` option directly into a `double` scalar, without a `float` intermediate. |

`Config.hpp:676–682,824–828,876,1015` defines vector lookup and the
`double` scalar/vector option types. Keep the matrix cast and the existing
`float` length and split calculations. Correct only the premature narrowing
of `flush_multiplier`, `grab_length`, and the direct interface-volume option.
Tool selection, travel placeholders, feedrates, tower calculations and
remaining-time accounting are outside this slice.

## Observable behavior

Slice a synthetic half-scale overhang with the X1C and prime tower disabled.
With two PLA slots, selecting slot 2 for support or support interface causes
a real tool change. The return to slot 1 uses the 229 mm³ cell and the 0.3
option: the stored volume is about 68.699997 mm³, not 68.700005 mm³. The stock
template renders `G1 E0.0972428 F50`. Compare the entire ordered public API and
CLI output with Native, normalizing only the generator line. A single-slot
version must remain unchanged and must not switch to a second tool. Also
check an existing multi-material, tower-disabled fixture and compare the
related tool-change, flush, multi-material and support inventories in both
Native modes.
