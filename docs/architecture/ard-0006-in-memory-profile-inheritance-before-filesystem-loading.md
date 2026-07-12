# ARD-0006: In-memory profile inheritance before filesystem loading

## Status
Accepted for M7.

## Decision
Ares will first port OrcaSlicer profile inheritance as an in-memory `ares-core` operation over JSON bytes/fragments. `ares-core` will parse profile fragments, resolve same-kind `inherits` chains, and return merged `SliceOptions`, but it will not scan resource directories or read files.

## OrcaSlicer structure evidence
- `OrcaSlicer/src/libslic3r/Preset.hpp` defines process, filament, and machine preset categories and the `inherits` JSON key.
- `PresetCollection::load_presets` in `OrcaSlicer/src/libslic3r/Preset.cpp` loads profile JSON and applies inherited parent profiles.
- `PresetCollection::get_preset_parent` / `get_preset_base` traverse inheritance through preset collections.
- Orca resource profiles use chained JSON fragments under `resources/profiles/<vendor>/{process,filament,machine}`.

## Consequences
- Browser/WASM callers can supply profile bytes without granting filesystem access to the core.
- CLI and future UI adapters can own profile discovery and pass fragments into the core.
- Full vendor bundle loading, compatibility filtering, aliases, and cross-kind composition stay explicit later milestones.
