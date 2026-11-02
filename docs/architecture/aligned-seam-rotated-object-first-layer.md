# Aligned-seam output for a rotated object's first layer

Status: public RED only; no production port or GREEN authorization.

## Upstream boundary and Rust destination

Upstream is OrcaSlicer v2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/GCode/SeamPlacer.cpp`:

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

This is a source-cited parity witness, not a new Ares seam strategy. No existing
destination is designated as a compatibility shell to replace until the first
unequal Native/API producer is observed. In particular, this RED does not
authorize another loop-normal port or a changed comparison policy.

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
- Save complete exports and the unfiltered first layer externally. Its two later
  M73 relocations around the other object remain separately owned; focused
  object equality is not whole-layer, whole-file or family closure.
- Preserve candidate order, ring origin, closing duplicate, raw numeric inputs,
  strict comparator priorities and the sharp branch. No max-Y preference,
  epsilon, score bias, forced candidate, geometry compensation or fallback.
- No private solver test, copied planner expectation, diagnostic public API,
  production instrumentation or platform-specific repair. Any later core port
  must remain portable to browser WASM, Windows, macOS and Linux.
- GREEN requires a separately reviewed causal source boundary and production
  authorization. This document records the owning score and sharp functions;
  it does not establish which producer must change.

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

Native internal rings, angles, visibility and scores remain unobserved. Do not
assign a Native winner index or claim an angle, visibility, geometry or ordering
producer as the cross-implementation cause from printed corners alone.

## Included and deferred

Included: one public complete-object RED, runtime selected-plate Native double,
strict output comparison, exact existing input and this upstream-boundary ARD.

Deferred: causal Native/Rust operand comparison, production changes, GREEN,
all-layer/family/FULL closure, M73/timing, generic travel, raw junction WIDTH,
sloped-Z, painting/PreciseSeam and unrelated geometry or score rewrites.

The original API observer's epilogue-scope failure remains an INCOMPLETE attempt;
the successful recovery does not relabel it. Both later offline50-layer grouping
validators failed and that direction was stopped. No verified all50-object
grouping or closure is claimed. The external evidence bundle for source
commit `1cf2968d507837c300c761f04af2e8cde649430a` retains both the original
incomplete attempt and the explicitly approved successful API recovery.
