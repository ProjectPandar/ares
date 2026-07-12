# Consume Different Extruders Slicing Guard Design

## Goal

Consume the existing `is_using_different_extruders()` and `support_different_extruders()` runtime option helpers at the slicing boundary. Ares currently emits single-tool G-code and has no tool-assignment pipeline, so every heterogeneous multi-extruder configuration detected by the current source-shaped helpers must fail early instead of silently producing misleading output.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:660-661` declares `DynamicPrintConfig::is_using_different_extruders()` and `DynamicPrintConfig::support_different_extruders(int&)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8716-8742` compares per-extruder `extruder_type` and `nozzle_volume_type` across configured nozzles.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8744-8765` determines whether the configured `extruder_variant_list` supports different extruders.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5239-5304` provides the extruder/nozzle-volume variant string context used by Orca's variant list logic.

## Ares Boundary

- `crates/ares-core/src/options/different_extruders.rs` already owns `SliceOptions::is_using_different_extruders()`.
- `crates/ares-core/src/options/support_different_extruders.rs` already owns `SliceOptions::support_different_extruders()`.
- `crates/ares-core/src/options/different_extruders.rs` will add one crate-internal validation helper that combines both source-shaped predicates for Ares' current single-tool slicing boundary.
- `crates/ares-core/src/pipeline.rs` will call that helper before model loading, next to the existing print-sequence guard.
- `crates/ares-core/src/pipeline/tests/different_extruders_guard.rs` will cover slice-time behavior without growing `crates/ares-core/src/tests/mod.rs`.

## Included Behavior

- Keep homogeneous or single-nozzle configurations accepted.
- Reject configurations where `is_using_different_extruders()` is true.
- Still call `support_different_extruders()` when `is_using_different_extruders()` is true, so malformed `extruder_variant_list` values are consumed and reported through the existing source-shaped helper before the unsupported-feature error is returned.
- Return `SliceError::InvalidInput` with a message mentioning `different extruders`.
- Propagate existing parse errors from `extruder_type`, `nozzle_volume_type`, `nozzle_diameter`, and `extruder_variant_list` instead of hiding them.
- Run the guard before model loading so invalid unsupported printer configuration fails even for otherwise valid STL input without doing slicing work.

## Deferred Behavior

- Do not implement multi-tool path assignment, toolchange G-code, filament mapping, wipe tower behavior, or extruder-specific speed/flow/temperature scheduling.
- Do not change `get_index_for_extruder_*` helpers or filament/extruder profile normalization.
- Do not add new option metadata, registry entries, public API, crates, dependencies, or Ares-owned pipeline features.
- Do not implement Orca's allowed heterogeneous multi-extruder slicing behavior in this slice; Ares will reject detected heterogeneous configurations until tool assignment and toolchange output are ported.

## Acceptance Criteria

- `run_slicing_pipeline()` and `slice()` reject a two-nozzle configuration with differing `extruder_type` values when `extruder_variant_list` is missing.
- `run_slicing_pipeline()` and `slice()` reject a two-nozzle configuration with differing `nozzle_volume_type` values when `extruder_variant_list` has only one effective variant token.
- A two-nozzle configuration with matching `extruder_type` and matching `nozzle_volume_type` still slices.
- A two-nozzle configuration with differing extruder/nozzle-volume values and multiple `extruder_variant_list` tokens is still rejected, because Ares has no multi-tool slicing/G-code behavior yet.
- An invalid `extruder_type`, `nozzle_volume_type`, or `extruder_variant_list` value returns the existing parse `SliceError::InvalidInput` mentioning the invalid option key instead of the generic unsupported-feature message.
- The different-extruders guard runs before model loading: invalid model bytes with a detected heterogeneous configuration return the `different extruders` error instead of `unsupported or malformed model input`.
- Existing by-object print sequence rejection remains unchanged.
- All touched Rust source files remain at or below 400 LOC.

## Verification

- Targeted tests:
  - `cargo test -p ares-core --lib different_extruders_guard`
  - `cargo test -p ares-core --lib print_sequence_gcode`
- Full checks before commit:
  - `cargo fmt --check`
  - `cargo test -p ares-core --lib`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `git diff --check`
  - `bad=0; while IFS= read -r -d '' f; do n=$(wc -l < "$f"); if [ "$n" -gt 400 ]; then printf '%s %s\n' "$n" "$f"; bad=1; fi; done < <(find crates/ares-core/src -name '*.rs' -print0); exit "$bad"`

## Documentation Impact

This spec and its implementation plan are the documentation for the runtime slice. No user-facing docs or roadmap changes are required because the change consumes existing source-shaped helpers in the current pipeline guard.
