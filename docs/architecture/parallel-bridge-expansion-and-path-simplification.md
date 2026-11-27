# Parallel bridge expansion and object path simplification

Follow OrcaSlicer v2.5.0 at `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
This slice moves independent bridge-over-infill work and object path
simplification to the existing native parallel dispatcher. It does not change
geometry, toolpath order, or the public API.

## Release profile

Median milliseconds across three fresh processes per fixture at
`cf243b2aad27fbefdf2dc0f1dc28100c65439f92`, with fixed affinity and
`RAYON_NUM_THREADS=4`. These are wall intervals from a temporary timer build;
parent and child intervals overlap. The ironing row is summed call time
inside the fill stage, not a separate wall phase.

| Stage | KSR | Arachne | Toolchange |
| --- | ---: | ---: | ---: |
| Whole CLI slice | 5255.7 | 150.7 | 298.9 |
| Region setup and composition | 1.15 | 0.03 | 0.04 |
| Arachne materialization | 0.08 | 6.20 | 0.09 |
| Infill preparation | 897.3 | 5.36 | 2.65 |
| Bridge candidate gathering | 260.3 | 0.54 | 0.25 |
| Bridge expansion including its setup | 242.8 | 1.64 | 0.52 |
| Second internal bridge pass | <0.01 | <0.01 | <0.01 |
| Fill entities | 328.8 | 8.10 | 4.04 |
| Ironing calls | 0.18 | 0.38 | 0.18 |
| Island order | 0.67 | 0.02 | 0.04 |
| Curled-perimeter estimation | 181.9 | 0.94 | 0.36 |
| Object path simplification | 222.6 | 1.35 | 0.56 |
| Seam placement | 966.2 | 61.94 | 31.78 |
| G-code schedule | 140.0 | 1.86 | 84.71 |
| G-code layer driver | 1243.5 | 17.72 | 72.03 |
| G-code processor and sink | 480.9 | 14.34 | 59.39 |

Bridge expansion and object path simplification are the two largest measured
serial intervals with directly corresponding upstream parallel loops. Together
they account for about 8.9% of the KSR run, not 8.9% available speedup: bridge
expansion includes serial generator setup and traversal within a cluster. The
benchmarks must measure the resulting gain rather than assuming that the whole
interval is parallelizable.

The G-code driver is the largest remaining serial interval on KSR, but
`src/libslic3r/GCode.cpp:5127-5147,5159-5250,5257-5343` does not parallelize
layer generation. Its parallel preparation filter computes overhang inputs;
its generator, cooling, and output are ordered. Ares's measured
`EntryBuilder::geometry` interval (22.2 ms on KSR) overlaps part of that
preparation but is not a measurement of upstream-style overhang precomputation.
Keep that port, the ordered G-code pipeline, and island ordering out of this
slice. Raw intersections and facet slicing, perimeter generation, ordinary
fill including ironing, support generation, and seam preparation already have
parallel dispatch. Region composition's remaining upstream complex-layer loop
(`src/libslic3r/PrintObjectSlice.cpp:408-548`) is deferred because its whole
measured interval is small on these fixtures. Curled-perimeter estimation
carries the previous layer's estimated lines into the next
(`src/libslic3r/Support/SupportSpotsGenerator.cpp:141-196`).

## Bridge expansion

The upstream boundary is `PrintObject::bridge_over_infill` in
`src/libslic3r/PrintObject.cpp`. Ares's transaction and staged records under
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/` are
storage for the upstream phases, not new slicing steps. Retain their current
phase and object order.

| Upstream `tbb::parallel_for` | Ares destination and ordering |
| --- | --- |
| 3374–3385: generate sparse anchor lines for each candidate's lower layer | `transaction/candidate_expansion/layers.rs::prepare_infill_lines`: run independent lower-layer jobs over the existing ordered candidate-layer keys. Keep region validation and polyline append order within a job. Collect then insert by original lower-layer key, not by completion order. Keep this dispatch serial when the Lightning generator exists. |
| 3405–3418: build candidate footprint per layer | `layer_clustering.rs::cluster_candidate_layers`: dispatch independent `layer_coverage` calls over candidate layers without changing each layer's union sequence. After joining, group adjacent layers serially in the existing order and with the existing Z and overlap predicates (upstream 3420–3433). |
| 3747–4032: expand bridge candidates by cluster | `transaction/candidate_expansion.rs::prepare_object` and `transaction/candidate_expansion/layers.rs::prepare_layer`: each cluster owns only its original candidate layers. Read immutable anchors, geometry and configuration. Process its layers and their candidates serially, preserving the existing sort, collision handling, and subtraction of earlier results from the same cluster. Return the finished layer records and install them in original cluster/layer order, including keys with empty results. |

