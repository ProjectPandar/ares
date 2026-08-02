# Task 22O: KSR-Reached Classic Perimeter Generator

> Historical execution frame: the Package-A0 qualification/recovery campaign
> in this document and its amendments is superseded as a production gate by
> the serial source-port plan beginning with Task 22O.1. It remains technical
> audit evidence only and must not be retried or block Rust implementation.

## Status and objective

This specification is a draft. Production and tracked-test implementation may
begin only after this specification and its implementation plan are frozen as
one exact content frame and receive independent fixed-source/specification and
current-Ares/plan approval.

Task 22O is the next bounded source rewrite in the persistent
`ksr_fdmtest_v4` project-to-G-code parity program. Released Ares commit
`e27e8c7736a768d1b4d7d291c62a553c722e1f21` produces the complete Task 22N
single-region perimeter input records but deliberately stops before either
generator body. Task 22O consumes those records and ports the complete
KSR-reached Classic perimeter-generator behavior from the fixed OrcaSlicer
v2.4.2 source.

For every populated Classic record, the stage produces ordered perimeter
island collections, loop and path metadata, supported/overhang path splits,
variable-width perimeter gap-fill entities, internal fill surfaces, and
fill-without-overlap polygons. It preserves the complete Task 22N predecessor
and typed project identity for later source slices. The public project API
executes the stage and continues to return
`SliceError::ProjectSlicingIncomplete`; Task 22O does not claim seam placement,
infill paths, G-code motion planning, extrusion scheduling, or final G-code.

This is not a completion of every Classic option combination. It is the exact
fixed-source behavior reached by the supplied KSR archive. An activated
adjacent branch that is explicitly deferred below must fail at the project
stage with its option key; it must never be silently ignored, approximated, or
routed through the old rectangular STL pipeline.

## Fixed identities and evidence

The fixed Ares baseline is commit
`e27e8c7736a768d1b4d7d291c62a553c722e1f21`, tree
`6a69f55b22a2638c383cfb811dac94d5389d5056`. Exact-SHA Tier-1 run
`29874293283` passed format, Ubuntu/Linux, Windows, macOS, and WASM/browser.

All upstream citations refer only to OrcaSlicer tag `v2.4.2`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored upstream checkout is
currently at another commit. Source discovery, oracle builds, and reviews must
read fixed Git objects or a detached fixed worktree. Tracked tests must never
inspect Git, the Orca checkout, source text, source line numbers, or blob IDs.

Principal fixed source blobs are:

- `src/OrcaSlicer_app_msvc.cpp`,
  `0f1953f55f65f45bc40b5a5b5d430bd84cd3c39a`;
- `src/OrcaSlicer.cpp`,
  `271759b5e3d4746847635a877b7710858ddbeccf`;
- `src/libslic3r/LayerRegion.cpp`,
  `22e0a26898c6fe07ad8ebd35de303b5911d84f4b`;
- `src/libslic3r/PerimeterGenerator.cpp`,
  `1a0f129c0d44cb5ff6c5b69ffee5ce5d211a0c80`;
- `src/libslic3r/PerimeterGenerator.hpp`,
  `e4f918d8bd772e53b925dfdbcd57dc799261f2af`;
- `src/libslic3r/ExPolygon.cpp`,
  `185e92508449a425064b26690e3d74d06a16fda8`;
- `src/libslic3r/Geometry/MedialAxis.cpp`,
  `7fece75e633653dccf59b21657e59de5f202ef3f`;
- `src/libslic3r/Geometry/MedialAxis.hpp`,
  `cd1404f915b5857130e4ce77aa35ea02d3526935`;
- `src/libslic3r/Geometry/VoronoiOffset.cpp`,
  `8ecd370235cb506892b5eb33d0c9e015dfa2bca0`;
- `src/libslic3r/Geometry/VoronoiOffset.hpp`,
  `747538e8b69e83eb7c91edff7c16e760e42b90d1`;
- `src/libslic3r/VariableWidth.cpp`,
  `c00fc13bdf2559bf1aa85f54562b0cffaadb2986`;
- `src/libslic3r/ExtrusionEntity.cpp`,
  `36955a19bdf4f5360a3fb053b9d59c3c183b7b1d`;
