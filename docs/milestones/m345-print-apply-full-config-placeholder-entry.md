# M345: PrintApply full_config_diff placeholder entry

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1248-1256`: after print-diff config invalidation, `Print::apply(...)` prepares placeholder-parser update state by reading the current extruder count from `m_config.filament_diameter.size()`, initializing `num_extruders_changed` to false, entering the block only when `full_config_diff` is non-empty, logging that `full_config_diff` changed, invalidating `psGCodeExport` through `update_apply_status(this->invalidate_step(psGCodeExport))`, and clearing the placeholder parser config.

```cpp
// Apply variables to placeholder parser. The placeholder parser is used by G-code export,
// which should be stopped if print_diff is not empty.
size_t num_extruders  = m_config.filament_diameter.size();
bool   num_extruders_changed  = false;
if (! full_config_diff.empty()) {
    //BBS: add more logs
    BOOST_LOG_TRIVIAL(info) << __FUNCTION__ << boost::format(" %1%: found full_config_diff changed.")%__LINE__;
    update_apply_status(this->invalidate_step(psGCodeExport));
    m_placeholder_parser.clear_config();
```

Supporting context is M343's staged max-based status update. This milestone models only initial extruder-count capture, `num_extruders_changed` initialization, the `full_config_diff` non-empty gate, staged log metadata, staged `invalidate_step(psGCodeExport)` call, status aggregation from that invalidation result, and staged placeholder-parser clear action.

## Exit criteria

- Preserve initial `num_extruders` from `m_config.filament_diameter.size()`.
- Preserve `num_extruders_changed = false` initialization.
- Preserve no full-config placeholder actions when `full_config_diff` is empty.
- Preserve staged log metadata only when `full_config_diff` is non-empty.
- Preserve staged `invalidate_step(psGCodeExport)` call only when `full_config_diff` is non-empty.
- Preserve feeding the staged G-code export invalidation boolean into max-based status update.
- Preserve staged `m_placeholder_parser.clear_config()` after G-code export invalidation within the non-empty branch.
- Preserve changed status when invalidation returns false from an unchanged prior status, invalidated status when it returns true, and no downgrade from invalidated.
- Keep all new Rust symbols private to `ares-core` staged `print_apply` modules.
- Defer placeholder preset assignments from `PrintApply.cpp:1257-1260`, placeholder `apply_config(filament_overrides)` from `PrintApply.cpp:1261-1263`, config mutation from `PrintApply.cpp:1264-1275`, extruder-count change handling from `PrintApply.cpp:1276+`, real logging, real invalidation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
