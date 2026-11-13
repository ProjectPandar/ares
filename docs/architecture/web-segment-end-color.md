# Web: per-end segment color (libvgcode ES segment shader)

This is the slice that was deferred in
`web-segment-positions-angles.md`.

## Upstream boundary (OrcaSlicer `8500fcdcc`)

| Upstream | Behavior | Ares destination |
|---|---|---|
| `src/libvgcode/src/ShadersES.hpp:60-61` | segment `gl_InstanceID` gives `id_a`; `id_b = id_a + 1` | already ported: instance `n` = vertex `i` with start `i - 1` (`toolpathMesh.ts` `buildToolpathMesh`). The web keys an instance by its **end** vertex `i` = upstream `id_b`; upstream `enabled_segments` keys a segment by its **start** `id_a` (`ViewerImpl.cpp:929-942` validates the line `i -> i + 1`, `:1183` pushes `i`), so `applyEnabled` maps segment `id` to instance end `id + 1` |
| `ShadersES.hpp:101` | `int id = vertex_id < 4 ? id_a : id_b;` — the template vertices 0..3 belong to the start end, 4..7 to the end | `toolpathMesh.ts` vertex shader: pick the start or end color by the same `vertex_id < 4` test |
| `ShadersES.hpp:36-43,145-146` | `color_base = decode_color(color_tex[id])`, then `color = color_base * lighting(...)`; `color` is not `flat`, so the rasterizer interpolates it between the two ends | same shader: `color = (vertex_id < 4 ? aStartColor : aColor) * lighting(...)` |
| `ViewerImpl.cpp:1218-1224,1227-1258,1261-1287` | `update_colors` stores `encode_color(get_vertex_color(v))` for **every vertex**, `update_colors_texture` greys some of them, and the result is uploaded as one texel per vertex | already ported: `ares-vgcode` `viewer/colors.rs` (`update_colors`, `update_colors_texture`, `get_vertex_color`) and wasm `VertexBuffers::vertex_colors` / `Toolpaths.colors` (RGB per vertex) |
| `ViewerImpl.cpp:1467-1553` | `get_vertex_color` per view type | already ported (`viewer/colors.rs`) |
| `src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:216-241` | `convert` adds a phantom vertex (the previous position, otherwise `curr` attributes) only when the type, role or `mm3_per_mm` changes | already ported (`ares-vgcode` `convert`); this is why Feature Type ends never differ (see Discovery) |

`encode_color`/`decode_color` round-trip a 24-bit RGB exactly (an integer below 2^24
in an f32), so the Uint8 RGB the web already carries per vertex is the same value
the ES shader decodes.

## Change (web only)

`buildToolpathMesh` adds an instanced Uint8 RGB attribute `aStartColor` filled from
`colors[i - 1]`; `aColor` stays the end color `colors[i]`. The vertex shader takes
`vertex_id < 4 ? aStartColor : aColor` as `color_base`. `applyVertexColors` writes
both ends of every toolpath instance, so view switches recolor both. Option markers
keep their single `aColor` (one vertex per marker, `ShadersES.hpp:203,213`).

No Rust change: the per-vertex colors already cross the wasm boundary. No `ares-core`
change. No ARD: no new crate, module or data boundary.

Included: start/end color selection for segments in every view type that the web
offers, plus the recolor path.

Deferred:
- ORCA bias;
- option-marker, cog and tool-marker shaders;
- GL texture packing (the web keeps instanced attributes; the values per instance are the same);
- Native bit-checks of view types other than Feature Type and Speed (see below).

## Discovery (Native-first, read-only)

This was measured over existing Native dumps
(`segment-hw/native.json` and `moves.csv` of `presetSlice` and `slice`), with the
same instance selection as the web: types 8/9/10, with zero-length instances skipped.

| Input | Instances | Feature Type key differs between ends | height | width | fan | temperature | feedrate |
|---|---|---|---|---|---|---|---|
| presetSlice | 5519 | 0 | 264 | 260 | 276 | 285 | 0 |
| slice | 5309 | 0 | 452 | 576 | 313 | 240 | 3 |

- **Feature Type:** the ends never differ, because `convert` gives every role or type change a phantom vertex of the new kind. Per-end color is therefore invisible there; it becomes the control.
- **Layer Height:** the ends differ on hundreds of instances in both inputs, so the RED uses this view.
- **Speed:** almost no instances differ (0 and 3), so it is not used.

### Revision: RGB-end measurement

The raw values above count instances whose ends are travel, wipe or option vertices.
Those are drawn in fixed colors, so raw differences do not imply RGB differences. The
built oracle (`orca-slicer-segment-colors-2.4.2`, `0j5bpnsd…`) measured the
`get_vertex_color` RGB ends over the same instances:

