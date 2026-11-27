# G-code thumbnail renderer

## Purpose

Port the upstream thumbnail pixel producer to the platform-neutral Rust core
without importing the desktop rendering stack. The existing image encoders and
G-code block layout remain the output boundary; this slice supplies their RGBA
input during the slice operation.

## Upstream source and Rust destination

The source authority is OrcaSlicer v2.5.0,
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/slic3r/GUI/Plater.cpp:7898,7913` installs the thumbnail callback on the
  background print process. `:14317-14328` iterates `ThumbnailsParams.sizes`,
  rounds each `Vec2d` to `Point`, renders each image, and removes invalid data.
- `src/slic3r/GUI/GLCanvas3D.cpp:6945-7060` selects visible model-volume
  meshes using instance-transformed `ModelVolume` triangles, the plate build
  volume, printable status, parts-only status, and modifier/wipe tower
  exclusions. It computes the mesh bounds and adds the source margins.
- `src/slic3r/GUI/GLCanvas3D.cpp:7061-7145` selects the view, fits the
  orthographic camera, and projects against the plate or mesh bounds.
- `src/slic3r/GUI/GLCanvas3D.cpp:7146-7293` clears the target, assigns volume
  colors, applies the thumbnail shading path, rasterizes mesh triangles, and
  reads bottom-up RGBA pixels from the framebuffer.
- `resources/shaders/140/thumbnail.vs:1-47` defines the top/front light
  directions, ambient/diffuse/specular intensities, vertex-normal lighting,
  interpolated intensity, and camera-space view vector. Its companion
  `resources/shaders/140/thumbnail.fs:1-24` applies lighting, emission, alpha,
  and the below-bed discard.
- `src/libslic3r/GCode/ThumbnailData.hpp:36-47` defines the callback parameter
  fields and callback result list. `src/libslic3r/GCode/Thumbnails.hpp:48-75`
  invokes that callback for every configured format and size before encoding.
- `src/libslic3r/GCode.hpp:261,450` and `src/libslic3r/Print.hpp:1086` carry the
  callback through export. `src/OrcaSlicer.cpp:7524-7573` shows that Native CLI
  rendering is conditional on a hidden GLFW/OpenGL context.

The Rust destination is a renderer module under
`crates/ares-core/src/gcode_thumbnails/`, called by the existing project slice
G-code emission boundary. The module consumes the current plate's printable
model-volume meshes (instance-transformed `ModelVolume` triangles) and
effective thumbnail definitions. `ares-wasm` exposes only the existing
byte-oriented slice result; it does not duplicate rendering or encoding.
`ares-cli` keeps the renderer option disabled by default because the Native
headless CLI path produces no thumbnail blocks in the parity environment.

## Render algorithm

1. Resolve the configured thumbnail definitions without re-parsing the
   serialized configuration. Preserve order, repeated sizes, and requested
   formats.
2. For each definition, round dimensions once as upstream does and allocate a
   validated RGBA8 `ThumbnailData` buffer. Keep the requested plate index and
   `ThumbnailsParams` flags with the render request.
3. Filter printable model-volume meshes against the selected plate build
   volume; omit modifiers and wipe-tower geometry when `parts_only` is set.
   Preserve the source mesh traversal order and each instance transform.
4. Compute the union bounds of selected meshes, apply the source x/y/z margins,
   select the source view direction, and construct an orthographic fixed-point
   projection using the plate bounds.
5. Rasterize triangles with integer edge functions, deterministic depth
   ordering, and integer alpha/color blending. Use filament or extruder colors
   and the upstream default color. Clear to transparent unless the effective
   background option requests otherwise. Do not draw the bed.
6. Apply the upstream `thumbnail.vs` lighting model with fixed-point constants:
   top direction `(-0.4574957, 0.4574957, 0.7624929)`, front direction
   `(0.6985074, 0.1397015, 0.6985074)`, ambient `0.3`, top diffuse `0.6 * 0.8`,
   front diffuse `0.6 * 0.3`, top specular `0.6 * 0.125`, and shininess `20.0`.
   Transform and normalize each mesh vertex normal in camera space, interpolate
   intensity across each triangle, add emission `0.1` via `emission_factor` as
   set at `GLCanvas3D.cpp:7132-7133`, preserve alpha, and discard fragments
   below world `z = 0`. Use vertex normals and interpolated vertex intensity;
   do not replace this with flat per-face normals. The normal thumbnail path
   keeps `ban_light` false. A flat-colour silhouette is not equivalent.
7. Return one image per successful render in callback order. The existing
   encoder maps each image to its requested PNG, JPG, QOI, BTT_TFT, or COLPIC
   block. A missing or invalid image is omitted rather than replaced.

The renderer must not use OpenGL, WebGL, a filesystem, a terminal, a native
process, or a new heavy dependency. It must not use wall-clock values, random
values, or worker-count-dependent reductions. One and eight Rayon workers must
produce identical RGBA bytes and identical encoded G-code.

The exact core switch is `ares_core::SlicerContext::render_thumbnails`, set by
`SlicerContext::with_render_thumbnails(bool)` and defaulting to `false`. The
exact `ares-wasm` profile request field is
`ProfileSliceOptions::render_thumbnails`, serialized as the web field
`render_thumbnails`; the web worker sends that boolean and no renderer-specific
field beyond it.

## Export behavior

The existing `gcode_thumbnails::append` path remains authoritative for image
encoding and block layout. Non-BBL exports with rendering enabled contain one
block for each successful configured definition, including repeated definitions.
BBL exports retain their existing no-block branch. Rendering disabled or an
empty definition list leaves the complete G-code stream unchanged.

The Native CLI source branch at `OrcaSlicer.cpp:7524-7573` disables thumbnail
groups when GLFW window creation or OpenGL initialization fails. The parity
runner is headless, so Native produces no thumbnail blocks there. Ares CLI
therefore does not enable rendering implicitly. The web adapter's explicit
`render_thumbnails` option is the only activation in this slice; a CLI switch
is deferred until Native exposes a matching switch.

## Included behavior

- configured sizes, formats, order, and current plate;
- printable-only and parts-only filtering;
- orthographic camera fit, source view direction, projection bounds, margins,
  transparent background, and deterministic object colors;
- software RGBA rasterization during slicing;
- existing image encoders, G-code insertion, and Moonraker metadata discovery;
- behavior independent of Rayon worker count;
- non-BBL browser downloads and BBL/no-definition suppression behavior;
- a KSR-sized model render budget of at most 250 ms per configured 160x160
  image at p95 in the release browser worker and at most 150 ms per image in
  the release CLI path, with added slice wall time no more than 10%;

## Deferred behavior

- 3MF `Metadata/plate_*.png` files;
- desktop Plater camera, selection, and viewport restoration;
- OpenGL/WebGL shader behavior and pixel equality with Native;
- new CLI switches without a matching Native option;
- unrelated display rendering and codec changes.

## Failure modes

Non-positive, non-finite, or otherwise invalid dimensions do not allocate an
image. Empty selected geometry or a failed render yields no block. Existing
format validation, BBL suppression, and encoder errors remain unchanged. No
format is silently replaced, no blank fallback image is fabricated, and no
Native rendering path is called.

## Test plan

Write public behavior tests before implementation:

- Playwright, on private port 4292, slices a non-BBL printer with configured
  thumbnail sizes and formats, downloads G-code, checks one block per entry,
  decodes every image, checks dimensions, and checks that object pixels are
  present. Run the pinned Moonraker `metadata.py` against the download and
  assert its thumbnail records.
- The same browser flow with a BBL printer confirms no block where Native
  writes none. A no-definition flow compares complete bytes with the unchanged
  behavior.
- A public core test supplies one model and compares RGBA and emitted G-code
  from Rayon counts one and eight.
- Focused tests cover size rounding, plate selection, filtering, transparent
  background, repeated definitions, and image insertion without changes to the
  remaining stream.
- Run formatting, strict workspace/all-target linting, the wasm target check,
  non-parity workspace tests, and paired scoped Official and Seeded checks.
  Browser artifacts retain the downloaded G-code, decoded image, metadata
  output, and hashes under the assigned artifact directory.
- In release builds, measure render-only time for every configured size on the
  KSR-sized model in both the browser worker and CLI path. Record median, p95,
  dimensions, slice wall time, and added wall time in the delivery report and
  enforce the per-image and 10% budgets above.
