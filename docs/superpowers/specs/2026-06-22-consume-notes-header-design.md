# Consume `notes` in G-code Header Design

## Goal

Consume OrcaSlicer's `notes` option as concrete Ares G-code header output instead of leaving it as registry-only metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1631-1634` declares the PrusaSlicer profile-note options and includes `((ConfigOptionString, notes))`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4723-4731` defines `notes` as a multiline string whose tooltip says the text is added to G-code header comments.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2532-2623` writes the header/config preamble near the start of generated G-code and preserves Orca's BTT-thumbnail header suppression.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5523-5576` appends full configuration key/value comments; `notes` is not in the banned-key set, so non-nil `notes` reaches G-code comments through this path.

## Ares Destination Boundary

- `crates/ares-core/src/gcode_header.rs` owns Ares' platform-neutral G-code header formatting and already receives `SliceOptions`.
- `crates/ares-core/src/gcode.rs` already skips the header when `thumbnails` contains `BTT_TFT`; this slice must preserve that behavior without adding lines to `gcode.rs`.
- `crates/ares-core/src/pipeline/tests/notes_header.rs` will cover concrete runtime output through `format_gcode`.
- `crates/ares-core/src/pipeline/tests.rs` will only register the new focused test module.

## Included Behavior

- Missing `notes` and empty `notes = ""` emit no new header line.
- Non-empty string `notes` emits header comments before normal G-code preamble output using the config-comment form `; notes = <text>`.
- Multiline notes are preserved as valid comments by prefixing each logical line independently:
  - `notes = "alpha\nbeta"` emits `; notes = alpha` and `; notes = beta`.
  - Internal blank lines emit `; notes = `.
  - A trailing newline does not add an extra empty comment line.
- A present non-string `notes` value is rejected at G-code formatting time with `SliceError::InvalidInput("notes must be a string")`.
- `notes` output is not gated by `gcode_comments`; upstream routes it through header/config comments, not movement-inline comment toggles.
- Existing BTT thumbnail header suppression remains authoritative: when Ares skips the header, `notes` is skipped with it.

## Deferred Behavior

- Full `CONFIG_BLOCK_START` / `CONFIG_BLOCK_END` emission and complete `append_full_config` parity.
- `filament_notes` and `printer_notes` runtime/header behavior.
- Orca's exact `ConfigOptionString::serialize` escaping details beyond safe line-wise G-code comments.
- BBL-printer-specific config blocks, thumbnail generation, label-object lists, and full placeholder parity.
- UI/profile-editor behavior and any Ares-owned pipeline redesign.

## Acceptance Criteria

- Focused tests fail before implementation and pass after implementation using `cargo nextest run -p ares-core notes_header_comments`.
- Generated G-code for `notes = "Calibrated profile"` contains exactly a comment line `; notes = Calibrated profile`.
- Generated G-code for missing or empty notes does not contain `; notes =`.
- Generated G-code for multiline notes keeps every note line commented and does not leak an uncommented line into the output.
- `notes = ["bad"]` returns `SliceError::InvalidInput` whose message contains `notes must be a string`.
- `notes` does not appear when `thumbnails` contains `BTT_TFT`.
- `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC checks pass before commit.

## Safety and Rollback

The implementation is isolated to header formatting and tests. Rolling back the slice removes the new helper and test module without affecting movement generation, slicing geometry, fan, speed, extrusion, or CLI behavior.
