# Spec: M328 PrintApply apply support-used flag

## Goal

Port the `Print::apply(...)` support-used flag assignment from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1134-1138` into `ares-core` private staged state.

## Upstream source mapping

```cpp
const ConfigOption* enable_support_option = new_full_config.option("enable_support");
if (enable_support_option && enable_support_option->getBool())
    m_support_used = true;
else
    m_support_used = false;
```

The Rust staging must model:

- querying the `enable_support` option key,
- the missing-option branch,
- the present false branch,
- the present true branch,
- the resulting staged `m_support_used` assignment.

## Non-goals / deferred behavior

- Do not implement real `ConfigOption` or `DynamicPrintConfig` lookup.
- Do not mutate a real `Print::m_support_used` field.
- Do not implement scarf-seam handling from `PrintApply.cpp:1140+`.
- Do not implement extruder variant expansion, public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Staged evaluation records the queried key as exactly `enable_support`.
- Missing option input produces staged `support_used = false`.
- Present false input produces staged `support_used = false`.
- Present true input produces staged `support_used = true`.
- The staged assignment is returned for each evaluation.
- All new symbols stay private to `ares-core` staged modules.
