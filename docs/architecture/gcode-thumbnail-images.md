# G-code thumbnail images

## Upstream rewrite boundary

Source authority is OrcaSlicer v2.5.0,
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/libslic3r/GCode/Thumbnails.cpp`: `make_and_check_thumbnail_list`,
  `get_error_string`, `compress_thumbnail` and its five encoding branches.
- `src/libslic3r/GCode/Thumbnails.hpp::export_thumbnails_to_file`: callback
  invocation, encoding and exact output layout.
- `src/libslic3r/GCode/ThumbnailData.{hpp,cpp}`: RGBA payload and validity;
  `ThumbnailsParams` and `ThumbnailsGeneratorCallback`.
- `src/libslic3r/GCode.cpp:4233-4239,4322-4334`: BTT header suppression and
  the non-BBL thumbnail export/error branch in `_do_export`.
- `src/libslic3r/PrintConfig.cpp:10046-10074::handle_legacy_composite`:
  validate and canonicalize loaded thumbnails with the same
  `make_and_check_thumbnail_list`; invalid values throw
  `Invalid value provided for parameter thumbnails`. The load paths call
  this conversion at `Config.cpp:1153,1245,1334,1516`.
- `src/slic3r/GUI/Plater.cpp:7898,7913,14317-14328` and
  `GLCanvas3D.cpp:6945-7293`: orthographic image generation and readback.
  These GUI sources constrain the browser adapter, not the core renderer.

This slice ports those boundaries, not a new Ares image or slicing pipeline.
The existing image-free API is a temporary compatibility shell for Native's
null callback; it remains the normal headless path.

## Reuse and destinations

Reuse `gcode_thumbnails` format/definition types and the existing corrected
`options::{legacy,typed_legacy}::thumbnails` parsing/normalization behavior.
The simpler public parser must not become a competing interpretation of
loaded values. Share the existing stream-number, extension and list rules
where the image-export branch needs them; do not add a third string parser
or change existing configuration-load error policy to activate thumbnails.

Keep ARD-0056's definition probes and `project_slice::gcode_emit::header` BTT
suppression. Add image encoders/output under `gcode_thumbnails`, split by
codec as needed, and call them from the existing G-code preamble boundary in
`project_slice::gcode_emit::orchestration`. Do not insert blocks by editing
finished G-code or parse the serialized config block back into configuration.

`SlicerContext` carries execution-local caller images via an explicit
`with_thumbnail_images` builder. Image data is not a persisted print option,
profile field, global cache or random-number input. Its default constructor
continues to mean no thumbnail callback. Transfer owned buffers once through
the slice; do not clone image pixels during object/layer traversal.

`ares-wasm` and `ares-wasm-js` transport dimensions and binary RGBA payloads
through the existing project/profile/plater slice calls. They do not encode
images, duplicate configuration resolution or serialize pixels as JSON/base64.
The browser's existing viewport supplies images before its worker slice call.
No GPU, OpenGL, filesystem or terminal access belongs in `ares-core`.

## Image and callback contract

`ThumbnailData` is a width, height and contiguous RGBA8 pixels, row-major in
Native framebuffer readback order (bottom row first). Public construction
validates dimensions and exact byte length with checked size arithmetic;
private encoders consume validated images without repeated defensive checks.
Pixel channels are bytes, not normalized floats or premultiplied RGBA.

Resolve requested definitions from the same effective configuration used by
the export call, including retained raw values rather than a UI typed-default
substitute. Expose that core-owned interpretation through WASM for the browser;
UI code does not reimplement the string grammar. Match `Point(Vec2d)` sizing
at the adapter boundary; tests must cover fractional dimensions as well as
integer definitions. No implicit image resizing or format substitution.

The caller supplies one frame per rendered requested size. Preserve definition
order, repeated sizes and repeated formats during encoding; do not deduplicate
output blocks. A deterministic frame for a repeated size may be reused without
rerendering, but every upstream definition still has its own export position.
A missing rendered frame emits no block, like an empty Native callback result.
An absent/empty caller image collection does not activate image export.

## Export behavior

- No supplied images: exactly the existing bytes, including invalid-definition
  load behavior and BTT header suppression. Do not newly validate thumbnails
  on the null-callback path.
- BBL front-config branch: no thumbnail blocks and no thumbnail-export
  validation, even if images were supplied. Preserve existing BBL detection;
  do not infer it from a preset/vendor name.
- Non-BBL with images: retain the upstream export validation guard, then emit
  at the upstream preamble position, before width/executable output. Loaded
  definitions are already valid; this guard is a no-op on those paths.
  Empty definitions emit nothing.
- Loading validates and canonicalizes definitions in
  `PrintConfig.cpp:10046-10074::handle_legacy_composite`, called by
  `Config.cpp:1153,1245,1334,1516`. Preserve the existing public load errors
  and assertions, whether images are absent, present or suppressed for BBL.
- The export kernel's `Invalid thumbnails value:` branch accumulates ordered
  invalid-value, range and extension errors. It is unreachable through these
  loaded public inputs. Prove its text/combinations only through the Native
  observer; do not create an entry point, bypass load validation or add a
  third parser to make it reachable. Investigate any apparent public bypass
  against its corresponding upstream load path before changing behavior.
- Never silently encode a PNG instead of the requested format.
- Ordinary encoded blocks use Native's tags, base64 byte length, 78-character
  rows, separators and `THUMBNAIL_BLOCK_START/END` comments verbatim.
- BTT and ColPic have different layouts, not base64 thumbnail wrappers.
  Preserve BTT's CRLF rows, dimension hex header and end-marker condition
  based on the definition's position. Preserve the first emitted ColPic
  `gimage` label and subsequent `simage` labels across a mixed list.
- Output parity concerns emitted bytes, not unused capacity or bytes after a
  Native C-string terminator in an overallocated compression buffer.

## Per-format byte equality

| Format | Owning upstream implementation | Required proof |
| --- | --- | --- |
| PNG | `compress_thumbnail_png`, miniz `tdefl_write_image_to_png_file_in_memory_ex` | RGBA, vertical flip, native compression parameters, PNG chunks/checksums and compressed bytes |
| JPG | `compress_thumbnail_jpg`, libjpeg defaults and quality85 | Flip, RGBA input/alpha treatment, subsampling, quantization, DCT/Huffman choices and complete JPEG bytes |
| QOI | `compress_thumbnail_qoi`, `qoi_encode` | Flipped RGBA, four channels, sRGB, exact header/opcodes/end marker |
| BTT_TFT | `compress_thumbnail_btt_tft` | Flip, integer alpha multiplication, RGB565 hex, three special zero colors, CRLF layout |
| ColPic | `compress_thumbnail_colpic`, `ColPicEncode`, `ColPic_EncodeStr` | Native capped-size handling, alpha-zero color, palette ordering/ties, run/page encoding, binary header and escaped six-bit text/padding |

Evaluate portable Rust codec dependencies against the Native observer before
selecting them. Equivalent decoded pixels, a valid container or the same
compression family do not establish byte equality. Port the required source
algorithm when a dependency differs, or explicitly defer that format and
report the failed comparison. No approximate encoder, runtime Native/FFI
fallback, silent skip or PNG replacement is accepted. The initial target is
all five formats; none is claimed supported before its own proof.

## Browser renderer

Use the existing three.js viewport's model/instance transforms and current
plate, not a second scene loader or screenshot of editor controls. Follow
`ThumbnailsParams`: printable parts only, no modifiers/wipe tower, requested
sizes, selected plate and orthographic camera. `render_thumbnail_internal`
filters with the plate build volume, fits the selected visible-volume bounds,
adds its small bounds margins and uses the plate bounds for projection.
It clears to transparent and does not draw the plate even though `show_bed`
is true in the export callback parameters. Do not assume that flag renders a
bed/grid overlay. Preserve the ordinary viewport's camera, selection, editor
markers, renderer target/viewport and scene state after readback.

Use framebuffer readback, not browser PNG/JPEG encoding. Preserve Native
size conversion and bottom-up row order. Send those exact RGBA bytes to core;
encoding and G-code placement remain core responsibilities. The existing
viewport is the visual authority; no new settings UX or visual redesign is
part of this slice. Browser and Native rasterizers need not produce identical
pixels; core encoding of identical supplied pixels must produce identical bytes.

## Failure model and test-first verification

Write tests before production implementation. Preserve every failed attempt,
Native command, source/dependency provenance, input hash and output artifact.

1. Original browser witness: select a non-BBL printer, configure explicit
   thumbnail sizes/format, slice and download G-code. Record the missing-block
   runtime failure before adding the image channel or encoders.
2. Build a test-only chain-probe observer invoking the actual upstream
   `GCodeThumbnails` routines at runtime on synthetic RGBA. It records input
   hash, definitions, callback order/parameters, encoded/output bytes and
   errors. Ordinary CLI FFF export passes nullptr (`OrcaSlicer.cpp:7193`);
   `Print.hpp:1086` also defaults the callback to nullptr. A CLI-only run is
   not a positive image oracle. No committed expected image/G-code goldens.
3. Public core E2E supplies the same RGBA through `SlicerContext` and compares
   returned G-code blocks byte-for-byte with the observer, separately for all
   five formats. An absent-API compilation failure is recorded separately from
   the subsequent runtime byte mismatch; neither is mislabeled as the other.
4. Include asymmetric/non-square, solid/run-heavy, high-color and varying-alpha
   frames; row orientation, BTT special colors and ColPic palette ties/runs;
   mixed/repeated definitions, format tags, exact lengths/wrapping/terminators,
   fractional sizes and empty/missing frames. Report any undefined or unstable
   Native encoder boundary rather than inventing expected bytes for it.
5. The observer checks all export-kernel error categories/combinations against
   Native text. Public API tests cover only reachable load errors and preserve
   existing normalization/project-import assertions. Export errors are
   unreachable after the cited load validation; do not invent a public bypass.
   Include non-BBL supplied images, BBL supplied images, empty definitions and
   no-image controls.
6. No-image E2E keeps complete ordered G-code equality under the existing
   generator/date-only normalization contract. Keep the BTT no-header raw-byte
   comparator unchanged. Also verify image insertion leaves the remaining
   complete stream unchanged, not just that a marker exists.
7. Playwright verifies the downloaded non-BBL G-code's configured dimensions
   and format, decodes its image and checks non-empty geometry, not merely a
   nonzero file length. Exercise current-plate selection and ensure rendering
   did not alter the visible camera/selection. Retain the download, decoded
   image, screenshot, request metadata and hash/comparison receipt.
8. Run rustfmt, strict workspace/all-target Clippy, wasm32 checks, TypeScript
   and oxlint. Report targeted versus full-suite coverage explicitly.

The unchanged beta thumbnail source may provide development observer results
while the v2.5.0 environment is prepared; keep source and binary provenance
separate and re-verify against the release environment before a
release-runtime parity claim.

## Deferred adjacent behavior

3MF plate thumbnail storage, CLI image rendering, GPU/rendering parity with
Native shaders, new viewer runtime and unrelated preset/configuration policy
are excluded. The legacy standalone STL API is not expanded into a second
image pipeline; the project/profile/plater path owns this upstream export
slice. Any unproven codec must be named explicitly here before declaring the
implemented subset complete; it must not be hidden behind a fallback.
