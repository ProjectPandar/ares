# ARD: G-code thumbnail software renderer

## Status

Accepted for the G-code thumbnail renderer slice.

## Source boundary

The upstream reference is OrcaSlicer v2.5.0 at
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

- `src/slic3r/GUI/Plater.cpp:14317-14328` implements
  `Plater::priv::generate_thumbnails`: it walks the configured `Vec2d` sizes,
  rounds each size to an integer `Point`, invokes the renderer, and discards
  invalid images.
- `src/slic3r/GUI/GLCanvas3D.cpp:6945-7060` implements
  `render_thumbnail_internal` volume selection over the model-volume meshes:
  instance-transformed `ModelVolume` triangles, printable and parts-only
  filtering, plate build-volume bounds, and object bounds margins.
- `src/slic3r/GUI/GLCanvas3D.cpp:7061-7145` selects the orthographic camera,
  fits those mesh bounds, uses the plate bounds for projection, and clears the
  image to transparent.
- `src/slic3r/GUI/GLCanvas3D.cpp:7146-7293` applies volume colors and the
  thumbnail shader lighting, rasterizes the selected mesh triangles, and reads
  the RGBA framebuffer in `render_thumbnail_framebuffer`.
- `resources/shaders/140/thumbnail.vs:1-47` defines the thumbnail light
  directions, ambient/diffuse/specular constants, vertex-normal lighting,
  interpolated intensity, and camera-space view vector. The paired
  `resources/shaders/140/thumbnail.fs:1-24` applies interpolated lighting,
  emission, alpha, and the below-bed discard.
- `src/libslic3r/GCode/ThumbnailData.hpp:36-47` defines `ThumbnailsParams`
  and `ThumbnailsGeneratorCallback`; `src/libslic3r/GCode/Thumbnails.hpp:48-75`
  invokes the callback for each configured size before encoding its images.
- `src/libslic3r/GCode.hpp:261,450` and `src/libslic3r/Print.hpp:1086`
  carry the callback into G-code export.
- `src/OrcaSlicer.cpp:7524-7573` creates a hidden GLFW context for Native CLI
  rendering and disables thumbnail groups when context creation or OpenGL
  initialization fails.

## Decision

Ares renders configured G-code thumbnail pixels in `ares-core` with a
platform-neutral software rasterizer during slicing. The renderer consumes
instance-transformed `ModelVolume` mesh triangles, the selected plate, and the
effective thumbnail definitions used by G-code export.
It produces deterministic RGBA8 buffers, which the existing
`gcode_thumbnails` encoders place into G-code. The browser and command-line
adapters therefore share the image and encoding path; neither adapter owns an
image codec or a GPU context.

The renderer uses fixed-point or integer arithmetic for coverage, depth, camera
projection, lighting, and color conversion. Traversal order is explicit and
reductions are not dependent on Rayon worker count. Rendering is an
execution-local slice operation, not a persisted profile field or a global
cache.

## Rewrite-policy exception

This is a recorded narrow exception to ARD-0020. The upstream pixel producer is
owned by `slic3r/GUI` (`Plater.cpp` and `GLCanvas3D.cpp`), while the Ares port
boundary is `ares-core` because browser and command-line slicing need one
platform-neutral producer. The exception permits moving the thumbnail render
semantics across the GUI/core boundary without importing OpenGL, wxWidgets, or
GUI state into core. The callback and byte emission remain the upstream
`libslic3r` boundary described above.

The exception does not authorize a new general rendering framework, a desktop
GUI port, or a second image pipeline. The existing `gcode_thumbnails` API is the
compatibility shell until its renderer-owned inputs match the cited upstream
concepts.

## Render contract

The effective definitions come from the parsed `thumbnails` list. For each
configured size, `ThumbnailsParams` semantics are preserved:

- `plate_id` selects the current plate;
- `printable_only` excludes volumes outside the selected build volume;
- `parts_only` excludes modifiers and wipe-tower volumes;
- `show_bed` and `transparent_background` select background behavior;
- the requested size is rounded to positive integer pixel dimensions exactly
  once at the render boundary.

The orthographic camera fits the selected printable model-volume mesh bounds
with the upstream margins, projects against the selected plate bounds, and
uses the upstream view direction. The output uses deterministic extruder or
filament colors and the upstream default color where no color is supplied. The
plate is not drawn into the thumbnail. Empty or invalid render results are
omitted from G-code just as an invalid callback image is omitted by the
existing encoder.

