# M183: GCodeThumbnails definition parser API

## Goal
Port the rendering-neutral thumbnail definition parser boundary from `libslic3r::GCodeThumbnails` into `ares-core` as a reusable API for option ingestion, CLI/WASM callers, and future UI consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundaries are `OrcaSlicer/src/libslic3r/GCode/Thumbnails.hpp:16-41`, `OrcaSlicer/src/libslic3r/GCode/Thumbnails.cpp:530-604`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:397-399`, and `OrcaSlicer/src/libslic3r/PrintConfig.cpp:542-549`. No thumbnail rendering, compression, filesystem output, UI runtime, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `ares-core` exposes thumbnail format, parser error, thumbnail definition, parser, and error-string API aligned with the upstream parser boundary.
- Supported formats preserve upstream enum/key order: `PNG`, `JPG`, `QOI`, `BTT_TFT`, `COLPIC`.
- Parser accepts comma-separated `XxY[/EXT]` definitions and default extension behavior matching M181/upstream parser rules.
- Parser reports invalid value, out-of-range, and invalid-extension errors without panicking.
- Existing legacy thumbnail composite normalization uses the shared parser instead of duplicating parser logic.
- Compression/export behavior from `GCode/Thumbnails.hpp:44+` and `Thumbnails.cpp:1-529` remains deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
