# ARD: Seam candidates borrow the physical Layer::slice_z

Status: proposed; see [SPEC](seam-candidate-slice-z-spec.md).
Base main c251aa616. No production change or permanent RED test is implied.

## Source owner

Pinned Orca8500fcdcc: PrintObjectSlice.cpp::compute_slice_z:24-47 and
new_layers:49-75 produce/store the physical plane, Layer.hpp::Layer:256-260
retains it. GCode/SeamPlacer.cpp::gather_seam_candidates:1014-1040, especially
1024-1032, consumes that plane; visibility:1042-1057 and place_seam:1500-1650
use the resulting candidates. Existing Rust planned_layers_with_zaa already
represents the producer. The correction boundary is seam_placement/runtime.rs::
layer_slice_zs:51-66 and seam_placement.rs::apply_objects:136-138, not a new pipeline.

## Non-negotiable decisions

- Physical planned layer, raft-shifted layer ID and region-major optional record
  slot are distinct coordinates. Region occupancy/owner cannot determine a
  physical slicing plane. Empty slots do not represent zero-height layers.
- Borrow the retained canonical planned layers and hand off slice_z. Do not
  reconstruct midpoint by scanning region heights: ZAA can select another plane
  and duplicate/empty records do not define physical layer progression.
- Keep canonical plane production, candidate f32 conversion and physical order.
  Replace only the runtime reconstruction compatibility shell. Do not add
  region0/first-nonempty/modulo/dedup/sort/fallback/guard behavior.
- Shared candidate consumers keep their existing visibility, association,
  selection and splitting algorithms. This is not an Ares-owned seam design.
  Unrelated lower-slice, brim, motion, region and loader boundaries stay deferred.
- Observed candidate z0 versus Native slice_z is a handoff witness, not a promise
  that changing it alone produces complete parity. A separately frozen public
  RED must precede production. Residual differences require a separate decision.
- Acceptance remains complete ordered G-code through the platform-neutral
  byte API and CLI, with only established generator identity normalization.
  Failed admissions/observer ABI checks and interrupted legs are not parity REDs.
- All diagnostic hooks remain external evidence only; none survive in core or
  public APIs. Browser WASM and all desktop Tier1 platforms retain the same API.

## Rejected alternatives

Using the first region's records retains the empty-prefix bug; choosing the
first occupied region substitutes occupancy for plane identity. Filtering None
or deduplicating height scans invents grouping/raft policy. Recomputing every
midpoint discards valid canonical ZAA planes. All are wider and less faithful
than borrowing the existing physical-layer producer.
