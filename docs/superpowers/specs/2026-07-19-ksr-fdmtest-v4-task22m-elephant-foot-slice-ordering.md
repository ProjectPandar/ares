# Task 22M: Single-Region Elephant-Foot Compensation and Slice Ordering

## Status and objective

This specification is a draft. Production or tracked-test implementation may
begin only after the exact specification and implementation-plan bytes receive
independent fixed-source/specification and current-Ares/plan approval.

Task 22M is the next bounded source rewrite in the persistent
`ksr_fdmtest_v4` project-to-G-code parity program. Released Ares commit
`fcd2c5728f4c0529f28bfc43c636507d61e263d8` produces the complete Task 22L
post-conical-overhang stream. Task 22M ports OrcaSlicer's immediately adjacent
single-region elephant-foot-compensation path and `Layer::make_slices()`
ordering. It consumes Task 22L state plus typed effective Options resolved only
from the supplied 3MF.

This is the complete enabled single-region geometry unit. It is not a
disabled-only gate, constant inset, rectangle-only approximation, full-scan
production shortcut, or fixture branch. It supports arbitrary valid
ExPolygons, holes, both coordinate scales, per-layer Flow, the source f32 ramp,
the source spatial grid, variable inner offset, two-pass NonZero union, and
deterministic island ordering. It remains platform-neutral on WASM, Windows,
macOS, and Linux.

Task 22M deliberately rejects nonzero XY compensation and valid multi-region
layer slicing before mutation. Those paths are materially different upstream
algorithms and remain later source-cited slices. Painted/MMU/fuzzy/interlocking
segmentation is also deferred; the released earlier capability boundaries
remain authoritative. Task 22M stops before surface classification,
perimeters, fill, supports, extrusion paths, G-code assembly, metadata, or
post-processing. The public project API executes Task 22M and continues to
return `SliceError::ProjectSlicingIncomplete`.

## Fixed identities and source blobs

