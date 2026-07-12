# M338: PrintApply apply filament_map auto-mode gate

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1194-1195`: inside the M337 `filament_map_mode`-absent guard, `Print::apply(...)` reads `filament_map_mode` from `new_full_config` as `ConfigOptionEnum<FilamentMapMode>` and enters the auto-mode branch when `map_mode < fmmManual`.

```cpp
FilamentMapMode map_mode = new_full_config.option<ConfigOptionEnum<FilamentMapMode>>("filament_map_mode", true)->value;
if (map_mode < fmmManual) {
```

Supporting enum ordering comes from `OrcaSlicer/src/libslic3r/PrintConfig.hpp:424-428`: `fmmAutoForFlush`, `fmmAutoForMatch`, `fmmManual`, `fmmDefault`. Supporting name-map context comes from `PrintConfig.cpp:577-582`: `Auto For Flush`, `Auto For Match`, and `Manual`. This milestone models only lookup identity, enum ordering, and the auto-mode decision as private staged data.

## Exit criteria

- Preserve source config identity `new_full_config`.
- Preserve option key `filament_map_mode`.
- Preserve required lookup flag `true`.
- Preserve local result identity `map_mode`.
- Preserve enum order `fmmAutoForFlush < fmmAutoForMatch < fmmManual < fmmDefault`.
- Preserve named input forms `Auto For Flush`, `Auto For Match`, `Manual`, and internal variant names for staged tests.
- Preserve auto-mode gate as true only when `map_mode < fmmManual`.
- Preserve `fmmManual` and `fmmDefault` as non-auto for this gate.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer the auto-mode inner `filament_map` branch from `PrintApply.cpp:1196-1203`, manual branch from `PrintApply.cpp:1205-1226`, `print_diff` reassignment from `PrintApply.cpp:1227-1228`, real config lookup, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
