# Ironing nozzle diameter from the applied filament map

## Upstream boundary

Pinned OrcaSlicer revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `src/libslic3r/Fill/Fill.cpp:1841–1843`: `Layer::make_ironing` obtains its nozzle diameter through `nozzle_diameter_for_filament` with the ironing filament ID and `Print::is_BBL_printer()`.
- `src/libslic3r/PrintConfig.cpp:148–155`: preserve that helper's exact selection. Start with the one-based filament ID. Only when BBL, the physical nozzle vector has more than one entry, and the filament ID is within `1..=filament_map.size()`, replace it with the corresponding one-based physical extruder. Read `nozzle_diameter.get_at(extruder - 1)`; an out-of-range index uses the vector's first entry.

The Rust destination is `project_slice/fill_entities/ironing/areas.rs::selected_nozzle`, with its private input wiring through `ironing.rs`, `ironing/spans.rs`, and `fill_entities/generate.rs`. Replace direct logical-ID indexing and its `InvalidInput` result; do not retain it as a fallback or add a new validation policy.

## Non-negotiable map timing

Ironing consumes the application-time map, not the later grouping result:

1. `PrintApply.cpp:1497–1503,1521–1529,1663–1670` reads the incoming map and applies it to `m_config`.
2. `Print.cpp:3482–3490` runs `PrintObject::ironing`; `PrintObject.cpp:846–853` calls `Layer::make_ironing`.
3. Only later does `Print.cpp:3608–3618` build static ToolOrdering; the tower path constructs it in `Print.cpp:5235–5245`.
4. `GCode/ToolOrdering.cpp:3079–3081` publishes grouping through `Print::update_filament_maps_to_config`; `Print.cpp:4656–4662` then writes the derived map into `m_config`. The sequential static path likewise writes back after geometry generation (`Print.cpp:3700–3734`).

Ares already retains this boundary as `resolved.applied_filament_map` in `project/effective_config/phases.rs`, captured after materialization/normalization and before the temporary recommendation shim. Ironing must borrow that snapshot. `resolved.views.full.project.gcode.filament_map` is not an interchangeable source: the temporary all-master shim can already have overwritten it before fill generation. Do not move grouping earlier, reconstruct a map from emitted G-code, or use the eventual grouping result.

Use the existing selected/system-aware `project_is_bbl` identity adapter. Preserve the physical nozzle vector and the upstream BBL/size/ID gates, including first-entry `get_at` behavior. The validated internal nozzle vector is nonempty; this helper must not introduce an error for a valid third logical filament on a two-head printer.

## Included and deferred behavior

Included: nozzle selection shared by ironing area inset and ironing extrusion dimensions, and the private context necessary to read the applied map.

Unchanged: ironing enablement, pattern choice, span ordering, geometry generation, flow arithmetic, role/selector values, retraction, motion, configuration-column materialization and automatic grouping. Other nozzle getters and different-head diameter parity are not qualified by this slice.

Public verification uses the four three-material Default/Fast and zero/high-load-time cases in `filament_group_flush_context`. Their Native runs use all three logical filaments; the initial API rejection is `invalid Orca option nozzle_diameter`. With this prerequisite, the cases must produce public API/CLI outputs and reach their separate matrix/map assertions. Reaching those assertions does not imply grouping or whole-output parity. Frozen input bytes, inherited ironing enablement and the existing replay/strict comparators remain unchanged.
