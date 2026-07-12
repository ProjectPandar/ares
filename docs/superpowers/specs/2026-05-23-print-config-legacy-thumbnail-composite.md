# M181 Spec: PrintConfig legacy thumbnail composite normalization

## Goal
Port the thumbnail composite conversion from `libslic3r::PrintConfigDef::handle_legacy_composite` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundaries:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8099-8130`: `handle_legacy_composite` thumbnail branch.
- `OrcaSlicer/src/libslic3r/GCode/Thumbnails.cpp:530-578`: `GCodeThumbnails::make_and_check_thumbnail_list` parser used by that branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:542-549` and `PrintConfig.hpp:397-399`: thumbnail format enum names and order.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8093-8096`: final `print_config_def.has(opt_key)` unknown-key validation. Ares keeps unknown non-obsolete keys until the option registry is complete enough to validate without dropping unported Orca options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8132+`: wiping-volume matrix composite behavior.
- Thumbnail image generation, compression, embedding, UI field behavior, G-code writer behavior, slicing behavior, extrusion behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: change deserialization to run composite legacy normalization after single-option normalization; keep `legacy.rs` below 400 LOC; expose no public API.
- `crates/ares-core/src/options/legacy/thumbnails.rs`: add a focused private helper for M181 thumbnail parsing and normalization.
- `crates/ares-core/src/options/tests/legacy_thumbnail_composite.rs`: add focused tests for valid normalization, default format behavior, invalid values, empty strings, non-string preservation, unknown-key preservation, and prior `thumbnail_size` alias participation.
- `crates/ares-core/src/options/tests/legacy_alias_top_wall.rs`: update the existing `thumbnail_size -> thumbnails` assertion to reflect that composite thumbnail normalization now runs after the single-option alias.
- `crates/ares-core/src/options/tests.rs`: register the M181 test module.
- `docs/roadmap.md` and `docs/milestones/m181-print-config-legacy-thumbnail-composite.md`: milestone sequencing docs.

## Included behavior

When `SliceOptions` is deserialized and has a string `thumbnails` value:

1. Split the string on commas into thumbnail entries.
2. Parse each entry as `XxY[/EXT]`.
3. Accept dimensions only when both parsed numbers are finite and `0 < dimension < 1000`.
4. If `EXT` is absent or empty, use `thumbnails_format` when it is one of `PNG`, `JPG`, `QOI`, `BTT_TFT`, or `COLPIC` after ASCII uppercase normalization; otherwise use `PNG`. Do not trim extension text before validation because upstream uppercases the substring returned by `std::getline` and validates it directly.
5. If `EXT` is present, uppercase it without trimming and require it to be one of `PNG`, `JPG`, `QOI`, `BTT_TFT`, or `COLPIC`.
6. Normalize valid non-empty lists back to `XxY/EXT, ...` and store the normalized string under `thumbnails`.
7. Keep empty `thumbnails` strings unchanged.
8. Reject invalid entry formats, dimensions outside `(0, 1000)`, invalid extensions, and extensions containing leading/trailing whitespace during deserialization.

## Functional requirements

1. Apply thumbnail composite normalization after the existing M169-M180 single-option legacy normalization, so prior aliases such as `thumbnail_size -> thumbnails` participate in the composite pass.
2. Preserve non-string `thumbnails` values unchanged because typed option validation is outside this milestone.
3. Preserve unknown non-obsolete options exactly as today.
4. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except covered thumbnail composite normalization/rejection.
5. Keep all modified Rust files below 400 LOC.
6. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, thumbnail rendering, or G-code behavior.
7. Do not implement final unknown-key validation from `PrintConfig.cpp:8093-8096` or wiping-volume matrix behavior from `PrintConfig.cpp:8132+` in this milestone.

## Acceptance checks

- Tests prove `thumbnails: "48x48, 300x300/jpg"` with `thumbnails_format: "QOI"` becomes `"48x48/QOI, 300x300/JPG"`.
- Tests prove missing extensions default to `PNG` when `thumbnails_format` is absent or unsupported.
- Tests prove supported extension normalization covers `png`, `jpg`, `qoi`, `btt_tft`, and `colpic`.
- Tests prove empty `thumbnails` string remains empty.
- Tests prove non-string `thumbnails` values remain unchanged.
- Tests prove invalid format, zero/out-of-range dimensions, invalid extension, leading extension whitespace, and trailing extension whitespace reject deserialization.
- Tests prove non-obsolete unknown keys remain preserved.
- Tests prove `thumbnail_size: "256x256"` is first aliased to `thumbnails` and then normalized to `"256x256/PNG"`.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8093-8096` and `PrintConfig.cpp:8132+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
