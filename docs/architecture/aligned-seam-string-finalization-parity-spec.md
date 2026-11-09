# Aligned seam-string finalization: source rewrite boundary

## Measured boundary

Upstream source is `345efec64c7b95a05ce5cf96df0dde66a158835c`. The Rust production tree is `8ba63961f0c42cea7b798b8d7e4082a88e334297`. Public comparison and first-endpoint context are described in [the painted split localization](paint-color-split-first-loop-start-localization.md).

Read-only observation of the existing seeded Native executable and existing Rust CLI distinguishes geometry from cached seam choice. Complete observed exports match their respective original complete outputs under the unchanged generator-line allowance. This is not an official-mode private observation or a repaired public comparison.

The first Inner wall enters Native `SeamPlacer::place_seam` and Rust `placement::place_loop` with the same single path: five exactly ordered integer XYZ points, including closure, at the four corners `(±9270002, ±9270002, 0)`. Both start at positive X/positive Y and proceed counterclockwise. Path role, width, height and extrusion volume per length also match exactly. No cyclic rotation, sorting, tolerance or coordinate transformation was used.

The first layer's five ordered candidate positions also match, at the corners `(±9.690000534057617, ±9.690000534057617, 0.10000000149011612)` with repeated closing point. Yet the cache consumed before projection differs:

| State | Native seeded | Rust |
|---|---|---|
| Selected index | 0 | 3 |
| Perimeter finalized | false | true |
| Selected XY | positive X, positive Y | positive X, negative Y |
| Final position | unfinalized candidate | stored aligned position at selected candidate |

Native's scaled pre-projection seam point is `(9690000, 9690000)`. Rust's selected and final-position XY are at the opposite Y corner. Rust's temporary final-position Z is optimized out at the observation site; its selected candidate Z and stored plan position are readable. No immediate Rust function-return observation is asserted.

Thus the first consumed loop geometry is equal, while aligned-mode choice/finalization already differs before projection or loop splitting. This does not establish equality of every layer's planner input or identify the earliest differing producer operation. Painted regions or later-layer perimeters may still influence cross-layer alignment indirectly. Neither painted-side segmentation nor a missing contour-continuation implementation is demonstrated as the cause by this observation alone.

## Owning upstream boundary

The consuming API remains `GCode.cpp::GCode::extrude_loop:8519` → `GCode/SeamPlacer.cpp::SeamPlacer::place_seam:2321`. Its finalized/candidate branch at2370–2384 consumes the state rather than constructing alignment strings.

The producer rewrite boundary is:

- `GCode/SeamPlacer.cpp::SeamPlacer::align_seam_points:1883`: immutable layer/perimeter geometry, stable initial ordering, alternative string starts, minimum string length, fitting and writes to `Perimeter::{seam_index,finalized,final_seam_position}`.
- `SeamPlacer::find_seam_string:1824`: forward/backward traversal using physical `slice_z`, the start perimeter's flow width and the preceding selected seam.
- `SeamPlacer::find_next_seam_in_layer:1739`: candidate traversal, finalized-state handling, comparator priorities, central enforcers and geometric-continuation restrictions.
- The called `GCode/SeamContour.hpp` and `SeamContour.cpp::{Contour,Layers,Options,Result,Resolver}` define immutable contour continuation. `Resolver::resolve` supplies the contour restriction consumed by `find_next_seam_in_layer`. This is an upstream dependency, not an Ares-owned matching algorithm.

`SeamPlacer.cpp::pick_seam_point:1339`, `SeamComparator` and `SeamPlacer::init` supply initial choices and priorities. Their output is an input prerequisite here; a mismatch before alignment belongs to its own earlier source boundary, not a finalization workaround.

## Exact Rust destination

Keep the rewrite private to `ares-core::project_slice::seam_placement`:

- `alignment.rs::{align,find_seam_string,find_next_in_layer,finalize_string}` and their existing scoring/choice helpers.
- `alignment/preparation.rs::{prepare,prepare_layer,PerimeterAssociation}` only for the existing producer handoff, initial-state observations and source-required immutable contour inputs. Do not change region grouping or entity membership.
- Source-owned contour continuation, if its responsibility is demonstrated, belongs in private `alignment/contour_continuation` modules matching `SeamContour::Resolver`, not a new public API, crate, configuration option or independent pipeline.