| Input | Instances | Feature Type RGB ends differ | Height RGB ends differ |
|---|---|---|---|
| slice | 5309 | 0 | 0 |
| presetSlice | 5519 | 0 | 0 |

Height changes only across layer or option boundaries, never inside an extrusion run,
so Layer Height cannot carry the RED. The oracle header therefore also dumps the
`Speed`, `Width`, `VolumetricFlowRate`, `FanSpeed` and `Temperature` range views
(header loop only; the kernel is unchanged). The RED view is a view with a non-zero
Native RGB-end count. If every count were 0, the slice would close as NO-RED. No
dedicated fixture is added.

Rebuilt oracle `pilsk1xa…` (`orca-segment-colors-2.4.2`), default visibility:

| Input | Instances | featureType | height | speed | width | volumetricFlowRate | fanSpeed | temperature |
|---|---|---|---|---|---|---|---|---|
| slice | 5309 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| presetSlice | 5519 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

All three Speed instances are travel→travel (vertices 3→4, 7339→7340, 7342→7343):
`get_vertex_color` colors travel by feedrate in Speed (`ViewerImpl.cpp:1472,1489-1491`);
the other range views give travel the fixed option color. Decision: **Speed RED,
slice only, with Travel shown**. Travel visibility changes the speed range
(`update_color_ranges` takes travel feedrates only when Travels is visible,
`:1860-1863`), so the oracle also dumps `speedTravels` = Speed with
`options_visibility[Travels] = true` (header only). `presetSlice` stays a control.

## Native oracle

A new read-only chain-probe `tools/chain-probe/segment-colors.nix`, built on
`procgcode.nix` like `segment-hw.nix`. It calls the real `libvgcode::convert` and links
the real `ColorRange` (`ColorRange.cpp:57-80`) and `Layers`. The source kernel is
extracted verbatim and checked by `extract-segment-colors.mjs --check`, like segment-hw.
It contains:
- `round_to_bin` (`ViewerImpl.cpp:47`);
- `update_color_ranges` (`:1817-1878`);
- `get_vertex_color` (`:1467-1553`);
- the default role and option colors (`:283-317`, `reset_default_*` `:1578-1597`);
- the default `Settings` (`Settings.hpp:19,33-71`).

`ViewerImpl::load` needs a GL context, so the oracle cannot call it. The kernel runs
over a struct holding the same members instead (the `role-times` precedent).

For each vertex, `--dump-segment-colors <json>` writes the RGB of `get_vertex_color`
under `featureType`, `height`, `speed`, `width`, `volumetricFlowRate`, `fanSpeed` and
`temperature`, with the default visibility (all roles shown,
Seams only among the options; the web default), and under `speedTravels` with Travels
also visible. Native outputs stay outside Git.

### OrcaSlicer 2.5.0 (`40ecc66f7`) source

The Native side of `slice.spec` (`oracle-moves.csv`) comes from 2.5.0. The segment
colors must come from the same source, so the kernel is re-extracted at `40ecc66f7`.
`procgcode.patch` and `segment-colors.patch` apply unchanged, and the oracle header calls
`convert` the same way (the new `belt_xform` argument defaults to `nullptr`).

The excerpts only moved:

| Excerpt | 8500fcdcc | 40ecc66f7 | Text |
|---|---|---|---|
| `IntegerOnly`, `fast_round_up`, `round_to_bin` | `:23-24,37-43,47-63` | `:35-36,49-55,59-75` | same |
| default palettes | `:283-305,307-317` | `:315-337,339-349` | same |
| members (`ViewerImpl.hpp`) | `:202,206,242,246,273,285-304` | `:245,249,300,304,331,346-365` | `Palette` → `PaletteWithAlpha` for `m_tool_colors` and `m_color_print_colors` |
| palette accessors | `:1567-1597` | `:1856-1886` | same |
| `update_color_ranges` | `:1817-1878` | `:2122-2183` | same |
| `get_vertex_color` | `:1467-1553` | `:1734-1823` | Tool and ColorPrint read RGB from a `ColorWithAlpha` |

The kernel adaptation does not change. No view the oracle dumps reads the tool or
color-print palettes, so the dump keeps the same views and fields.

2.5.0 also changes code the Ares viewer port (`ares-vgcode`) covers. None of these
changes touches a view this oracle dumps, and they are not ported in this slice:
- `LibVGCodeWrapper.cpp` `convert_filament_colors` (`:84-99`): filament colors go
  through `adjust_color_for_rendering` (`3DScene.cpp:114-125`). This lifts near-black
  colors to a grey and keeps an alpha. `convert.rs` `decode` still keeps the RGB as
  written. This affects the Tool and ColorPrint views only.
