# Wall Infill Order Infill-First Design

## Goal

Consume the remaining runtime behavior carried by OrcaSlicer `wall_infill_order` legacy values: legacy values whose order starts with `infill/` must make Ares print infill before perimeters on non-first layers through the existing `is_infill_first` print-path ordering path, while preserving the already-implemented `wall_sequence` migration.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:108-116` defines `WallInfillOrder` variants, including `InfillInnerOuter` and `InfillOuterInner`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7946-7958` migrates `wall_infill_order` into `wall_sequence`.
- `OrcaSlicer/src/libslic3r/Config.cpp:938-947` derives `is_infill_first = true` for `infill/outer wall/inner wall` and `infill/inner wall/outer wall`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5385-5397` prints non-infill-first perimeters, then infill, then infill-first perimeters.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6063-6073` keeps first-layer walls first and otherwise gates perimeter output by `is_infill_first`.

## Ares Destination Boundary

- `crates/ares-core/src/options/legacy.rs` owns the current `wall_infill_order` compatibility ingestion and must preserve both resulting knobs:
  - `wall_sequence` for wall ordering.
  - `is_infill_first = true` for upstream `infill/...` variants.
- `crates/ares-core/src/options.rs::SliceOptions::is_infill_first()` remains the runtime accessor consumed by the pipeline.
- `crates/ares-core/src/print_paths.rs::generate_print_paths()` remains the only print-path ordering implementation for this slice. It already keeps first-layer walls first and places infill before perimeters on later layers when `is_infill_first` is true.

## Included Behavior

- Accept legacy `wall_infill_order = "infill/inner wall/outer wall"` and produce both:
  - `wall_sequence = "inner wall/outer wall"`.
  - `is_infill_first = true` unless the user explicitly supplied `is_infill_first`.
- Accept legacy `wall_infill_order = "infill/outer wall/inner wall"` and produce both:
  - `wall_sequence = "outer wall/inner wall"`.
  - `is_infill_first = true` unless the user explicitly supplied `is_infill_first`.
- Preserve existing legacy mappings for:
  - `inner wall/outer wall/infill`
  - `outer wall/inner wall/infill`
  - `inner-outer-inner wall/infill`
- Preserve explicit `is_infill_first` when present. The explicit modern key is more specific than the legacy combined key.
- Make the behavior visible in generated G-code by using an input with `wall_infill_order = "infill/inner wall/outer wall"` and no explicit `is_infill_first`; the second layer must emit sparse infill before perimeter, while the first layer still emits perimeter before infill.

## Deferred Behavior

- No new `WallInfillOrder` runtime enum is added.
- No change to Ares perimeter wall ordering beyond the existing `wall_sequence` behavior.
- No support for arbitrary custom `wall_infill_order` values beyond preserving the current invalid wall-sequence rejection.
- No multi-region/tool-ordering parity beyond Ares' current single-region print-path order.
- No changes to Orca preset UI, project diff tracking, `different_settings_to_system`, wipe tower behavior, or object-by-object scheduling.

## Acceptance Criteria

- A new options test proves the two `infill/...` legacy values set `is_infill_first()` to true and still set the expected `wall_sequence`.
- A new options test proves explicit `is_infill_first = false` is preserved when combined with a legacy `infill/...` value.
- A new pipeline/G-code test proves `wall_infill_order = "infill/inner wall/outer wall"` changes second-layer path/G-code order through existing `is_infill_first` behavior without changing first-layer wall-first output.
- Existing wall sequence, infill-first, and legacy normalization tests remain green under `cargo nextest run`.
- `docs/roadmap.md` is updated to mark the `wall_infill_order` infill-first runtime slice as consumed while keeping unrelated wall-order and full Orca parity behavior deferred.
