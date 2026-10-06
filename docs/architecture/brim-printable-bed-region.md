# Brim printable-bed region

## Decision status

Approved source-cited rewrite slice (`2bf5a2a14`). Product implementation follows public RED and prewritten controls.

## Upstream ownership

OrcaSlicer v2.4.2, commit `8500fcdccaa10b5099ac20d252af3a7c560046f1` owns this behavior:

- `src/libslic3r/Model.cpp:3014–3026`: bed-polygon construction inside `Model::setPrintSpeedTable`; `Model.hpp:1462–1472,1614`: its stored polygon and getter.
- `src/libslic3r/Print.cpp:3256–3265`: per-extruder printable polygons.
- `src/libslic3r/Brim.cpp:27–45,636–686`: instance coordinates, empty-list fallback, mapped physical bed selection and final brim-area subtraction.
- `src/libslic3r/Brim.cpp:819–878`: the existing `makeBrimInfillImpl` downstream geometry/ordering consumer.
- `src/libslic3r/Brim.cpp:899–956`, `Print.cpp:2535–2544,3154–3163`, `GCode.cpp:2931–2936`: clipped-area footprints and first-layer hull/template consumers.
- `src/libslic3r/PrintConfig.cpp:11511–11541`: the fallback printable shape.

The destination is `crates/ares-core/src/project_slice/gcode_emit/brim.rs::BrimPlan::generate` and, if needed, a private `brim/bed_area.rs` geometry module. Existing explicit-outer, single-object eligibility remains unchanged. `BrimPlan` is a temporary compatibility shell around the cited upstream brim concepts, not an independently designed Ares pipeline. The separate legacy `brims` API is not this behavior's owner.

## Non-negotiable contracts

1. Extract only the source-owned bed geometry needed by brim. Do not introduce `GlobalSpeedMap` singleton state, port unrelated speed calculations, add dependencies on GUI/native runtime, or put I/O and probes in core.
2. Preserve the cited `Model::setPrintSpeedTable` interpretation: complete four-point exclusion groups, original orientation, discarded incomplete trailing group, flat difference traversal and first-polygon selection. A general polygon-with-hole interpretation is a different algorithm and is not an acceptable replacement.
3. Preserve the distinction between an empty configured per-extruder outer list and existing per-slot groups. The former uses the source Model bed polygon; configured groups follow `Print::get_extruder_printable_polygons`. Do not invent empty-inner-group fallbacks.
4. Select the physical bed region using the source logical usage and applied filament map. Do not alter existing Flow/nozzle selection or the recommendation lifecycle as part of this slice.
5. Follow the source's `scale_(30.)`, `jtRound`, `SCALED_RESOLUTION` offset and expanded-bed difference literally. Take `.front()` only when the result vector is nonempty, exactly as the source does; do not search/reorder entries. Combine this region with the existing object-island no-brim region in the final brim-area difference before simplification/loop stepping. Preserve contour traversal semantics rather than substituting a similar-looking intersection.
6. Upstream's `shift_without_plate_offset` describes its native storage interface; it does not authorize a second Ares origin convention. Ares/O6 keeps geometry global, with bed geometry in the same coordinate system as the print-volume gate. Brim clipping consumes that shared coordinate system without independently adding or subtracting plate origin. Only writer motion formatting and footprint output subtract origin, as owned by O6. Preserve input placements; do not rewrite G-code text. The Kobra fixtures are single plate with origin zero, so this slice does not depend on O6. Whichever slice merges later rebases and reconciles this interface.
7. Covered-area footprint data must come from the clipped brim region consumed by the source footprint/hull path. An unmasked outer offset is not the same region.
8. Empty brim geometry is not a new object-volume validation failure. Preserve inactive brim behavior and existing eligibility. Native undefined empty first-polygon accesses are not permission to add a legacy fallback or a new processing-time validation policy.

The print-volume gate owns the shared `PartPlate::set_shape` point transform (`PartPlate.cpp:3179–3187`), exposed internally as `project_slice::state::global_bed_points`. Brim uses it for fallback bed, exclusion groups and configured tool areas; it has no separate plate-origin arithmetic.

The separately approved closed-loop seam integration is owned by `ExtrusionEntity.cpp:265–312::ExtrusionLoop::split_at` and `Polyline.cpp:757–885::Polyline3::split_at_index/split_at`. Its Rust destination is only `gcode_emit/skirt/geometry.rs`: a repeated terminal vertex represents closure, not another cyclic vertex. The shared skirt caller remains subject to zero-byte-change full official/seeded controls. Wipe, Flow and ring connection behavior are not part of this correction.

## Included and deferred behavior

Included: supported explicit-outer single-object brim bed-region construction, mapped slot selection, final subtraction, source coordinate-frame composition and corresponding footprint consumption. Reuse the current downstream loop algorithms; this is not a Flow, connection or ordering rewrite.

Deferred: generalized multi-object/support/ears/auto-brim generation, the rest of `GlobalSpeedMap`, new configuration validation/loading, Flow selector corrections, bridge processing, and independent O6 implementation.

The centered Kobra-3 0.4 input is an inactive `auto_brim` control, not a successful explicit-brim oracle. Existing original-position rejection fixtures remain separate from centered slicing coverage. Complete ordered-byte parity with only the existing generator normalization is the acceptance policy; diagnostic selected-move or point checks cannot replace it. Prewritten controls include ordinary single-object explicit brim and machines without `bed_exclude_area`, with zero output change, plus independent complete official/seeded current-base comparisons.
