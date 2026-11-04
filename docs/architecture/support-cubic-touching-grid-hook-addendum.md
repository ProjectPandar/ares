# S-C0 localization STOP: hook inputs precede the grid panic

## Disposition

Docs-only SPEC amendment for the public S-C0 RED. The original SPEC
requires stopping if the first divergence is outside its connector/grid
boundary; this observation triggers that clause. Do not remove the
touching-grid assert merely because upstream release assertions are compiled out.

## Frozen observation

Exact S-C0 SHA256 remains
9580fd4f17fad55829f561c13ee0ae336b447016c7c9682ce301f79afc9d0bc1.
Read-only gdb used the built Ares CLI and canonical official Native.
No inferior calls/writes, probes, code edits or compilation were needed.

Ares fails on layer_index3, print_z0.8, from
`fill/adaptive/filler.rs::fill_surface` into `connect_infill` and
`mark_boundary_segments_touching_infill`. Its first failing trace starts at
(-3672296,9736935), with grid min(-9485403,-9485403), max(9485408,9485408),
resolution10692035, rows2/cols2. Although Y exceeds the geometric bbox, its
computed cell is in range (row1,col0). This alone does not justify removing
an assertion: Native must actually receive the same trace to make that claim.

Native's 1246 observed calls to the touching visitor's specialized grid
walker all have endpoints inside its bbox. The source-line breakpoint attempt
was unresolved and is excluded from this claim; symbol-entry captures and
source/assembly-validated argument layouts supply the actual trace evidence.

For the second connector invocation, both sides have bbox
(-9019688,-9019688)..(9019693,9019693), spacing0.40707963705062866mm, and ten
ordered input polylines. Nine polylines are identical. The sixth differs:

- Native front endpoint: (-4835402,7071064).
- Ares front endpoint: (-3207077,9891390), already beyond the fill boundary.
- Both have hook_start(-4021240,8481227) and following point(4021253,8481227).
- The last endpoint differs by one coordinate unit as well.

This is an input difference before FillBase's clipping/thick-trace code.
The earliest *inspected collision-decision* difference is in `add_hook`:
Native forward=621767.7732422437, backward=1628318.5, hook_length=1628318.5;
Ares forward=hook_length=1628318.5482025146. Ares backward is optimized out
and is not claimed. Both direction components agree:
(0.5000019134345626,0.8660242990596604).
The initial equal-length-choice hypothesis is therefore unproven and cannot
explain away Native's smaller forward limit. No trial production edit was made.

The first captured connector call also has a bbox/input difference of about
0.20354mm per side. Its earlier cause and relationship to output are not
localized here. This report does not claim the inspected hook is the first
global pipeline divergence or that fixing it alone establishes strict parity.

## Source boundary amendment requested

All upstream citations are in `OrcaSlicer/src/libslic3r/`:

- `Fill/FillAdaptive.cpp:1398-1401`: scaled hook lengths finish through
  `std::min<float>` before widening to coordf_t; call `connect_lines_using_hooks`.
- `Fill/FillAdaptive.cpp:801-1262`: `connect_lines_using_hooks` owns the
  source/working lines and collision index, including source insertion
  at870 and contour/hole segment insertion at1038/1044.
- `Fill/FillAdaptive.cpp:600-612`: `Intersection::other_hook` and its intersection.
- `Fill/FillAdaptive.cpp:670-778`: `add_hook`; start intersection695-698,
  other-hook700/713, finite forward query705-711, exclude-the-intersecting-line
  filter708, closest thick-line distance718-751, reverse query757-763,
  longer direction and integer displacement-before-addition767-768.
- `Fill/FillAdaptive.cpp:1410` dispatches the resulting polylines into
  `FillBase.cpp:1580-1594`; graph/touching remains at1432-1565/996-1241.
- `EdgeGrid.hpp:172-201` visits the start cell after translating/dividing;
  its stepped-cell bounds exit is at227-230 (and the other direction branches).
  NDEBUG removes assertions, but does not modify the incoming hook geometry.

Requested Rust boundary: extend this slice to the existing
`crates/ares-core/src/fill/adaptive/hooks.rs::add_hook` and the mutable segment
collision data owned by `connect_lines_using_hooks`; use the existing
`hooks/geometry.rs`/`hooks/pair.rs` only where the cited collision operation
requires it. Source hook-length completion belongs in
`fill/adaptive/filler.rs::fill_surface`. These are temporary compatibility
shells for the cited upstream hook/index algorithm, not a new Ares connector.

Before production, localize which actual collision candidate or self-hook
limits Native's forward length and which source-owned candidate is absent
or evaluated differently in Ares. Port that specific operation, including
its source precision/cast order; do not treat the observed scalar as a
constant, patch this fixture, or assume the visible tie/cast differences are
the sufficient fix. If the cause requires geometry, ordering or caller
changes beyond these functions, the boundary must be revisited.

## RED reuse, failure modes and acceptance

Reuse the frozen public S-C0 RED unchanged. First failure modes to
check are missing mutable connected segments, missing other-hook collision,
incorrect excluded source-line identity, finite query extent, thick-line
clearance, reverse-query condition, scaled-float completion and displacement
cast order. Check actual data before choosing an implementation.

Retain grid assertions unless equivalent Native
trace evidence justifies a separately explained removal. Preserve release
cell visitation and ordered edge processing; do not enlarge bbox, clamp or
skip traces, catch panic, or add guards. No frozen Native expected output,
unsafe, new suppression, or weakened assertions. Every Rust file <=400LOC.
The requested stable test comment belongs in the GREEN commit: describe
CLI/API evidence capture order without referencing the current panic.

Require S-C0 strict whole-output and raw API==CLI checks in both modes, then
existing B-S ten-case and connector/Adaptive controls, strict/fmt/wasm32.
If the earlier input difference or any output residual remains, do not
broaden the patch. Object-region spacing stays blocked. A-C1/C2-C4
Inner-wall remains separately deferred.

## Observed runs

All observed Native runs completed result0 and match the original public
reference line-for-line except the generator date on line2; no motion or
configuration differences. Ares read runs reproduce the original exit101.