- `src/libslic3r/ExtrusionEntity.hpp`,
  `180312aa6d3dd9ccfda9ae35989037171a2b6458`;
- `src/libslic3r/ExtrusionEntityCollection.cpp`,
  `9a37ff3ac12b644b0032eff4b6c54b8b8109845b`;
- `src/libslic3r/ShortestPath.cpp`,
  `e2fc258e316c8e9ded30a3003ee3d534399b8a1b`;
- `src/libslic3r/ClipperUtils.cpp`,
  `2f97e08f536e93c5fd27b4614980072285d2ce22`;
- `src/libslic3r/Surface.cpp`,
  `58ac7294cc002a8518bb12c9a32c2607a25aef25`;
- `src/libslic3r/Print.cpp`,
  `e25ef7db84f0d29324c77bc925feade57d3ca12e`;
- `src/libslic3r/Thread.cpp`,
  `3030b6d194d4ba27a0a94c5476caba7d6a7d2fc1`;
- `src/libslic3r/Thread.hpp`,
  `c4071590725b55f79da0b3208a0d01f6e56f7ad5`; and
- the fixed `Flow.*`, `PrintConfig.*`, `libslic3r.h`, `Polygon.*`,
  `MultiPoint.*`, `Polyline.*`, `Line.hpp`, `BoundingBox.hpp`, and
  `Feature/FuzzySkin/FuzzySkin.*` objects cited by the oracle manifest.

The supplied archive is 183,007 bytes with SHA-256
`698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`.
The supplied reference G-code is 6,339,134 bytes and 269,330 lines with
SHA-256
`10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3`.
The released Task 22N KSR checkpoint is 7,083,888 bytes with SHA-256
`42e0053bffb3093a44597abd0a2b4e8b8c8c11d6f07003cb894399ad7dce3c6e`.

The ordinary unchanged-scheduler Package 0 control is RED. Two fresh primary
runs used identical input and content- and metadata-identical datadir clones.
Both emitted 4,842,892-byte payloads, but their SHA-256 values were
`fe2f6523772f175484a93fad3899e9fa35a2ee08d6ce89939f08d1289284a78e`
and
`b33bbe42c5ef9a0a8d30dade183f3cde7417f5ab52af59e708ab6fcba5464bb3`.
Their structure, identities, and semantic totals were equal, but 8,158 aligned
fields differed, all of them coordinates. Neither run is selected or
canonicalized; both remain negative evidence that ordinary Release scheduling
does not provide a freezeable Task 22O payload.

The supplied reference remains the immutable later end-to-end Ares target, but
it is not a replay-authentication oracle for Package 0. The surviving July 10
log shows that Orca loaded the DRC as model-only input, completed G-code export,
and only afterward exported the supplied project state. This provenance
correction does not relax final acceptance: eventual Ares G-code may differ
only in the generated-by timestamp and the producer token `OrcaSlicer`
becoming `Ares`; every other byte must match the supplied reference.

## Correct upstream boundary

The Classic body is exactly `PerimeterGenerator.cpp:1144-1692`,
`PerimeterGenerator::process_classic()`. Its directly called fixed helpers
needed by the KSR path are:

- `traverse_loops()`, lines 100-280;
- `split_top_surfaces()`, lines 574-671;
- the false-body gate of `apply_extra_perimeters()`, lines 1087-1114;
- the inactive reorientation gate at lines 1117-1142;
- `process_no_bridge()`'s `chbNone` return at lines 1728-1732;
- `PerimeterGeneratorLoop::is_internal_contour()`, lines 2537-2546; and
- `generate_lower_polygons_series()`, lines 2548-2570.

`add_infill_contour_for_arachne()` at lines 1695-1725 and the Arachne
touching/reorder helpers at lines 1931-2088 are not part of Classic. The old
roadmap wording that treated all lines 1144-2092 as one Classic body is
superseded by this boundary.

`LayerRegion.cpp:82-142` defines the caller/output seam. Task 22O must rewrite
that seam directly over Task 22N state. Existing `crates/ares-core/src/perimeters.rs`
and its rectangle-oriented children are an older public STL scaffold. They may
not be called as a project-path fallback and are not evidence of parity.