Factor the cluster job away from the whole `BridgeCandidateObject` so the
worker cannot access another cluster's mutable layers or the non-thread-safe
Lightning generator. The ordered merge must finish before the existing
surface rewrite. In both anchoring and expansion, keep the existing calling
thread and traversal for objects with sparse or solid `~`/`^` rotation
templates: `group_fills` and the lower-direction calculation can draw from
slice-local RNG state. Do not replace that state or change the draw sequence.
Preserve the first geometry error in original traversal order. Do not clone
whole object geometry to dispatch a cluster.

The earlier candidate gathering and later surface rewrite are already
parallel. The Lightning setup (3270 and 3294), adaptive field and octree setup,
and the separate second internal bridge pass (4132 and 4165) are not included.

## Object path simplification

`src/libslic3r/PrintObject.cpp:1061-1092`,
`PrintObject::simplify_extrusion_path`, runs separate per-layer wall and infill
passes (`tbb::parallel_for` at 1064 and 1082). `Layer.hpp:117-118,232-233` and
`LayerRegion.cpp:1085-1128` define the entity traversal. In
`crates/ares-core/src/project_slice/path_simplification.rs::apply`, dispatch
one job per mutable `OrderedExtrusionLayer`. Ares already visits walls and fills
in a single layer pass; keep that traversal, including the order of entities
within each island and of paths within each entity. Preserve all existing arc
fitting and linear tolerances, the spiral-mode decision, stored fitting data,
and the contour-Z bypass. No job can inspect or mutate another layer. Keep the
stage after curled-perimeter estimation and before seam placement and G-code
emission. The support path pass at 1099, raft adapters, and the order of status
callbacks in the G-code driver are unchanged.

## Cancellation and platforms

Pass the active `CancelToken` from `slice_plate_to_sink` / `prepare_infill`
through bridge expansion and path simplification. Use the indexed helpers in
`crates/ares-core/src/parallel.rs`, with the same serial behavior on WASM and
when the native `parallel` feature is disabled. Check on phase entry, per
anchor/coverage job, per cluster and between its layers and candidates, and
per simplification layer and between its islands. Check again before committing
bridge results and handing each stage to its consumer. A canceled slice returns
`SliceError::Canceled`; neither partially processed geometry nor any G-code
reaches the sink. Workers do not emit progress callbacks. Do not add a WASM
thread pool or change the clipping, fitting, or random algorithms.

## Verification

- Reuse the seven raw G-code SHA-256 witnesses in
  `crates/ares-core/tests/parallel_slicing.rs`: KSR, Arachne, toolchange, both
  interface-shell settings, the external extra bridge, and volume ranges.
  Compare exact base and tip outputs with separate `RAYON_NUM_THREADS=1,2,7,16`
  processes and a feature-disabled serial build, with fixed generation metadata
  and no output normalization. Also exercise independent and dependent bridge
  clusters, Lightning and random-template ordering, and path simplification
  with both arc fitting and its linear path through public slicing cases.
- Keep the public sink cancellation checks: `Canceled`, no bytes written,
  and byte-identical output from a later slice with a fresh token. Do not
  assert elapsed time or private scheduling behavior in correctness tests.
- Run workspace non-parity tests and compare complete base/tip Official and
  Seeded OrcaSlicer v2.5.0 suites with the canonical flags and lane profiles:
  no pass-to-fail changes and no selected tests skipped. Run formatting,
  workspace all-target Clippy, the `wasm32-unknown-unknown` checks for
  `ares-core` and `ares-wasm`, and the repository pre-push checks.
- Benchmark KSR, Arachne, and toolchange on matched release builds with fixed
  affinity and options, three or more repeats at one and four Rayon threads,
  and exact byte comparison. Report end-to-end wall times as well as bridge
  expansion and object simplification intervals, including any regressions.
  Do not commit profiling timers or generated G-code.
