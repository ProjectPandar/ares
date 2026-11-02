# Aligned-seam output for a rotated object's first layer

Status: public regression; the original RED is closed by the reviewed
TriangleMeshSlicer port integrated in source commit
`3d11c2284553266db1113a353adc74eafcbbeeca`. No seam-placement production
change or new seam strategy is part of this regression.

## Upstream boundary and Rust destination

Upstream is OrcaSlicer v2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`.
The causal producer boundary is `src/libslic3r/TriangleMeshSlicer.cpp`:

- `coplanar_edges:169-240` classifies artificial triangulation junctions;
  `chain_lines_by_triangle_connectivity:1180-1287`, especially1257-1262 and
  1282-1287, suppresses those junctions while retaining real corners.
  Rust destinations: `mesh_slicer/coplanar.rs`, `mesh_slicer/chaining.rs`,
  and the coplanar carrier setup in `project_slice/raw_intersections.rs`.
- The canonical raw-line tuple sort at623-635 stabilizes the input order
  used by `make_loops`. Rust destination: `mesh_slicer.rs::slice_mesh_on_planes`.

The downstream witness remains `src/libslic3r/GCode/SeamPlacer.cpp`:

- `calculate_polygon_angles_at_vertices:260-308` and
  `process_perimeter_polygon:673-736` generate the ordered candidates and angles.
  Rust destination: `project_slice/seam_candidates.rs::{process_polygon,vertex_angles}`.
- `compute_angle_penalty:82-89` and
  `GlobalModelInfo::calculate_point_visibility:392-418` supply the score terms;
  `SeamComparator::is_first_better:1178-1221`, especially1214-1221, combines
  overhang, visibility and angle importance with a strict comparison.
  Rust destination: `seam_placement.rs::{candidate_penalty,angle_penalty}`,
  `visibility.rs::GlobalVisibility::at`, `alignment.rs::is_better`.
- `pick_seam_point:1339-1350` retains the first strict minimum.
  Rust destination: `alignment/preparation.rs::best_candidate`.
- `align_seam_points:2052-2062` computes the sharp-angle blend; Neutral
  candidates with `t>=1` retain `current_pos`, otherwise `place_on_loop` runs.
  Rust destination: `alignment.rs::finalize_string:275-288` and the already
  ported `alignment/loop_normal.rs`; later loop splitting is in `placement.rs`.

This is a source-cited parity regression, not a new Ares seam strategy.
The observed unequal producer was the incoming ordered ring, before seam
candidate construction. The accepted TriangleMeshSlicer rewrite replaced
that earlier mismatch; no comparator, angle/visibility policy, loop-normal
solver or compatibility shell is changed by this test.

## Non-negotiable decisions

- Compare runtime Native against the public
  `ares_core::slice_project_plate_validated` API, selected plate1, default
  overrides/context0. Require whole-export Native double stability first,
  normalizing only the shared generator identity. No Native goldens or fixed
  Native output counts are committed.
- Preserve the entire first-layer rotated-object block from its printing label
  through its stop-object terminal, including inherited entry wipe, both walls,
  bottom surface, XYZ/E, travel/feed, TYPE/WIDTH, M73 and all retract/wipe records.
  No filtering, tolerance, sorting, cyclic rotation or coordinate-only acceptance.
- Save complete exports and the unfiltered first layer externally. Preserve
  the historical other-object M73 differences as historical evidence, not
  filtered acceptance. The regression asserts complete-object equality;
  whole-export validation receipts do not close unrelated families or FULL.
- Preserve candidate order, ring origin, closing duplicate, raw numeric inputs,
  strict comparator priorities and the sharp branch. No max-Y preference,
  epsilon, score bias, forced candidate, geometry compensation or fallback.
- No private solver test, copied planner expectation, diagnostic public API,
  production instrumentation or platform-specific repair. Any later core port
  must remain portable to browser WASM, Windows, macOS and Linux.
- Attribute closure to the reviewed TriangleMeshSlicer source boundary,
  not to an inferred equal-score tie or a new seam repair. Any later core
  change still requires its own source-cited causal review and authorization.

## Frozen input and observed qualification

Reuse the GUI-exported platerSession input without changing any ZIP payload:
`tests/parity/aligned-seam-rotated-object/rotated-two-cubes.3mf`, SHA256
`350250ac3b6f54b8ef79949d3232980bb4cd2a7b3bdbae74f71b3b62aa9faf79`.
It holds an unrotated cube and a45-degree cube centered at110,95.5 on plate1;
`seam_position=aligned`. The test is
`option_coverage::aligned_seam_rotated_object::rotated_object_first_layer_aligned_seam_matches_native_output`.

Prior qualification on `1cf2968d5` established
official Native double stability. A separately authorized same-ELF API recovery
reused those outputs with explicit reused receipts, adding no Native stability
credit. Its completed166805B API output exactly matches the retained Browser
witness SHA256
`317f955f709f8ae0fb1e55285a57778aadf453906afa253475443d2873e17f16`.
The full-export first difference is Native Y101.569 versus API Y89.431 at line126.
This historical observation is not a substitute for the new RED's runtime oracle.

Read-only API capture observed the first ring in centered integer coordinates:
`[(6632662,0),(0,6632662),(-6632661,0),(0,-6632663),(6632662,0)]`.
Its origin is right, not bottom; it is not exactly symmetric on the integer grid.
Candidate3 has score bits`0x3fa86efc`, versus`0x3fa86efd` for the other candidates,
with zero overhang and embedding. Thus this API choice is not an equal-score
first-wins tie. Its actual sharp-test and return checkpoints show candidate3,
blend1.0 and current position`(0,-6.632662773132324,0.10000000149011612)`:
the normal-field solver is bypassed for this target.

A separately approved seeded observation, qualified against the complete
saved official export with only generator-line normalization, observed Native's
incoming ring:
`[(0,6632662),(-6632661,0),(0,-6632661),(6632662,0),(0,6632662)]`.
The historical API ring differs in origin and bottom Y before any candidate
scoring. Native's actual sharp-test and return hit layer0/candidate0 at the
top with blend1.0. Initial Native scores and initial-pick winner were not
captured; no tie, angle or visibility cause is inferred.

The earlier producer was confirmed as TriangleMeshSlicer coplanar-junction
suppression and canonical raw-line sorting, already ported in
`85f43af4528f463c07ceb48e25c7f729eb201005` and
`81f82a49d766e7c47fae606bccf59b417463ca06`.
A read-only first-layer trace on the latter source commit recorded eight
raw segments becoming a four-corner, top-origin chained loop, then a final
seam input exactly equal to Native's ring. A transient right-origin union
contour did not persist into the final external loop. No ablation or further
Native probe was needed or performed.

## Merged-source regression qualification

On source commit `3d11c2284553266db1113a353adc74eafcbbeeca`, tree
`520e55cc10f92bbebfa31d131824ea199c8fe289`, the unchanged accepted RED test
and fixture were overlaid without a production delta. One selected test ran
in each of ordinary official and explicitly seeded modes: both passed,
none ignored, no retry. Each mode generated two fresh Native exports with
whole-export double stability before one public API call.

Complete rotated-object equality passed in both modes, and the retained
whole-export comparator also passed, preserving all M73 and other records.
The closure is this public witness on the merged source, not a new seam core
port, broad family/FULL certification or a substitute for the runtime oracle.

## Included and deferred

Included: one public complete-object regression, runtime selected-plate Native
double, strict output comparison, exact existing input, and source-cited
closure by the TriangleMeshSlicer port.

Deferred: new production changes, all-layer/family/FULL closure, unrelated
M73/timing, generic travel, raw junction WIDTH, sloped-Z, painting/PreciseSeam
and unrelated geometry or score rewrites.

The original API observer's epilogue-scope failure remains an INCOMPLETE attempt;
the successful recovery does not relabel it. Both later offline50-layer grouping
validators failed and that direction was stopped. No verified all50-object
grouping or closure is claimed. The source-commit-qualified external evidence
bundle retains the original incomplete attempts, successful recovery, Native
byte qualification, producer trace, and both merged-source regression runs.
