# ARD: Brim.cpp combined outer/inner runtime boundary

Status: accepted. SPEC 53b22ddc9; RED b35dc2bd5 precedes code.

## Source and destination

`Brim.cpp::outer_inner_brim_area:448-570,661-711` owns the combined selector,
area construction, exclusions and contact policy. `tryExPolygonOffset` /
`makeBrimInfillImpl:813-875` owns shared path construction and emission.
Rust destination: private `brims/combined.rs`, the shared hole calculation in
`brims/inner.rs`, and existing project `BrimPlan::generate` dispatch.

## Non-negotiable decisions

1. Port explicit OuterAndInner for one object/instance/volume/group/tool.
   The existing BrimPlan is a compatibility shell around those upstream
   functions, not a new adhesion pipeline. No API or crate is added.
2. Outer uses contour-only Round offsets and its line527 modified width;
   inner uses reversed holes, original line450 paired width and default
   Miter3. These join/width boundaries cannot be collapsed into one rule.
3. Preserve per-ExPolygon outer-then-inner append order. Combined mode has
   neither no-inner nor no-outer mask; its objectIsland is the Round offset
   of actual ExPolygons. Reusing inner-only's complete mask is invalid.
4. Reuse source-owned bed clipping, contact retention and shared area-to-path
   emission. No independent concatenated outer/inner G-code streams, global
   offset-default change, posthoc ordering fix or rectangular fallback.
5. Keep existing InnerOnly, OuterOnly and ear assertions/comparators intact.
   Public rectangular adapter retirement, EFC, support, painting, automatic
   widths and multi-object/tool composition need separate scopes.
6. Input-only fixtures and admitted Native/API/CLI full-byte RED precede the
   production port in a separate commit. Regional XY/E
   witnesses cannot replace the full comparison. Core remains platform-neutral;
   Native processes and repeatable artifacts live only in CLI tests.

[SPEC, source citations, failure inventory and acceptance](outer-inner-brim-rewrite-spec.md).
