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