The fixed Ares baseline is commit
`fcd2c5728f4c0529f28bfc43c636507d61e263d8`, tree
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`. Exact-SHA Tier-1 run
`29718329104` passed format, Ubuntu/Linux, Windows, macOS, and WASM/browser.

All upstream citations refer only to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored upstream checkout and
ignored C++ oracle are review evidence only. Tracked tests never inspect them.

Fixed source blobs are:

- `src/libslic3r/PrintObjectSlice.cpp`,
  `07eb885eda83a495001467c22c0452dfc36e55c2`;
- `src/libslic3r/Layer.cpp`,
  `5bdc156d0172ec19894b630cc70d73b5aef8f82d`;
- `src/libslic3r/Layer.hpp`,
  `cb2e6c7c1a166a028ac8fceffaf9f42f3c2426b0`;
- `src/libslic3r/LayerRegion.cpp`,
  `22e0a26898c6fe07ad8ebd35de303b5911d84f4b`;
- `src/libslic3r/PrintRegion.cpp`,
  `5c08de8b36d469b583425524c9948b92117236e8`;
- `src/libslic3r/Flow.cpp`,
  `42fd6e8ea132f8012217c38db7d3b7a36e2bbc76`;
- `src/libslic3r/Flow.hpp`,
  `79cb1b324d6343e41ed11a5f2984f52c0ea61412`;
- `src/libslic3r/ElephantFootCompensation.cpp`,
  `0adff1ba4e578d733f1575b1a0e3f8def6989e90`;
- `src/libslic3r/ElephantFootCompensation.hpp`,
  `596a3e9588e69ffba59c107fff7b36393fe4e64e`;
- `src/libslic3r/EdgeGrid.cpp`,
  `93bf7d48ec87da91cde5800a7c1d099ed4337c0d`;
- `src/libslic3r/EdgeGrid.hpp`,
  `6bbed1e9fa5375ac9bb221407460c8eb3994a5ac`;
- `src/libslic3r/ClipperUtils.cpp`,
  `2f97e08f536e93c5fd27b4614980072285d2ce22`;
- `src/libslic3r/ClipperUtils.hpp`,
  `9c2fa239263c0cb097a4b4c3db823821615bd7c7`;
- `src/libslic3r/ExPolygon.cpp`,
  `185e92508449a425064b26690e3d74d06a16fda8`;
- `src/libslic3r/ExPolygon.hpp`,
  `ce7ebe892f64b3d4e2e9fb0c85bd77b99e889d54`;
- `src/libslic3r/Polygon.cpp`,
  `32b4d062f1b8f898866a0e0e55672dcd5f54ac89`;
- `src/libslic3r/Polygon.hpp`,
  `7d996055e5d9403f871071ef82baa140c03492b5`;
- `src/libslic3r/MultiPoint.cpp`,
  `694d3ea9d0b59d81181f05b7bbd4fb617751bb6d`;
- `src/libslic3r/MultiPoint.hpp`,
  `de386b501cbb21056b068c5e456d305afa9089b4`;
- `src/libslic3r/ShortestPath.cpp`,
  `e2fc258e316c8e9ded30a3003ee3d534399b8a1b`;
- `src/libslic3r/ShortestPath.hpp`,
  `88389b20dede43fe7d6413e8d55c0599e1499e71`;
- `src/libslic3r/KDTreeIndirect.hpp`,
  `37c10827b18ad4313e51630138407e45c7c4e48f`;
- `src/libslic3r/MutablePriorityQueue.hpp`,
  `2565f417f8499155599d33e434f4050eeb921b23`;
- `src/libslic3r/Point.hpp`,
  `039f361eaa18db9c6e7d2c35d1c61af78bcad51b`;
- `src/libslic3r/Line.hpp`,
  `d8240702b24168c3e7efa90971ac2babab1dffaf`;
- `src/libslic3r/Utils.hpp`,
  `f1bdd897126049dc195107ea1754ac15587010d2`;
- `src/libslic3r/Geometry.hpp`,
  `e610a9ed4f48cde31a0e794643a516b72cdaa424`;
- `src/libslic3r/Config.hpp`,
  `5fedaa9b288e206b2dbf454927479c745d20e45d`;
- `src/libslic3r/Surface.hpp`,
  `b63c283251d63154a3a7071694c87b637ec7dff7`;
- `src/libslic3r/SurfaceCollection.hpp`,
  `1895516aa2eb1fa30be3cf63bb211f7db420f3af`;
- `src/libslic3r/BoundingBox.hpp`,
  `26b840ade672b2489a7878f0742d62057837788f`;
- `src/libslic3r/BoundingBox.cpp`,
  `a2a510b64cf8eed685f527a69b51750bb3dfdc9c`;
- `src/libslic3r/libslic3r.h`,
  `f4291d36df8175c700fa9374c5b5c07e6880e706`;
- `deps_src/clipper/clipper.hpp`,
  `06637effce040fa7d87c368437cb32398f19ee92`;
- `deps_src/clipper/clipper.cpp`,
  `1f16446ac8da1f0b9c802d8a9dee33f766919f6b`.

## Exact upstream rewrite boundary

The owning boundary is:

- `PrintObjectSlice.cpp:1246-1269` for the compensation gate, scaled f32
  value, backup length, per-layer visit, and linear ramp;
- `PrintObjectSlice.cpp:1270-1276,1287-1292` for the single-region enabled
  branch, uncompensated backup, kernel invocation, two-pass union, and Internal
  surface replacement;
- `PrintObjectSlice.cpp:1364-1387` for `make_slices()` on every retained layer
  and restoration of ordered uncompensated `lslices`;
- `Layer.cpp:38-66` and `Layer.hpp:123-178` for single-region island extraction,
  `lslices`, and deterministic `chain_points` ordering;
- `LayerRegion.cpp:21-29` and `PrintRegion.cpp:6-54` for actual layer height,
  first-layer identity, width selection, 1-based logical outer-wall filament
  selection, and its direct unmapped nozzle-vector lookup;
- `Flow.cpp:20-35,129-144,200-206`, `Flow.hpp`, and
  `Config.hpp:624-628,1259-1285` for auto width, raw float-or-percent
  conversion, f32 width/height/spacing, and element-zero fallback;
- `ElephantFootCompensation.cpp:20-28,233-447,465-532,544-644` for
  `ResampledPoint`, filtered contour distance including the strict
  `search_radius + SCALED_EPSILON` boundary, resampling, banded smoothing, tiny
  gate, variable offset, and the only source-defined result-count fallback;
- `EdgeGrid.cpp:28-334` and `EdgeGrid.hpp:15-356` for contour segment topology,
  complete production grid creation, padding, all line-raster quadrants and
  boundary/corner cases, raster/count/prefix/fill, box traversal, and segment
  access;
- `ClipperUtils.cpp:169-207,303-344,634-668,737-739,813-816,1019-1031,
  1065-1248,1378-1421` and `ClipperUtils.hpp:34,183-222,548-550` for ordered
  PolyTree conversion, provider order, StrictlySimple normalization followed by
  two-pass NonZero union, variable miter inner offset and shortest-edge factor,
  contour/hole repair, and hole subtraction;
- `ExPolygon.cpp:50-56,229-254`, `Polygon.cpp:52-68`,
  `MultiPoint.cpp:164-230`, and `Line.hpp:41-77,155-188` for signed area and
  closed-contour Douglas-Peucker simplification;
- `ShortestPath.cpp:83-419,1000-1011,1106-1115`, `KDTreeIndirect.hpp`,
  `MutablePriorityQueue.hpp`, and `Utils.hpp:305-408` for exact greedy point
  chaining, deterministic tie/update behavior, power-of-two growth, and cyclic
  helpers;
- `Surface.hpp:9-47` and `SurfaceCollection.hpp:11-78` for Internal identity
  and replacement metadata;
- `BoundingBox.hpp:13-110,208-230`, `BoundingBox.cpp:172-179`,
  `Point.hpp`, `Geometry.hpp`, and `libslic3r.h:46-96,124-125,299-303` for
  bounds, signed coordinates, scale, epsilon, generic interpolation, and typed
  unscale arithmetic.

The upstream parallel scheduler and cancellation checks are classified but
deferred. Ares has no public AbortSignal contract, and deterministic sequential
layer execution has identical uncancelled output. Painted segmentation,
interlocking, nonzero XY compensation, multi-region compensation and
`make_slices`, later raw-slice backup, classification, perimeter/fill/support,
and G-code remain deferred. The superseded `#if 0` smoother and old
`contour_distance` implementation are not ported.

