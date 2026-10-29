# Web preview: libvgcode segment positions and join angles

Status: SPEC `ffe2c63e6` APPROVED by COORD, split into two slices (see COORD ruling at
the end). Follow-up rewrite slice to
`web-segment-heights-widths.md` (chain `5e164a404..50ca38eb4`, stack-19). It ports the
rest of libvgcode `extract_pos_and_or_hwa` (the positions branch and the join angle)
and the segment geometry that consumes them in the ES segments shader, which is the
WebGL2 (GLSL ES 3.0) counterpart of the Native preview.

## Upstream boundary

| Upstream | What it does | Ares destination |
|---|---|---|
| `libvgcode/src/ViewerImpl.cpp:945-952` (`extract_pos_and_or_hwa`, positions branch) | per vertex: `v.position`, Extrude pushed down by `0.5f * v.height`; Travel/Wipe/options keep the nozzle z | `ViewerImpl::positions` in `crates/ares-vgcode/src/viewer/heights_widths.rs` (64 LOC; stays one file with the hwa branch) returning `[f32; 3]` per vertex |
| `ViewerImpl.cpp:928-934,979-980` (`prev_line` / `this_line`, angle) | `atan2(prev.x*this.y - prev.y*this.x, dot(prev, this))` (z cross over a 3D dot), `prev_line` zero unless `valid_lines_bitset[i - 1]`, `this_line` zero unless the line `i -> i+1` is valid | `heights_widths` extended to `[f32; 3]` `[height, width, angle]` (same pass, reusing the `valid_lines` bitset already ported in `viewer/enabled.rs:9-21`) |
| `ShadersES.hpp:13-148` (`Segments_Vertex_Shader_ES`, `POINTY_CAPS` and `FIX_TWISTING` defined at `:16-17`, `main` `:58-148`) | 8-vertex segment between `id_a`/`id_b`: view-dependent cross-section (signs table `:81-100`, `is_vertical_view` `:114-115`), half sizes `:122-128`, caps `:129-141` (`angle == 0`: pointy cap of `half_width`; else `sin`/`cos` of `abs(angle)/2`) | `web/app/src/viewport/toolpathMesh.ts` vertex shader (GLSL port, GPU code cannot live in Rust) |
| `SegmentTemplate.cpp:13-27` (`VERTEX_DATA`, 24 indices over vertex ids 0..7) | instanced template geometry | `toolpathMesh.ts` index buffer replacing the `THREE.BoxGeometry` |

## Existing Ares scaffolding (replaced)

`toolpathMesh.ts` is a temporary compatibility shell around the Native segment
geometry: an axis box spanning `[z - height, z]` from the raw vertex z, flat caps
grown by half the width at both ends. Known differences from upstream:

1. Travel/Wipe boxes sit below the nozzle (`[z - r, z]`) where Native centres them on it
   (`[z - r/2, z + r/2]`, no push-down for non-extrusions, `:948-950`). Extrusions
   occupy the same z span in both.
2. No join angle: every cap is the `angle == 0` cap. Census of the existing segment-hw
   oracle dumps (`/tmp/ares-estimation/segment-hw/angle-census.mjs`, Native
   `hwaBits[2]`): non-zero angles on roles 30, marker-time 6, preset-slice 2645,
   slice 2444 vertices (Travel, Wipe and Extrude); color-change 0.
3. Box cross-section instead of the camera-facing 8-vertex template.

The box geometry, its `[z - height, z]` convention and the inline cap rule are deleted.

## Included

1. Rust `positions` and the angle in the hwa pass, f32 bit-equal, same branch order.
2. wasm transfer: `heightsWidths` becomes `heightsWidthsAngles` (3 per vertex) and a new
   `segmentPositions` (3 per vertex); the raw `values` positions stay for the other
   consumers (bounds, tool marker).
3. web: template indices plus the ES vertex shader geometry (`:58-141`) over the
   transferred positions/sizes/angles; layer/visibility filtering and the colour
   attribute stay as they are.

## Deferred