## Predecessor and output contract

Task 22O accepts only the complete `PreparedPostPerimeterInputs` state. The KSR
predecessor has one object and 460 populated, nonspiral Classic records. It has
2,890 current surfaces, 395 holes, 3,285 polygon boundaries, and 59,160 points.
All current surfaces are internal, nonbridge, and have zero extra perimeters.

The immutable Task 22O output for each populated record contains:

1. An ordered top-level collection whose children are per-island collections.
2. Each perimeter `ExtrusionLoop`, including loop role, inset depth, and one or
   more ordered `ExtrusionPath` values.
3. Each path's fixed-coordinate polyline, extrusion role, width, height, and
   exact `mm3_per_mm` bits. Supported and overhang portions remain separate
   paths within the same logical loop when fixed traversal does so.
4. A separate ordered gap-fill collection containing open paths or loops
   produced from variable-width `ThickPolyline` values.
5. Internal `fill_surfaces`, retaining source-compatible surface metadata.
6. Ordered `fill_no_overlap` ExPolygons.

The object wrapper retains the complete Task 22N object and aligned layer
slots. Empty input slots remain empty. A failure is transactional across the
whole project: preflight and geometry failure return an error without exposing
a partially advanced object set.

The new fixed-coordinate extrusion model is crate-private to the project
slicing path. It must not adapt through the old public f64 `ExtrusionPath`,
flatten island collections, discard loop/path distinction, or erase loop role,
inset depth, width, height, or volume metadata.

## KSR option and branch inventory

All values are resolved from the supplied 3MF. No production code may inspect
fixture paths, hashes, object names, point counts, layer counts, or reference
G-code.

Directly result-affecting KSR values are:

| Option | Effective value | Reached behavior |
|---|---:|---|
| `wall_generator` | `classic` | dispatches to this stage |
| `wall_loops` | `2` | outer plus one inner where geometry survives |
| `precise_outer_wall` | `true` | first inner distance uses width average |
| `wall_sequence` | `InnerOuter` | retains inner-before-outer traversal order |
| `wall_direction` | `ccw` | contours CCW and ordinary holes CW; the lone-hole exception is defined below |
| `detect_overhang_wall` | `true` | splits loops against lower support masks |
| `raft_layers` | `0` | starts overhang splitting after layer zero and defines the first printable layer |
| `detect_thin_wall` | `false` | selects smaller-external-width fallback |
| `only_one_wall_top` | `true` | topmost one wall and dynamic top splitting |
| `gap_infill_speed` | `250` | enables Classic perimeter gap discovery |
| `sparse_infill_density` | `15%` | enables the gap-only extra offset pass |
| `filter_out_gap_fill` | `0` | keeps every positive-length gap polyline |
| `enable_arc_fitting` | `true` | uses `0.2 * resolution` surface simplify |
| `resolution` | `0.012` | 0.0024 mm wall and 0.012 mm fill simplify |
| `infill_wall_overlap` | `15%` | intermediate fill overlap |
| `top_bottom_infill_wall_overlap` | `25%` | first/top and exposed-top overlap |
| `min_width_top_surface` | `300%` | dynamic top split width threshold |
| `interface_shells` | `false` | top split uses all upper layer slices |
| `sparse_infill_line_width` | `0.45` | top split fill clipping width |
| `fuzzy_skin` | disabled | grouping is a geometry no-op |
| `outer_wall_filament_id` | effective `1` | selects nozzle element zero |
| `nozzle_diameter` | `[0.4, 0.4]` | 0.2 mm lower support growth |

Inactive gates that the real-archive inventory must freeze are
`alternate_extra_wall=false`, `only_one_wall_first_layer=false`,
`spiral_mode=false`, `overhang_reverse=false`,
`overhang_reverse_internal_only=false`, `brim_type=auto_brim`,
`extra_perimeters_on_overhangs=false`, and
`counterbore_hole_bridging=none`.

`gap_fill_target=nowhere` is not a Classic perimeter-gap gate. The obsolete
master-plan statement that it suppresses perimeter gaps is invalid. Fixed
`process_classic()` enables gaps from `gap_infill_speed > 0`; the KSR reference
contains 470 Gap infill feature markers and 758 continuous gap paths.