## KSR activation and branch inventory

The committed KSR Task 22L input contains one print object and one occurrence,
460 retained layers, exactly one region with ID zero per layer, 2,890 Internal
surface ExPolygons, 58,902 points, and 395 holes. Layer zero contains six
ExPolygons, 720 points, and six holes. Layer 459 contains nine ExPolygons and
36 points.

Its effective Task 22M values are:

- `elefant_foot_compensation = 0.15` mm;
- `elefant_foot_compensation_layers = 1`;
- `raft_layers = 0`;
- `xy_hole_compensation = 0` and `xy_contour_compensation = 0`;
- planned layer height `0.2` mm for every retained layer;
- `initial_layer_line_width = 0.5` mm;
- region `outer_wall_line_width = 0.42` mm;
- object `line_width = 0.42` mm;
- effective region `outer_wall_filament_id = 1`, normalized from the raw
  zero/fallback semantics before this stage;
- `filament_map = [1, 1]`;
- `nozzle_diameter = [0.4, 0.4]` mm; and
- normal coordinate scale.

The KSR values cannot distinguish direct nozzle lookup from an incorrect
logical-to-physical pre-map because both physical nozzles have the same
diameter and logical selector one maps to physical nozzle one.

The resulting first-layer f32 external Flow has spacing
`0.4570796489715576` mm and minimum contour width
`0.9570796489715576` mm. Only layer zero is compensated. Its region surfaces
are compensated and rebuilt, while its final `lslices` are the ordered
uncompensated six ExPolygons. Every later layer's `lslices` are the ordered
current region surfaces. The raw layer-zero world bounds are X
`[95.539205, 169.639205]`, Y `[81.892105, 150.992105]`; the reference's later
first-layer outer extrusion bounds are consistent with the 0.15 mm
compensation plus half the 0.5 mm external width. That downstream observation
is evidence, not a Task 22M expected-output shortcut.

## Option ownership, resolution, and validation

Task 22M consumes existing typed effective records; it does not add a dynamic
map, second parser, or old `ExtrusionOptions` compatibility path:

- object `elefant_foot_compensation`, `elefant_foot_compensation_layers`,
  `raft_layers`, `xy_hole_compensation`, `xy_contour_compensation`, and
  `line_width`;
- global process `initial_layer_line_width`;
- per-region `outer_wall_line_width` and `outer_wall_filament_id`;
- global project `nozzle_diameter`; and
- each retained `PlannedLayer.id` and `PlannedLayer.height`.

