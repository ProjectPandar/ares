# Task 22O.1 Implementation Plan: Classic Prelude

## Objective

Implement the bounded source slice in the matching Task 22O.1 specification.
Advance real 3MF project slicing from Task 22N through Classic prelude
preparation while retaining `ProjectSlicingIncomplete` as the public sink.

## Execution plan

1. Add focused RED tests for fixed-coordinate bounding boxes and
   `chain_expolygons` ordering, including negative coordinates, holes, equal
   centers, empty input, and repeatability.
2. Add transactional Classic capability tests over typed real-archive Option
   mutations. Preserve predecessor error precedence and assert exact owning
   Option keys.
3. Add `project_slice::perimeters::classic` with owned aligned predecessor and
   prelude types. Validate all records before moving any predecessor object.
4. Port the fixed prelude arithmetic from `process_classic`: existing Flow
   consumption, scaled width/spacing, precise spacing, overlap tolerances,
   smaller external `Flow::with_width`, gap enablement, and loop counts.
5. Port lower-support growth and two-sample polygon series from
   `generate_lower_polygons_series` with existing Clipper offsets.
6. Port counterbore-none, arc-aware simplification, union, and bounding-box
   surface ordering. Keep surface metadata and ordered geometry for the onion
   milestone.
7. Wire `slice_project` through the new stage, consume the complete state, and
   retain the intentional incomplete error.
8. Replace the opaque Task 22N synthetic binary parser fixture with readable
   in-test behavioral bytes; delete the binary.
9. Record the exact included/deferred source boundary in the parity
   architecture and roadmap.
10. Run focused tests, Task 22 regressions, workspace Nextest, rustfmt, strict
    Clippy, workspace/all-feature checks, installed WASM checks, LOC audit, and
    forbidden-pattern scan. Send the resulting diff to independent six-axis
    review; repair concrete findings and repeat review until approved.

## File boundary

Production changes are limited to:

- `crates/ares-core/src/geometry.rs` and a normal `geometry/bounding_box.rs`
  module;
- existing Clipper offset exports needed for source-compatible lower paths;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/perimeters.rs`, its existing Task 22N
  types, and new normal modules under `perimeters/classic/`.

Tests are limited to separate geometry and
`project_slice/tests/perimeters/classic/` modules plus the existing Task 22N
parser test. Documentation changes are the matching spec/plan,
`docs/architecture/option-parity-v4.md`, and `docs/roadmap.md`.

## Verification contract

The real KSR archive must produce a populated, repeatable, slot-aligned Classic
prelude and public `ProjectSlicingIncomplete`. Option-only mutations must alter
precise spacing and gap enablement or return the exact deferred key. Task 22N
checkpoint semantics must not drift. Every changed/new Rust file must be below
400 lines, no production fixture/reference read may appear, and no opaque Task
22N binary embedding may remain.

The next milestone starts at `split_top_surfaces()` and the onion loop in
`PerimeterGenerator::process_classic`; it may not reinterpret this prelude as
an Ares-owned pipeline or route project data through the old rectangular STL
perimeter implementation.