`LayerPlan`, `PerimeterChoice`, candidate vectors and collection associations remain temporary private representation shells around Native `LayerSeams`, `Perimeter` and `SeamCandidate` state. Reuse an existing equivalent source port only after checking its input/output contract. Split affected modules before they exceed400lines. Do not retain the old algorithm as an alternative execution path.

## Required producer measurements before replacement

1. At alignment entry, compare complete ordered layer/perimeter/candidate inputs for the target object: physical Z, contour geometry and role/reliability, initial choice/finalized state, flow/nozzle values, candidate flags and comparator priorities. Preserve all axes, repeated points and associations. Equality of only the first layer cannot satisfy this condition.
2. If inputs differ, stop at the earliest input-producing function. Record that boundary; do not alter alignment to compensate. An earlier visibility, context, candidate-generation, painted-region or perimeter difference is outside this replacement scope.
3. With equal relevant inputs, locate the first different string transition or finalization decision: source/target contour, preceding candidate, direction, radius, nearby traversal order, continuation result, chosen candidate, alternative start and string-length threshold. Record an input-equivalent counterexample before changing its Rust kernel.
4. Replace only the responsible upstream kernel and necessary called source behavior. A static difference in continuation code is insufficient. If fitting is responsible, preserve its source arithmetic and cached-state writes rather than forcing a desired loop corner.

The present observation stops before these all-layer producer measurements. No particular kernel replacement is yet demonstrated. This boundary prevents a first-layer skip, forced index0, reordered candidate list, special case for painted objects or emission-time nearest-seam substitution from masquerading as an aligned-seam rewrite.

## Included and deferred behavior

Included is source-equivalent aligned seam-string construction/finalization for the demonstrated responsible kernel, with physical layer coordinates, immutable contour restrictions, ordered candidates, stable ties, source thresholds and exact state propagation. Keep valid no-continuation behavior exactly as upstream specifies; do not introduce a separate compatibility branch.

Deferred are new seam modes, random/nearest redesign, region/island grouping, painted segmentation, perimeter geometry generation, nozzle/configuration mapping, G-code formatting, projection/splitting changes, estimator/M73 behavior, cache architecture and viewer logic. A differing input owned by one of these boundaries requires its own source-defined work rather than scope expansion. No Native rebuild, instrumentation or inferior assignment/call is part of the measurement protocol.

## Tests

Use the existing public E2E contract before and after any replacement. Both modes' original `paint_color_native_preflight` finish before Ares legs; keep positive painted activity, count/export and Native double-replay checks. Run `paint_color_split` separately per mode and retain its complete generator-only ordered comparisons over headers, configuration, time, M73, tools, roles and motion. No geometry-only substitute or added mask is permitted.

Run the distinct existing `paint_color_unpainted` and `paint_color_leaves` controls in the same environment. Preserve API and CLI raw products and actual nextest exits. The present baseline is preflight1pass/exit0, target1failure/exit100 and controls2passes/exit0 per mode. External target API/CLI raw equality does not mean the helper's later assertion ran after its Native/API failure.

A replacement must reproduce the measured source-equivalent internal transition, restore the complete target comparison in both modes and preserve controls without weakening assertions. First-loop or scoped internal agreement alone is not complete-file parity. Use source-derived/live Native data, never new coordinate, count, exit or output goldens. Build checks include formatting, strict workspace/all-target Clippy and browser WASM; core changes remain platform-neutral.

## Failure modes

1. Treating equal first-loop geometry as equal all-layer planner inputs hides later-layer producer differences.
2. Ignoring `finalized` confuses a selected candidate with a fitted aligned position.
3. Rotating, sorting or dropping the closing point changes candidate identity or stable tie behavior.
4. Comparing pre-seam Native with already placed Rust loops mistakes the effect for an input difference.
5. Replacing a physical `slice_z` with print Z changes cross-layer search inputs.
6. Omitting contour role/reliability, nozzle bounds or continuation outcome changes admissible strings.
7. Initializing nearest/best from an excluded or finalized candidate changes the source traversal semantics.
8. Changing forward/backward restart or alternative-start traversal changes string length and finalization.
9. Fusing arithmetic or changing float precision, thresholds or stable comparison ties changes choices.
10. Forcing an unfinalized first layer or a chosen corner repairs one output without porting its owner.
11. Reading optimized-out fields as zero invents input equality; failed/partial observations remain distinct.
12. Calling build success, unchanged controls or raw API/CLI equality a target parity pass hides the still-failing complete comparison.