`outer_wall_filament_id` remains a 1-based logical filament selector. The fixed
`PrintRegion::flow` source nevertheless indexes the physical
`nozzle_diameter` vector directly with `selector - 1` and element-zero
fallback. Task 22M must not apply `filament_map` before Flow resolution;
logical-to-physical mapping belongs to a later Tool/G-code boundary. The typed
`filament_map` remains available in the resolved project, but is deliberately
not a Task 22M Flow input.

Raw `FloatOrPercent` variants remain intact until external-perimeter Flow
resolution. A percent zero is not the same as an absolute zero: percent zero
resolves to zero and fails the positive-spacing boundary, while an absolute
zero can participate in source-defined width fallback or auto width.

After effective resolution and Task 22L, the Task 22M orchestration boundary
preflights every object before mutating any object. In resolved-object order it
requires finite nonnegative `elefant_foot_compensation`, a strictly positive
`elefant_foot_compensation_layers`, exact zero XY hole compensation, exact zero
XY contour compensation, and no valid multi-region retained layer. Nonzero XY
values return `SliceError::UnsupportedProjectFeature` with keys
`xy_hole_compensation` then `xy_contour_compensation`; multi-region input uses
key `multi_region_layer_slices`. The already released capability gate rejects
every nonzero real-project `raft_layers`, including negative values, earlier
with `UnsupportedProjectFeature("raft_layers")`; that result precedes Task 22M
raw validation. The pure Task 22M stage still locks the upstream rule that any
signed nonzero raft value disables compensation, without first converting it
to an unsigned type.

For every layer that can receive positive compensation, preflight resolves its
Flow without mutation. The selected nozzle vector must be nonempty, the
selected/fallback nozzle must be finite and positive, the planned height must
be finite and positive, and the resulting spacing must be finite and positive.
Invalid raw values use exact errors `invalid Orca option
elefant_foot_compensation`, `invalid Orca option
elefant_foot_compensation_layers`, `invalid Orca option nozzle_diameter`, and
`invalid Orca option layer_height`. A nonpositive derived spacing uses
`invalid external perimeter flow spacing`.

Compensation-layer validation explicitly covers zero, negative one, and
`i32::MIN` before any conversion to a layer count. A later invalid object must
still leave every earlier object unchanged. Public real-project tests lock
both positive and negative nonzero raft values to the earlier capability error;
pure-stage tests lock both signs to zero compensation with normal `lslices`.

All object configs, structural gates, and required per-layer Flow records are
successfully preflighted before the first mutation. A later invalid object
therefore leaves earlier objects byte-for-byte unchanged. Empty retained
objects may have zero regions and remain empty. Every nonempty retained object
must otherwise have exactly one region for this slice. No Option is inferred
from geometry, object names, hashes, timestamps, reference G-code, or fixed KSR
counts.

## External-perimeter Flow semantics

For each positively compensated retained layer:

1. Use `PlannedLayer.height`, converted once to f32. First-layer identity is
   `PlannedLayer.id == 0`, not vector position alone.
2. If this is the first layer and the raw numeric value of
   `initial_layer_line_width` is strictly positive, select that complete raw
   `FloatOrPercent`; otherwise select region `outer_wall_line_width`.
3. If the selected raw numeric value equals zero, replace it with object
   `line_width`, preserving that replacement's percent/absolute variant.
4. Use the selected 1-based logical outer-wall filament ID directly as the
   nozzle-vector selector, without consulting `filament_map`. Preserve the
   exact source underflow/out-of-range behavior: selector zero, underflow, and
   an index past the vector all select element zero.
5. Convert the selected nozzle f64 to f32 before width evaluation.
6. Absolute width converts directly to f32. Percent width evaluates as
   `f64::from(nozzle_f32) * configured_f64 / 100.0`, then converts to f32.
7. Only a final non-percent width `<= 0` selects auto width
   `1.125f32 * nozzle_f32`. A percent zero never selects auto width.
8. Compute spacing as the exact source sequence
   `width_f32 - height_f32 * f32(1.0_f64 - 0.25_f64 * PI_f64)`.
9. Compute minimum contour width as f32 `width + spacing`, then promote it for
   the geometry kernel.

Flow is resolved once for each compensated layer. Uncompensated layers do not
need Flow merely to execute `make_slices`. There is no cached global first-layer
Flow and no f64 rewrite of the source f32 sequence.

## Compensation ramp and coordinate arithmetic

The runtime `CoordinateScale` is already selected before Task 22A:

