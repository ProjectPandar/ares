# SupportCubic B-S S45 residual: localization amendment

Localization amendment; not GREEN. The B-S RED and B-T slice remain
prerequisites; no additional production change is proposed here.

## Frozen attempt and public evidence

The attempt ports PrintObject.cpp:972-1007/2728-2735 object preparation:
configured double spacing, original ordered candidate identities and float
bottom_z, runtime GLU triangles, Support Vec3f mesh/bbox and true overhang
filtering, shared per-slice tree for anchoring and emission. Ares surface
coordinates include XY placement; bridge points subtract the same object
center before rotation, keeping double coordinates separate from mesh casts.
This explains the implementation boundary, not a claim that all intermediate
Native/Ares state has now been measured equal.

The existing bridge candidate type is re-exported only inside project_slice.
The production change builds without dead_code warnings in the non-test
CLI/core compilation.

With distinct fresh Native cache/artifacts per mode, official and seeded each
run 10 tests: 9 PASS/1 FAIL. No retry or comparator/fixture/assertion edits.
All 40 Native export/slice commands exit0; all 20 public APIs admitted/CLI exit0;
API and CLI raw SHA identical per case, with matching layer markers/line counts.

S50/S125/SAUTO now strict PASS in both modes. The six unchanged controls
A45/A50/A125/AAUTO/R1/L1 strict PASS in both modes, with all12 API and CLI
raw outputs identical to their RED baselines. Every regenerated input
has identical uncompressed ZIP members to RED; only export timestamps differ.

S45 remains a deterministic complete-output FAIL in both modes. First difference
is byte64438, layer13 (print_z2.6, height0.2), line2607, X rounding. Both modes
have5425 lines. Outside the existing generated-by metadata distinction, there
is exactly one differing raw G-code line. The original RED first difference
was layer4/line1418; that difference is gone. This is partial progress, never
a parity PASS. No new guard, emitter rounding fix or tolerance is in scope.

Official S45 input SHA
fe76c98e5e3e4d698968bb81a1a76a91ef81b1faa433af26138131342b6b16cf;
seeded S45 input SHA
0f77beb88fc7d9b77834829269db845800f330390b4b41e1b6f9a1fe7bf0febe.
Inputs/reference streams are runtime artifacts, not committed Native outputs.

## Proposed bounded read-only localization

Use the established read-only GDB flow and the frozen S45 projects above;
one fresh Native official and seeded capture each, plus the already compiled
Ares CLI. No source probes, inferior calls/writes, numeric substitutions,
sorting/chaining changes, Native rebuild or store deletion. Record actual
inferior exit status and preserved stdout/stderr, not just GDB exit status.
Unavailable optimized locals remain unavailable rather than reconstructed
and claimed as actual observations. No new tests are needed for this localization.

Find the earliest measured divergence in this ordered chain:

1. PrintObject.cpp:977/984/995-1007 and FillAdaptive.cpp:1495-1527:
   spacing, transformed float mesh/bbox, original double bridge triples,
   root/ladder and relevant child centers. Compare source-normalized coordinate
   frames explicitly; do not compare world Ares points to local Native points.
2. FillAdaptive.cpp::generate_infill_lines_recursive:450-510 and
   Filler::_fill_surface_single:1321-1388: layer2.6 inputs, ordered cube-wall
   segments, scaled integer endpoints, clipping output. Source point creation
   versus adding object translation is a hypothesis to check, not a conclusion.
3. FillAdaptive.cpp:1397-1413: ordered points before/after hooks and final
   chain_or_connect_infill. Preserve all points and order in runtime evidence.
4. Only if geometry endpoints match, read GCode.cpp:8124-8127
   point_to_gcode and GCodeWriter.cpp:933/1224 emit_axis inputs to distinguish
   local integer point/origin/extruder offset/double formatting values.
   This is observation only: G-code changes are deferred from B-S.

Corresponding Ares observation destinations are
fill_entities/adaptive/preparation.rs::prepare_support,
fill_entities/adaptive.rs::build_object_octree, fill/adaptive/octree.rs,
fill/adaptive/filler.rs::generate_infill_lines_recursive/::fill_surface,
fill/adaptive/hooks, and finally gcode_emit/motion/state.rs/format.rs.
Existing bridge/fill carriers remain temporary compatibility shells around
the cited libslic3r boundary. No new API or pipeline is proposed.

Exit: report first measured divergence with exact source ownership and a
minimal source-cited implementation amendment before changing production.
If the mismatch is outside object preparation, it becomes a separate upstream
boundary. If not located, STOP with observations; no indefinite expansion,
retries or downstream patch to manufacture GREEN. Strict/fmt/WASM and the B-T
directed rerun are not claimed as final gates.
