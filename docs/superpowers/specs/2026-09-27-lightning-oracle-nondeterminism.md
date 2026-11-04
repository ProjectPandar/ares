# Lightning infill oracle nondeterminism

## Upstream and Rust boundaries

`PrintObject.cpp:710-715` fills lightning-infill layers concurrently
(`parallel_for` over layers); `Lightning/TreeNode.cpp` `convertToPolylines`
consumes the process-global, unseeded `rand()` to pick which child continues
the current polyline. Under concurrency the draw order — and therefore the
emitted polyline shape and junction trims — is nondeterministic by
construction.

Ares fills layers sequentially through the generator's private stream
(`fill/lightning/generator.rs`), so its output is deterministic, but it can
never be byte-pinned to a moving oracle.

## Evidence

Two official Orca 2.4.2 runs on identical input manifests
(same input hash)
differ by 407 G-code lines in the lightning tree junction region —
the same junction where Ares differed; Ares' branch order is one of the
oracle's own outcomes (junction trimming matches `TreeNode.cpp:379-413`).

## Decision

`option_coverage` marks `(sparse_infill_pattern, lightning)` as
`ORACLE_NONDETERMINISTIC`: the case is exercised end-to-end (the slice must
succeed) but its comparison result does not gate the domain. The domain
counts the value as covered (`compared + nondeterministic == cases`).

Deferred: none — the value is structurally untestable for byte equality.