- normal factor `0.000001` mm per integer coordinate;
- large-bed factor `0.00001` mm per integer coordinate.

The exact stage arithmetic is:

1. If `raft_layers != 0`, scaled compensation is f32 zero. Otherwise compute
   `f32(compensation_f64 / scale_factor)` without integer rounding.
2. Allocate uncompensated backups only when scaled compensation is strictly
   positive, with length `min(configured_layers, retained_layers)`.
3. For layer vector index `i`, positive compensation is
   `scaled - (scaled / configured_layers_f32) * i_f32` when `i` is below the
   configured count, otherwise zero. Preserve this source f32 operation order.
4. Pass the layer value through exact
   `f64::from(elfoot_f32) * scale_factor`, then let the kernel rescale it by
   division. Do not substitute the original option f64.
5. `SCALED_EPSILON` is `0.0001 / scale_factor`, specifically
   `100.00000000000001` at normal scale and `10.0` at large-bed scale.
6. Geometry coordinates stay signed i64. Floating-to-coordinate casts use the
   same truncation/rounding sites as the cited upstream algorithms; the stage
   does not pre-quantize f32 offset deltas.

Nonfinite derived geometry or a Clipper coordinate error maps once at the
Task 22M boundary to `SliceError::InvalidInput("project elephant-foot
compensation geometry is nonfinite or outside the supported Clipper range")`.
It never becomes identity output.

## Tiny gate, simplification, and resampling

For each input ExPolygon, derive scaled compensation, scaled minimum contour
width, compensated minimum width, and search radius exactly as upstream. The
input is returned unchanged only when the source tiny gate holds: contour bbox
width or height is below compensated minimum width plus scaled epsilon, or
signed ExPolygon area is below five times the squared compensated minimum
width.

Otherwise simplify the closed ExPolygon with Douglas-Peucker tolerance
`SCALED_EPSILON`, preserving upstream closed-contour seam selection, f64
point-to-segment squared distance, contour/hole orientation, signed area, and
union normalization. Normalization executes StrictlySimple to Paths, a normal
union to Paths, then a fresh union to PolyTree. The supported valid input must
yield the source-owned first simplified ExPolygon; Clipper failures propagate.

Resample the simplified contour and every hole at scaled 0.5 mm. Each source
segment receives `ceil(length / interval)` subdivisions, intermediate points
are cast at the source site, and every output point retains source-segment,
interpolated, step-length, and accumulated curve-parameter information. Empty
or two-point invalid closed contours are not invented by this stage.

## EdgeGrid and filtered contour distance

Production must implement the cited `EdgeGrid::Grid` spatial-index contract.
It may not use the oracle's exhaustive full scan as the production algorithm.
The simplified ExPolygon bbox is expanded by scaled epsilon; grid resolution
is `coord_t(0.7 * search_radius)` at the source cast boundary. Contours and
holes retain their indices and segment indices in cell data. Segment raster
insertion and query-box cell traversal must preserve deterministic candidate
enumeration and avoid duplicate semantic results.

For each resampled point, query its search-radius box and inspect candidate
segments. The closest accepted foot uses f64 finite-segment projection, clamps
`t` to `[0,1]`, requires the candidate vector to point into the contour, and
chooses strict shorter distance. Same-contour candidates additionally use the
accumulated cyclic arc distance:

- reject below `0.5 * scaled_compensation * PI`;
- when distance is strictly below `search_radius + SCALED_EPSILON`, accept only
  the exact corner/segment inside predicate and arc distance greater than
  `0.6 * PI * chord distance`;
- otherwise retain the nearest accepted distance, capped to search radius.

Contour orientation controls the inward normal naturally; no contour/hole
special-case sign is added outside the cited predicates.

## Distance mapping, smoothing, and variable inner offset

Each accepted/capped distance becomes a per-vertex f32 offset:

- below minimum contour width: `0`;
- above compensated minimum width: negative scaled compensation;
- otherwise: negative half of the width excess.

Run exactly three banded smoothing passes, strength `0.3f`, band
`0.8 * resample_interval`, f32 point vectors and compensation values. Each side
walks cyclically to the band distance, linearly interpolates at the crossing,
computes the source Laplacian, and uses `max(laplacian, current)` so smoothing
cannot increase the magnitude of the negative compensation. The disabled old
smoother is not ported.

