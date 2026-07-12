# M326: PrintApply apply normalization prelude

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1115-1127`: at the start of `Print::apply(...)`, materialize `print_settings_id`, `filament_settings_id`, and `printer_settings_id` options, collect `used_filaments` via `this->extruders(true)`, build `used_filament_set` from that vector, run `new_full_config.normalize_fdm_1()`, then call `new_full_config.normalize_fdm_2(objects().size(), used_filaments.size())` and keep the returned changed keys for later logging/use.

This milestone depends on existing Ares `SliceOptions::normalize_fdm_2(...)` context, prior option registry coverage for the three profile id keys, and the `Print::apply(...)` source context in `OrcaSlicer/src/libslic3r/PrintApply.cpp:1107-1133`.

## Exit criteria

- Preserve materializing the three profile id options before normalization.
- Preserve `used_filaments` source order and a membership set derived from the same values.
- Preserve call order: materialize profile ids, collect used filaments/set, run `normalize_fdm_1`, then run `normalize_fdm_2` with object count and used-filament count.
- Preserve changed-key output from `normalize_fdm_2` for later apply stages.
- Defer changed-key logging from `PrintApply.cpp:1127-1133`, support flag handling from `PrintApply.cpp:1134-1138`, scarf-seam handling, extruder variant expansion, real `DynamicPrintConfig`, real `Print`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