## Exact reached behavior

### Prelude and surface preparation

Task 22N Flow values are consumed without recomputation. For later layers,
internal/external/overhang/solid widths are 0.45/0.42/0.4/0.42 mm. Precise
InnerOuter spacing uses `(external width + internal width) / 2`, producing
0.5 mm on layer zero and 0.435 mm later. For the smaller external Flow, fixed
source first truncates external width and spacing to scaled `coord_t`, evaluates
`SCALING_FACTOR * (scaled_width - 0.5 * 0.22 * scaled_spacing)` using the source
double constants and operation order, and narrows exactly once through
`Flow::with_width(float)`. Tests freeze the resulting width, recomputed spacing,
and `mm3_per_mm` bits; they may not replace this sequence with direct f32 or
rounded millimeter arithmetic.

Lower support preparation grows lower slices to half the selected nozzle and
builds the same two-element sampled offset series for normal internal, normal
external, and smaller external widths. Surface processing calls the
counterbore helper but returns immediately for `none`, simplifies at 0.0024 mm,
unions the result, and greedily chains ExPolygons by bounding-box center, not by
their first contour point.

### Onion shells and dynamic top surfaces

Normal surfaces start with zero-indexed `loop_number=1`; the topmost layer is
forced to zero by `only_one_wall_top`. The first onion offset selects either
normal or smaller external width using the fixed narrow-area test. Later onion
offsets use precise external/internal spacing for depth one and normal
perimeter spacing thereafter. A positive gap speed collects differences and a
positive sparse density permits one additional offset iteration used only for
gap discovery.

On intermediate exposed top areas, `split_top_surfaces()` must execute the
fixed upper/lower masking, bridge exclusion, 300% minimum-width threshold,
0.45 mm fill clip, top/non-top split, and union behavior. Geometry-dependent
collapse, number of surviving loops, hole topology, and selection of smaller
width are required behavior, not deferrals.

### Hierarchy, traversal, and overhangs

Holes are nested before contours. Contours are nested deepest to shallowest.
Traversal creates external or internal loop paths, splits paths against the
appropriate lower polygon series only when `layer_id > raft_layers`, reorders
split paths from their first point, chains entities from `(0,0)`, applies
contour/hole direction, and emits children before a contour or after a hole as
fixed source requires. KSR has `raft_layers=0`, so its specialization starts
splitting after layer zero.
For the fixed single-contour/single-hole topology, `reverse_thin_wall_hole`
makes the lone hole counterclockwise under KSR `wall_direction=ccw` and applies
the fixed reverse ordering step; ordinary holes remain clockwise. Direction,
role, and ordering for both topologies are oracle-frozen behavior.
The KSR sequence does not execute outer-first, outer-only-brim, sandwich, or
overhang-reverse reordering.

The reference G-code observes 2,075 Outer, 1,973 Inner, and 148 Overhang
feature markers. It contains 3,272 external and 1,971 inner physical wall
strands; 56 wall strands are split into overhang portions on 48 layers. These
are downstream observations, not substitutes for the complete Task 22O oracle.
The oracle must freeze the exact pre-seam loop/path totals and ordered bytes.

The smaller external width is observed on 152 physical external paths. The
reference contains only seven `LINE_WIDTH 0.37852` state-change comments;
those comments are stateful and are not an entity count.

### Gap fill and fill remainder

Gap regions use the fixed opening/difference sequence, 0.0024 mm Douglas-
Peucker simplification, fixed medial-axis width bounds, endpoint extension and
short-branch pruning, then fixed variable-width grouping with 0.05 mm width
tolerance. Actual extrusion coverage, grown by 10 internal units, is subtracted
from the fill remainder. The KSR reference observes 470 gap feature markers,
758 continuous paths, and widths from approximately 0.0933202 to 0.70341 mm.

The remaining fill geometry uses the complete 0.012 mm simplify, the fixed
solid-flow collapse threshold, 15% or 25% overlap according to layer/top
classification, top-fill union, internal surface append, and no-overlap
calculation. `apply_extra_perimeters()` is called but its body is inactive for
the KSR option.

## Geometry and dependency contract