- ORCA z-fighting bias (`ViewerImpl.cpp:970-975`, `Shaders.hpp:93-96,137-138`): the
  desktop GL shader applies it, the ES shader (`ShadersES.hpp:143`) does not; the web
  follows the ES shader. Question to COORD below.
- ES `lighting()` (`:44-57`) and `decode_color`: the web fragment lighting is unchanged
  (separate rendering slice).
- Options shader / option markers (`ShadersES.hpp:159-216`), cog/tool markers.
- GL texture upload (`ViewerImpl.cpp:1071-1099`) and `update_heights_widths`: GL only.

## Native-first discovery (before RED)

The segment-hw oracle (`tools/chain-probe/segment-hw-oracle.hpp`) already runs the
verbatim kernel with `positions != nullptr` but dumps only the input `v.position`.
One-field extension: also dump `positions[id]` bits (`positionOutBits`). That is a new
derivation (about 42 min, CPU 16,17,20,21, ≤4 cores) replacing `yi8nyfhl…`; I will
report the new store path so COORD can swap the gcroot, and `ARES_SEGMENT_HW_BIN` in
`tier1.yml` (worker4) moves with it. Discovery compares, per fixture (roles,
marker-time, color-change, presetSlice/slice plates), the Ares API output (current
wasm `values` z and zero angle) with Native `positionOutBits` / `hwaBits[2]`; if no
difference shows, NO-RED STOP.

## Tests (TDD, E2E-first)

- RED 1 `crates/ares-vgcode/tests/heights_widths.rs` (new test fn, existing fixtures):
  `positions()` bits and angle bits per vertex equal the oracle. The scaffold returns
  the raw position and a zero angle, so the failure is an assertion.
- RED 2 web: the existing `data-segment-sizes` read-out is widened to hash the instance
  geometry inputs (`aStart`/`aEnd` positions and `[h, w, angle]` per end) against the
  oracle; pure read-out, no test switch.
- GREEN gates as in the previous slice (directed nextest with `ARES_REQUIRE_ORCA=1`,
  strict clippy, fmt, wasm32, web build/lint/typecheck, gzip < 125000, browser desktop
  bundle/presetSlice/slice/import/firstFrame/visual on 16,17 with BROWSER-START/END).
  Every preview PNG moves (new cross-section); I report before/after crops and change
  no threshold or baseline without COORD.

## Failure modes enumerated before coding

1. Push-down applied to travels/wipes/options, or by the full height.
2. Angle sign flipped (cross product order) or computed in 3D instead of the z cross.
3. `prev_line` taken from an invalid line (bitset of `i - 1` ignored), so caps bend at
   type changes and seams.
4. Template winding or vertex ids mismatched, giving inside-out or missing faces.
5. `is_vertical_view` evaluated per vertex instead of per segment end (FIX_TWISTING uses
   the closer end).
6. Positions transferred through f64 and drifting from the f32 bits.

## COORD ruling (on `ffe2c63e6`)

- Slice A: Rust `positions` plus the join angle in ares-vgcode / ares-wasm
  (`ViewerImpl.cpp:928-934,945-952,979-980`) with the oracle extension, RED 1 and GREEN.
  No web geometry change and no preview baseline change in A.
- Slice B (after A merges): ES segment geometry (`ShadersES.hpp:13-148`,
  `SegmentTemplate.cpp:13-27`) in `toolpathMesh.ts`, RED 2 and every preview baseline
  move.
- Bias: the ES shader is the WebGL target upstream ships, so the web follows it. The
  desktop ORCA bias (`ViewerImpl.cpp:970-975`, `Shaders.hpp:93-96,137-138`) is deferred.

## Slice B addendum: ES segment geometry (SPEC, awaiting COORD review)

Slice A merged in stack-19 (`5c7383991`). Slice B replaces the box shell in
`web/app/src/viewport/toolpathMesh.ts` with the upstream WebGL2 segment draw.

