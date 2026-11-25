# Web: toolpath texture packing in the slicer worker

## Problem

When a slice result reaches the page and Preview is shown, `Viewport.setToolpaths`
(`web/app/src/viewport/Viewport.ts:202-227`) runs on the main thread, in the task that delivers
the result. For a 512-facet, 60 mm cylinder it blocks the main thread for about 580-600 ms
(Chromium performance trace of the Pixel flow, two runs). Sampled profile of that task:

| Part | Share of the task |
|---|---|
| The three E2E probes: `segmentSizesProbe`, `segmentGeometryProbe`, `segmentColorsProbe` (`web/app/src/viewport/toolpathMesh.ts:167-216`, FNV-1a over every byte of every segment) | 65-69% |
| The texture packing of `buildToolpathMesh` (`toolpathMesh.ts:34-65`) | 21-24% |
| Everything else (option markers, React commit) | about 10% |

On `main` the same work runs when Preview opens after a slice, and Ctrl+R opens Preview while the slice runs, so its
result arrives on Preview. The page does not answer input during that time.

## Upstream reference

OrcaSlicer v2.5.0 (`40ecc66f7`), `src/libvgcode/src/ViewerImpl.cpp`:

- `width_height` (`:352-361`): the texture extent, rows of up to the maximum texture size.
- `ViewerImpl::load` (`:1121`, GPU buffers at `:1235-1249`): `extract_pos_and_or_hwa` fills one `Vec4` position and one
  `Vec4` height/width/angle per vertex, then `TextureData::set_positions` and
  `TextureData::set_heights_widths_angles` upload them.
- `TextureData::set_positions` (`:384-435`), `set_heights_widths_angles` (`:437-488`),
  `set_colors` (`:490-541`): the per-vertex RGBA32F / RGBA8 texture layout.

These are the source of the packing layout. Upstream packs on the thread that renders; it has no
separate worker. This slice changes only where the existing Ares packing runs, not its layout.

## Ares today

- `@ares-slicer/wasm` worker (`web/packages/ares-wasm-js/src/worker.ts:85`,
  `processedResult`, toolpaths at `:93-125`) reads the vertex arrays out of ares-wasm (`workerTransfer.ts:30-41`) and
  transfers them as `Toolpaths`.
- `buildToolpathMesh` (`toolpathMesh.ts:33-103`) packs them on the main thread:
  - `positions`: RGBA32F, `[segment position xyz, layer id]` per vertex.
  - `heightsWidthsAngles`: RGBA32F, `[height, width, angle, role or option kind]` per vertex.
  - The drawn segments: one per extrude / travel / wipe vertex whose position differs from its predecessor.
  - The bounding box of the extrusions other than Custom.

  It sizes the textures by `renderer.capabilities.maxTextureSize`.
- `Viewport.setToolpaths` then writes the three probes to `data-segment-sizes`,
  `data-segment-geometry` and `data-segment-colors` in the same task. `setToolpathColors`
  (`Viewport.ts:230-236`) recomputes `data-segment-colors` on the main thread for every recolor
  (`viewColors`, `viewRange` colors). E2E specs read these attributes.

## Change

Move, unchanged, into the package worker:

- The packing loop, the drawn-segment selection and the bounds (`toolpathMesh.ts:34-65`).
- `fnv`, `bytesOf`, `probe` and the three probes (`toolpathMesh.ts:167-216`).

The moved code goes to a new `web/packages/ares-wasm-js/src/toolpathTextures.ts`.

- `AresSlicer.prepareViewerTextures(maxTextureSize)` tells the worker the extent limit. Until a
  client calls it, the worker packs nothing and `Toolpaths.textures` is absent, so a consumer
  without a viewer pays nothing. The app calls it when its viewport renderer exists.
- `processedResult` adds `Toolpaths.textures`, transferred with the other arrays:
  - `extent`;
  - `positions`;
  - `heightsWidthsAngles`;
  - `segments`, the start vertex of each drawn segment;
  - `bounds` (min and max xyz);
  - `probes` (sizes, geometry, and colors for `Toolpaths.colors`).

  The worker keeps the drawn-segment list with the vertices it already keeps for `viewRange`.
- `ViewColors` (`viewColors`) and `MovesView` (`movesHigher`) carry `colorsProbe`, the colors probe of the colors they return.
- `buildToolpathMesh` builds the three.js textures, attributes and bounding volumes from
  `toolpaths.textures`. It copies `segments` for the `aSegment` attribute, which `applyEnabled` writes into, as each
  mesh had its own array before. `setToolpaths` and `setToolpathColors` write the probes they receive.

The attributes keep their values and are written in the same task as before. No E2E spec changes.

The package worker bundle stays self-contained: it imports no runtime module of the client entry, so the build emits no
shared chunk and the packed file list stays the same. `toolpathTextures.ts` writes the `MOVE_TYPES` / `EXTRUSION_ROLES`
indices, the `VERTEX_VALUES` length and `KIND_OPTION` as literals whose types are derived from those `as const`
constants, so a mismatch fails the type check. Its declaration file is left out of the package like the other
worker-only modules. `KIND_OPTION` moves from the app's `segmentShader.ts` to `toolpaths.ts`, the shader contract
the worker now packs for. `AresSlicerOptions` moves to `types.ts` to keep `index.ts` under 400 lines.
The color texture (`writeColors`), option markers and enabled-segment updates stay where they are.

## Verification

- New Playwright spec `web/e2e/previewResultResponsive.spec.ts` (desktop Chromium): import the generated
  cylinder, select the LH presets, press Ctrl+R, so Preview is open when the result arrives.
  A 16 ms main-thread heartbeat runs from the key press until `data-segment-sizes` is set. Assert
  that no gap reaches 200 ms, the threshold of `worker.mobile.spec.ts`. It fails before the change.
- The whole web suite, base `origin/main` vs tip, with no pass-to-fail. Every existing spec runs
  unmodified, among them those that compare the probe attributes with Native-derived values
  (`segmentSizes.ts`, `segmentGeometry.ts`, `segmentColors.ts`, `boundedViewerQueries.spec.ts`).
- A diagnostic performance trace of the same flow on base and tip: the longest main-thread task
  after the result arrives, and its sampled profile split. Not a pass/fail record.

## Deferred

- Porting the packing to Rust in `ares-vgcode` (the `extract_pos_and_or_hwa` + `TextureData`
  layout), run in the worker through ares-wasm. This slice keeps the JS code as it is, as a
  temporary shell around that upstream concept.
- The synchronous shader program compile of the first toolpath frame. It is outside the measured
  task, but it blocks for seconds under software GL.
- Moving the color texture fill (`writeColors`) and the option markers off the main thread.