The stage uses Ares' released fixed-coordinate Clipper booleans, offsets,
strict simplification, and nearest-neighbor infrastructure where they already
match the fixed source. Missing source boundaries must be added as real
modules: bounding boxes, fixed polylines/lines, open-polyline clipping,
opening/closing adapters, extrusion coverage, ThickPolyline, medial-axis
extraction, and variable-width conversion.

`boostvoronoi 0.12.1` is a candidate BSL-1.0, pure-Rust port with Rust 1.87
minimum support. It is not automatically accepted as an exact Orca substitute.
Before it becomes a normal dependency, an ignored compatibility probe must
show the same ordered finite primary edges, source categories, vertices, and
widths as the fixed Orca oracle for ordinary, hole, degenerate, and KSR gap
cases, and default plus WASM builds must pass. If qualification fails, no
approximate centerline library or raster skeleton is allowed; the documents
must be amended to port the required fixed BSL Voronoi subset instead.

All scaling, f32/f64 conversions, integer truncation, safety offsets, and
ordering remain source-compatible. In particular, preserve
`INSET_OVERLAP_TOLERANCE=0.4`, `SMALLER_EXT_INSET_OVERLAP_TOLERANCE=0.22`,
`overhang_sampling_number=6`, `narrow_loop_length_threshold=10`,
`ClipperSafetyOffset=10`, the `-1/+1` inner onion correction, the 10-unit gap
safety and coverage values, `BRIDGE_INFILL_MARGIN=1 mm`, and the 0.05 mm
variable-width tolerance.

## Independent behavioral oracle

Before production GREEN work, an ignored, out-of-tree oracle must execute the
fixed v2.4.2 implementation and emit a versioned `ARES22O` behavioral wire.
The official v2.4.2 x64 portable release has SHA-256
`feba3009dfb9d268779cca5758a1a5bc3b7d0722bf8fa48d5c57340de975d6be`
and supplies independent provenance and negative replay evidence. Its exact
asset identity, commands, raw outputs, timestamp-only comparisons, historical
timeline, and same-input run-to-run differences are retained without further
normalization. Portable or historical G-code equality does not authenticate
the Task 22O payload. Internal generator data must come from the fixed-source
instrumented build, not from reverse-engineering seam-cut G-code.

Because ordinary fixed-source scheduling failed exact payload repeatability,
the only qualified retry is an explicitly oracle-only deterministic execution
mode. When a nonempty `ORCA22O_PAYLOAD_PATH` is present before process launch,
an explicit RAII runtime guard is constructed as the first statement of the
exported `orcaslicer_main`, after the launcher has returned from
`LoadLibraryExW` and before argument conversion, `CLI` construction, or any
slicing work. The guard captures that path exactly once.

Before constructing scheduler control, the guard first calls unchanged fixed
`set_current_thread_name("orcaslicer_main")` on the calling main thread. Fixed
`Thread.hpp:14-15` requires main-thread naming before workers are spawned
because the Windows thread-description API is initialized through a
non-thread-safe dynamic lookup; this early call reproduces fixed
`CLI::run:1191` without reading arguments or constructing `CLI`. The guard then
synchronously invokes unchanged fixed
`name_tbb_thread_pool_threads_set_locale()` once at ordinary arena concurrency.
These two fixed calls are the only permitted pre-control operations after path
capture.

Fixed `Print.cpp:2181` otherwise first invokes the pool-naming function inside
`Print::process`, while fixed `Thread.cpp:222-246` launches the arena-sized
worker naming/locale barrier and waits for every task. Priming that barrier
before constraining TBB sets its fixed function-local `initialized` state,
names workers, and sets their per-thread C locales; it must not read or mutate a
model, Option, CLI argument, or slicing state. Fixed `CLI::run:1191` may repeat
the same main-thread name, and its later pool call must be an immediate no-op.
An exception from either pre-control call fails closed with the dedicated
nonzero exit before CLI work. A priming call that does not return is an invalid
run that the external bounded supervisor terminates and retains as negative
evidence; no scheduler source may be patched to bypass the barrier.

