# Multi-instance brim provenance barrier and destination amendment

Source/input localization only; **no production edits** after the public RED.

## Verified producer and frame/lifetime chain

The frozen target and no-skirt witness each have object2 instances at80/140,
Y110, and object3 at110/Y110. Source instance areas are generated before clipping
and contact: `Brim.cpp:36-45` shifts by instance.shift_without_plate_offset(),
subtracts destination geometry and appends under the SAME PrintObject ID;
:573-581 repeats for all instances and associated masks/contact islands.
:636-689 applies common-frame no-brim/bed masks; subsequent contact retention
operates on these owner areas. :914-956 computes bounds across instance slices
and complete brim areas, then preserves owner footprints with plate shift.
:964-977 ordinary combine0 generates infill once for each complete owner area.
`Print.cpp:2794-2816` consumes all area contours for occupied skirt points.
`GCode.cpp:4486-4531` consumes emitted owner IDs once, origin0, not once per copy.

The public RED establishes actual distinct consequences: Native two brim blocks,
first collection spanning both outer copies; Ares three per-copy collections.
Target's initial skirt geometry already diverges; no-skirt witness subsequently
fails whole comparison at M73 placement. No internal Native vertex values or
proof that M73 alone is repaired by a geometry port are claimed. Any remaining
whole divergence after a faithful owner port remains a mandatory STOP.

## Current Rust producer and consumers

- `brim.rs::generate`: ordinary combine0 loses pre-infill area after first-
  instance bed clipping and converts a local template to paths.
- `brim/bed_area.rs::clip`: only first print_object_origin enters global clipping,
  then returns local polygons. Instance expansion needs to precede common-frame
  masking/contact, not duplicate already clipped paths.
- `skirt/occupied.rs::items`: model hull expands over every instance, but brim
  hull is appended only at first instance origin.
- `layers/brim_emit.rs::emit`: every copy's layer0 re-emits the same local plan;
  once-only state currently exists for global combine only.
- `brim/emit.rs`: covered bounds/hull are local; existing helpers/callers assume
  they describe one instance. The ordinary owner rewrite must make that frame
  explicit, retain per-owner occupied areas and avoid changing global-combine
  behavior. Do not re-use global_brim_done for each ordinary owner.

## Necessary destination addition before production

SPEC's explicit Rust destination list omitted this existing consumer:
`crates/ares-core/src/project_slice/gcode_emit/footprint/first_layer.rs`.

`first_layer_bounds` and `first_layer_convex_hull` currently shift each brim's
bounds/hull at first instance; `first_layer_object_center` instead loops over
EVERY copy and re-shifts the SAME local brim bounds. That is coherent for the
current per-instance template, but not for the requested whole-owner collection.
For object2, a complete owner hull expressed relative to first center80 spans
roughly -12..72 in X. Re-shifting it at both80 and140 would invent an extra
right-hand extent to212 rather than Native's actual extent152. A plate-frame
owner hull would likewise be shifted twice. This is a static consumer-frame
contradiction, not an external-input defensive check.

Narrow amendment: allow adapting ONLY the above existing footprint
bounds/hull consumers to recognize the ordinary multi-instance owner collection,
consume its already-expanded occupied bounds/hull ONCE in the correct frame,
and retain existing singleton/global-combine behavior. This implements exactly
the already-cited `Brim.cpp:914-956` owner bounds/footprint boundary, not a new
milestone, estimator feature, pipeline stage or guard. No M73/estimator changes.
No new geometry, public API, validation policy or fallback. Other destinations
remain exactly those in the SPEC; per-object multi-copy skirt guard stays.

A minimal representation may retain whole-owner footprint relative to first
instance and separate plate-frame emission paths; whichever private form is
chosen, both the skirt and first-layer consumers must respect the same frame
and owner identity. It cannot preserve old per-copy bounds multiplication while
truthfully claiming complete owner bounds. Coordinate snapping or a copy-count
workaround is not a port. Source/global-combine RED must remain green.

No production implementation started: per SPEC's explicit rule that changes
outside named destinations require STOP/amendment. This single caller-file
addition precedes the owner-area/collection port and
exact-hash public target/no-skirt/width0/global-combine checks. Residual strict
whole divergence still means STOP/report.
