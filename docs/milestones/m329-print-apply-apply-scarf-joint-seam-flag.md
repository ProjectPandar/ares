# M329: PrintApply apply scarf joint seam flag

## Source boundary

Source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:1140-1154`: immediately after the support-used assignment, `Print::apply(...)` scans model objects for a non-`None` `seam_slope_type` on the object resolved config, any volume override config, or any layer-range override config. When any source has a scarf joint seam, it sets `has_scarf_joint_seam` to `true` in `new_full_config`.

Supporting enum boundary is `OrcaSlicer/src/libslic3r/PrintConfig.hpp:216-220` and `OrcaSlicer/src/libslic3r/PrintConfig.cpp:360-365`, where `SeamScarfType` maps `none`, `external`, and `all` to `None`, `External`, and `All`.

This milestone depends on M326-M328 staged apply prelude/logging/support context and existing scarf seam option metadata. It models scarf joint seam detection as private staged data rather than wiring real `DynamicConfig`, `ModelObject`, `ModelVolume`, config mutation, or logging.

## Exit criteria

- Preserve the `seam_slope_type` queried key for object fallback, volume override, and layer-range override sources.
- Preserve `SeamScarfType` values `None`, `External`, and `All`, where only `None` is inactive.
- Preserve object-level detection: any object resolved `seam_slope_type` that is not `None` makes `has_scarf_joint_seam` true.
- Preserve volume override detection: any volume config that has `seam_slope_type` and is not `None` makes `has_scarf_joint_seam` true.
- Preserve layer-range override detection: any layer-range config that has `seam_slope_type` and is not `None` makes `has_scarf_joint_seam` true.
- Preserve empty and all-source-`None` inputs as `has_scarf_joint_seam = false`.
- Preserve a staged config-set record only when `has_scarf_joint_seam` is true, matching the upstream guarded `new_full_config.set("has_scarf_joint_seam", true)`.
- Defer logging from `PrintApply.cpp:1155`, real `BOOST_LOG_TRIVIAL`, real `DynamicConfig`, `ModelObject`, `ModelVolume`, `ConfigOptionEnum`, mutation of `new_full_config`, extruder variant expansion from `PrintApply.cpp:1157+`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.
