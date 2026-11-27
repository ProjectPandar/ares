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
- `src/slic3r/GUI/GLCanvas3D.cpp:6945-7060` selects visible volumes using the
  plate build volume, printable status, parts-only status, and modifier/wipe
  tower exclusions. It computes the object bounds and adds the source margins.
- `src/slic3r/GUI/GLCanvas3D.cpp:7061-7145` selects the view, fits the
  orthographic camera, and projects against the plate or object bounds.
- `src/slic3r/GUI/GLCanvas3D.cpp:7146-7293` clears the target, assigns volume
  colors, applies the thumbnail shading path, renders triangles, and reads
  bottom-up RGBA pixels from the framebuffer.
- `src/libslic3r/GCode/ThumbnailData.hpp:36-47` defines the callback parameter
  fields and callback result list. `src/libslic3r/GCode/Thumbnails.hpp:48-75`
  invokes that callback for every configured format and size before encoding.
- `src/libslic3r/GCode.hpp:261,450` and `src/libslic3r/Print.hpp:1086` carry the
  callback through export. `src/OrcaSlicer.cpp:7524-7573` shows that Native CLI
  rendering is conditional on a hidden GLFW/OpenGL context.

The Rust destination is a renderer module under
`crates/ares-core/src/gcode_thumbnails/`, called by the existing project slice
G-code emission boundary. The module consumes the current plate's printable
slice geometry and effective thumbnail definitions. `ares-wasm` exposes only
the existing byte-oriented slice result; it does not duplicate rendering or
encoding. `ares-cli` keeps the renderer option disabled by default because the
Native headless CLI path produces no thumbnail blocks in the parity
environment.

## Render algorithm

1. Resolve the configured thumbnail definitions without re-parsing the
   serialized configuration. Preserve order, repeated sizes, and requested
   formats.
2. For each definition, round dimensions once as upstream does and allocate a
   validated RGBA8 `ThumbnailData` buffer. Keep the requested plate index and
   `ThumbnailsParams` flags with the render request.
3. Filter printable slice volumes against the selected plate build volume;
   omit modifiers and wipe-tower geometry when `parts_only` is set. Preserve
   the source volume traversal order.
4. Compute the union bounds of selected geometry, apply the source x/y/z
   margins, select the source view direction, and construct an orthographic
   fixed-point projection using the plate bounds.
5. Rasterize triangles with integer edge functions, deterministic depth
   ordering, and integer alpha/color blending. Use filament or extruder colors
   and the upstream default color. Clear to transparent unless the effective
   background option requests otherwise. Do not draw the bed.
6. Return one image per successful render in callback order. The existing
   encoder maps each image to its requested PNG, JPG, QOI, BTT_TFT, or COLPIC
   block. A missing or invalid image is omitted rather than replaced.

The renderer must not use OpenGL, WebGL, a filesystem, a terminal, a native
process, or a new heavy dependency. It must not use wall-clock values, random
values, or worker-count-dependent reductions. One and eight Rayon workers must
produce identical RGBA bytes and identical encoded G-code.

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
slice option is the only activation in this slice; a CLI switch is deferred
until Native exposes a matching switch.

## Included behavior

- configured sizes, formats, order, and current plate;
- printable-only and parts-only filtering;
- orthographic camera fit, source view direction, projection bounds, margins,
  transparent background, and deterministic object colors;
- software RGBA rasterization during slicing;
- existing image encoders, G-code insertion, and Moonraker metadata discovery;
- behavior independent of Rayon worker count;
- non-BBL browser downloads and BBL/no-definition suppression behavior.

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
