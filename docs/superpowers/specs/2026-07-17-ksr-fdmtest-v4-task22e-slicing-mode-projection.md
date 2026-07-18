# Task 22E: Project Slicing-Mode Projection and Raw Polygon Policy

## Status and objective

This specification is a draft. Production and test implementation may begin
only after independent upstream/spec, independent Ares/plan, and direct
default-model reviewers approve the exact frozen specification and plan bytes.

Task 22E is the next bounded source-rewrite package in the persistent
`ksr_fdmtest_v4` parity program. Released Task 22D commit
`a06deedecdc1e7b21b16c38e1d9bd28893eaf0fc` produces ordered closed integer
polygons for every object, volume, and planned layer. Task 22E ports the
adjacent Orca slicing-mode boundary in two deliberately distinct parts:

1. the four-mode raw-polygon policy used by direct `slice_mesh`; and
2. the real project `slice_mesh_ex` adaptation that derives mode per object,
   volume, and layer from resolved 3MF Options, applies only the raw-stage
   portion now, and retains the original mode for a later ExPolygon stage.

The distinction is required for correctness. In the real project path,
`PositiveLargestContour` is converted to `Positive` before raw polygon
processing. All contours are retained and oriented counter-clockwise at this
stage. The largest ExPolygon is selected only after Clipper conversion, which
is outside Task 22E. The implementation must not discard project contours
early or claim that raw largest-polygon selection is project-path parity.

The public project API still returns `SliceError::ProjectSlicingIncomplete`
after traversing the new owned result. This package does not create
ExPolygons, infer holes, apply fill rules or closing offsets, create regions or
surfaces, generate toolpaths, or emit G-code.

The committed fixture resolves `slicing_mode=regular`, `spiral_mode=0`,
`bottom_shell_layers=3`, and `bottom_shell_thickness=0`. Task 22E is therefore
an exact geometry no-op for all 460 fixture layers and must preserve the Task
22D polygon counts, point counts, encodings, and hashes.

## Fixed upstream rewrite boundary

All source citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored Orca checkout may
have another HEAD. Citations are verified with `git show <sha>:<path>` without
checking it out or changing it.

- `src/libslic3r/PrintConfig.hpp:162-170` defines the three external
  `SlicingMode` values.
- `PrintConfig.cpp:307-312` defines their exact serialized keys: `regular`,
  `even_odd`, and `close_holes`.
- `PrintConfig.hpp:916-947` owns `slicing_mode` in `PrintObjectConfig`;
  `PrintConfig.cpp:6030-6042` defines its enum values and Regular default.
- `PrintConfig.hpp:1073-1080` owns per-region `bottom_shell_layers` and
  `bottom_shell_thickness`; `PrintConfig.cpp:1156-1176` defines their
  nonnegative ranges and defaults.
- `PrintConfig.hpp:1558-1574` owns global `spiral_mode` in the print config;
  `PrintConfig.cpp:5829-5835` defines its false default.
- `src/libslic3r/TriangleMeshSlicer.hpp:11-33` defines the four internal
  `MeshSlicingParams::SlicingMode` values, the normal-below threshold, and
  `mode_below`.
- `src/libslic3r/PrintObjectSlice.cpp:76-95` proves that project slicing calls
  `slice_mesh_ex`; it also contains the already-adjacent mirrored-transform
  winding context, which Task 22E does not alter.
- `PrintObjectSlice.cpp:138-142,148-225` defines sliceable volume types, calls
  the upstream volume-ID sort, maps external modes to internal modes, and
  defines the model-part-only spiral gate, per-volume region lookup, bottom
  threshold, and multi-range invariant.
- `src/libslic3r/Model.hpp:1227-1230` defines that upstream ordering as
  ascending `ModelVolume::id()`. Task 22E does not port this order.
- `PrintObjectSlice.cpp:1149-1185` is the project call-chain context.
- `src/libslic3r/Layer.hpp:334-341` converts ordered `Layer::slice_z` values
  to `float` before slicing.
- `src/libslic3r/libslic3r.h:48-52` defines `EPSILON = 1e-4`.
- `src/libslic3r/Polygon.cpp:52-87` defines signed area and calls the winding
  and reversal operations.
