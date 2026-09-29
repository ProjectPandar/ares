# ARD-0034: Temperature probes must select their owning plate

`GCode.cpp:3015-3047` selects first-layer temperature through `curr_bed_type`.
Therefore `hot_plate_temp_initial_layer` coverage must export High Temp Plate
and verify its actual heating commands, not merely the config comment.
Keep this dependency local to the option case, never modify global presets.

`PrintConfig.cpp:1061-1069` declares maximum 300 but no minimum. Zero is a
documented unsupported-filament sentinel, not evidence of a schema minimum.
Retain INCOMPLETE until the missing domain coverage is addressed; never weaken
the full-output comparator or count configuration round-trip as activation.
