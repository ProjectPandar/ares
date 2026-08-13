# Task 22O.76 — CrossHatch fill entities

## Goal

Port pinned OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1` source
`Fill/Fill.cpp:1213-1224,1234-1357` plus
`Fill/FillBase.cpp:133-184` for the first KSR-active `Layer::make_fills`
vertical slice: complete grouped CrossHatch geometry becomes owned extrusion
entities.

## Interface and output

```rust
pub(in crate::project_slice) fn generate_crosshatch_layer(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
) -> Result<LayerFillEntities, SliceError>;
```

`LayerFillEntities` owns ordered collections. Each collection owns ordered paths;
each path owns a fixed-coordinate `Polyline`, grouped `ExtrusionRole`, exact
`mm3_per_mm`, width, height, and source flags required by later ordering.
Empty generated polylines produce no collection.

## Behavior

Call `group_fills` exactly once. Iterate groups and authoritative ExPolygons in
order. A configured CrossHatch group uses aligned layer `print_z`, grouped
spacing/overlap/angle/multiline/anchor values, and source
`float(0.01 * density)`. Convert each returned polyline to an extrusion path.
For KSR sparse Internal CrossHatch, `using_internal_flow` is true, so
`FillBase.cpp:148-163` keeps grouped Flow `mm3_per_mm` and width unchanged;
height is grouped Flow height. Preserve the grouped extrusion role.

A reached non-CrossHatch pattern contributes no output to this explicitly
CrossHatch-only result; do not run it through CrossHatch, fallback to generic
Ares infill, or hardcode layer/fixture output. Propagate
full grouping errors unchanged and map filler Clipper errors consistently.
The prepared graph remains unchanged on success and failure.

## Tests

Tests call only the graph-native layer seam. Required TDD witnesses:

1. a focused CrossHatch graph freezes collection/path/point order and all path
   metadata bits;
2. empty generated geometry omits a collection;
3. a reached non-CrossHatch group emits no CrossHatch fallback;
4. natural grouping and filler range errors are atomic;
5. repeatability and graph immutability;
6. selected real KSR sparse layers plus an aggregate all-layer checkpoint,
   independently compared with pinned Orca output.

Tests live in separate ordinary modules. Every changed/new Rust file stays below
400 LOC. Do not use `include!` or `include_bytes!` for Rust source splitting.

## Deferred

Adjusted solid/bridge flow, bridge density, elephant-foot density, no-overlap
gap fill, all other filler classes, thin fills, `Layer::make_fills` lifecycle,
ironing, ordering, extrusion sequencing, motion, G-code, CLI completion, and
normalized golden parity.
