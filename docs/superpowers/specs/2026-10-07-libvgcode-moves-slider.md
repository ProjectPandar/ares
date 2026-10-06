# libvgcode visible view range and the preview moves slider (web preview)

Source-cited rewrite slice of AGPL-licensed OrcaSlicer `src/libvgcode` and the rendering-neutral part of `src/slic3r/GUI/GCodeViewer.cpp` (moves slider). No ares-core change.

## Upstream boundary (ported)

| Upstream | Rust destination |
|---|---|
| `ViewerImpl.cpp:1450-1459` `set_view_visible_range` (full range refresh, `ViewRange::set_visible`, enabled entities, colors texture) | `crates/ares-vgcode/src/viewer.rs` `ViewerImpl::set_view_visible_range` |
| `ViewerImpl.cpp:1130-1184` `update_enabled_entities` (top-layer-only start, one extra step for an option at the current vertex, spiral vase step, valid line / visibility filters, segment and option lists); `Settings.hpp` `update_enabled_entities` flag set by the layer range and visibility setters (`ViewerImpl.cpp:1337, 1347, 1433, 1446`) and consumed as `render` does (`ViewerImpl.cpp:1294-1295`) | new `crates/ares-vgcode/src/viewer/enabled.rs` (`impl ViewerImpl`: `enabled_segments`, `enabled_options`) |
| `ViewerImpl.cpp:911-943` `extract_pos_and_or_hwa` `valid_lines_bitset` part (a line `i -> i+1` is valid when the next vertex exists, moved, has the same type and is not a seam), built at `load` (`ViewerImpl.cpp:1059-1071`) | `viewer/enabled.rs` (`Vec<bool>` filled in `load`) |
| `ViewerImpl.cpp:1227-1245` `update_colors_texture` (dark grey below the top layer while the visible range ends before the full range, top-layer-only view) | `viewer/colors.rs` `ViewerImpl::texture_colors` |
| `ViewerImpl.cpp:1342-1347` `toggle_top_layer_only_view_range` with `GCodeViewer::load_as_gcode` (`GCodeViewer.cpp:1128-1131`) applying the `seq_top_layer_only` app config default `1` (`AppConfig.cpp:202-203`) | already in `viewer.rs`; called once by the wasm `VertexBuffers` loader |
| `GCodeViewer::update_moves_slider` (`GCodeViewer.cpp:1765-1821`): slider values over the enabled range, consecutive vertices with the same `gcode_id` (G2/G3 splits) merged into the last one, alternate values = G-code line ids, selection span from the visible range | new `crates/ares-vgcode/src/moves_slider.rs` (`MovesSlider`) |
| `GLCanvas3D::_render_gcode` (`GLCanvas3D.cpp:8263-8266`): slider thumbs to the visible range, `values[thumb] - 1` | `MovesSlider::vertex_range` |

## wasm: thin bindings only (`crates/ares-wasm/src/moves.rs`)

- The loader turns on the top-layer-only view range, as above.
- `VertexBuffers.movesSlider(minLayer, maxLayer, visibleRoles, visibleOptions) -> Uint32Array`: visibility and layer range as `viewRange` does (`GCodeViewer::set_layers_z_range` then `update_moves_slider(true)`, `GCodeViewer.cpp:1864-1868`), keeps the `MovesSlider`, and returns `[spanMin, spanMax, ...gcodeIds]`.
- `VertexBuffers.setMovesRange(low, high) -> Uint32Array`: `set_view_visible_range(vertex_range(low, high))`, returns `[first, last, current]` like `viewRange`.
- `VertexBuffers.enabledSegments()` / `enabledOptions()` -> `Uint32Array`.
- `vertexColors` returns `texture_colors` (`update_colors` ends with `update_colors_texture`, `ViewerImpl.cpp:1285`).

## Library and app

- `AresSlicer.movesSlider(layers, roles, options)` and `AresSlicer.movesRange(low, high, viewType, roles, options)`, the latter returning the view range, enabled segment / option ids and the vertex colors.
- A horizontal moves slider under the preview canvas (`<input type=range>`, the higher thumb). It resets to its maximum when the layer range or a legend toggle changes. The toolpath mesh and option markers draw only the enabled entities; the G-code window follows the current vertex. TS only maps slider position to the call and an id list to instance flags.

## Deferred

- The lower thumb, the slider's alternate value labels (`SetSliderAlternateValues`) and `IMSlider` keyboard details. Recorded in `web/README.md` "Known differences from OrcaSlicer".
- The tool marker and its estimated-time label (`ViewerImpl::get_estimated_time_at`).

## Tests (RED first)

- **Browser E2E** (extend `toolpathPreview.ts`, spec count stays 14). The oracle reads the procgcode `--dump-moves` CSV only: it rebuilds the vertices with the phantom path-start vertices of `LibVGCodeWrapper.cpp:191-268`, the default visibility (`Settings.hpp:33-42`), the full / top-layer range of `ViewerImpl.cpp:1728-1808` and the `gcode_id` merge of `GCodeViewer.cpp:1790-1807`.
  - Slider maximum = stops - 1 at the top layer and at layer 1.
  - At the first, a middle and the last stop the G-code window marks that stop's line; the canvas differs from the full view except at the last stop.
  - Screenshot of the middle stop.
- No Rust unit tests: the browser E2E drives every ported function through the bindings.