Only after successful priming does the guard construct
`tbb::global_control(max_allowed_parallelism, 1)`. It must fail closed before
argument conversion or `CLI().run` unless
`global_control::active_value(max_allowed_parallelism)` is exactly 1. The
control then spans every argument-derived algorithm and the complete
`CLI().run`; it is destroyed after `CLI().run` returns but before
`orcaslicer_main` returns. Guard construction, main-thread naming,
ordinary-concurrency priming, control construction, and control destruction all
occur outside the Windows DLL loader lock. The exporter reads only the guard's
captured state; late environment activation is forbidden.

When the variable is absent, the same guard owns no scheduler control and the
same binary performs neither early naming nor pool priming and emits no payload.
No fixed Orca scheduler source is patched. The only allowed fixed-source
instrumentation paths are `src/OrcaSlicer.cpp`,
`src/libslic3r/CMakeLists.txt`, `src/libslic3r/PrintObject.cpp`,
`src/libslic3r/Ares22OOracle.hpp`, `src/libslic3r/Ares22OOracle.cpp`,
`src/libslic3r/Ares22OOracleRuntime.hpp`, and
`src/libslic3r/Ares22OOracleRuntime.cpp`. This mode selects one deterministic
schedule through unchanged fixed-source algorithms and Options. It is not
claimed to reproduce ordinary Release scheduling, the historical GUI run, or
the supplied final G-code.

The superseded unprimed deterministic build is required negative evidence. Its
first env-on KSR process reached only 0.25 seconds of CPU time, emitted no
payload or G-code, and was boundedly terminated after 141.808 seconds. It may
not be retried, selected, or counted as one of the two qualification runs. The
primed runtime requires a new clean build, a new env-off check, and two new
fresh env-on qualifications.

The composite wire is unambiguous little-endian framing:

1. eight bytes `ARES22O\0`;
2. a `u64` predecessor length and exactly that many released `ARES22N` bytes;
3. a `u64` payload length and exactly that many fixed-Orca payload bytes; and
4. exact outer EOF.

The fixed-Orca payload starts with eight bytes `ORCA22O\0`, then a little-endian
`u32` version equal to 1, followed by every object/layer/region slot, collection
nesting, loop role, inset depth, path role, coordinates, widths, heights,
volumes, gap entities, fill surfaces, and fill-no-overlap polygons. The parser
validates the predecessor as a complete `ARES22N` frame inside its declared
length, validates payload magic/version/canonical fields and payload EOF inside
its declared length, binds object/layer/region identities across both parts,
and then requires outer EOF.

The ignored packer prefixes the already released exact Ares `ARES22N` bytes;
only the appended Task 22O generator payload is exported from fixed Orca. The
`ORCA22O` version-1 payload schema and the outer `ARES22O` framing remain
unchanged. Execution mode is provenance, not generator output, so no scheduler
field is appended to the behavioral wire.

For the supplied KSR archive and each supported synthetic oracle-payload case,
two fresh qualified processes use fresh output and payload paths and fresh
clones of one hashed run-state base. Their fixed-Orca
payloads must be byte identical; because the predecessor is fixed, their
composite `ARES22O` wires must also be byte identical. No payload field, order,
coordinate, or byte may be normalized, sorted, omitted, majority-selected, or
chosen from one run after observing a difference.

Active-deferred probes are a disjoint Ares-only preflight-rejection set, not
fixed-Orca payload inputs. Each probe is a deterministic 3MF with every Option
inside the archive and exactly one active deferred condition. Package 0 freezes
the archive, Options identities, expected `UnsupportedProjectFeature` key, and
current `ARES22N` predecessor status, but it does not implement or claim a new
rejection. Existing earlier capability gates, including positive `raft_layers`
or `multi_region_layer_slices`, may already supply an observed key and may
reject before an N frame exists. Every newly specified Task 22O gate instead
records a null observed key and `pending_package_b`.

Package B must turn each pending definition into a RED and then an observed
exact-key GREEN before any Task 22O generator or checkpoint can run. No
`ORCA22O` payload or `ARES22O` composite may be created for these probes. Fixed
Orca must not be used to accept them because fixed upstream implements the
deferred branches. These probes authorize no production behavior.

