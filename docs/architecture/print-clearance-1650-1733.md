# Print.cpp:1650-1733 printable-clearance port

## Source boundary

This is a source-cited Rust rewrite slice of Native beta `345efec64c7b95a05ce5cf96df0dde66a158835c`, specifically `OrcaSlicer/src/libslic3r/Print.cpp:1650-1733` inside `Print::layered_print_clearance_valid` (the upstream spelling is retained in the source citation). The Ares destination is the private applied-print validation boundary under `crates/ares-core/src/print/validate/`, called by the existing layered collision path. Existing Ares estimate helpers are a temporary compatibility shell around the cited upstream `WipeTowerEstimate.cpp`/`WipeTower.cpp` concepts, not a new Ares validation pipeline.

The prerequisite `PrintApply.cpp:2207-2216` normalization port is already landed separately at `ec357f05a`; this slice consumes its resolved applied configuration and does not change normalization predicates.

## Included upstream behavior

1. Count the Native effective tower filaments through `print.extruders(true)`, including current-plate ToolChange membership from `Print.cpp:784-793` as already ported by the prerequisite. Use the effective applied configuration and selected plate, not raw slot cardinality.
2. Read selected-plate tower X/Y with Native vector-at semantics, add the public plate origin, narrow to the Native f32 boundary, and retain the configured rotation angle with the pinned promotion/order. Construct the four body corners, rotate about the scaled anchor, and preserve existing integer polygon rounding and offset semantics.
3. Before generation, use the shared estimated tower width/depth/brim producer. For explicit nonnegative brim, offset the checked body hull by the resolved brim. Automatic brim and estimated cone/base warning geometry remain deferred from the hard rejection.
4. Obtain the live shared printable polygon from the existing public-volume/instance carrier, including Native’s fewer-than-two-nozzles, missing-zone, and ordered zone-intersection fallback behavior. Translate it by the same scaled plate origin.
5. Apply the existing integer polygon difference to the complete checked hull. A nonempty difference returns the existing `SliceError::LayeredPrintCollision` diagnostic, which the CLI maps through its existing `-64` family. Do not add a tolerance, alternate placement, bounding-box shortcut, or new public estimator API.

## Deferred upstream behavior

Post-generation mesh-bottom validation, exact generated footprint checks, estimated warning/proximity diagnostics, tower exclusion and wrapping hard checks, cone/exclusion/wrapping geometry, PartPlate fitting/relocation, and generated-bottom mesh validation remain deferred to their owning upstream slices. Purge paths, G-code output, footer/timing fields, and comparator policy are unchanged. The unchanged-input rejection must remain a failing public E2E until this slice is implemented and verified; no intermediate rejection-green claim is allowed.

## Verification contract

First retain the existing unchanged-input Native `-64` / no-G-code and Ares-success failure-first artifacts. Then implement this exact boundary and run the public E2E in official and seeded modes with actual source, input, resource, catalog, and ELF bindings. Require Native-equivalent rejection, exact diagnostic family, no G-code, no pass-to-fail transitions in the retained controls, and strict `cargo fmt`, diff checks, and locked/offline workspace/all-targets/all-features Clippy with `-D warnings`. Any newly exposed difference requires a separate source-first slice rather than comparator weakening.