- `src/libslic3r/MultiPoint.hpp:34` defines reversal as a complete
  `std::reverse` of the point vector.
- `deps_src/clipper/clipper.hpp:199-200` defines orientation as
  `Area(poly) >= 0`, including zero-area orientation.
- `src/libslic3r/TriangleMeshSlicer.cpp:1483-1532` applies all four modes in
  the direct raw `slice_mesh` path.
- `TriangleMeshSlicer.cpp:1864-1902` shows that direct `slice_mesh` returns
  raw polygons after that policy.
- `TriangleMeshSlicer.cpp:2003-2049` is the real project `slice_mesh_ex`
  boundary: it adapts `PositiveLargestContour` to `Positive` before raw
  slicing, then later chooses Clipper fill rules and the largest ExPolygon.
- `src/libslic3r/ExPolygon.cpp:532-548` defines that deferred post-union
  largest-contour selection.
- `src/libslic3r/LayerRegion.cpp:87-94` is the downstream spiral threshold
  synchronization constraint.

The Task 22E semantic stop is the owned, per-layer raw polygon state plus its
retained original internal mode immediately before `make_expolygons` at
`TriangleMeshSlicer.cpp:2029-2037`.

## Two source APIs that must not be conflated

### Direct raw `slice_mesh`

The private core policy supports every internal mode exactly as the source
`make_loops` layer policy does:

- `Regular`: preserve all polygons, polygon order, point order, and winding;
- `EvenOdd`: identical to Regular at this raw boundary;
- `Positive`: preserve all polygons and polygon order, reversing each
  clockwise point vector to make it CCW;
- `PositiveLargestContour`: select one polygon by strictly greatest absolute
  signed area, keep the first polygon on equal absolute area, and reverse the
  selected polygon only when its signed area is negative.

This helper is a source-complete direct raw policy. It is not itself the
project adapter.

### Real project `slice_mesh_ex`

The project adapter first determines the original per-layer internal mode.
Before calling the raw helper it maps `PositiveLargestContour` to `Positive`,
exactly as `TriangleMeshSlicer.cpp:2011-2016` does. It stores the original
`PositiveLargestContour` mode on the resulting layer so that the future
ExPolygon slice can select the largest result at the correct boundary.

Therefore a spiral project layer above its threshold retains every raw polygon
in Task 22E, makes every polygon CCW, and records
`PositiveLargestContour`. It does not reduce the layer to one raw polygon.

## Existing Ares input seam

Task 22E consumes only released, typed state:

- `mesh_slicer::LoopedLayer`: ordered, closed integer polygons from Task 22D;
- `project_slice::looped_intersections::LoopedPrintObject`: complete plan,
  object order, volume ordinal/type, and layer-slot ownership;
- `ResolvedProjectObject.object.slicing_mode`: process base plus object-level
  3MF overlay already resolved into `ObjectOptions`;
- `resolved.views.full.process.print.spiral_mode`: the final normalized typed
  print Option, not unnormalized source JSON;
- `ResolvedModelPartCandidate.region`: process/object/layer/volume region
  precedence already resolved into typed `RegionOptions`;
- `PlannedLayer.slice_z`: the ordered source value used to form Orca's float
  Z vector.

The new path may not call or adapt the legacy f64 STL
`planning`/`segments`/`contours`/`pipeline` path. It may not inspect the
reference G-code, branch on fixture names or coordinates, or parse Orca source
at test runtime.

## Source-volume association

`ProjectedVolume` already owns two different identities:

- `source_volume_index`: the actual index in `ProjectObject::volumes()`;
- `volume_ordinal`: the one-based ordinal of nonempty volume occurrences.

The released Raw, Chained, and Looped wrappers accidentally stop carrying the
first identity. Task 22E must propagate `source_volume_index` through all three
wrappers without changing ordinal semantics or order.

The ordinal cannot be used to reconstruct the source index. Empty volumes are
not ordinal occurrences; support blockers and enforcers may consume ordinals
and then be filtered out; existing Task 22B tests deliberately freeze ordinal
gaps. `ResolvedModelPartCandidate.volume_index` is the real source index.
Per-volume region lookup must match that field exactly.

No 3MF leaf ID, occurrence ordinal, vector position after filtering, or
fixture-specific assumption may replace this association.