A payload is valid only with a manifest that pins the fixed commit and tree,
the complete instrumentation source manifest and diff, the compiler and build,
the pre-launch execution mode, the observed active parallelism value, input and
datadir identities, exact parser and predecessor binding, and both run hashes.
For preflight-probe definitions, the manifest instead pins the archive and
Options identities, expected key, current predecessor status, absence of
`ORCA22O` and `ARES22O` output, and execution status. Package B replaces each
pending status with the observed exact key and rejection seam.
The same qualified executable must finish slicing in every oracle-payload run.
Corresponding G-code is liveness and diagnostic evidence only; record its raw
and timestamp-only hashes and diff classes without requiring or claiming
equality with another run, the portable binary, or the historical reference.

If the qualified payload pair differs, the oracle is invalid and Package 0
remains blocked. Any further execution-mode change requires another explicit
document amendment and independent approval.

Tracked tests store only behavioral expected values, synthetic vectors, and
the KSR wire length/SHA/semantic summary. They do not compile Orca, inspect
Orca source, assert source hashes or line text, or require the ignored oracle
artifact. Any remaining obsolete source-pinning test found in the touched
scope is deleted rather than updated.

The production path never reads the supplied G-code. Tests may parse it only
to assert downstream observations and to keep generator concerns separate from
seam, speed, arc, retraction, and G-code formatting concerns.
Nothing in this Package 0 authentication correction authorizes production
reference reads, fixture-derived constants, additional G-code normalization, or
a weaker later exact-G-code acceptance comparator.

## Deferred activated behavior

The following activated modes are outside Task 22O and must produce an explicit
project-stage unsupported error until a later source-cited slice replaces that
gate:

- Arachne dispatch;
- active fuzzy deformation or multi-region fuzzy masks;
- `detect_thin_wall=true` thin-wall medial-axis output;
- spiral keep-largest behavior;
- alternate extra wall and only-one-wall-first-layer;
- overhang reverse and steep-overhang reorientation;
- OuterInner and InnerOuterInner reorder modes;
- outer-only-brim forced reversal;
- active extra perimeters on overhangs;
- Filled or Bridges counterbore handling; and
- positive `raft_layers`, whose actual raft planning/generation boundary is not
  part of this Classic-generator slice; and
- multi-region compatibility merging beyond Task 22N's single-region gate.

These gates may not reject the supplied archive. Ordinary geometry-dependent
branches inside the KSR path, including collapse, holes, nesting, smaller
external width, overhang splitting, top splitting, medial-axis topology, and
gap filtering at zero, are fully in scope.

## Structure, platforms, and acceptance

Production and tests use real Rust modules. Every Rust source and test file is
below 400 physical lines. No `include!`, generated textual Rust, or
`include_bytes!` may be used to split source; binary fixture embedding is also
unnecessary because native and browser tests can hash the runtime checkpoint.
No new unsafe, filesystem access in `ares-core`, native-only algorithm,
reference-G-code branch, broad lint allowance, or legacy fallback is allowed.

The complete stage must run on WASM, Windows, macOS, and Linux. Native tests use
Cargo Nextest. Required release evidence includes focused synthetic and KSR
tests, full workspace Nextest, rustfmt, strict Clippy, default/all-feature
checks, both wasm32 checks, release WASM, Node syntax, and two Chromium runs of
the real archive checkpoint.

Success requires:

1. independently approved spec and plan before tracked implementation;
2. an independently generated and frozen complete Task 22O behavioral oracle;
3. RED-before-GREEN evidence for every package and every result-affecting
   option branch;
4. exact native and browser KSR `ARES22O` bytes and semantic totals;
5. mutation tests proving values come from 3MF Options rather than fixture
   identity;
6. no Task 22N predecessor drift and no old rectangular project fallback;
7. all source/test LOC and module-structure gates;
8. full Tier-1 verification on the exact candidate commit; and
9. one independent read-only reviewer validating requirement completeness,
   logic, edge cases, code quality, test coverage, and actual execution.

The final reviewer returns a concrete repair list to the main thread. The main
thread repairs it and sends the same complete frame back to that reviewer for
revalidation. That loop continues until the reviewer returns approval or a
specific external blocker is documented. Only then may Task 22O be committed,
pushed, and marked complete. The persistent full KSR exact-G-code goal remains
active after Task 22O.
