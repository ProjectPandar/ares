# Consume `combine_brims` in First-Layer Brim Generation

## Goal

Consume the existing Orca `combine_brims` option in Ares brim generation so first-layer outer brims can be emitted as one merged brim envelope instead of one brim envelope per outer contour.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1619`: `combine_brims` is a `PrintConfig` bool option tuple.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1658-1663`: `combine_brims` option definition, default `false`.
- `OrcaSlicer/src/libslic3r/Brim.cpp:931-969`: Orca reads `print.config().combine_brims`, disables it for by-object printing, merges all object brim areas with `union_ex`, cleans the merged area, and calls `makeBrimInfill` once.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5078-5099`: Orca prints the unified first-layer brim before object output when `combine_brims` is enabled and print sequence is by layer.

## Ares Destination Boundary

- `crates/ares-core/src/options.rs` and `crates/ares-core/src/options/brim.rs`: parse the existing `combine_brims` bool into `BrimOptions`.
- `crates/ares-core/src/brims.rs`: use `BrimOptions::combine_brims()` when generating first-layer outer brim paths.
- `crates/ares-core/src/brims/tests.rs` and focused pipeline tests under `crates/ares-core/src/pipeline/tests/`: cover option parsing and observable first-layer brim output.

## Current Ares Behavior

Ares already parses and generates `brim_width`, `brim_object_gap`, `brim_type`, `brim_ears_max_angle`, and `brim_ears_detection_length`. It generates one rectangular outer brim loop sequence per outer contour on layer 0. The existing `combine_brims` registry metadata is not consumed by runtime brim generation.

## Required Behavior

- `combine_brims` defaults to `false`, preserving existing per-contour brim output.
- When `combine_brims` is `true` and the brim type generates ordinary outer brim paths, Ares generates each outer brim loop from the merged bounding box of all first-layer outer contours instead of generating one loop sequence per outer contour.
- The merged bounding box applies the existing `brim_object_gap`, requested `brim_width`, and effective line width clamping rules exactly once per loop.
- `BrimType::OuterOnly`, `BrimType::OuterAndInner`, and `BrimType::AutoBrim` use the combined outer brim envelope.
- Inner hole brims remain per-hole because Orca's relevant source combines object brim areas, not hole brim loops.
- `BrimType::BrimEars` keeps existing corner-ear behavior; local ears are not converted into a global envelope.
- `BrimType::Painted`, `BrimType::NoBrim`, zero `brim_width`, non-first layers, and empty contour layers remain unchanged.
- Pipeline diagnostics and print path ordering continue to count and emit the resulting brim paths from the existing `LayerBrims` output.

## Deferred Behavior

This slice does not implement Orca's full polygon boolean union, `offset2_ex` cleanup, support brim maps, `ObjectID` carrier assignment, `m_brimMap`, `m_objsWithBrim`, by-object print sequence behavior, multi-object instance transforms, support-layer brim integration, G-code-specific pre-object unified brim emission, or new option registry metadata.

## Acceptance Criteria

- A focused brim-generation test proves two separate outer contours produce two first-loop outer brim paths when `combine_brims = false`.
- A focused brim-generation test proves the same two contours produce one first-loop outer brim path spanning the merged outer-contour bounds when `combine_brims = true`.
- A focused test proves `OuterAndInner` combines only the outer envelope while preserving inner hole brim output.
- A runtime options test proves `SliceOptions::brim_options()` reads `combine_brims: true` and rejects non-bool values.
- A pipeline-level test proves a sliced model/options path can carry `combine_brims` into generated brim output and diagnostics without adding option metadata.
- Existing brim behavior tests continue to pass.

## Safety And Rollback

The change is local to first-layer brim generation and option parsing. Rollback is removing the `combine_brims` field/accessor and the combined-bounds branch in `brims.rs`, restoring all existing default behavior because the default remains `false`.
