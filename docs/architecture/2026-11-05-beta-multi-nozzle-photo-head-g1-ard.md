# Multi-nozzle photo-head G1 inline snapshot

## Source and destination

Port the multi-nozzle photo-head clause of `GCode::FarthestPointTimelapseContext` and `GCode::_extrude::check_and_insert_timelapse` from OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. The owning source is `src/libslic3r/GCode.hpp:789–807` and `GCode.cpp:6451–6455,6683–6687,9831–9853`. The Rust destination is `ares-core::project_slice::gcode_emit::layers::farthest::{begin_layer,check_and_insert}`; existing G1 post-endpoint calls stay in `motion/path/constant.rs`. This is an upstream rewrite slice, not a separate Ares snapshot pipeline.

The existing target search, tool-ordering winner and shared timelapse renderer remain implementations of their cited upstream concepts. Their temporary Ares compatibility boundaries must not acquire new target heuristics or a second camera-command generator.

## Photo head is not nozzle count

Upstream makes a single nozzle always eligible as the target owner. For multiple nozzles, eligibility instead requires that the selected farthest target belongs to the ordering-derived physical photo head (`GCode.cpp:6451–6455`). The inline hook separately requires the currently active physical head to equal that photo head (`:9832–9837`). It does not reject a machine merely because its nozzle vector has more than one entry.

Use the existing computed target-owner flag and existing active-head check. Remove the additional single-nozzle-only rejection, not the physical-head checks. Authoring filament-map IDs, zero-based runtime nozzle indices and firmware-facing `physical_extruder_map` labels are different domains; only the shared template overlay maps runtime indices to firmware labels. Do not compare an input map ID or a rendered camera E argument with a runtime head index.

Retain global print-frame endpoints, strict distance below 0.5 mm, source-order target ties and per-physical-layer reset. Do not subtract the nozzle offset on only one side of the distance test.

## Generation and lifetime

Use `timelapse::append_inline` and its existing shared generator as the implementation of `generate_timelapse_gcode(..., true)` (`GCode.cpp:6467–6550,9847–9851`). Preserve template bytes, labels, physical-ID overlay, skipped safe-position selection, writer-position/Z bookkeeping and nonempty-generation insertion semantics. The existing layer-end union of the inline insertion flag (`:8165`) prevents a second traditional snapshot. Preserve first-extrusion synchronization and all deferred-hop/force-Z/unknown-position handling. No change to cooling or custom-G-code invalidation belongs to this boundary.

## Included and deferred

Included: enabled traditional non-I3 machines with multiple configured nozzles, one active physical photo head in each retained witness, and the existing plain/fitted-linear G1 endpoint hooks. Cover zero-based runtime heads 0 and 1 with remapped firmware labels and a nonzero nozzle offset. Preserve existing single-nozzle behavior and the disabled-subsystem behavior.

Deferred: mixed-head interleaving qualification; new target selection or most-used-head policy; safe-position/picture-head travel; a farthest target owned by another head; I3/smooth placement; G2/G3 endpoint hooks (`:10027`); segmented/scarf hooks (`:10174`); and the no-inline-hit fallback (`:8171–8177`). Their existing behavior and guard conditions remain unchanged; this slice does not claim parity for them. No option materialization, retraction arithmetic, parser/cache invalidation, tolerance adjustment, command suppression or acceleration relocation is included. Core remains byte-oriented, platform-neutral and independent of filesystem, terminal and viewer APIs.
