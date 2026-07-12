# M183 Spec: GCodeThumbnails definition parser API

## Goal
Port the rendering-neutral thumbnail definition parser part of `libslic3r::GCodeThumbnails` into `ares-core` and make the M181 legacy thumbnail composite normalization reuse it.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundaries:

- `OrcaSlicer/src/libslic3r/GCode/Thumbnails.hpp:16-41`: `ThumbnailError`, `ThumbnailErrors`, `GCodeThumbnailDefinitionsList`, and `make_and_check_thumbnail_list` declarations.
- `OrcaSlicer/src/libslic3r/GCode/Thumbnails.cpp:530-604`: thumbnail definition parser and error string behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:397-399`: `GCodeThumbnailsFormat` enum order.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:542-549`: thumbnail format string map.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/GCode/Thumbnails.hpp:24-35` compressed image buffer and compression declarations except parser error-string naming.
- `OrcaSlicer/src/libslic3r/GCode/Thumbnails.hpp:44-105`: thumbnail export to G-code/file output.
- `OrcaSlicer/src/libslic3r/GCode/Thumbnails.cpp:1-529`: PNG/JPG/QOI/BTT/COLPIC compression implementation.
- Thumbnail image generation, filesystem output, UI runtime, slicing behavior, extrusion behavior, G-code writer behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/gcode_thumbnails.rs`: public rendering-neutral parser API.
- `crates/ares-core/src/lib.rs`: module and re-exports for UI/adapter consumers.
- `crates/ares-core/src/options/legacy/thumbnails.rs`: replace private parser implementation with calls to the shared API.
- `crates/ares-core/src/options/tests/legacy_thumbnail_composite.rs`: keep existing legacy ingestion coverage.

## Included API

Expose these `ares-core` types/functions:

- `GCodeThumbnailFormat` enum with variants `Png`, `Jpg`, `Qoi`, `BttTft`, `ColPic` and `as_str()` returning upstream keys `PNG`, `JPG`, `QOI`, `BTT_TFT`, `COLPIC`.
- `GCodeThumbnailDefinition { format, width, height }` with dimensions stored as `f64` to preserve parser values before future rendering/export milestones choose integer image sizes.
- `ThumbnailParseError` enum with variants `InvalidValue`, `OutOfRange`, and `InvalidExtension` matching upstream `ThumbnailError` flags.
- `parse_thumbnail_definitions(thumbnails: &str, default_extension: Option<&str>) -> Result<Vec<GCodeThumbnailDefinition>, ThumbnailParseError>`.
- `thumbnail_error_string(error: ThumbnailParseError) -> &'static str` with messages corresponding to `GCode/Thumbnails.cpp:593-604`.

## Functional requirements

1. Empty input returns an empty definition list.
2. Valid `XxY[/EXT]` entries parse into definitions in input order.
3. Missing or empty `EXT` uses a valid default extension when provided, otherwise `PNG`.
4. Invalid default extension falls back to `PNG`, matching M181 legacy behavior.
5. Present extensions are ASCII-uppercased without trimming and must match upstream keys.
6. Dimensions must parse as finite numbers and satisfy `0 < value < 1000`.
7. Invalid value format returns `InvalidValue`; out-of-range dimensions return `OutOfRange`; invalid extensions return `InvalidExtension`.
8. Existing M181 legacy composite normalization must produce the same normalized stored strings while using the shared parser API.
9. Do not add thumbnail compression/rendering/export behavior, filesystem behavior, UI runtime, slicing behavior, extrusion behavior, G-code writer behavior, new crates, or dependencies.

## Acceptance checks

- Tests prove all five upstream thumbnail formats parse and expose the correct `as_str()` names.
- Tests prove missing/empty/default extensions follow upstream/M181 rules.
- Tests prove empty input returns no definitions.
- Tests prove invalid value, out-of-range, and invalid-extension errors are distinguishable.
- Tests prove error strings contain the upstream message fragments from `GCode/Thumbnails.cpp:597-602`.
- Existing `legacy_thumbnail_composite` tests continue to pass without behavior changes.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