Apply the fixed variable miter inner offset with miter limit `2.0` to the
resampled ExPolygon and its contour/hole delta vectors. Preserve the source
per-vertex shifted-line construction, convex/concave join decisions, miter
limit, inner/outer repair, NonZero rules, and hole subtraction. This is not a
constant Clipper offset and must not be approximated by averaging deltas.

The only permitted identity fallback is the explicit upstream rule: if the
variable-offset result contains anything other than exactly one ExPolygon,
return the original input ExPolygon. Actual arithmetic, Clipper, allocation,
or nonfinite errors propagate; there is no broad `unwrap_or(input)` path.

## Two-pass union and surface replacement

After compensating every original ExPolygon independently, preserve provider
order by adding each contour followed by its holes as Subject paths. Execute
NonZero Union first to Paths, then add those Paths to a fresh Clipper instance
and execute NonZero Union to PolyTree. Convert that second-pass PolyTree to
ordered ExPolygons. A direct one-pass PolyTree execution is forbidden because
the algorithms have observably different sibling and nested-island ordering.

Replace the compensated region's complete surface collection with Internal
surfaces. Every new surface resets metadata to exactly
`(Internal, -1.0, 1, -1.0, 0)` for kind, thickness, thickness layers, bridge
angle, and extra perimeters. A tracked Rust test must seed nondefault metadata
and observe this reset; the C++ checkpoint wire cannot encode those fields.
When compensation is disabled for a layer, `make_slices` alone does not rebuild
region surfaces and must preserve their metadata.

## Single-region `make_slices` and backup restoration

Every retained layer executes single-region `make_slices`, regardless of
whether compensation is enabled. It copies/moves that layer's current region
surface ExPolygons, collects each contour's first point, and orders islands via
the cited `chain_points` implementation. Empty input produces empty `lslices`.

`chain_points` uses squared signed-coordinate distance and exact deterministic
source behavior for nearest selection, equal-distance ties, mutable-priority
updates, KD-tree capacity/growth, and input-index output. It must not replace
ties with an unstable sort or a different nearest-neighbor crate whose order
is only geometrically equivalent.

For each layer with a positive source ramp, save the pre-compensation
ExPolygons before surface replacement. After every layer has run
`make_slices`, independently chain each saved vector and replace that layer's
`lslices` with the ordered uncompensated vector. Thus compensated region
surfaces and raw `lslices` intentionally differ. When scaled compensation is
not positive, there is no backup vector and `lslices` remain ordered current
surfaces.

## Rust destination and state boundary

Task 22M introduces a narrow wrapper rather than changing the released
`PostRegionPrintObject` contract:

```rust
PostCompensationPrintObject {
    post_regions: PostRegionPrintObject,
    lslices: Vec<Vec<ExPolygon>>,
}
```

The wrapper preserves the complete plan, occurrence-keyed volume sidecars,
region IDs, region Options, layer/surface order, and object order. `lslices`
has exactly one entry per retained planned layer. An object with zero retained
layers has an empty outer vector; a zero-region object with N retained layers
has N empty inner vectors. No duplicate parallel surface model is introduced.

Resolved contexts expand in existing order: each `ResolvedProjectObject` in
vector order, then each of its `print_objects` in vector order, produces one
matching post-L object. Task 22M uses that source object's `ObjectOptions`, the
global typed print/project views, and the ordered `PostRegion.options` already
carried by the region composition stage. No context is re-looked-up by name or
geometry.

Production modules are real Rust modules under `geometry` and
`project_slice`: the released two-pass boolean union, variable offset, EdgeGrid/raster,
chain-points/KD/priority support, Flow resolution, elephant-foot kernel, slice
ordering, orchestration, and checkpoint framing. `project_slice.rs` is already
348 lines; existing G-L and new M checkpoint entrypoints move to a real
`project_slice/checkpoints.rs` module before M wiring. Source splitting with
`include!`, `include_bytes!`, or textual source inclusion is forbidden.

The old general extrusion-width compatibility APIs are not reused where they
erase the raw absolute-zero versus percent-zero distinction. No new third-party
geometry engine, native-only code, filesystem access, terminal behavior,
unsafe code, or public slicing pipeline is introduced.

## Pre-implementation oracle contract

An ignored C++20 oracle was built before this specification from the fixed
source identities above. It uses fixed modified Clipper 6 but deliberately
replaces production EdgeGrid enumeration with an exhaustive segment scan. The
independence lets the tracked production grid be checked against identical
observable geometry without sharing spatial-index failure modes.

