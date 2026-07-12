# M328: PrintApply apply support-used flag

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1134-1138`: immediately after the changed-key logging block, `Print::apply(...)` reads `new_full_config.option("enable_support")` and sets `m_support_used` to `true` only when the option exists and `getBool()` is true; otherwise it sets `m_support_used` to `false`.

This milestone depends on the staged apply prelude/logging context from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1115-1133` and `enable_support` option metadata already covered by the option registry. It models the support-used assignment as private staged data rather than wiring real `DynamicPrintConfig` or `Print` mutation.

## Exit criteria

- Preserve querying exactly the `enable_support` option key.
- Preserve `true` assignment only when the option exists and its bool value is true.
- Preserve `false` assignment when the option is missing.
- Preserve `false` assignment when the option exists and its bool value is false.
- Preserve an assignment record for every evaluated input so later apply stages can observe the staged `m_support_used` value.
- Defer scarf-seam handling from `PrintApply.cpp:1140+`, extruder variant expansion, real `ConfigOption`, real `DynamicPrintConfig`, real `Print::m_support_used` mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
