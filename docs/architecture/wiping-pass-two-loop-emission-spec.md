# Spec: process_layer print_wipe_extrusions two-loop emission order for wiping-pass objects

Slice status: DRAFT (awaiting review)
Base: `4b4798ecff1652a94a02bdfc866b443807311d9d`
Upstream boundary: `OrcaSlicer/src/libslic3r/GCode.cpp:5318-5360` —
`GCode::process_layer`'s `print_wipe_extrusions` two-loop structure
(`:5326-5360`), with `generate_object_skirt_group` called inside the
wiping loop (`:5336-5338` -> `:4465-4485`), object labels `:5349` and
`:5471-5475`, and `"; PURGING FINISHED"` opening the normal loop
(`:5327-5328`). Support footprints enter the per-object skirt grouping
at `Print.cpp:2865-2899`.

## Cause (discovery-approved)

Upstream, for each extruder/layer with `is_anything_overridden`:

1. **Loop 1 (wiping pass)** — per instance: per-object skirt
   (`generate_object_skirt_group`, `:5336-5338`), brim, `"; printing
   object"` (`:5349`), overridden extrusions, `"; stop printing
   object"` (`:5471-5475`).
2. `"; PURGING FINISHED"` (`:5327-5328`) opens loop 2.
3. **Loop 2 (normal pass)** — per instance: labels again, normal
   extrusions.

Ares (`gcode_emit/layers/tool_buckets.rs`, `tool_entry.rs`) flushes the
object label/PURGING triple at the bucket head (before the layer-change
retract/travel) and emits per-object skirts on `Pass::Normal` only.
The flush_into_infill witness never exposed this because its skirts
exist on the first layer only; tree support (`pf-supb`) regenerates
the skirt every layer from support footprints, exposing both
sub-symptoms (label/PURGING triple position; Skirt block displacement,
302 normalized lines).

## Included

- Mirror the two-loop emission order in the Ares wiping-pass bucket
  structure: the wiping pass owns the per-object skirt (when
  `is_anything_overridden`) and its `printing/stop` labels;
  `"; PURGING FINISHED"` opens the normal pass.
- Public RED: single leg on the ACTIVE support fixture
  (`pf-supb`: overhang mesh, tree support, prime tower, 2 filaments,
  `skirt_type=perobject`, `flush_into_support=1`; native rc=0, 23
  `;TYPE:Support` + 8 interface blocks inside wiping windows) plus a
  native ACTIVE assertion (Support extrusion inside a wiping window).
  Today Ares rejects the input via the `flush_into_support` guard
  term, so the leg FAILS before the GREEN.

## Deferred

- `flush_into_objects` per-object skirt (tool-change/print ordering
  residual) — separate slice.
- No skirt geometry work: the tower-obstacle grouping and support
  footprint hulls are already at parity; only emission order changes.

## Exit criteria (blast radius: all wiping-pass output)

- RED FAIL at the RED hash, PASS at the GREEN hash (canonical
  official+seeded, targeted, CPU7,8).
- GREEN gate includes the FULL tower/wipe/toolchange/skirt families
  both modes at the exact GREEN hash.
- fmt/clippy/wasm clean; no file over 400 LOC.
- If GREEN shows the skirt-ownership and label-order fixes are
  independent, split into separate slices per the coordinator's rule.
