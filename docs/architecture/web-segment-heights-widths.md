# Web preview: libvgcode segment heights and widths (travel / wipe radius)

Rewrite slice of the height/width part of
libvgcode `extract_pos_and_or_hwa` and of the `ViewerImpl` travels/wipes radius
state, which together size every toolpath segment in the Native preview.

## Upstream boundary

| Upstream | What it does | Ares destination |
|---|---|---|
| `libvgcode/src/ViewerImpl.cpp:911-983` (`extract_pos_and_or_hwa`, height/width part of the `heights_widths_angles` branch, `:954-968`) | per vertex: Travel → `{travels_radius, travels_radius}`, Wipe → `{wipes_radius, wipes_radius}`, otherwise `{v.height, v.width}`; early return when either radius `<= 0` | `Viewer::heights_widths` in `crates/ares-vgcode/src/viewer/heights_widths.rs` (new; `viewer.rs` is 372 LOC) returning `[f32; 2]` per vertex |
| `ViewerImpl.hpp:234,238`, `Types.hpp:21-30` | `m_travels_radius` / `m_wipes_radius` default `0.1f`, limits `[0.05, 1.0]` (constants already in `crates/ares-vgcode/src/types.rs:10-15`) | `Viewer` fields `travels_radius` / `wipes_radius` |
| `ViewerImpl.cpp:1648-1658`, `Viewer.hpp:209-223` (accessors `Viewer.cpp`) | `get/set_travels_radius`, `get/set_wipes_radius`: `std::clamp` to the limits, then `update_heights_widths` | getters/setters with the same clamp; the recompute is the next `heights_widths` call |
| `Shaders.hpp:91-118` (segments vertex shader) | each segment end takes the height/width of its own vertex (`id_a`, `id_b`); `half_height = 0.5 * hwa.x`, `half_width = 0.5 * hwa.y` (full sizes, not radii) | `ares-wasm` toolpaths transfer `heightsWidths` (`Float32Array`, 2 per vertex); `web/app/src/viewport/toolpathMesh.ts` sizes each instance end from its start/end vertex |

Native release builds keep the default radii: the only `set_travels_radius` /
`set_wipes_radius` callers (`GCodeViewer.cpp:2455-2467`) are inside
`#if ENABLE_NEW_GCODE_VIEWER_DEBUG` (`:2312-2471`).

## Existing Ares scaffolding (replaced)

`toolpathMesh.ts:17-18,105-106` is a temporary compatibility shell: travels and
wipes get `OPTION_SIZE = 0.08` and extrusions the end vertex size for the whole box.
Known differences from upstream by reading:

1. travel/wipe cross-section `0.08` vs Native `0.1` (`DEFAULT_*_RADIUS_MM`);
2. one size per segment (end vertex) vs per-end sizes (`id_a`/`id_b`).

`OPTION_SIZE` and the inline size selection are deleted.

## Included

1. `heights_widths` port (single pass, same branch order and f32 values), radius
   fields with clamp.
2. wasm transfer of the per-vertex heights/widths.
3. web instance sizes from that transfer at both segment ends.

## Deferred

- positions branch (`:945-952`, extrusion z pushed down by half height): the web box
  already spans `[z - height, z]` from the nozzle, the same volume; a positions port
  would be a shell swap with no observable difference.
- the join angle (`atan2`, `:979-980`) and the ORCA z-fighting bias (`:970-975`):
  they only drive upstream shader corner/offset math the web shader does not have;
  a separate rendering slice.
- radius UI (debug-only upstream).
- `update_heights_widths` GL buffer mapping (`:1880-1911`): GL only.

## Native-first discovery (before RED)

New oracle `tools/chain-probe/segment-hw-oracle.hpp` + `segment-hw.nix`, same shape
as the role-times / paths-bbox oracles: real `LibVGCodeWrapper` conversion, the
verbatim `extract_pos_and_or_hwa` body as an extracted kernel (only adaptation:
`static` → `inline`), called with the `ViewerImpl.hpp:234,238` defaults; dumps the
f32 bits of the height/width of every vertex. Fixtures: role-times `roles.gcode`
(has travels), marker-time `path.gcode`, color-change `normal.gcode`, and the
presetSlice/slice plates. Discovery replays the current JS size rule over the same
vertices and counts segments whose JS size differs from the Native size at either
end. Gap 1 holds whenever a fixture has a valid Travel or Wipe segment; if
discovery shows no difference at all, I send NO-RED STOP (as for paths-bbox).

## Tests (TDD, E2E-first)

- RED 1 `crates/ares-vgcode/tests/heights_widths.rs`: per-vertex heights/widths
  bit-equal to the oracle per fixture; radius clamp at 0.01/2.0 against the
  `std::clamp` limits. Scaffold returns zeros, so the failure is an assertion.
  `ARES_REQUIRE_ORCA=1` with `ARES_SEGMENT_HW_BIN` missing FAILs.
- RED 2 web (`toolpathPreview.ts`): the instance sizes of the presetSlice/slice
  meshes (exposed to the E2E as f32 bits, the same way as `data-*` probes in earlier
  slices) equal the
  oracle sizes of their start/end vertices.
- GREEN gates: directed nextest, clippy strict, fmt, wasm32, web build/lint/typecheck,
  gzip < 125000, browser desktop bundle/presetSlice/slice; pixel baselines that move because travels widen are reported
  with before/after crops, not re-thresholded.

## Failure modes enumerated before coding

1. Radius treated as a radius (doubled) instead of the full shader size.
2. Wipe takes the travel radius, or option vertices take a radius instead of their own height/width.
3. Clamp applied with the wrong limits or not at all.
4. Only the end-vertex size used, losing the per-end taper.
5. Sizes transferred through f64/JSON and drifting.
6. Early return on a radius `<= 0` not reproduced (cannot happen through the clamp; kept for parity, not tested).
