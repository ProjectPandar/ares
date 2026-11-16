# Deterministic automatic orientation

The `AutoOrienter` rewrite of OrcaSlicer 2.5.0
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, `Orient.cpp:134–203,281–344`,
preserves first insertion order for normal groups and candidate directions.
Area ranking and score ranking use stable sorting, followed by the upstream
upright-preference scan. Native `unordered_map` iteration and equal-key
`std::sort` ordering are library implementation details, not orientation
semantics to reproduce. Do not reverse-engineer libstdc++ tie ordering.

Arithmetic within each mesh has a fixed evaluation/reduction order. Execution
may distribute independent meshes but cannot change group representatives,
ranking, selected rotation or input/result association. Symmetric and tied
meshes verify only Ares reproducibility and worker-count invariance; no test
asserts their equality or inequality against Native. Native complete-output
comparisons use asymmetric meshes without candidate-ranking or selection ties.

The three-dimensional hull is dependency-free deterministic Quickhull under
`ares-core::geometry`, with triangulated facets and outward normals, usable on
WASM and all native targets. `TriangleMesh.cpp:1358–1447` uses Qhull `Qt`, but
AutoOrienter consumes quantized normal-group area sums and each group's largest
facet's actual normal. Coplanar triangulation differences preserve these
quantities apart from floating-point noise. Native complete-output witnesses
must prove observable agreement; the comparator is not relaxed to accommodate
hull differences. A two-dimensional hull, Native fallback or new dependency
is not an alternative implementation of this decision.