Lighting follows `thumbnail.vs`, not a flat-colour silhouette: transform each
mesh vertex normal into camera space, normalize it, and interpolate the
resulting intensity over each triangle. Preserve the top light direction
`(-0.4574957, 0.4574957, 0.7624929)`, front light direction
`(0.6985074, 0.1397015, 0.6985074)`, ambient `0.3`, top diffuse `0.6 * 0.8`,
front diffuse `0.6 * 0.3`, top specular `0.6 * 0.125`, shininess `20.0`, and
thumbnail emission `0.1` supplied by `emission_factor` at
`GLCanvas3D.cpp:7132-7133`. Use the camera-space view vector for the top
specular term, preserve alpha, and discard fragments below world `z = 0`.
Use the mesh's vertex normals and interpolated vertex intensity; do not replace
this with flat per-face normals. The normal thumbnail path keeps `ban_light`
false. Fixed-point approximations must use documented constants and rounding
so lighting is deterministic.

Rendering is enabled by `ares_core::SlicerContext::render_thumbnails`, set by
`SlicerContext::with_render_thumbnails(bool)`. Its default is `false`. The
`ares-wasm` profile request carries the boolean field
`ProfileSliceOptions::render_thumbnails`, serialized as the web field
`render_thumbnails`; the web worker sends that field and no other renderer
control. A headless CLI slice keeps this option disabled by default. In the
parity environment the Native CLI cannot use its hidden GLFW/OpenGL path, so
its export has no thumbnail blocks; the Ares CLI default consequently emits no
blocks. CLI exposure is added only if a matching Native command-line switch is
identified. A browser request may enable the core renderer and receive the
resulting blocks.

Non-BBL export keeps the existing configured format, order, repeated entries,
wrapping, and encoder behavior. BBL export follows the existing suppression
branch, including the case where rendered images are supplied. With no
thumbnail definitions, output bytes remain unchanged. Pixel equality with the
OpenGL image is not required; RGBA encoding for the same produced pixels
remains byte-exact through the existing encoder tests.

## Included and deferred behavior

Included:

- the configured-size callback semantics and current-plate selection;
- printable and parts-only volume filtering;
- orthographic fit, view direction, plate projection, transparent background,
  and deterministic volume colors;
- fixed-point software rasterization at slice time;
- existing PNG, JPG, QOI, BTT_TFT, and COLPIC emission;
- identical results for one or eight Rayon workers;
- a KSR-sized model render budget of at most 250 ms per configured 160x160
  image at p95 in the release browser worker and at most 150 ms per image in
  the release CLI path, with added slice wall time no more than 10%;
- browser download and metadata visibility of emitted image blocks.

Deferred:

- GPU, OpenGL, WebGL, shader, and Native pixel equivalence;
- desktop Plater state restoration and viewport interaction;
- 3MF `Metadata/plate_*.png` generation;
- CLI image-rendering switches without a matching Native switch;
- unrelated viewer, display, and image-codec changes.

## Failure model

Invalid or non-positive dimensions do not create an image. Missing geometry,
unsupported geometry, or an unavailable render result produces no thumbnail
block rather than a fabricated image. The existing export validation and BBL
suppression rules remain authoritative. No fallback renderer, format
substitution, native process, or thread-count-dependent output is introduced.

## Verification

The implementation is introduced after tests are written. Public behavior is
verified by:

1. A Playwright browser slice on a private port using a non-BBL printer with
   configured sizes and formats. The downloaded G-code has one block per
   configured entry; decoded images have the requested dimensions and contain
   non-background object pixels. The pinned Moonraker `metadata.py` reports the
   same thumbnail entries.
2. A BBL browser slice confirming no block when the Native branch writes none,
   plus a no-definition slice whose complete bytes match the base behavior.
3. A public `ares-core` test that renders the same model with Rayon counts one
   and eight and compares the resulting RGBA buffers and emitted G-code.
4. Focused G-code emission and thumbnail tests, strict formatting and linting,
   the wasm target check, non-parity workspace tests, and paired Official and
   Seeded scoped checks. Browser results are compared on the same fixed input;
   no Native/OpenGL pixel comparison is asserted.
5. A release-profile KSR-sized model run measures render-only time for every
   configured size in the browser worker and CLI path. Record median, p95,
   image dimensions, slice wall time, and added wall time in the delivery
   report; enforce the per-image and 10% budgets above.
