# ARD: preserve the effective default Voronoi wrapper, not raw-Boost parity

Status: proposed, docs-only. [Source boundary and review gates](voronoi-default-repair-prerequisite-spec.md).
Upstream8500fcdcc; planning base9bbc63c93; measured witness remains bound to0ce6.

## Decision

Port libslic3r `Geometry/VoronoiDiagram`'s default detection and ordered rotation
repair outside `vendor/boostvoronoi`, with its effective storage and REAL existing
MedialAxis default-constructor consumer. A raw backend with corrected anchors
cannot stand in for a Native repaired diagram. Do not grow an Ares geometry
pipeline, admit painting early, or merge an unused facade as a prerequisite.

## Non-negotiable constraints

1. Source owner: `Geometry/Voronoi.cpp::construct_voronoi:28-73`,
   `copy_to_local:90-158`, detection166-240, repair248-362; dependencies and
   exact Rust private `ares-core/src/geometry/voronoi/*` destinations are in SPEC.
   No vendor algorithm change, C++/CGAL FFI or alternate geometry engine here.
2. Existing patched Boost remains raw construction only. Raw versus effective
   local ownership, all source categories, anchors, flags, colors, f64 bits and
   graph order must survive. No approximate accessor substitution/debug parsing.
3. Default-true detection, issue precedence, exact filtered predicates, all4
   source angles, integer point rotation, endpoint encoding/remapping and mark
   cleanup are required; no witness-only bypass or assumption that repair is rare.
4. Existing `geometry/medial_axis/diagram.rs::build` is the real consuming handoff
   (`MedialAxis.cpp:458-460`), not a fake call returning/discarding a facade.
   Existing caller internals remain temporary compatibility shells; downstream
   closing retry and unrelated annotation/chaining behavior explicitly defer.
   Do not quietly fall back to raw output on repair failure.
5. The frozen painted-split Native witness is actually reachable and repair-active.
   Its initial issue and chosen angle are UNKNOWN, not deducible from final state.
   Constructor RED must run the actual Rust constructor and compare complete
   Native effective records; public XML rejection is not this RED.
6. Every new phase begins with source-cited failure inventory/Native evidence.
   Infrastructure, absent tests, compile errors, historical hashes or optional
   omissions cannot count as RED/GREEN. Keep durable hash-audited artifacts under
   `${EVIDENCE_ROOT}`; `/tmp` alone is insufficient retention.
7. Portable core is byte/data-only on WASM/Windows/macOS/Linux. No debugger,
   filesystem, terminal or native runtime in production core; files stay <400LOC.
8. Painted producer/consumer work remains paused until this independent slice
   is integrated. Its approved phase order and one-complete-unit integration
   requirement are unchanged. No FULL/push/merge under docs-only authorization.

## Consequence

Source-exact predicates and effective ownership cost more than simply rotating
all inputs or retrying a raw builder. That is necessary upstream parity, not
permission for tolerances, normalization, new fallback policy or broad unrelated
refactors. Any unsupported backend API/predicate/consumer requirement causes a
new bounded STOP and review, not scope inflation.
