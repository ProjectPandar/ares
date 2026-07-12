# Spec: M338 PrintApply apply filament_map auto-mode gate

## Goal

Port the `filament_map_mode` lookup and `map_mode < fmmManual` gate from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1194-1195` into `ares-core` private staged state.

## Upstream source mapping

```cpp
FilamentMapMode map_mode = new_full_config.option<ConfigOptionEnum<FilamentMapMode>>("filament_map_mode", true)->value;
if (map_mode < fmmManual) {
```

Supporting enum order:

```cpp
enum FilamentMapMode {
    fmmAutoForFlush,
    fmmAutoForMatch,
    fmmManual,
    fmmDefault
};
```

The Rust staging must model:

- source config identity `new_full_config`,
- option key `filament_map_mode`,
- required lookup flag `true`,
- local value identity `map_mode`,
- auto-mode decision from upstream enum ordering.

## Non-goals / deferred behavior

- Do not implement the inner auto-mode `filament_map` diff-prune branch from `PrintApply.cpp:1196-1203`.
- Do not implement the manual-mode `else` branch from `PrintApply.cpp:1205-1226`.
- Do not implement reassignment of `print_diff` from `PrintApply.cpp:1227-1228`.
- Do not perform real `DynamicPrintConfig` lookup or mutation.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Staged gate records source config `new_full_config`.
- Staged gate records option key `filament_map_mode`.
- Staged gate records required lookup flag `true`.
- Staged gate records value identity `map_mode`.
- `fmmAutoForFlush` evaluates as auto mode.
- `Auto For Flush` evaluates as auto mode.
- `fmmAutoForMatch` evaluates as auto mode.
- `Auto For Match` evaluates as auto mode.
- `fmmManual` evaluates as non-auto mode.
- `Manual` evaluates as non-auto mode.
- `fmmDefault` evaluates as non-auto mode because it is ordered after `fmmManual`.
- All new symbols stay private to `ares-core` staged modules.
