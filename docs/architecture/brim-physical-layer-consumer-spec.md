# SPEC: compatible later-owner first-physical-layer outer-brim consumer

## Dependency

This consumer depends on the physical-layer producer identity handoff to
Classic ordering (the compatible later-owner wall-ordering slice). It does not
move ordering into BrimPlan/GCode emission or invent an object-config override.
Before public RED, refresh discovery on the producer baseline and confirm this
consumer still diverges separately from the sandwich ordering.

## Owning upstream rewrite slice

Vendored libslic3r source boundary, not an Ares-owned brim pipeline:

- Brim.cpp:421-485, outer_inner_brim_area: configured brim type/width, initial
  flow spacing and object first-layer (layers().front()->lslices or EFC outline)
  producer; object firstLayerObjGroups own brim volume footprints.819-878,
  makeBrimInfillImpl materializes the brim paths. Geometry ownership stays here.
- GCode.cpp:4456-4540, generate_object_brim, and5325-5340: first-layer owned
  brim is emitted before the object's wall/content; first_layer is a physical
  layer condition, not a compatible-region record slot.
- **PerimeterGenerator.cpp:1459-1471** is the exact connecting consumer:
  configured OuterInner reverses collection; alternatively layer_id==0 AND
  OuterOnly AND configured width>0 reverses it so wall export continues inward
  after brim. Later InnerOuterInner sandwich is a distinct branch.
  Do not attribute perimeter permutation to Brim.cpp's area generation.

Exact Rust destination boundary:
project_slice/perimeters/classic/perimeter_append.rs::transform_object's
index==0 / OuterOnly / width>0 effective wall-sequence consumer (currently81-87),
then transform_record/transform_surface/reorder_walls. Read the physical
ClassicTraversalRecord identity from the producer; retain enumeration for
region_options/record slots. Replace record-zero compatibility scaffold with
physical-layer-zero semantics, preserving both brim predicates and reorder
algorithm. Do not confuse physical ID with planned-layer index or raft offset.

brim.rs::BrimPlan::generate and layers::brim_emit / brim::emit remain the
source-owned producer/export context, not correction targets. No geometry,
area, owner grouping, flow, path splitting, import or new scheduling policy.

## Runtime Native-backed discovery (current main, not public RED)

The original M-v3 compatible later-owner input was used, preserving every mesh,
modifier, transform, volume ID, model_instance and plate metadata. Four pure
3MFs change only global Classic / counterbore None / skirt0 / min0 plus the
listed brim/sequence settings.

Using InnerOuter instead of InnerOuterInner deliberately excludes the
later-layer sandwich branch from this brim-specific candidate observation.
First printed Z0.32, later Z0.52. Default gap0.1, explicit brim width8 (zero
for no-brim), original support/raft-free one-filament fixture retained.

| Input | First-layer Native / Ares wall | Discovery disposition |
|---|---|---|
| no brim + InnerOuter | Inner / Inner | all ordered positive-XY role blocks equal |
| OuterOnly + InnerOuter | **Outer / Inner** | true movement/role divergence, both modes |
| OuterOnly + OuterInner | Outer / Outer | all ordered positive-XY role blocks equal |
| InnerOnly + InnerOuter | Inner / Inner | all ordered positive-XY role blocks equal |

Native **8/8** actual exports accepted, both modes. Current-main CLI all4
exit0, real G-code, no rejection masquerading as difference. Both mode positive
cases: after identical first-layer positive-XY Brim blocks, Native first body
wall is Outer, G1 X139.25 Y160.75 E1.20908; Ares first body wall is Inner,
G1 X159.75 Y159.75 E1.09661. This is not ETA/footer formatting or missing brim.

Wall witnesses count positive XY+E, not E-only unretract, as a wall segment.
Controls' ordered positive-XY role blocks equality is **not** a whole-stream
parity claim. This observes the independently falsifiable brim operand
candidate, not an internal Native trace.

### Immutable discovery hashes

| Input | SHA256 |
|---|---|
| none-inner-outer | `a87421a8693ffab880d2b0c838eb0f191fa8855c8743b978cea042b8a6d7b5ab` |
| outer-inner-outer | `f2938a05b5f4a81606f2bed1c24605ec9c17445318e3f9545befab92a3bedbc0` |
| outer-outer-inner | `4db60ddbc830466f539075adb474041c90c7b987832e843f563b8443be0b5576` |
| inner-inner-outer | `f8212c0883c17ac79f799cac8c1e4dbed0242555410971aab43198a1dcc5e787` |

## Included, deferred and dependency-gated exit sequence

Included only configured OuterOnly/nonzero-width first physical layer
collection reversal for later compatible-region owner, on top of the
physical-layer producer. Existing non-brim and explicit OuterInner behavior
must remain; later layers must not gain first-layer reversal. No
source-record-zero special case.

Deferred: lower_slices(index) identity/overhang reversal, planned-index or raft
policy, Arachne ordering, owner selection/compatibility, Filled/counterbore
restoration, loop nesting/chaining/sandwich math, EFC/area generation, brim
footprint/split/ownership variants, support and multi-filament scheduling.
This is a dependent consumer of source layer identity, not a separate Ares API.

1. On the producer baseline, re-run the same pure inputs and ensure positive
   remains a brim-specific Native difference; if the producer already fixes it,
   report no remaining diff instead of manufacturing RED or duplicating
   correction.
2. Separate public Native-first API+CLI RED: exact4 inputs/full streams,
   expected1true ordered divergence+3strict controls each mode; Native legality,
   real first-layer brim/wall order first. Preserve every role, travel, arcs,
   M73/stat/footer and established generator-only treatment.
3. Minimal actual-layer operand consumption; no indirect config rewrite,
   fallback, new guard or owner0 shortcut.
4. GREEN same-input complete strict Native/API/CLI parity, first+later physical
   layers; raw controls unchanged; producer M REDs/counterbore controls and prior
   skirt/brim suites retained. No assertion/input/comparator relaxation.
5. Directed modes/strict/fmt/WASM with fresh Native caches; <=400LOC.
