# Preserve nullable wire input through raw legacy handling

Source boundary: OrcaSlicer libslic3r revision
345efec64c7b95a05ce5cf96df0dde66a158835c.
Rust boundary: project/raw_settings.rs::ProjectSettingsRaw::handle_legacy.

Config.cpp:597-611 copies the original deserialize input before handle_legacy;
626-666 looks up the canonical option and assigns it. PrintConfig.cpp:8779-8809
constructs nullable filament retraction overrides from their extruder options,
including filament_deretraction_speed. Config.hpp:932-958 distinguishes the
empty whole wire string (accepted as a single zero at 937-939) from literal nil
(accepted as the nullable sentinel at 945-949). They are not equivalent.

The raw load/merge path must therefore retain a Text value's literal deserialize
input when calling legacy normalization. The export-only token() intentionally
suppresses literal nil; reusing that projection as a deserialize input loses it.
A profile's scalar nil previously became an empty string, and per-filament
set_at-style overlay stored it as a vector element. The new numeric export
serializer exposed this pre-existing corruption as invalid FloatsNullable "".
Preserve nil at the raw import/legacy owner instead of converting invalid
export elements to nil, substituting zero, skipping validation or weakening
alias assertions. Nil output omission remains owned by the typed vector and
append_full_config policies.

The legacy alias contract itself is unchanged: PrintConfig.cpp:9768-9795
renames legacy feature-filament keys and maps legacy 1 to canonical 0. That exact
block equals 2.4.2 PrintConfig.cpp:8050-8077 byte-for-byte. Config.cpp:937
iterates the JSON map and 597-611/626-666 assigns each renamed value; sorted
source-key last-write precedence remains the owning upstream behavior.

Adjacent test-data boundary: PrintConfig.cpp:10002-10035 retires silent_mode
(at 10017). It must not return to the registry to maintain an old test counter.
The WASM codec witness keeps its minimum 643 real-value and zero-failure
assertions and broadens coverage to a second unchanged real 3MF. Both input
paths are recorded in its report and per-case source labels; every original
valid case still executes. No synthetic production defaults, Native output
fixtures or revived obsolete key is introduced. The remaining scalar/vector
migration is outside this fix.
