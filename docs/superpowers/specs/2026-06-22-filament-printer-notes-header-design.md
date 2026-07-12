# Filament and Printer Notes Header Design

## Goal

Consume OrcaSlicer `filament_notes` and `printer_notes` as concrete Ares G-code header comments. This slice extends the existing `notes` header behavior so profile notes already preserved in `SliceOptions` are visible in generated G-code instead of remaining metadata-only values.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1631-1634` declares the profile notes options: `filament_notes`, `notes`, and `printer_notes`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2375-2382` defines `filament_notes` as multiline `ConfigOptionStrings` with default `[""]`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4723-4731` defines `notes` as multiline `ConfigOptionString` whose text is added to G-code header comments.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4963-4970` defines `printer_notes` as multiline `ConfigOptionString` with an empty default.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2602-2614`, `3488-3497`, and `5523-5575` append full config key/value comments into generated G-code, including non-banned profile-note keys.

## Ares Destination Boundary

- Extend `crates/ares-core/src/gcode_header.rs` only; this module already owns Ares header comments and current `notes` output.
- Extend focused coverage in `crates/ares-core/src/pipeline/tests/notes_header.rs`.
- Update `docs/roadmap.md` after implementation review.

## Included Behavior

- Keep existing `notes` behavior unchanged:
  - missing or empty `notes` emits nothing;
  - non-empty `notes` emits one `; notes = ...` line per logical line;
  - non-string `notes` returns `SliceError::InvalidInput`.
- Add `printer_notes` header comments:
  - missing or empty `printer_notes` emits nothing;
  - non-empty `printer_notes` emits one `; printer_notes = ...` line per logical line;
  - non-string `printer_notes` returns `SliceError::InvalidInput`.
- Add `filament_notes` header comments:
  - missing or empty vector, or vectors containing only empty strings, emit nothing;
  - a JSON string is accepted as a single note value for profile-fragment tolerance;
  - a JSON array must contain only strings;
  - non-string array entries, object, bool, number, and null return `SliceError::InvalidInput`;
  - each non-empty string entry emits one or more `; filament_notes = ...` lines, splitting embedded newlines the same way `notes` currently does.
- Preserve header ordering by appending notes after existing header fields and optional `filament_colour`, in this order: `notes`, `filament_notes`, `printer_notes`.
- Preserve BTT thumbnail header suppression: when `should_skip_header_for_btt_thumbnail` suppresses the whole header block, the new notes are also suppressed.
- Do not change movement, extrusion, speed, fan, temperature, or footer G-code commands.

## Deferred Behavior

- Full Orca config-block serialization and exact `ConfigOptionString(s)::serialize()` quoting for notes.
- BBL/non-BBL branch-specific config block placement.
- Preset/profile UI behavior and note editing.
- Multi-extruder note mapping beyond preserving valid `filament_notes` vector entries.
- `printer_notes`-based Prusa XL detection behavior from `PrintConfig.cpp:11341-11355`.

## Acceptance Criteria

- A slice with `filament_notes: ["PLA dry", "Second spool"]` emits both values as `; filament_notes = ...` header comments.
- A slice with multiline `filament_notes` splits embedded newlines into repeated `; filament_notes = ...` comments.
- A slice with `printer_notes: "Garage printer"` emits `; printer_notes = Garage printer`.
- Missing and empty note values do not emit note comments.
- Invalid `filament_notes` and `printer_notes` values fail through `format_gcode` with `SliceError::InvalidInput`.
- BTT thumbnail header suppression hides `notes`, `filament_notes`, and `printer_notes`.
- Existing movement and extrusion command lines remain unchanged when only these notes change.
- `cargo nextest run -p ares-core notes_header` covers the focused behavior.