| Upstream | What it does | Ares destination |
|---|---|---|
| `SegmentTemplate.cpp:13-27,46-52,81` | 24 template vertices, each carrying its vertex id 0..7, drawn non-indexed and instanced (`glDrawArraysInstanced`) | `toolpathMesh.ts`: `vertexId` attribute (Uint8, 24 entries) instead of `THREE.BoxGeometry` |
| `ShadersES.hpp:58-142` | segment `id_a -> id_a + 1`: line frame (`:64-80`), view signs table (`:81-100`), endpoint choice (`:101-103`), FIX_TWISTING closer end (`:104-113`), `is_vertical_view` (`:114-115`), half sizes and cross-section (`:116-128`), caps from the angle (`:129-142`) | `toolpathMesh.ts` vertex shader, transcribed line by line |
| `ShadersES.hpp:44-49,143-147,150-157` | per-vertex `lighting()` with `eye_normal = normalize(pos - endpoint_pos)`; pass-through fragment | same vertex shader; fragment becomes the pass-through (see question 1) |
| `ViewerImpl.cpp:1300-1302,1940` | camera position from the inverse view matrix, uploaded as a uniform | three.js `cameraPosition` built-in (the toolpath mesh has an identity model matrix) |
| `ViewerImpl.cpp:1929,1942` | face culling disabled while drawing segments | `THREE.DoubleSide` material |

Data: the worker transfers slice A's `segmentPositions` and `segmentAngles` next to
`heightsWidths`. `aStart`/`aEnd` come from `segmentPositions` (pushed down) instead of
the raw `values`, and a new `aAngles` vec2 holds the angles of `id_a`/`id_b`. `aSize`
keeps its layout, so the approved `data-segment-sizes` probe and its test do not change.
The paths bounding box keeps the raw positions, as `GCodeViewer::load` does. Instance
selection (types 8/9/10, zero-length skip), colours, layer/visibility filtering and
`applyEnabled` are unchanged.

Removed: `BoxGeometry`, the `[z - height, z]` box frame, the half-width flat caps, the
`normal`-based fragment light.

Deferred: ORCA bias (as ruled), option markers (`ShadersES.hpp:159-216`), cog/tool
markers, and the GL texture packing of positions/sizes (the web uses instanced
attributes; the values per instance are the same).

Deferred by COORD ruling on RED `0c3a5f021`: per-end segment color. Upstream colors
each end by its own vertex (`ShadersES.hpp:101,145`, `color_tex` at `id`); the web
keeps one color per instance, that of the end vertex.

RED 2 (test-only, after a scaffold): a new production read-out
`data-segment-geometry` = `instanceCount:FNV-1a32` over the f32 bytes of
`aStart`, `aEnd` and `aAngles` per instance (pure read-out, like `data-segment-sizes`).
The e2e computes the expected value from the Native oracle `positionOutBits` and
`hwaBits[2]` over the same instance selection, in `presetSlice` and `slice`. The
scaffold adds `aAngles` filled with zeros and the probe, so RED fails on the hash
(raw z and zero angles). The GLSL itself cannot be bit-checked against Native (no
headless Native GL); it is a transcription of the cited lines, and I'll review it
through before/after preview crops for every preview PNG. Thresholds stay unchanged.

Failure modes: (1) vertex ids or template order wrong, giving missing or inside-out
faces; (2) culling left on, so back faces vanish; (3) `cameraPosition` taken in view
space instead of world space; (4) `id`/`endpoint_pos` chosen per triangle instead of
`vertex_id < 4`; (5) the closer end swapped in FIX_TWISTING; (6) angles of the wrong
end fed to the caps; (7) raw positions left in `aStart`/`aEnd`.

Questions to COORD:
1. Lighting: the box normals disappear with the template, so the shader needs the
   upstream normal anyway. I propose porting the ES `lighting()` and the pass-through
   fragment (`:44-49,143-157`) in this slice rather than inventing a web normal.
2. Probe: is the new `data-segment-geometry` read-out acceptable (it leaves the approved
   `data-segment-sizes` test untouched), rather than widening `data-segment-sizes` as
   the original SPEC text said?

## Questions to COORD (answered above)

1. Bias: follow the ES shader (no bias) as above, or also port the desktop `hwa.w` bias?
2. Scope: is the ES geometry port (template plus shader) acceptable in this slice, or
   should the data part (Rust/wasm positions plus angle) land alone first, with the web
   geometry as a second slice?
