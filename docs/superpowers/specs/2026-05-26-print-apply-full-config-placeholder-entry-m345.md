# Spec: M345 PrintApply full_config_diff placeholder entry

## Goal

Port the `full_config_diff` placeholder-parser entry block from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1248-1256` into `ares-core` private staged state.

## Upstream source mapping

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

The Rust staging must model:

- current extruder count captured from `m_config.filament_diameter.size()`,
- `num_extruders_changed` initialized to false,
- branch entry only when `full_config_diff` is non-empty,
- staged log metadata for the changed full-config diff branch,
- staged `invalidate_step(psGCodeExport)` action,
- status aggregation from the staged invalidation result using max semantics,
- staged `m_placeholder_parser.clear_config()` action after invalidation.

## Non-goals / deferred behavior

- Do not implement placeholder preset `set(...)` calls from `PrintApply.cpp:1257-1260`.
- Do not implement placeholder `apply_config(filament_overrides)` from `PrintApply.cpp:1261-1263`.
- Do not implement config `apply_only` / `apply` / full-config assignment from `PrintApply.cpp:1264-1275`.
- Do not implement extruder-count change handling from `PrintApply.cpp:1276+`.
- Do not emit real logs or perform real `invalidate_step` calls.
- Do not perform real `DynamicPrintConfig` or `PrintConfig` lookup/mutation.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Empty `full_config_diff` records the extruder count, keeps `num_extruders_changed` false, records no branch actions, and leaves prior status unchanged.
- Non-empty `full_config_diff` records staged log metadata before invalidate/clear actions.
- Non-empty `full_config_diff` records staged `invalidate_step(psGCodeExport)` before placeholder clear.
- Non-empty `full_config_diff` records staged placeholder parser clear after G-code export invalidation.
- Invalidation result false updates unchanged prior status to changed.
- Invalidation result true updates status to invalidated.
- Invalidation result false does not downgrade an already invalidated status.
- All new symbols stay private to `ares-core` staged modules.
