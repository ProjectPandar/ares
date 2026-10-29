# High-valence indexed mesh edges

Source: TriangleMesh.cpp:520-613 (`create_edge_map`,
`its_face_edge_ids_impl`), TriangleMeshSlicer.cpp:1865-1889.
Approved SPEC49c31802f and ordering addendum8b4162da3.

`generate.py` authors input meshes into the stock Ender3 3MF. The first
tracer joins two closed prisms at one indexed vertical edge; split indices
preserve identical world geometry as a manifold control. Reversed,
interleaved and rotated variants change only triangle order. Six-use cases
repeat the first prism's faces. Odd-original adds one nonzero-area face to a
cube. Same-direction repeats one cube face twice (3 versus 1 orientation
uses on each of its edges), exercising the fallback after opposite pairing.
Arachne shared/split cases keep the same geometric control.

Thirteen cases require Native result0 and compare complete G-code with only
the existing generator-header comparison, plus raw API/CLI byte equality.
Ten currently reach Ares's mesh_topology guard; cube-control, split-original
and arachne-split are strict controls. `provenance.json` records input hashes,
triangle counts, high-valence edges and admission classification. No Native
outputs are committed.

Odd-reverse and odd-rotated are retained diagnostic inputs, not strict
success-parity cases. Both Native modes return -100 (Unix156), stderr
`Errors`, with no G-code. Source attribution: GCode.cpp:1774-1778/1866,
Exception.hpp:40-46, OrcaSlicer.cpp:6286-6291. The longer first-layer error
text comes from source and is not verbatim CLI stderr. One diagnostic test
records both public Ares API/CLI outcomes without asserting Ares's guard;
its successful evidence collection is never counted as a parity PASS. Any
Ares G-code on these inputs is a future gap outside this slice.

Retained Native admission roots:

- /tmp/worker3-edge-native-preflight-t8_h596m
- /tmp/worker3-edge-native-order-risk-xz5652uy
- /tmp/worker3-edge-native-failure-logs-kort1uyl
- /tmp/worker3-edge-native-logfile-ycvz867g
- /tmp/worker3-edge-native-extra-r6nl3zlk (Arachne and same-direction)

Public runs require fresh Native caches/artifact roots and CPU12 outer
binding. Run nextest with `-E 'test(runner::mesh_edge_pairing::)'` in official
and seeded environments. Evidence lives in each retained root's result.json,
Native G-code, Ares G-code when present, and captured CLI stdout/stderr.
