# Stable KSR island-order rewrite slice

Initial candidate upstream boundary:
`libslic3r/GCode.cpp:4982-5068,5432-5435` extrusion-island membership and
enumeration, `Layer.cpp:38-66` slice ordering and
`ShortestPath.cpp:1096-1120` polygon/point chaining. Rust destinations are
the existing `project_slice::extrusion_islands`, `slice_ordering` and
`geometry::chain_points`; revise these if input capture proves an earlier
producer divergence. Existing emission stages remain temporary upstream
compatibility shells, not Ares-owned pipeline design. Captures establish the
actual repair owner: `PerimeterGenerator.cpp:238-270` calls
`ShortestPath.cpp:1026-1042::chain_extrusion_entities`; Rust destination is
`perimeters::classic::shortest_path::chain::chain_extrusion_loops` and its
`entity_collections::traverse` consumer. Preserve incoming loop enumeration
instead of the extra Ares coordinate canonicalization; keep existing
endpoint, KD-tree, heap and ignored-loop-reversal behavior unchanged.

Include only the proven pre-chaining canonicalization affecting complete public
G-code. Defer top-split/seam changes, M73 timing and unrelated native nondeterminism, generic ordering
refactors, random-generator unification and unrelated option behavior.
Do not change production code to match a nondeterministic native sample.

First verify reference self-replay on the exact input/build/seed/affinity.
Retain complete outputs for every variant. Single-CPU KSR self-replay is
stable in five runs, but the current simple grid probes become complete
matches under that same affinity and cannot justify a repair. Minimize a
stable public RED using the existing ordered generator-only comparator.
Diagnostic movement-only differences and private stage capture are not
acceptance and must not become source-pinning tests.

Exit requires sensitive public RED/GREEN against an independently identified
reference, original KSR revalidation with honest residual reporting, affected
full-output consumers, workspace Nextest without fail-fast, fmt/clippy,
WASM/Windows/macOS/Linux core builds and WASM adapter build. Rust files remain
below400LOC with separate test modules and no include macros for splitting.
Independent review verifies requirements, logic, boundaries, quality, tests
and actual execution; fix/re-review until approved, then commit/push code and
docs together. All-printer/default/all-option completion remains a separate
global requirement, not implied by this slice.
