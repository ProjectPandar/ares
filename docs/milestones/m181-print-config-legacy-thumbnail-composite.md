# M181: PrintConfig legacy thumbnail composite normalization

## Goal
Port the thumbnail composite normalization branch from `libslic3r::PrintConfigDef::handle_legacy_composite` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8099-8130`, using the thumbnail-list parsing rules from `OrcaSlicer/src/libslic3r/GCode/Thumbnails.cpp:530-578`, into `ares-core` option ingestion.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the thumbnail composite branch in `PrintConfig.cpp:8099-8130` plus the called thumbnail parser in `GCode/Thumbnails.cpp:530-578`. No new Ares pipeline, crate, dependency, option registry expansion, UI behavior, slicing behavior, extrusion behavior, thumbnail rendering, or G-code writer behavior is added.

## Exit checklist
- `thumbnails` string values are parsed as comma-separated `XxY[/EXT]` entries.
- Missing thumbnail extensions use `thumbnails_format` when it is a supported format string, otherwise `PNG`.
- Supported extensions are normalized to Orca enum names: `PNG`, `JPG`, `QOI`, `BTT_TFT`, and `COLPIC`.
- Valid thumbnail lists are normalized back to `XxY/EXT, ...` form, including values that reached `thumbnails` through the older `thumbnail_size` alias.
- Empty `thumbnails` strings remain unchanged.
- Invalid thumbnail values, out-of-range dimensions, invalid extensions, and extensions with surrounding whitespace reject deserialization.
- Non-string `thumbnails` values remain unchanged because typed option validation is outside this milestone.
- `PrintConfig.cpp:8093-8096` final unknown-key validation, `PrintConfig.cpp:8132+` wiping-volume matrix composite handling, thumbnail image generation/rendering, UI behavior, slicing behavior, extrusion behavior, and G-code behavior remain deferred.
- `crates/ares-core/src/options/legacy.rs` remains below 400 LOC by moving thumbnail-specific helper code into a focused submodule.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
