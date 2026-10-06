# libvgcode color ranges and view types (web preview)

Source-cited rewrite slice of AGPL-licensed OrcaSlicer `src/libvgcode`. No ares-core change.

## Upstream boundary (ported)

| Upstream | Rust destination |
|---|---|
| `libvgcode/include/ColorRange.hpp` `DEFAULT_RANGES_COLORS` (11 colors), class `ColorRange`: `m_type`, `m_palette`, `m_range{FLT_MAX,-FLT_MAX}`, `m_count`, `update`, `reset`, `get_range`, `get_values`, `get_color_at`; `ColorRange.cpp:11-139` incl. `get_step_size` (log step 0 when `range[0] == 0`) | new `crates/ares-vgcode/src/color_range.rs` (`ColorRange`, `DEFAULT_RANGES_COLORS`) |
| `ViewerImpl.cpp:47-63` `round_to_bin` | `color_range.rs` (crate-private) |
| `ViewerImpl.cpp:1817-1878` `update_color_ranges` (height/width/fan/temperature/volumetric on extrusions, width+volumetric gated by Custom role visibility, speed also on travels/wipes when those options are visible, layer time linear+log from `Layers::get_times`), incl. the settings cache | new `crates/ares-vgcode/src/viewer/colors.rs` (`impl ViewerImpl`) |
| `ViewerImpl.cpp:1313-1317` `set_view_type`; `ViewerImpl.cpp:1467-1553` `get_vertex_color`; `ViewerImpl.cpp:1599-1621` `get_color_range`; `Settings.hpp:19-20` `view_type`, `time_mode` | `viewer/colors.rs`; `Settings` gains `view_type` (default `FeatureType`) and `time_mode` (default `Normal`) |
| `ViewerImpl.cpp:283-317` `DEFAULT_EXTRUSION_ROLES_COLORS`, `DEFAULT_OPTIONS_COLORS` (needed by `get_vertex_color` for FeatureType, travels, wipes and options) | `viewer/colors.rs` consts; the TS `ROLE_COLORS` / `OPTION_COLORS` tables are deleted and read from wasm instead |

The `ViewType` and `ColorRangeType` enums already exist in `ares-vgcode/src/types.rs`, in `Types.hpp:80-103` order.

## wasm: thin bindings only (`crates/ares-wasm/src/moves.rs`)

- `VertexBuffers.vertexColors(viewType) -> Uint8Array`: calls `set_view_type`, then `get_vertex_color` for each vertex as RGB. Visibility comes from the latest `viewRange` call, which already sets the role and option visibility.
- `VertexBuffers.colorRange(viewType) -> {type, range:[min,max], values, palette}` as a typed serde struct, `get_color_range` + `get_values`.
- `viewTypes()`, `roleColors()`, `optionColors()` export the names and default palettes.

## App

- The legend gets a view-type `<select>` with the `get_view_type_string` labels (`GCodeViewer.cpp:66-101`). It uses the `view_type_items` order (`GCodeViewer.cpp:1077-1093`), filtered to the view types this slice supports.
- Range views use `append_range` (`GCodeViewer.cpp:3293-3313`): one swatch row per value, from the top of the palette down. Decimals follow `GCodeViewer.cpp:3911-3986`: Height/Width/Flow 2, Speed/Fan/Temperature 0, Layer Time 1. Speed also keeps the Travel option row (3913-3929).
- The toolpath mesh takes its per-vertex colors from `vertexColors`. TS does no range, color or binning math.

## Included view types

FeatureType, Height, Width, Speed, FanSpeed, Temperature, VolumetricFlowRate, LayerTimeLinear and LayerTimeLogarithmic. Time mode is Normal.

## Deferred

- **ActualSpeed, ActualVolumetricFlowRate, Acceleration, Jerk, PressureAdvance:** ares-core `MoveVertex` has no `actual_feedrate`, `acceleration`, `jerk` or `pressure_advance`, and `convert.rs` sets them to 0. Fixing that is a core gap, which I will report.
- **ColorPrint, Tool, Summary:** they need filament usage statistics for the legend, and their colors come from the colour-print / tool palettes.
- **Not included:**
  - the stealth time mode;
  - `set_color_range_palette`;
  - the dark-grey colors from `update_colors_texture` (layers outside the range);
  - the legend's time/percent columns.

## Tests (RED first)

- **Rust** (`crates/ares-vgcode/tests/color_range.rs`). Each oracle value is derived by hand from the upstream code:
  - **Unset range:** `get_color_at` keeps the `std::clamp` semantics without panicking when min > max.
  - **One value:** `get_values` has length 1, and the color is `palette[0]` because the step is 0.
  - **Two values:** length 2; `update(1), update(2), update(1)` keeps the count at 2.
  - **Linear:** midpoint color and the 11 `get_values`.
  - **Logarithmic with min 0:** the step is 0, so every color is `palette[0]`.
  - **Logarithmic with min > 0:** values are geometric.
  - **`round_to_bin` thresholds:** 0.2, 0.0949, 1e-7.
  - **`update_color_ranges` visibility gates:** the Custom role on width; travels on speed.
- **Browser E2E** (extend `toolpathPreview.ts`, so the spec count stays at 14):
  - Expected min and max per view type are computed from the native procgcode `--dump-moves` CSV. The test applies the same vertex filters and `round_to_bin` from the cited upstream lines.
  - For each supported view type, the legend's top and bottom rows must equal `%.Nf` of max and min. Choosing a view type must change the canvas digest.
  - Screenshots for Height, Speed and Layer Time (log).
  - The legend for FeatureType stays unchanged.
