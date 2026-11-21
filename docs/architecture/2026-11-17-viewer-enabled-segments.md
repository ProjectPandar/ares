# The web viewer draws only the enabled segments

Upstream baseline: OrcaSlicer v2.5.0 (40ecc66f7), `src/libvgcode/src`. The web viewer runs on WebGL2,
so its counterpart is the `ENABLE_OPENGL_ES` path of libvgcode.

## Upstream source

Per-vertex data, uploaded when the G-code is loaded or recoloured:

- `width_height` and `TextureData::init` (`ViewerImpl.cpp:352-382`): one texture row of up to the maximum
  texture size per `width` vertices.
- `TextureData::set_positions` (`ViewerImpl.cpp:384-435`) and `set_heights_widths_angles`
  (`ViewerImpl.cpp:437-488`): float RGBA textures, one texel per vertex.
- `TextureData::set_colors` (`ViewerImpl.cpp:490-541`): one colour texel per vertex.

Enabled segments, on every range or visibility change:

- `update_enabled_entities` (`ViewerImpl.cpp:1300-1396`) collects the ids `i` of the visible segments
  `i -> i + 1`. The ES branch passes them to `TextureData::set_enabled_segments`
  (`ViewerImpl.cpp:1357-1361`).
- `TextureData::set_enabled_segments` (`ViewerImpl.cpp:543-616`) uploads only those ids, 4 bytes each
  (`GL_R32UI`).

Drawing:

- `render_segments` (`ViewerImpl.cpp:2218-2398`) returns early when `get_enabled_segments_count()` is 0
  (`:2233-2238`).
- Otherwise it draws `count` instances per enabled-ids texture (`draw`, `:2311-2320`, called at
  `:2331-2350`), so the instance count is the number of enabled segments.
- The ES segment vertex shader (`ShadersES.hpp:84-97`) turns an instance into its segment:
  `id_a = segment_index_tex[instance]` and `id_b = id_a + 1`. It fetches the positions, heights, widths,
  angles and colours of both ends from the per-vertex textures by those ids (`tex_coord`).

So the vertex shader runs only for the enabled segments. A range or visibility change uploads only the
enabled ids, never the segment geometry.

## Ares today

`web/app/src/viewport/toolpathMesh.ts` copies every drawable segment into per-instance attributes
(`aStart`, `aEnd`, `aSize`, `aAngles`, `aStartColor`, `aColor`, `aLayer`, `aKind`) and draws all of them on
every frame. `applyEnabled` writes a per-instance `aEnabled` flag, and the vertex shader collapses a
disabled instance to a point outside the clip volume.

The result:
- every frame runs the vertex shader for every segment the mesh holds, enabled or not;
- a range change re-uploads one flag per segment.

## Ares destination

The change stays in `web/app/src/viewport/toolpathMesh.ts`. The segment shader moves next to it, into
`segmentShader.ts`.

**Per-vertex textures**, built once per toolpaths with the row layout of `width_height` and the renderer's
maximum texture size:
- positions: `xyz`, with the vertex's layer in `w`;
- heights, widths and angles: `xyz`, with the shader's role/option kind in `w`;
- colours: RGBA8.

The layer and the kind stay in `w` for the shader's own layer/role/option filter, which this slice keeps.

**Enabled ids:** one `uint` per enabled segment, the `id_a` of `set_enabled_segments`, in a per-instance
integer attribute. The geometry's `instanceCount` is their number.
- `applyEnabled` writes the engine's enabled segments (`MovesView.segments`, the ids of
  `update_enabled_entities`) into that attribute and uploads only that prefix.
- Until the engine's moves view arrives, the ids are those of every segment the mesh holds, which is what
  the viewer draws today.
- Upstream re-creates an `R32UI` texture on every update. A per-instance attribute carries the same ids in
  the same order and uploads the same 4 bytes per enabled segment, and it can be updated in place.
- The engine enables only valid lines (the next vertex moved, same move type, not a seam) of a visible
  extrusion, travel or wipe. Every enabled id is therefore a segment the mesh holds.

**No enabled segment:** the segment material is not drawn (the early return of `render_segments`). The
option markers, a child mesh, are still drawn.

**Shader:**
- reads both ends from the textures by `id_a` and `id_b = id_a + 1`, with upstream's `tex_coord`;
- reads the layer and kind of the segment from `id_b`, as the per-instance copies held them;
- keeps today's geometry, lighting and filter.

**Colours:** `applyVertexColors` writes the vertex colours into the colour texture instead of two
per-instance copies. The option markers keep their own instance colours.

**Probes:** the segment probes (`segmentSizesProbe`, `segmentGeometryProbe`, `segmentColorsProbe`) hash the
same per-segment bytes in the same order as today. The bytes now come from the toolpath arrays by the
mesh's segment list, and the probes still report the number of segments the mesh holds. Their values do
not change.

What the viewer draws does not change: the same triangles of the same segments, in the same order, with
the same inputs to the same shading.

## Included

- One instance per enabled segment. The vertex shader reads the segment from per-vertex textures by its id.
- A range or visibility change uploads the enabled ids (4 bytes each) and the colours, and never the
  segment geometry.
- No segment draw while no segment is enabled.

## Deferred

- The indexed 8-vertex template (`SegmentTemplate.cpp:24-37,92-104`). The web template keeps its 24
  non-indexed corners.
- Compiling the tool marker's program when the preview loads (`ViewerImpl.cpp:790,911-913`).
- The v2.5.0 shading and passes of the ES segment shader that the web shader does not have yet:
  - the uniform light direction, gloss and the stronger specular;
  - shadows and the section view;
  - the joint caps;
  - the separate opaque and translucent passes, with `first_instance`, `reverse_order` and the
    translucent partition of `set_enabled_segments`.
- The layer greying and dimming in the shader instead of in the colours (`update_colors`,
  `ViewerImpl.cpp:1458-1494`, the note at `:1474-1477`).
  Today the engine returns layer-greyed colours on every moves result, so the colours are still uploaded
  on every range change.
- Several textures per data set when a toolpath has more vertices than one texture holds (`m_count > 1`,
  `ViewerImpl.cpp:363-382`).
- The option markers (`render_options`, `ViewerImpl.cpp:2399-2486`), which keep their own instanced mesh
  and enabled flags.
- Removing the shader's own layer/role/option filter, now that the engine's enabled ids apply the same
  visibility.

## Verification

`web/e2e/toolpathDrawWorkload.spec.ts` slices a tall project, opens its preview and counts the GL work of
the segment program on every layer range change: the first layer, the first tenth of the layers, one layer
at that tenth, one middle layer, and all layers again.
- The settled draw has one instance per enabled segment the engine returned.
- The change uploads no float texture, so no segment geometry is re-sent.
- The buffer upload is the 4-byte enabled ids plus the option markers' enabled flags and colours.

The segment probes keep their expected values in the existing preview specs (`toolpathPreview.ts`,
`segmentGeometry.ts`), and the preview screenshots stay the same.