Task 22E preserves the released Ares source/BFS occurrence order. Orca instead
sorts its selected model-volume pointers by ascending `ModelVolume::id()`
before slicing. This package ports mode behavior per retained volume, not that
upstream cross-volume order, and makes no claim of volume-order parity.

## External and internal mode mapping

The base mode is mapped exhaustively from the resolved object Option:

```text
ProcessSlicingMode::Regular    -> SlicingMode::Regular
ProcessSlicingMode::EvenOdd    -> SlicingMode::EvenOdd
ProcessSlicingMode::CloseHoles -> SlicingMode::Positive
```

`PositiveLargestContour` has no serialized `slicing_mode` key. It is derived
only for spiral model-part volumes.

For every print object and retained volume:

1. set `base_mode` and `mode_below` to the mapped object mode;
2. if resolved `spiral_mode` is false, use `base_mode` for every layer;
3. if spiral is true but the volume type is NegativeVolume or
   ParameterModifier, still use `base_mode` for every layer;
4. if spiral is true and the volume is ModelPart, match its
   `source_volume_index` to its resolved model-part region, derive
   `PositiveLargestContour` above the bottom threshold, and keep `base_mode`
   below it.

Task 22B already rejects explicit `layer_config_ranges` before intersections.
Task 22E preserves that public boundary and therefore consumes exactly one
resolved all-height candidate. It does not add a multi-range fallback or
expand range support.

Mode application remains per volume and per layer. Two model-part volumes may
have different resolved bottom policies. No object-level aggregation may pick
one global largest polygon or one global threshold.

## Bottom threshold and float semantics

For a spiral model-part volume, derive:

```text
threshold = usize(bottom_shell_layers)
boundary = bottom_shell_thickness - 1e-4

while threshold < planned_layers.len
  and f64(f32(planned_layers[threshold].slice_z)) < boundary
    threshold += 1
```

The per-layer original mode is selected with the strict source condition:

```text
layer_index < threshold ? base_mode : PositiveLargestContour
```

Required consequences:

- the comparison uses `slice_z`, never `print_z`, height, or layer ID;
- every `slice_z` is cast to `f32` and then promoted for the comparison;
- equality with `bottom_shell_thickness - 1e-4` stops extension;
- `bottom_shell_layers == 0` may still extend through thickness;
- a layer count smaller than `bottom_shell_layers` is not clamped; all
  existing layers remain below mode;
- f32 rounding is observable and must be tested;
- `mode_below` remains the mapped base mode, including EvenOdd or Positive.

The 3MF is an external input boundary. A negative `bottom_shell_layers`, or a
negative/nonfinite `bottom_shell_thickness` when spiral model-part policy
consumes it, returns the existing keyed form
`SliceError::InvalidInput("invalid Orca option <key>")`. Values are not
clamped, wrapped, or replaced by defaults. Planned `slice_z` finiteness is an
already-proved internal invariant and is not revalidated here.

## Raw polygon arithmetic and point order

Signed area follows `Polygon.cpp:52-63`:

```text
0.5 * sum(cross(previous, current))
```

Coordinates convert to `f64` only for the source-equivalent area calculation.
Positive area is CCW. A zero-area polygon is treated as already CCW by the
raw Positive policy. Reversal reverses the complete point vector in place;
the old final point becomes the new first point. It does not rotate to retain
the old start.

For direct raw `PositiveLargestContour`, initialize the greatest absolute area
to zero and replace the selected index only on strict `>`. Equal absolute
areas retain the earlier polygon. Empty input remains empty. A nonempty set in
which every polygon has zero area violates the same internal nondegenerate-loop
invariant as Orca's assertion; the private helper must not invent a fallback.

Within each retained Ares volume, all other layer, polygon, and point ordering
is unchanged. The implementation is sequential and deterministic. Native TBB
scheduling is not ported to WASM. Task 22E does not change Ares volume order or
claim that it matches Orca's ID-sorted order.

## Normative executable test vectors

These vectors are part of the implementation contract, not illustrative
examples. Raw policy tests use these exact ordered point vectors:

```text
A = [(0,0), (4,0), (0,3)]                  area +6
B = [(10,0), (10,2), (14,2), (14,0)]       area -8
C = [(20,0), (20,5), (26,5), (26,0)]       area -30
```

