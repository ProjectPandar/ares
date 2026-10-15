# Web preview: libvgcode segment positions and join angles

Status: SPEC (awaiting COORD review). Follow-up rewrite slice to
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

## Questions to COORD

1. Bias: follow the ES shader (no bias) as above, or also port the desktop `hwa.w` bias?
2. Scope: is the ES geometry port (template plus shader) acceptable in this slice, or
   should the data part (Rust/wasm positions plus angle) land alone first, with the web
   geometry as a second slice?
