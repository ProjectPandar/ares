# PrintConfig Option Registry Foundation Spec

## Goal
Port the first `libslic3r::PrintConfig` option-definition boundary into `ares-core` by adding a source-cited registry for the options Ares already parses, without changing option parsing behavior or generated G-code.

## Background
Ares already preserves arbitrary Orca option keys in `SliceOptions` and has typed accessors for a small subset. To eventually port all OrcaSlicer options in small milestones, Ares needs an upstream-aligned option-definition registry that records each option key, value kind, default, and `PrintConfig.cpp` source location.

Relevant upstream source:
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp` defines `PrintConfigDef`, `DynamicPrintConfig`, and option-definition concepts.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp` calls `this->add("...", co...)` for each option and assigns defaults, labels, ranges, enum values, and ratio behavior.
- Current typed Ares options map to `PrintConfig.cpp` entries including `layer_height`, `initial_layer_height`, `nozzle_diameter`, `filament_diameter`, `min_layer_height`, `max_layer_height`, infill, width, speed, skirt, brim, and bridge options.

Current constraints:
- `crates/ares-core/src/options.rs` is close to the 400 LOC limit and must not grow significantly.
- `ares-core` remains WASM-safe and filesystem-free.
- `ares_core::slice` and `ares slice --options option.json -o output.gcode input.stl` behavior must remain unchanged.

## Requirements
- Add an `ares-core` option registry module under `crates/ares-core/src/options/` instead of expanding `options.rs`.
- Define an `OptionValueKind` enum covering current typed kinds: `Float`, `FloatOrPercent`, `Percent`, `Int`, `Bool`, `Enum`, `Floats`.
- Define an `OptionDefinition` data type with at least:
  - `key: &'static str`,
  - `kind: OptionValueKind`,
  - `default_value: &'static str`,
  - `source: &'static str` pointing to a concrete `PrintConfig.cpp` line or line range.
- `default_value` is upstream registry metadata from OrcaSlicer, not a behavioral parsing source of truth for this milestone. Existing Ares parsing defaults remain unchanged even when upstream metadata differs, such as `initial_layer_height` using Ares fallback behavior today.
- Register every option currently parsed by `SliceOptions` or delegated bridge/brim/skirt parsing:
  - `layer_height`, `initial_layer_height`,
  - `nozzle_diameter`, `filament_diameter`, `min_layer_height`, `max_layer_height`,
  - `sparse_infill_density`, `infill_direction`, `sparse_infill_line_width`, `is_infill_first`,
  - `line_width`, `outer_wall_line_width`,
  - `travel_speed`, `outer_wall_speed`, `sparse_infill_speed`,
  - `skirt_loops`, `skirt_distance`, `skirt_height`, `skirt_speed`,
  - `brim_width`, `brim_object_gap`, `brim_type`,
  - `bridge_flow`, `internal_bridge_flow`, `bridge_speed`, `internal_bridge_speed`, `bridge_no_support`, `thick_bridges`.
- Expose registry accessors from `ares-core`:
  - `option_definitions() -> &'static [OptionDefinition]`,
  - `option_definition(key: &str) -> Option<&'static OptionDefinition>`.
- Add `SliceOptions::known_definition_count()` or equivalent read-only helper that counts known definitions for its provided keys without rejecting unknown keys.
- Preserve unknown-option behavior: unknown keys remain stored and must not become validation errors.
- Do not change existing default values, parsing, validation, G-code output, public byte API, CLI behavior, crates, or dependencies.
- Update M22 roadmap/milestone docs to describe this option-registry foundation and move the G-code writer parity milestone later.
- Plan/spec review must receive independent APPROVE before implementation.
- Final implementation must receive independent spec-compliance APPROVE and code-quality APPROVE before commit.
- Verification must include registry unit tests, unknown-option preservation tests, an exact-byte G-code no-change regression for a fixed fixture, LOC check under 400, `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, and `git diff --check`.

## Non-goals
- Do not port all `PrintConfig.cpp` options in this milestone.
- Do not add new typed option accessors.
- Do not change option validation behavior.
- Do not introduce generated-code tooling for PrintConfig parsing.
- Do not add workspace crates or dependencies.
