# Fixed-order fill gap neighbors

## Upstream boundary

This slice ports `9ccc83fc644187f2cae19b74776edfb20db7fae8` into the Rust rewrite of `FillBase.cpp::insert_gap_fill`.

At OrcaSlicer `40ecc66f7`, `src/libslic3r/Fill/FillBase.cpp:255–282` computes the last original extrusion bordering each raw gap. Lines 295–300 constrain the insertion slots. `tests/fff_print/test_fill.cpp:2509–2558` exercises gaps touching later extrusions in fixed-order spiral inset, concentric, monotonic and monotonic-line fills on V and holed-cube geometry.

The Rust destination is `crates/ares-core/src/project_slice/fill_entities/gap_residual/insert.rs`. Its existing travel-minimizing insertion is an incomplete representation of the upstream function. Extend that function rather than introducing another fill-ordering pipeline or changing public APIs.

## Included behavior

- Flatten the original fill entities into XY line segments, retaining the original entity index for every segment. Every path in an extrusion loop belongs to that loop's entity index.
- Build the existing integer-coordinate line-distance tree once per fill, matching `AABBTreeLines::LinesDistancer<Line>`.
- For each raw gap with widths, use half its largest width plus the nominal flow's scaled spacing as the neighborhood radius.
- Sample every gap segment at both endpoints and uniformly between them, with `max(1, ceil(length / (0.5 * spacing)))` intervals. Interpolate in double precision and truncate the resulting XY coordinates to the integer coordinate type before querying the tree.
- Select the largest original entity index found by the radius queries. Preserve the absence of an owner when no segment is found or the upstream width vector is empty.
- Count original extrusions before each insertion slot, excluding previously inserted gaps. Only consider a slot after the last bordering original extrusion, when one exists.
- Preserve the existing strict travel-cost minimum, orientation choices, closed-gap vertex selection, appended-order comparison and variable-width materialization.

The nominal flow spacing is distinct from the adjusted fill-generator spacing. Querying only gap endpoints, looking only at extrusion endpoints, or counting previously inserted gaps as original extrusions changes the upstream rule and is not part of this slice.

## Deferred behavior

No changes to gap-area extraction, medial-axis geometry, variable-width conversion, sortable collections, pattern generation, spiral-inset wall generation, cooling, motion emission, retraction or configuration identity are included. Existing scaffolding outside the cited insertion function remains a temporary compatibility shell for its own upstream source boundaries.

## Failure modes

A gap inserted before a later bordering extrusion violates the source ordering. Missing intermediate samples or using a smaller width-derived radius can miss that later extrusion. Flattening loop paths as separate owners or counting inserted gaps can shift the allowed slot. Altering float interpolation, integer truncation, squared-distance admission or travel tie handling can change path orientation and emitted motion. Rebuilding the spatial tree for each gap or scanning every original line for every sample discards the upstream spatial-index performance boundary.

## Behavioral verification

Use the existing public byte-input slicing and CLI comparisons against original OrcaSlicer, retaining complete Native and Ares outputs outside the source tree:

- both sequential-origin label cases;
- the classic one-wall top-hole case;
- the sequential spiral-lift case;
- both XL curled-perimeter segmentation cases, including the disabled control;
- the custom G-code integer-formatting case on the holed cube.

These cases already exercise the changed insertion behavior through complete-output comparisons. Their gap strands move behind the neighboring fixed-order fill; total time, cooling feedrates, progress placement, subsequent travel and retraction can change as a consequence. Require exact existing comparisons in both Native modes without changing comparators, expected outputs or identity checks. Include adjacent residual-gap fill cases to retain sortable and fixed-order collection behavior. Do not substitute private-function tests for the public slicing boundary.

The implementation remains allocation-bounded to the fill's segment index, owner map and one owner entry per raw gap. It uses no filesystem, terminal, platform-specific runtime, unsafe code or concurrency-dependent ordering, preserving the browser WASM, Windows, macOS and Linux core boundary.
