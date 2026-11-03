# ARD-0079: Overhang segmentation investigation

The demonstrated upstream boundary is
`libslic3r/GCode/ExtrusionProcessor.hpp::estimate_points_properties` and
`ExtrusionQualityEstimator::estimate_extrusion_quality`, called by
`GCode.cpp::GCode::_extrude`, with the floating
`AABBTreeLines.hpp::LinesDistancer::intersections_with_line` and
`Line.hpp::line_alg::intersection` dependencies. Rust destinations are the
existing `project_slice::gcode_emit::motion::overhang::sections` and
`geometry::line_distance_tree` modules. This is an upstream rewrite slice,
not a new Ares pipeline.

Include only demonstrated segmentation or scaling behavior responsible for
different public movement output. Defer M73 timing placement, later path
ordering, other random-generator families, and unrelated perimeter behavior.
Existing Ares emission scaffolding remains a temporary compatibility shell
around the upstream GCode concepts where parity has not been demonstrated.

Acceptance is complete ordered G-code with the existing generator/timestamp
normalization only. Diagnostic removal of M73 records cannot establish
acceptance. Do not replace the original KSR fixture, inject coordinates,
introduce fixture-dependent behavior, or retain private implementation pins.
Production repair requires a sensitive public minimized RED and subsequent
complete RED/GREEN output comparison against the approved native build.

## Current evidence

Baseline is commit `50fe97a6edd65fbbed850771eb96aa7f91f5a73c`. Strict KSR
replay exits100 in18.791s and retains complete input/native/Ares outputs in
`${TMP_ROOT}/ares-ard79-ksr-ordered-current/case-YF3e5N`. Both normalized outputs
have6321717 bytes; the first difference is M73 placement at line9946.
Ares raw output SHA256 remains
`ee645de0861e0d780ab1aeed653f2b7b1520cd032daa1d5702c54a91b440093d`.

A diagnostic traversal past M73 records identifies the first coordinate
witness at physical layer227/Z45.4, outer wall: native emits
`G1 X137.693 Y114.955 E.94471`, Ares emits
`G1 X137.693 Y114.956 E.94473`. This does not imply that only one movement
or only progress records remain different.

Read-only Rust debugger runs finish normally and capture the20 input points,
19 previous-layer boundary segments, and42 processed output points of this
outer wall. The fourth processed point is local `(4.653836,-1.036605)` with
speed50. Artifacts are `${TMP_ROOT}/ares-ard79-rust-overhang-input.log` and
`${TMP_ROOT}/ares-ard79-rust-overhang-boundary.log`; complete diagnostic outputs are
retained separately. Their full raw diff contains only the generated-header
timestamp change; movement and remaining records agree. The first native
capture attempt stops on an optimized
lambda variable and is excluded from causal evidence despite debugger
exit0. The second exact-signature attempt also stops on an unavailable
aggregate return address and is excluded. Third capture68259 completes0
with a normal inferior exit; `${TMP_ROOT}/ares-ard79-native-overhang-sret.log` retains
the structured capture and `${TMP_ROOT}/ares-ard79-native-sret-capture/plate_1.gcode`
retains complete output. Its20 raw path points unscale to exactly the Rust
input coordinates. Both return42 processed points, with three integer XY
differences at indices3,8,32. Index3 is native `(4653837,-1036606)` versus
Rust `(4653836,-1036605)` in scaled units. The mismatch already exists in
the estimator result, before downstream G-code coordinate formatting.
Native previous-boundary capture81624 now completes0 with a normal inferior
exit. All19 previous-boundary lines exactly equal Rust after unscaling.
The native42 floating extended points are retained in
`${TMP_ROOT}/ares-ard79-native-overhang-boundary.log`. Optimized Rust capture16069
cannot read the extended vector, and61443 cannot read the current point;
both stop early and are excluded. Unoptimized diagnostic build51115
completes0/17.90s, and capture56898 completes0 with a normal inferior exit.
Its complete output differs from the optimized output only by the generated
header timestamp. The optimized binary is frozen independently at
`${TMP_ROOT}/ares-ard79-optimized-ares` with the previously verified SHA256.

