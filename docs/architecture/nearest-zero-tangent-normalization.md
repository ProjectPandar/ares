# Nearest-seam zero-tangent normalization prerequisite

## Upstream owner and Rust boundary

The owner is `GCode/SeamPlacer.cpp:1564-1593`, specifically the two Eigen
`normalized()` calls used to form `dir_to_middle` for concave inner-perimeter
seams (`:1584-1587`). The Rust destination is
`project_slice/seam_placement/runtime/nearest.rs::place_nearest_penalized`.
Reuse the existing `seam_placement/split.rs::normalized`, which already ports
Eigen's zero-vector behavior, instead of the nearest path's duplicate
unconditional division. No independent seam-placement algorithm is introduced.

Included: the source's zero-length tangent semantics, preserving the existing
f32 operations, candidate selection, concave gate, projection and split order.
Deferred: other seam penalties, candidate generation/closure representation,
staggering, interlocking geometry and thin-area arithmetic. The existing
nearest runtime is a temporary compatibility shell around SeamPlacer, not an
Ares-owned policy. No new public API, fallback, tolerance or guard is needed.

## Difference from the mesh-normal repair

The mesh-normal repair changes only
`seam_placement/mesh.rs::Vec3::normalized`, used by three-dimensional mesh
normal/occlusion sampling, owned by `TriangleSetSampling.cpp:67` and Eigen
`src/Core/Dot.h:92-102`. This slice does not call or modify that method.
It concerns the separate two-dimensional `(f32,f32)` closure inside
`runtime/nearest.rs`, called directly on `to_prev` / `to_next` when forming
`dir_to_middle` at `GCode/SeamPlacer.cpp:1584-1587`. A repeated closing point,
not a zero-area mesh face, supplies its zero tangent. Both bugs share Eigen's
normalization semantics but have distinct call sites and failure mechanisms.
With that repair in place, rerun this unchanged RED before claiming
an independent witness; changed occlusion scores must not be mistaken for a
repair of the two-dimensional local division.

## Observed failure mechanism

Interlocking localization found
matching input/output geometry before seam placement at the first remaining
G-code difference. Layer-44 runtime seam snapshots show both
implementations select the same concave candidate at
(-5.95703077, -3.07977605), with local angle -1.57079601 and staggering false.
The candidate range is [0,5); positions 0 and 4 are equal (closed contour).
For candidate 0, the previous tangent is therefore exactly zero.

Eigen's `normalized()` leaves a zero vector unchanged. The shared Rust
`split::normalized` does the same, but the nearest runtime's local closure
unconditionally divides both components by length. The resulting NaN prevents
its second projection from being constructed. Native splits the inner loop at
(-6806290,-3431555), while Ares retains the initial projected foot
(-6205774,-3680298). These are diagnostic values, not test expectations or an
output oracle.

## Witness refinement after the square-hole control

The first ordinary 8×8 mm square-hole nearest case passes unchanged main
(2/2 PASS with its existing AH1 control). It is not a RED and justifies no
production change. Refine the input to an authored 0.8×6.4 mm through-slot in
the same 64×48×4 mm board, rotated 22.5 degrees. These are explicit physical
model dimensions (the beam-width/angle scale of the observed short concave
edges), not copied Native coordinates or expected-output constants. Generate
the ASCII STL solely by transforming the existing authored holed prism's
vertices and normals; freeze its input SHA. Retain the square-holed aligned
control and all strict assertions. If this does not expose a public Native
witness, do not relabel a passing candidate as RED.

## Arrangement check for the slot candidate

The whole-board rotation candidate also passes unchanged main (2/2);
it is a passing control, not RED.
A debugger check of that exported input's first-layer candidates shows an
axis-aligned slot, so the export's `export-v2/arrange-1` recipe did not retain
the intended relative slot angle. It selected candidate 3, not the duplicated
candidate 0. Keep the board fixed and rotate **only the authored slot** by
-22.5 degrees (the observed source beam frame's unapply rotation), leaving
its physical dimensions 0.8×6.4 mm. This separates slot orientation from
whole-object arrangement. Freeze the revised input SHA, rerun the same strict
public witness before any production change, and do not use these diagnostic
candidates as test expected output.

## Standalone RED requirement and plan

This prerequisite must have a Native-observable RED with
the mesh-normal repair in place,
without any interlocking code or fixture. Use the existing authored
`tests/parity/multiline-union/holed-prism.stl`, ordinary aligned rectilinear
fill, classic two-wall perimeters and `seam_position=nearest`. Add only that
explicit seam choice to the existing input builder; all old controls retain
their previous settings. The existing hole/nearest representation should
exercise a concave candidate with a repeated closing point, but the actual
strict E2E result must establish the witness before any production edit.

The test must require a runtime Native oracle, verify exported settings,
assert Native hole clipping, compare complete Native/API/CLI output with the
existing strict generator-only comparator and retain raw API == CLI equality.
Artifacts include input 3MF, Native G-code, both Ares outputs and comparison
JSON; no Native output is committed. Run an existing aligned holed control as
well. Do not weaken assertions if this candidate does not isolate the defect;
find another source-owned public witness or keep the fix in the interlocking
chain instead of inventing a standalone GREEN.

## Observed standalone RED after the mesh-normal repair

With **no core production difference**, both official and seeded
runs of the new relative-slot case fail while the unchanged AH1 control passes.
The authored STL SHA is
`a5107761081b7afefe1ba1952548876ca93aead50b0c858ae5fcc5b50b74f88f`.
The API and CLI raw byte-equality assertion passes, but strict Native output
has 6579 lines versus Ares's 6573. The first differing move is line 587:
Native `G1 X108.424 Y106.459 Z.8`, Ares `G1 X108.831 Y106.29 Z.8`, before
returning to Z0.4. This is a public G-code witness independent of interlocking
and still present after the three-dimensional repair. The test expected
output remains runtime Native, not these recorded diagnostic literals.

The exact filter for both prescribed control scripts was
`test(nearest_slotted_rectilinear_inner_seam_matches_native) |
test(aligned_rectilinear_multiline::aligned_multiline_ah1)`.
GREEN now changes only the two-dimensional nearest normalizer; leave the test,
fixture, input SHA, raw equality and all strict comparisons unchanged.

After observing RED, GREEN may change production code only: replace the local
normalizer with the existing source-cited helper. Re-run the exact same test
and controls, require clippy **zero warnings**, rustfmt, then official and
seeded full suites with 0 FAIL. No raw
inherited WIP commits belong in this clean prerequisite slice.