Required raw results are:

- Regular and EvenOdd return `[A, B, C]` byte-for-byte and in that order;
- Positive returns A unchanged, then
  `[(14,0), (14,2), (10,2), (10,0)]`, then
  `[(26,0), (26,5), (20,5), (20,0)]`;
- direct raw PositiveLargestContour returns only
  `[(26,0), (26,5), (20,5), (20,0)]`.

The strict tie vector is:

```text
first  = [(0,0), (0,2), (2,2), (2,0)]       area -4
second = [(10,0), (12,0), (12,2), (10,2)]   area +4
```

PositiveLargestContour must retain the first and output exactly
`[(2,0), (2,2), (0,2), (0,0)]`. Empty input remains empty. A single CCW
polygon remains exact; a single CW polygon is completely reversed. A nonempty
all-zero-area set triggers the internal invariant instead of choosing a
fallback polygon.

Threshold tests use `slice_z = [0.10, 0.30, 0.50, 0.70]` and freeze:

```text
(bottom_shell_layers, bottom_shell_thickness) -> threshold
(1, 0.0)   -> 1
(1, 0.61)  -> 3
(5, 0.0)   -> 5
(0, 0.0)   -> 0
```

The strict-equality vector fixes `bottom_shell_thickness=0.5001`, hence the
exactly representable `boundary=0.5`, and uses
`slice_z_f64=[0.4999, 0.5, 0.5001]`. A zero starting threshold advances only
past the first value and stops when the second casts to f32 `0.5`. Separately,
the f32-rounding regression uses `slice_z_f64=0.5-1e-9`, which also casts to
f32 `0.5`; the strict comparison is false and the threshold does not advance.

Projection tests freeze all three external mappings. With spiral enabled,
EvenOdd and Positive remain their respective below modes; negative and
modifier volumes retain the base mode on every layer; two model parts with
different source-index regions derive different thresholds; and an
ordinal-gap object selects its region only by `source_volume_index`.

When spiral model-part policy consumes invalid external values,
`bottom_shell_layers=-1` and each of `bottom_shell_thickness=-0.1`, NaN,
positive infinity, and negative infinity return exactly:

```text
SliceError::InvalidInput("invalid Orca option bottom_shell_layers")
SliceError::InvalidInput("invalid Orca option bottom_shell_thickness")
```

## Rust destination and ownership

### Pure core policy

Add private `ares-core::mesh_slicer::slicing_mode` with:

- four-value crate-private `SlicingMode`;
- source-equivalent signed-area calculation;
- in-place application to one `LoopedLayer`.

`geometry::Polygon` gains only an in-place point-vector reversal operation.
`LoopedLayer` gains only the minimum parent-visible mutable polygon seam. No
public geometry API, dynamic option map, general polygon normalization layer,
or new crate is introduced.

### Project policy state

Add private `ares-core::project_slice::slicing_mode_intersections` owning:

- `SlicingModeLayer { mode, looped_layer }`;
- a volume wrapper carrying source index, ordinal, type, and ordered layers;
- a print-object wrapper carrying the complete plan and ordered volumes;
- the resolved Option-to-mode projection and raw-stage adaptation.

The wrapper consumes each previous stage once. It does not clone production
geometry or retain independently mutable copies. Test-only readers may expose
mode and geometry without changing production visibility.

The production path becomes:

```text
raw intersections
  -> chained intersections
  -> looped intersections
  -> Option-derived slicing-mode raw policy
  -> ProjectSlicingIncomplete
```

## Fixture and mutation acceptance

The committed fixture must still resolve exactly:

- one print object and one nonempty ModelPart volume;
- 460 ordered layers;
- `Regular` on every layer;
- 3,288 polygons and 116,472 points;
- face-order encoding length 2,190,993 and SHA-256
  `6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe`;
- semantic encoding SHA-256
  `7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd`;
- config-block length 49,004 and SHA-256
  `b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`;
- public `ProjectSlicingIncomplete` lifecycle.

Test-only archive mutations must prove provenance rather than hardcoded
behavior:

- changing only 3MF `slicing_mode` changes the retained mapped mode;
- `regular -> even_odd` leaves raw fixture polygons exact, while the normative
  CW vectors above prove `close_holes -> Positive` reversal without assuming
  that this fixture contains a clockwise contour;
