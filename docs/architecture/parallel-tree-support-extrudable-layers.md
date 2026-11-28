# Parallel extrudable-layer preparation for tree support

Follow OrcaSlicer v2.5.0 at `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Port the parallel layer preparation in
`src/libslic3r/Support/TreeSupport.cpp:880–892`, the first loop of
`TreeSupport::detect_overhangs`. Keep geometry, warnings and G-code unchanged.

## Release profile

Five fresh four-thread release CLI processes ran the KSR project at
`e9b5b5626c8278f54696fa653b85bb909e8daf8c` with fixed affinity. Timers were
added to a temporary source snapshot. All five outputs matched the previous
KSR run at that revision byte-for-byte. The wall median was 5,314.5 ms
(range 5,098.2–5,476.1 ms). Parent and child intervals overlap.

| KSR interval | Median, ms | Dispatch decision |
| --- | ---: | --- |
| G-code layer driver | 1,262.1 | Keep ordered. Orca `GCode.cpp:5127–5147,5159–5250,5257–5343` parallelizes overhang preparation but not the stateful layer generator. The 22.9 ms Ares entry-geometry timer overlaps only part of that work. |
| Seam placement, inclusive | 982.4 | Already parallel visibility raycasts account for 735.6 ms. Already parallel per-layer preparation takes 104.0 ms. |
| Tree support necessity, inclusive | 278.0 | Largest measured preparatory stage with an unported upstream parallel loop and no need to alter the consumer. |
| Tree first-pass detection | 192.7 | Defer the second Orca loop (`TreeSupport.cpp:899–1080`). Ares samples elapsed time after each layer's raw overhang calculation; its 30-second and 100-polygon checks change flags for later layers. Parallel speculation could change which layers contribute to warnings, so keep the original loop and clock. |
| Curled-perimeter estimation | 183.3 | Keep ordered: `Support/SupportSpotsGenerator.cpp:141–196` supplies the previous layer's estimated lines to the next layer. |
| Extrudable layer preparation | 81.9 | Largest measured independent serial work with a corresponding upstream loop. Its median is about 1.5% of KSR wall time, not a speedup prediction. |

The support generation body is not exercised on KSR, which runs the necessity
check for support-disabled objects. The 172.3 ms vertical-shell cache,
98.4 ms surface typing and 87.6 ms external surface intervals already
include parallel work; those parent intervals are not serial measurements.

Within seam placement, cross-layer alignment takes 115.3 ms and remains
ordered in `GCode/SeamPlacer.cpp:1883–2140`. The visibility mesh assembly
and collapse total 3.4 ms, including 1.2 ms for face neighbors. Orca
`ShortEdgeCollapse.cpp:37–40` uses `its_face_neighbors_par` via
`TriangleMesh.cpp:1620–1623` and `MeshSplitImpl.hpp:233–279`; it does not
parallelize the collapse traversal. Triangle sampling takes 3.7 ms, with
parallel work available in `TriangleSetSampling.cpp:17–27,53–72`, but
sampling is not the leading candidate. The triangle BVH and sample KD tree
take 2.7 ms and 7.2 ms respectively; their construction in this path has no
corresponding upstream `tbb::parallel_for`. Leave those seam stages unchanged.
The complex region composition loop in `PrintObjectSlice.cpp:408–548`
is also deferred (1.1 ms for the Ares region-composition interval).

## Rewrite boundary

`TreeSupport.cpp:880–892` derives each `Layer::lslices_extrudable` from that
layer's source slices by opening at the configured extrusion width and
intersecting with the original slices. Each job reads and writes its own
layer; the next loop starts after all layers are ready. For support-disabled
objects `PrintObject::is_support_necessary` (`PrintObject.cpp:5292–5305`)
uses the same preparation under the invocation-local automatic-tree override.

In `crates/ares-core/src/project_slice/support_material/tree/overhangs.rs`,
move only the `SmartOverhangs::new` extrudable construction into the existing
indexed dispatcher. Keep the same `offset2_ex` and `intersection_ex` calls,
their arguments, rounding and result order. Each job reads one immutable
`LayerInput` and produces its existing `OverhangLayer`, including empty
layers and the original height. These scratch records carry the upstream
extrudable slices and empty overhang state; no layer layout or public API
changes. Collect by physical layer index, not completion order. Preserve the
first clipping error in input order and the existing public error mapping;
cancellation must remain `SliceError::Canceled`, not a clipping error.

Use one preparation path for `tree/necessity.rs`,
`tree/organic/generation.rs::detect` and `tree/non_organic/generation.rs`.
Thread the existing cancellation token into preparation. Check on stage
entry, before each layer's geometry, and after the join before consuming the
records. The same closure and indexed order apply on WASM and with the native
`parallel` feature disabled; no new thread pool. No worker emits progress
or writes G-code. Object order, status callbacks and warning order stay on
the caller. Canceled preparation must not write any bytes to the sink or
install support geometry.

The prepared layers feed the existing serial detection and later propagation,
clustering and path generation without changes to their geometry or order.
Keep the detection clock start after preparation and keep all guard behavior,
including first-layer handling, the skipped tail/cantilever work and the
final warning thresholds. Do not introduce a different clipping algorithm,
parallelize the whole `detect_layer` call, or import adjacent belt/tilt behavior.

## Verification

- Add test-first public witnesses for the support-disabled warning path and
  enabled organic and non-organic tree support. Compare complete G-code and
  warning content/order across fresh Rayon 1/2/7/16 processes and a build with
  the `parallel` feature disabled. Retain the existing twelve byte witnesses
  (including KSR, Arachne and toolchange) and require exact byte identity
  against the predecessor. Byte-preservation witnesses are expected to pass
  with the predecessor geometry; do not require a failing comparison by
  introducing an implementation-dependent assertion.
- Cancel from the public support-necessity phase callback; require `Canceled`,
  no sink bytes, and unchanged output and warnings on a later slice with a
  fresh token. Do not assert thread scheduling or elapsed time in tests.
- Run focused support warnings and tree-style parity cases, paired complete
  Official and Seeded OrcaSlicer v2.5.0 lanes on base and tip (zero
  pass-to-fail changes, zero selected skips), and workspace nonparity tests.
  Use the canonical lane settings. Run formatting, native all-target Clippy,
  core/WASM checks, all-target wasm32 Clippy excluding the CLI, and the
  repository pre-push checks.
- Benchmark matching release builds on KSR, Arachne and toolchange at one
  and four Rayon threads with fixed affinity and options, and at least five
  interleaved base/tip pairs. Record wall and preparation timings, keep
  outputs byte-identical, and disclose one-thread costs or an inconclusive
  wall result. Include an enabled tree-support fixture if available to measure
  the other callers. Do not commit timers or generated G-code.