The ordered 19-case matrix covers normal and large scales, narrow/hole and tiny
geometry, disabled and raft gates, a two-layer ramp, layer-count clamp, zero
and empty layers, per-layer heights, every width fallback, absolute auto width,
percent second-nozzle selection, selector fallback, and a sibling/hole/nested
two-pass-union discriminant. Startup self-checks lock zero-percent spacing
failure, both XY rejections, multi-region rejection, chain empty/nearest/tie
semantics, and a direct-one-pass mutant kill. The fixed two-pass result is
ordered `[left, nested, right]`; the mutant is `[right, left, nested]`.

The pure oracle intentionally has no `filament_map` input because the fixed
Flow boundary does not consume one. Its percent-second-nozzle case locks direct
selector behavior. A tracked real-3MF integration case additionally varies a
nonidentity map and unequal nozzle diameters so orchestration cannot secretly
pre-map the selector before calling the pure resolver.

The oracle build fails closed on the fixed Orca tree and every listed blob, the
live fixed Clipper sources, and the shared Task 22L type header SHA-256
`1a7834789dc80a8347b5e0c3dfe8e619ee4f96c49d8183e086c372407beb50e7`.
Every owned C++ file is below 400 LOC; the maximums are 396 and 394. It uses
real translation units and no textual `.cpp` inclusion.

Independent clean compilation and two complete runs froze:

| Mode | Kind | Bytes | SHA-256 |
|---|---|---:|---|
| synthetic | binary | 10,351 | `c112246ff48b280eb803082749d74315e771d073b0407e45afde536e37fcf46d` |
| synthetic | text | 17,407 | `daa902bf4d1bf93d16e8c1b22427432ffe37d0c5d73728967f08bcf7a5d57e72` |
| KSR | binary | 3,008,346 | `91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19` |
| KSR | text | 2,528,073 | `abbe1ce7bdfdda06f4e9e6e581c2e08b4ff29051322bf22c92c5daaf62e79833` |

Both local clean runs matched byte-for-byte after deleting all Task 22M objects
and the executable, rebuilding from zero, and regenerating all four outputs.
The fixed Clipper translation unit uses `/W0`; every oracle-owned translation
unit uses `/W4 /WX`; both use C++20 `/O2 /fp:precise /DNDEBUG`.
Independent fixed-source and current-Ares approval of this exact document frame
is still required before production or tracked-test implementation begins.

Ignored oracle sources and outputs are never staged or loaded at runtime.
Tracked tests copy only bounded independently frozen inputs, exact coordinates,
bit patterns, counts, and hashes needed to prove the Rust behavior.

## Real 3MF anti-hardcoding vectors

Tracked native tests load the committed KSR project through the public byte
boundary and prove its effective Task 22M Options, 460-layer/one-region branch
inventory, released L input identity, exact M output identity, layer-zero
surface mutation, uncompensated `lslices`, unchanged later geometry, unchanged
plan/sidecars, repeatability, and continuing public incomplete result.

A small synthetic 3MF pair is also generated in memory. Both archives have the
same model, plan, geometry, profiles, and semantic entry order and differ only
in one embedded elephant-foot Option. The disabled archive proves
`make_slices` without surface rebuilding; the enabled archive proves real
compensation, metadata reset, raw backup restoration, and ordering. Additional
in-memory archives vary width forms, nozzle selection, compensation layers,
XY gates, and region count through real serialized Options. One anti-map pair
uses `nozzle_diameter = [0.4, 0.6]`, 125 percent width, and selector one; the
archives differ only between `filament_map = [1, 2]` and `[2, 1]`. Direct
lookup must produce f32 width 0.5 mm and byte-identical M output for both;
incorrect pre-mapping changes the second width to 0.75 mm. They are not
filename-conditioned test objects.

Tests freeze ordered semantic archive entries rather than assuming native ZIP
bytes equal browser `fflate` bytes. No production or tracked test embeds the
reference G-code, invokes OrcaSlicer, reads `/OrcaSlicer`, inspects Git, or uses
fixture hashes to select behavior.

## Task 22M checkpoint contract

