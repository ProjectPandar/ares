# Honeycomb lattice parity slice

## Upstream boundary and destination

Port `FillHoneycomb.cpp::_fill_surface_single` and
`FillHoneycomb.hpp::_layer_angle` into `ares-core/src/fill/honeycomb.rs`.
The existing generator is a temporary compatibility shell for those
upstream functions, not a separately designed Ares fill algorithm.

Included:
- Preserve the scaled integer/floating arithmetic used by CacheData.
- Use the upstream 60-degree, three-layer rotation sequence, respecting
  fixed-angle, bridge-angle and thickness-layer inputs.
- Pass `5 * spacing` directly to polyline simplification, as upstream does;
  do not multiply this tolerance by the coordinate scale.

Deferred: generic clipping/connection changes, cooling or time estimation,
other fill patterns, configuration coverage expansion and oracle stability.

## Evidence and plan

Strict sweep at `033f4a40` produces ~110888 reference versus ~94722 Ares
bytes for honeycomb. `/tmp/honeycomb-current.diff` and paired artifacts at
`/tmp/parity-adaptive-trunc-full/case-nsJaS9` show missing hexagon corners,
long multi-cell chords and resulting cooling/M73 changes at Z1.2.

1. Red regressions at the real fill seam: three-layer periodicity and
   preservation of hexagon corners with connection anchors disabled.
2. Correct source-derived arithmetic, rotation and tolerance; check both
   coordinate scales and override behavior in separate tests.
3. Run strict live honeycomb comparisons, workspace and KSR, fmt/clippy.
4. Independent read-only review across requirements, logic, edges, quality,
   tests and actual evidence; fix and re-review, then code+docs commit/push.

No output-tolerance acceptance, production reference lookup, or unexplained
short-path deletion is permitted. Whole-project completion still requires
all-printer/all-option verification, not this one fill pattern.

## Verification

`/tmp/honeycomb-red.log`: both original-seam tests fail (incorrect layer
period and multi-cell chords). `/tmp/honeycomb-green.log`: four tests pass,
including bridge/fixed-angle/thickness overrides and a grid vertex from
standalone C++ CacheData arithmetic (`/tmp/honeycomb-grid.cpp`). The latter
uses spacing 0.40707963705062866 and density 0.15f: distance=2713860,
x_offset=203539, y_offset=117513. Its origin-aligned test domain avoids
confusing accumulated integer row pitch with the cache's local vertex.

`/tmp/honeycomb-oracle.log`: the honeycomb value now passes strict ordered
bytes; the filtered option domain correctly remains INCOMPLETE, because
25 other values were not requested. Full paired artifacts are under
`/tmp/parity-honeycomb-fix`. `/tmp/honeycomb-workspace.log`: 7315 passed,
10 skipped; `/tmp/honeycomb-ksr.log`: 84 passed, 2 skipped. Fmt/clippy complete
with existing warnings. Touched Rust files are 225 and 104 lines, with tests
in a separate module. This verifies the scoped correction, not the complete
printer/option inventory or oracle determinism.

## Follow-up: multiline offset units

`FillHoneycomb.cpp:77` supplies millimetre spacing to
`FillBase.cpp::multiline_fill:2712–2784`, which scales each ring offset once.
The Rust destination is the `multiline_offset::apply` call in
`fill/honeycomb.rs`; that helper already owns conversion to coordinates.
Remove the caller's duplicate division by the coordinate scale. No helper,
connection policy or other pattern changes are included. Generic multiline
Clipper2 fidelity and all-printer/all-option coverage remain deferred.

Plan: reproduce erased even bundles at the actual fill seam, correct the
single argument, exercise even/odd bundles at both coordinate scales, then
run the real oracle for honeycomb with fill_multiline=2,3,4. Preserve complete
paired bytes regardless of result. Run workspace/KSR, fmt/clippy and obtain
independent review before committing this code and documentation together.

Red evidence: `/tmp/honeycomb-multiline-red.log` fails because multiline=2
produces no paths in a 20 mm square. `/tmp/honeycomb-multiline-green.log`
passes all five honeycomb tests after the unit correction. The regression
also requires a minimum total path length. This is a geometry regression,
not a substitute for strict complete G-code comparison.

The initial live run was invalid evidence: the generic override helper
ignored fill_multiline because that key was absent from the selected preset.
Artifact inspection caught exported fill_multiline=1 for all three labels.
The new test now inserts the value into the owning process preset and
asserts both exported keys before slicing. Only the verified rerun at
`/tmp/parity-honeycomb-multiline-verified` is relevant: values 3 and 4 PASS
strict ordered bytes; value 2 remains DIVERGENT across three references.
Its first reference differs at line 747 in rounded corner coordinates and
extrusion, not merely metadata or timing (`/tmp/honeycomb-multiline-residual.diff`).
The new opt-in test deliberately fails until that residual is resolved;
this scoped unit fix does not establish full multiline parity.

`/tmp/honeycomb-multiline-oracle-verified.log` preserves the failed result.
`/tmp/honeycomb-multiline-ksr.log`: 84 passed / 2 skipped. Clippy completed
with warnings (`/tmp/honeycomb-multiline-clippy.log`). No new runtime or
platform dependency was added; production change is one argument only.
Final workspace log `/tmp/honeycomb-multiline-workspace-final.log` records
7323 passed / 10 skipped; the opt-in live test is inactive in that run.
Independent six-axis review `a3178d08` found no scoped issues and approved
the unit correction only. Count-2 corner fidelity remains an explicit
follow-up; neither the review nor this commit accepts whole-goal parity.
