# ARD: Brim.cpp inner-only runtime boundary

Status: accepted. SPEC `696cd308a`. Tests/RED precede implementation.

## Source and observed boundary

`OrcaSlicer/src/libslic3r/Brim.cpp::outer_inner_brim_area:448-458,529-570`
owns InnerOnly hole-area construction. `tryExPolygonOffset` and
`makeBrimInfillImpl:813-875` own area-to-path construction and extrusion.
At main5fc560ca0 the actual project `BrimPlan::generate` excludes InnerOnly.
The independent public LayerContours adapter's rectangular hole paths are
not that observed runtime call stack and are not Native parity evidence.

## Non-negotiable decisions

1. Rewrite the named upstream project boundary, not a new adhesion pipeline.
   A private `ares-core::brims::inner` owns hole-area geometry; the existing
   project BrimPlan is a compatibility shell around the upstream functions.
2. Admit one object/instance/model-part/group/filament, ordinary first-layer
   holes and explicit InnerOnly. Import existing option/geometry/flow types;
   no public API or candidate crate is added.
3. Reuse the existing shared source area-to-path port and emitter. Pass the
   upstream Miter limit3 for hole offsets; never change a global geometry
   default to repair this slice. Use the original paired-quantized brim_width,
   not the outer branch's brim_width_mod. Only objectIsland uses Round.
   Include single-object masks/contact rules.
4. Defer public generate_brims replacement and its assertion disposition.
   No retired square assertion or existing guard is removed by this SPEC.
   The real project implementation never calls that scaffold as fallback.
5. Preserve complete Native/API/CLI byte comparisons and all existing outer
   and automatic-ear controls. No geometry-only or footer-only substitute.
6. Core remains platform-neutral and byte-oriented for all four Tier1
   platforms. Native execution and artifacts belong solely in CLI tests.

Broader brim selectors, EFC, painting, multi-object composition and all other
other destinations remain outside this decision. Expansion requires a
revised source-cited SPEC before implementation.