- `convert` (`:272-282`) renumbers `layer_id` consecutively over the moves. A source
  layer without moves no longer leaves a gap. `convert.rs` still copies
  `curr.layer_id`.
- `convert` takes `belt_xform` for belt printers (`:244-253`); the port has no belt path.
- `Layers::get_layer_id_at` (`Layers.cpp:78`) compares with `item.z > z`.
  `layers.rs` `get_layer_id_at` still uses `item.z < z`. Upstream calls it only from the
  conflict layer lookup (`GCodeViewer.cpp:1847`).
- `Settings` adds `dim_previous_layers`, `dim_previous_layers_brightness` and
  `translucent_filaments` (`Settings.hpp:24-30`). These are render-only.

## RED (after a scaffold) and control

The scaffold commit adds `aStartColor` filled with the **end** vertex color. That is the
current behavior: both ends are drawn in `colors[i]`. It also adds the production
read-out `data-segment-colors` = `instanceCount:FNV-1a32` over the bytes of
`aStartColor`, then `aColor`, of the toolpath mesh. The read-out is set when the mesh is
built and again after every `setToolpathColors`.

The e2e test, `web/e2e/segmentColors.ts`, is called from `toolpathPreview.ts` in
`presetSlice` and `slice`. It computes the expected value from the Native dump over the
same instance selection: start `rgb[i - 1]`, end `rgb[i]`. It runs three steps:

1. **Control, Feature Type** (the initial view): the read-out equals the Native value. It already passes at the scaffold, because the Native ends are equal there.
2. **Speed.**
   - `slice` (**RED**): the test selects "Speed", turns the Travel legend row on (asserting `aria-pressed=true`), and waits on the read-out. It must equal the Native `speedTravels` value. It fails on the hash until GREEN, because the scaffold start colors are the end colors. Then it turns Travel off again.
   - `presetSlice` (**control**): the test selects "Speed" at the default visibility (Travel off, asserted). The read-out must equal the Native `speed` value. It passes at the scaffold and at GREEN.
3. **Control, back to Line Type:** the read-out equals the step-1 Native value again. This exercises the recolor path for both ends.

Native-only coverage preconditions read only the Native rows. They are kept separate from
the Ares-vs-Native assertions and compare `get_vertex_color` **RGB** ends, not raw values
(two values can fall into the same `round_to_bin` / `ColorRange` color):
- `slice` requires at least one travel→travel instance whose `speedTravels` RGB ends differ, so the RED cannot pass vacuously.
- `presetSlice` requires zero differing `speed` ends, so the control really has equal ends.
- Both require zero differing `featureType` ends.

The counts are written to `segment-colors/expected.json`.

The existing `data-segment-sizes` and `data-segment-geometry` read-outs and tests do
not change.

The GLSL change cannot be bit-checked against Native, because there is no headless
Native GL. It is checked through before/after crops of every preview PNG at the
true base (an exact before-run). Pixel thresholds stay unchanged. Feature Type previews
should be pixel-identical; range views should change only where Native ends differ (travel in Speed with Travel shown).

## Failure inventory

1. Start color taken from `colors[i]`, or from `colors[i + 1]`, instead of `colors[i - 1]`.
2. Ends swapped in the shader (`vertex_id >= 4` picks the start color).
3. Color selected per triangle or per instance instead of per template vertex.
4. `applyVertexColors` updates only `aColor`, so view switches keep stale start colors (step 3 above).
5. The read-out is not refreshed after a recolor, so step 2 compares stale bytes.
6. Option markers break because they share `applyVertexColors` but have no `aStartColor`.
7. The `normalized` flag is missing on `aStartColor`, so the color is 255× too bright.
8. The instance selection in the test and the mesh differ (count mismatch in the read-out).
9. A Native-vs-Ares per-vertex Speed color mismatch unrelated to the ends (range or `get_color_at` drift in `viewer/colors.rs`). The RED would then fail for a different reason. If that happens, it is reported as a separate finding and not fixed in this slice.
10. The Native visibility differs from the web's: the RED compares against `speedTravels` only after the Travel row is asserted pressed, and the controls only at the default (asserted unpressed). The web default `visibleOptions` stays the Seams-only default.
11. Gzip: the change adds only a few shader lines and stays under 125,000 B. It is checked at GREEN.

## Exit criteria

- `slice` RED step 2 (Speed + Travel) fails on the hash at the scaffold and passes at GREEN. Steps 1 and 3, and all of `presetSlice`, pass at both.
- Browser E2E passes in full on desktop. clippy, fmt, the web build and the wasm32 build all exit 0.
- Initial JS gzip is under 125,000 B.
- The before/after crops show changes only in range views; thresholds are unchanged.