The Task 22M checkpoint uses magic `ARES22M\0`. Its object framing preserves
the Task 22L object, plan, occurrence-sidecar, retained-layer, region,
surface-kind, and complete ExPolygon byte order. For each retained layer it
writes the layer index, complete ordered regions and surfaces, then immediately
writes that same layer's complete ordered `lslices` ExPolygon vector before the
next layer. This exact per-layer framing matches the fixed oracle and is frozen
by parser EOF/truncation tests; it does not duplicate Options or trace
instrumentation.

`task22m_browser_input_oracle` returns the exact released `ARES22L` frame.
`task22m_browser_oracle` executes the actual M stage and returns the complete
`ARES22M` frame. For committed KSR input the registered L frame is 2,008,706
bytes with SHA-256
`7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07`;
the M frame is 3,008,346 bytes with SHA-256
`91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`.

The parser validates magic, counts, IDs, Internal kind, every point/hole,
`lslices`, and exact EOF. Truncation and trailing bytes fail. Task 22L native
regressions remain available under `cfg(test)`; the nondefault browser feature
is replaced, not aliased.

## WASM and browser boundary

The previous `task22l-browser-oracle` feature is replaced by
`task22m-browser-oracle` in core and adapter. Default WASM exports no Task 22
hook. The feature build exports exactly:

- `task22mBrowserInputOracle`;
- `task22mBrowserOracle`.

The old `task22l-vectors.mjs` file is deleted and replaced by a real
`task22m-vectors.mjs` module. The existing browser page/parser and explicit
test server routes are updated rather than duplicated. Browser tests first run
independent small M parser KATs, then load the real committed KSR 3MF and the
small Option-only synthetic archives. They verify exact export sets, semantic
archive identities, L/M hashes, complete summaries, enabled surface changes,
uncompensated `lslices`, repeatability, public incomplete behavior, and exact
EOF. Chromium runs twice from fresh optimized bindgen output.

## Structural constraints and deferrals

Every changed Rust production and test file must remain strictly below 400
physical lines. Tests live under separate real `mod` files. Rust source may not
be split with `include!`, `include_bytes!`, `include_str!`, or equivalent macro
indirection; `include_bytes!` remains permitted only for bounded fixture input,
not source organization. New unsafe code is forbidden.

Task 22M tracked tests contain no executable Orca source pinning. Source
identities live in this specification and ignored evidence only. Existing
runtime code is changed only where required by this slice. No legacy browser
feature/export alias, general-pipeline fallback, hardcoded KSR constant,
filename/hash branch, or speculative later-stage abstraction is retained.

Explicitly deferred source boundaries are:

- painted/MMU/fuzzy/interlocking segmentation and their input parsers;
- nonzero XY contour and hole compensation;
- multi-region compensation, merged trimming, region redistribution, and
  multi-region `make_slices` safety union;
- cancellation and parallel scheduling control planes;
- later raw backup/classification, perimeters, fill, supports, extrusion and
  travel planning, G-code, metadata, and post-processing; and
- complete normalized `ksr_fdmtest_v4.gcode` equality.

These deferrals do not permit silent identity behavior. Unsupported activated
inputs fail before mutation with their specified feature keys.

## Acceptance and review gate

Task 22M is complete only when:

1. the fixed-source oracle and exact document frame have independent approval;
2. every implementation package follows RED then GREEN with expected failures
   recorded before production code;
3. external Flow including direct unmapped nozzle selection, two-pass union,
   variable offset, EdgeGrid, filtered distance, smoothing, full elephant-foot
   geometry, chain ordering, metadata reset, backup restoration, and
   transaction gates match their fixed vectors;
4. committed KSR loads only embedded typed Options, yields exact L and M
   identities, and remains publicly incomplete;
5. native, WASM, optimized bindgen, and real Chromium checks pass twice where
   required;
6. `cargo fmt`, strict workspace clippy/check, focused Task 22A-M tests, core,
   and full workspace nextest pass;
7. manifest, LOC, macro, unsafe, source-pinning, fixture, hardcoding, stale-L
   feature/export, and `git diff --check` audits pass;
8. one dedicated read-only reviewer validates requirement completeness,
   logical correctness, boundary cases, code quality, test coverage, and
   actual execution, returning an empty P0-P3 repair list;
9. every reviewer finding is repaired by the main thread, fully rerun, and
   revalidated by the same reviewer until approval; and
10. the unchanged reviewed frame is committed, pushed normally, and all five
    exact-SHA Tier-1 jobs pass.

Task 22M does not emit G-code and does not complete the persistent goal. The
next source-cited slice begins only after this release gate is green.
