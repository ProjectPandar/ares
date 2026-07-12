# Spec: M326 PrintApply apply normalization prelude

## Goal

Port the `Print::apply(...)` normalization prelude from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1115-1127` into `ares-core` private staged state.

## Upstream source mapping

```cpp
// Normalize the config.
new_full_config.option("print_settings_id",            true);
new_full_config.option("filament_settings_id",         true);
new_full_config.option("printer_settings_id",          true);

// BBS
std::vector <unsigned int> used_filaments = this->extruders(true);
std::unordered_set <unsigned int> used_filament_set(used_filaments.begin(), used_filaments.end());

//new_full_config.normalize_fdm(used_filaments);
new_full_config.normalize_fdm_1();
t_config_option_keys changed_keys = new_full_config.normalize_fdm_2(objects().size(), used_filaments.size());
```

The Rust staging must model:

- materialized profile id keys in source order,
- used-filament vector and set derived from it,
- object count and used-filament count passed to the staged `normalize_fdm_2` call,
- changed-key output from the staged normalization call,
- operation/call ordering for verification.

## Non-goals / deferred behavior

- Do not implement changed-key logging from `PrintApply.cpp:1127-1133`.
- Do not implement support flag handling from `PrintApply.cpp:1134-1138`.
- Do not implement scarf-seam handling, extruder variant expansion, real `DynamicPrintConfig`, real `Print`, public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` normalization APIs for this private staged milestone.

## Acceptance criteria

- Profile id keys are materialized in exact source order: `print_settings_id`, `filament_settings_id`, `printer_settings_id`.
- Used-filament vector preserves the provided extruder order.
- Used-filament set deduplicates the same values.
- `normalize_fdm_1` is recorded before `normalize_fdm_2`.
- `normalize_fdm_2` receives the supplied object count and the used-filament vector length, not the deduplicated set length.
- Changed keys returned by the staged `normalize_fdm_2` behavior are preserved in source order.
- Empty used-filament input passes used-filament count `0` and produces an empty set.
- All new symbols stay private to `ares-core` staged modules.
