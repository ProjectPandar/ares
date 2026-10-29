# SupportCubic B-S S45 residual: localization amendment

Status: STOP / SPEC-REVIEW-REQUEST, docs only; not GREEN or merge-ready.
The approved B-S RED e27170322 and B-T chain remain prerequisites to a single
stacked merge. No partial merge or additional production change is requested.

## Frozen attempt and public evidence

The attempt ports PrintObject.cpp:972-1007/2728-2735 object preparation:
configured double spacing, original ordered candidate identities and float
bottom_z, runtime GLU triangles, Support Vec3f mesh/bbox and true overhang
filtering, shared per-slice tree for anchoring and emission. Ares surface
coordinates include XY placement; bridge points subtract the same object
center before rotation, keeping double coordinates separate from mesh casts.
This explains the implementation boundary, not a claim that all intermediate
Native/Ares state has now been measured equal.

Initial commit9e7a175fe had an internal private-module import error; root
/tmp/worker3-support-b-s-green-fzk3bpu6 exit101, zero Native/tests, not GREEN.
The sole visibility fix27535f18d reexports the existing bridge candidate type
only inside project_slice. Per coordinator ruling, only these two production
commits were squashed into1fc075c55415294b9a3161e4c049e303f3be73fb.
Tested27535f18d and squashed1fc075c55 have identical tree
238e354e025e5886b1c55aac7a5e86082c9db6ac; git diff exit0.
RED/B-T/docs history was untouched. The production commit builds; no dead_code
warnings occurred during the actual non-test CLI/core compilation.

Fresh root /tmp/worker3-support-b-s-green-nmjt0eon, Rust1.99.0/default test
opt1, CPU12 outer taskset, jobs1/threads2, distinct fresh Native cache/artifacts
per mode. Official15.575s and seeded22.745s: each10 tests,9 PASS/1 FAIL,
nextest100. No retry or comparator/fixture/assertion edits. All40 Native
export/slice commands exit0 and reaped; all20 public APIs admitted/CLI exit0;
API and CLI raw SHA identical per case, with matching layer markers/line counts.
All67 sampled descendants used CPU12, including28 sampled actual Native jobs.

S50/S125/SAUTO now strict PASS in both modes. The six unchanged controls
A45/A50/A125/AAUTO/R1/L1 strict PASS in both modes, with all12 API and CLI
raw outputs identical to their approved RED baselines. Every regenerated input
has identical uncompressed ZIP members to RED; only export timestamps differ.

S45 remains a deterministic complete-output FAIL in both modes. First difference
is byte64438, layer13 (print_z2.6, height0.2), line2607, X rounding. Both modes
have5425 lines. Outside the existing generated-by metadata distinction, there
is exactly one differing raw G-code line. The original RED first difference
was layer4/line1418; that difference is gone. This is partial progress, never
a parity PASS. No new guard, emitter rounding fix or tolerance is authorized.

Official S45 input SHA
fe76c98e5e3e4d698968bb81a1a76a91ef81b1faa433af26138131342b6b16cf;
seeded S45 input SHA
0f77beb88fc7d9b77834829269db845800f330390b4b41e1b6f9a1fe7bf0febe.
Inputs/reference streams are runtime artifacts, not committed Native outputs.

## Proposed bounded read-only localization

Request one60min window starting with the first observation, CPU12 only.
Use the established read-only GDB flow and the frozen S45 projects above;
one fresh Native official and seeded capture each, plus the already compiled
Ares CLI. No source probes, inferior calls/writes, numeric substitutions,
sorting/chaining changes, Native rebuild or store deletion. Record actual
inferior exit status and preserved stdout/stderr, not just GDB exit status.
Unavailable optimized locals remain unavailable rather than reconstructed
and claimed as actual observations. No new tests are needed for this window.

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
If the mismatch is outside object preparation, submit that separate upstream
boundary for approval. If not located within60min, STOP with observations;
no indefinite expansion, retries or downstream patch to manufacture GREEN.
Strict/fmt/WASM and B-T directed rerun remain pending; they were not launched
after the mandatory residual STOP and are not claimed as final gates.

## Durable evidence

/home/indexyz/ares-evidence/worker3/support-cubic-b-s-residual/
residual-evidence.json SHA256
fdcbf3c141603757b7334ff4d953255c1609aedfbe07bf7f94bd1f0c5b698448.
Archive1fc075c55-residual.tar.gz in the same directory, SHA256
772383d8c613bd915af340a8a68f75e9bbf2eb448cd7fb3f38b0588a1a10981c.
It retains complete projects/streams/comparisons/command receipts, affinity
observations, run logs, squash proof and the initial compile failure. Targets
and caches are excluded. All /tmp roots are preserved. This committed summary
survives tmpfiles cleanup without freezing Native outputs in Git.