- an object-level 3MF override wins over the process base;
- changing only 3MF `spiral_mode` and bottom Options changes the per-layer
  original mode at the strict threshold; the normative CW vectors separately
  prove project raw adaptation and orientation;
- the real project adapter retains all polygons for
  `PositiveLargestContour` layers;
- an ordinal-gap synthetic project selects the region by real source index.

Fixture mutations are test inputs only. Production may not recognize their
archive path, values, hashes, layer count, or geometry.

## Included behavior

Task 22E includes only:

- propagation of `source_volume_index` through Raw, Chained, and Looped state;
- the four internal direct raw modes;
- resolved external `slicing_mode` mapping;
- resolved global `spiral_mode` and per-model-part bottom policy;
- f32 `slice_z`, strict EPSILON threshold, and mode-below selection;
- project-path `PositiveLargestContour -> Positive` raw adaptation while
  retaining the original mode;
- exact no-op KSR acceptance and public lifecycle preservation.

## Explicitly deferred behavior

The following remain source-cited future slices:

- `make_expolygons`, Clipper union, `pftNonZero`, `pftEvenOdd`, and
  `pftPositive` behavior;
- post-union `keep_largest_contour_only` and ExPolygon hole ownership;
- `slice_closing_radius`, extra offset, resolution simplification, XY contour
  and hole compensation;
- support for explicit layer-config ranges in project mesh slicing;
- ascending `ModelVolume::id()` reordering from `Model.hpp:1227-1230`;
- slab slicing beginning at `TriangleMeshSlicer.cpp:2052`;
- negative/modifier boolean operations after independent volume slicing;
- regions, surfaces, perimeters, fill, supports, toolpaths, G-code assembly,
  metadata, post-processing, and normalized reference-G-code parity.

EvenOdd is complete only as an Option mapping and raw identity mode in Task
22E. No claim of EvenOdd geometric fill parity is allowed before the Clipper
slice.

## Structural, platform, and quality constraints

- Every Rust source file remains below 400 physical lines; split with real
  `mod` files before reaching 400.
- Tests live in separate test modules.
- Do not use `include!`, `include_bytes!`, or related macros to split source.
  Existing test fixture byte inclusion is data loading, not source splitting,
  and is not expanded by this package.
- Core code remains WASM-safe and contains no filesystem access, terminal/UI
  behavior, native threads, Rayon, platform branches, unsafe code, mutable
  globals, or native Clipper dependency.
- No legacy fallback, feature flag, compatibility shim, fixture-specific
  production branch, reference-G-code read, or Orca source-pinning executable
  test is introduced.
- Changes remain surgical to the listed source seam.

## Verification and review exit criteria

Task 22E is implemented only when all of the following are true:

1. genuine RED/GREEN evidence exists for identity propagation, pure mode
   policy, project Option projection, threshold boundaries, and fixture no-op;
2. every new test is named `task22e_*` and passes under Cargo Nextest;
3. Task 22A-D focused suites and all relevant mesh/project tests remain green;
4. full workspace Nextest, formatting, warning-denying Clippy, native checks,
   both WASM checks, release WASM build, and the real-3MF browser gate pass;
5. fixture hashes and the released Task 22D geometry hashes are unchanged;
6. structural audits prove the constraints above and an exact tracked manifest;
7. an independent reviewer validates requirement completeness, logic, edge
   cases, code quality, test coverage, and actual execution; the main thread
   fixes its list and the same reviewer rechecks until all six pass;
8. fresh whole-spec, whole-quality, and direct default-model implementation
   reviews approve the exact candidate;
9. architecture and roadmap documents record the project `slice_mesh_ex`
   distinction and the next exact upstream boundary;
10. one conventional commit is pushed normally and its exact SHA passes all
    five Tier-1 jobs.

Passing Task 22E is not completion of the original user-visible G-code parity
goal. The next bounded slice must begin at
`TriangleMeshSlicer.cpp:1738-1823,2003-2049` and the directly used Clipper/
ExPolygon helpers. It must also port the deferred `Model.hpp:1227-1230`
volume-ID order before any cross-volume combination, or freeze that ordering
as an immediately preceding package. Its exact included/deferred boundary is
frozen before implementation.