The Rust extended intersection at index4 is `(4.059867,-0.442636)`, whereas
native preserves `(4.059867949609615,-0.442636945390813)`. The following
segmentation point carries this upstream rounding difference. Baseline Rust
uses integer `LineDistanceTree::intersections_sorted` and unscales its result;
native uses floating `LinesDistancer<Linef3>::intersections_with_line`.
Capture alone was not treated as repair authority; the public complete-output
RED below was established before changing production code.

Exploratory closed six-vertex stepped-profile prisms with14 extension/angle
variants and24 additional angle/length variants produce only generated-header
differences. They are not sensitive REDs. Arrangement must disable rotations
with `--allow-rotations=0`; `--no-allow-rotations` is invalid. Export without
arrangement produces no loaded plate identity and is not a usable Ares case.
The24 angle/length fixtures also pass the existing strict comparator in
Nextest (0.283s); they are explicitly insensitive explorations, not proof
that the captured bug is fixed.

## Public RED and narrow repair

Native current-boundary capture98098 finishes normally. Reducing its lofted
outline to11 vertices preserves the failure; removing both orthogonal arms
to a four-vertex bar loses it. The two-layer closed mirrored loft at ordinary
input placement `(110,110.000105)` produces a complete strict RED in0.311s,
pair `${TMP_ROOT}/ares-ard79-loft-public-red/case-n1g9K2`. This input position exposes
the existing precision loss at the output rounding boundary; no production
coordinate injection or comparator tolerance is introduced.

Permanent live regression18041 finishes100/1.354s after all20 effective
settings and XY input placement checks pass. Pair
`${TMP_ROOT}/ares-ard79-permanent-public-red-output/case-9NZ2G7` differs at line209:
native `G1 X114.654 Y108.963 E.94471`, Rust
`G1 X114.654 Y108.964 E.94473`. Early test setup failures for the incorrect
`arc_fitting` option name and an inappropriate auto-drop Z assertion are
excluded; use canonical `enable_arc_fitting` and verify intended XY placement.

Repair keeps the existing tree and prunes by unscaled node bounding boxes;
only candidate boundary lines are converted to f64. Port the cited cross2,
EPSILON, segment-parameter and distance-sorted floating intersection behavior
without quantizing hits before segmentation. Existing integer queries,
speed bands, curl behavior and output scaling remain unchanged. General
3D/scarf intersection behavior is outside this demonstrated XY slice.

Regression73698 finishes0/1.425s at `${TMP_ROOT}/ares-ard79-public-green`, preserving
complete outputs with the unchanged ordered comparator. Clippy12304 and
Tier-142572 finish0, covering all four core targets and the WASM adapter.
Native wrapper SHA256 is
`3d05d4ccab740ca5e13aa94d0d00867a75f867ba2159b9f5b024dc0fcaa3bfaa`;
actual ELF SHA256 is
`8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d`.
This is the separately approved initialized/seeded reference, not an untouched
official build; the original official KSR fixture remains unchanged.

Original strict KSR69601 finishes100/42.184s, complete pair
`${TMP_ROOT}/ares-ard79-ksr-after-float-intersections/case-BV7aDL`. Both normalized
streams contain6321717 bytes and still first differ at M73 line9946. A
diagnostic traversal past progress records now reaches line221229/Z52 before
the first movement mismatch: native travels to `(150.165,101.366)` and Rust
to `(135.165,86.366)`, with different helical-lift I/J values. The earlier
Z45.4 coordinate/E witness is gone; this diagnostic is not strict acceptance.
Workspace72018 finishes0: 7460 PASS /9 skipped in256.939s, log
`${TMP_ROOT}/ares-ard79-workspace.log`. Eight live complete-output consumers6493
finish0/1.969s, with full artifacts at `${TMP_ROOT}/ares-ard79-public-consumers`.
The24 cached stepped fixtures also finish0/0.295s after the repair, retaining
outputs at `${TMP_ROOT}/ares-ard79-step-postfix-replay-artifacts`.
Fmt and diff checks finish0. Independent six-axis initial review finds no
required production repair. Final independent six-axis review approves the
ten-file patch with SHA256
`b9c5d742fc26de987437417fc84b5f9ba96e865078997c43671af6b310745ab5`
with no remaining required fixes; this review-status documentation update
is re-submitted before committing. Rust stable sorting does not claim the complete equal-key
ordering contract of native `std::sort`; duplicate same-point intersections
are indistinguishable to this consumer, which discards the line index.
Original KSR and global printer/option parity remain open.
