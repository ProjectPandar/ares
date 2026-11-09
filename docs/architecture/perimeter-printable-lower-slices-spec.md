# Printable lower slices for perimeter generation

## Upstream boundary

Source revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

The owning boundary is `src/libslic3r/LayerRegion.cpp:163-173`,
`LayerRegion::make_perimeters`, and
`src/libslic3r/PerimeterGenerator.cpp:3189-3202`,
`PerimeterGenerator::printable_slices`.

`LayerRegion::make_perimeters` takes the preceding physical layer's complete
`Layer::lslices`, calls `g.printable_slices`, and assigns the result to
`g.lower_slices` before either generator runs. This is not a mutation of
`Layer::lslices`. An absent preceding layer leaves the pointer null; a present
layer whose printable result is empty still supplies a non-null pointer.

## Numerical contract

Let `d` be the minimum of all nozzle diameters in the print configuration,
including nozzles not selected for the current region. Start with `min_width = d / 3`.

For object configuration `wall_generator = Arachne`, obtain the physical-unit
`min_feature_size` from `Arachne::make_paths_params`
(`src/libslic3r/Arachne/WallToolPaths.cpp:47-56`). Its assignment computes
`min_feature_size_percent * 0.01 * d` in double, then stores it in the float
`WallToolPathsParams::min_feature_size` field (`Arachne/WallToolPaths.hpp`).
The field starts at zero; a zero percent leaves that default unchanged.

Promote that float value to double for `min_width` unless print configuration
enables spiral mode; in that case use `min(d / 3, double(min_feature_size))`.
The object generator choice, not the current record's classic/Arachne dispatch,
controls this branch. A spiral record may dispatch classic while its object
configuration is Arachne.

When `min_width > EPSILON`, return
`opening_ex(slices, float(scale_(min_width / 2.)))`; otherwise return the input
geometry without an opening. After the float-field promotion, retain double
arithmetic through division and scaling, then the explicit float conversion
at the opening radius. Do not derive this radius from the integer
`ArachneWallParams::min_feature_size`, or omit the source float-field rounding.

`src/libslic3r/ClipperUtils.hpp:447-448`, `opening_ex`, delegates to
`offset2_ex` with negative radius followed by positive radius and the source
join and miter defaults. Retain that operation and its hole topology; replacing
it with a polygon-area cutoff or a centerline check is not equivalent.

## Rust destination

The Rust function belongs in
`crates/ares-core/src/project_slice/perimeters/printable_slices.rs` as the
rewrite of `PerimeterGenerator::printable_slices`. It uses the existing geometry
opening implementation, object generator configuration, print spiral setting,
nozzle vector, and coordinate scale. It has no filesystem or platform runtime.
`classic.rs::finish_classic_prelude` and
`classic/preflight.rs::validate_project` supply this configuration to the
common perimeter context without changing option decoding or defaults.

`perimeters/classic/prelude.rs::prepare_record` is the common perimeter-generator
entry for classic and Arachne records. It must compute the printable lower
geometry before counterbore geometry and lower polygon series. Keep this
record-local geometry in `classic/types.rs::ClassicPreludeRecord`, preserving
the distinction between no preceding layer and an empty printable result.

Route the result to the consumers of the upstream generator's `lower_slices`:

- `classic/prelude.rs` lower support polygons and lower flow sample series;
- `classic/counterbore.rs::process` and `classic/top_split.rs::prepare_record`;
- classic perimeter steep-overhang orientation in `classic/perimeter_append.rs`,
  with a record-index accessor in `classic/traversal.rs`;
- `arachne/materialize.rs::materialize_record`, including its fuzzy, top-surface,
  steep-overhang, and extra-perimeter inputs.

The current common prelude is a temporary compatibility shell for the upstream
`PerimeterGenerator` transaction, not an independent slicing pipeline. Its raw
lower-slice access must not stand in for `printable_slices` at this boundary.
The existing Arachne conversion, path role assignment, endpoint chaining, and
entity transport remain consumers rather than new implementations.

## Downstream behavior

`PerimeterGenerator.cpp:2664-2670`, `process_arachne`, grows printable lower
slices by half the current outer-wall nozzle diameter for overhang support.
That nozzle selection is distinct from the all-nozzle minimum above.

`PerimeterGenerator.cpp:407-608`, `traverse_extrusions`, clips each Arachne
subject against that support geometry. Intersection paths retain their ordinary
role; difference paths become `erOverhangPerimeter`. Existing endpoint selection
and `ExtrusionMultiPath` construction then determine direction and emission.
Do not compensate for missing support geometry by forcing a role or direction.

## Unchanged boundaries

Keep original physical-layer slices for slicing, seam placement, travel
boundaries, and the G-code extrusion quality estimator. In particular,
`GCode.cpp:6970-6973` prepares that estimator from object layers, not from
`PerimeterGenerator::printable_slices`. Do not globally replace the existing
raw-layer accessors or current/upper-layer geometry.

This slice includes both perimeter generators and the spiral minimum rule.
It excludes changes to Arachne wall generation, Clipper algorithms, fuzzy
sampling, seam selection, travel planning, curl estimation, cooling, formatter
arithmetic, progress calculation, retraction placement, and configuration schema.
Those consumers receive the geometry or paths from their existing source
boundaries; no alternate pipeline or legacy path is introduced.
