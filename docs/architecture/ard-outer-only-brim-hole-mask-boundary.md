# ARD: Brim.cpp OuterOnly contour and hole-mask boundary

Status: accepted. SPEC a68680255; separate Native RED precedes code.

## Source and destination

`Brim.cpp::outer_inner_brim_area:448-570,661-711` owns explicit OuterOnly
area/exclusions/contact; `make_brim:903-904` supplies spacing as no_brim_offset.
`ClipperUtils.hpp:19-27,344-350` supplies default Miter3 for hole inset.
`tryExPolygonOffset` / `makeBrimInfillImpl:813-875` owns shared paths/emission.
Rust destination: private `brims/outer.rs`, existing BrimPlan dispatch and
reuse of current reversed-hole, bed clipping, contact and path producers.

## Proposed non-negotiable decisions

1. Replace the whole-ExPolygon OuterOnly area scaffold with the named upstream
   contour-only area producer. BrimPlan stays a temporary source compatibility
   shell; no independent adhesion pipeline, public API or candidate crate.
2. Outer ring uses Round/resolution, line450 paired width and line527 outer
   re-quantization. No automatic additions/clamp in explicit OuterOnly.
3. No-inner hole exclusion is holes minus the negative-spacing default Miter3
   inset. Neither positive expansion nor gap/width replaces no_brim_offset.
   OuterOnly has no no-outer mask. ObjectIsland uses real ExPolygons and Round.
4. Reuse existing subtraction, contact against gap-expanded objectIslands/
   own other fragments, and shared path emission; no G-code postprocessing.
5. One object/instance/volume/group/tool with non-nested footprint islands;
   support/EFC/painting/auto modes, cross-object policy and public rectangular
   adapter/assertion retirement require separate scopes.
6. Admitted Native/API/CLI fixtures+RED are a separate commit before
   implementation. Existing assertions/full comparator stay intact. Core
   stays platform-neutral; Native process/artifact handling stays in CLI tests.

[SPEC, source citations, failure inventory and exits](outer-only-brim-hole-mask-spec.md).
