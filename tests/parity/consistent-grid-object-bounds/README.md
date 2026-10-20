# Consistent-grid object reference bounds — input-only RED

Approved SPEC `f12f35120`:
`docs/architecture/2026-10-20-consistent-grid-object-bounds-spec.md`.
Owner: `PrintObject.cpp:95-112` / `Print.hpp:347` -> `Fill.cpp` ->
`FillRectilinear.cpp:2741-2807`, shared by ZigZag/CrossZag/LockedZag.

Eight SHA-pinned input-only3MFs, each nine ZIP members without G-code,
preview or stored Native output. Non-settings ZIP members are identical;
settings differ only `sparse_infill_pattern` and `xy_contour_compensation`.
Existing authored10mm classic single object/instance, direction45, sparse20%,
multiline1, layer0.2/top6, ZAA0/0. No changed mesh or geometry search.
+0.8 crosses the source-derived grid snap; +0.1 is its below-boundary control.

| File | Role | SHA256 |
|---|---|---|
| zigzag-compensation08.3mf | target | 2764c00429118e2285aeac74e36242c93fb0f6e3b2841d6576527a11abc3f8c2 |
| crosszag-compensation08.3mf | target | 2f1e3004772244b83ed0aa8df94858f9e2e7d95ec225bfdca471e1a503ccd077 |
| lockedzag-compensation08.3mf | target | e4d5d41399bdd49f59dee8942b84d0d72802f05651158eb9ec3268a1c76b7d6e |
| zigzag-compensation0.3mf | phase control | e413f331e008147446c4ef63ddda32d5c8c1b8694eedace21948c2df1829bb76 |
| zigzag-compensation01.3mf | below-grid-boundary control | b9bba1156611846a0e1aef3efc56b8520403cf4307e2129fb1c0cc9b23bfd7fa |
| crosszag-compensation0.3mf | phase control | 1531d7401eb1f9dfa7d99beda680da61ddd37ed124baef6350ebf418fde0ffd8 |
| lockedzag-compensation0.3mf | phase control | 1ff705f79f4b97bdc6e8e8d80bb56d7412e7bbc3c4ad2293a2d0af3b4988cf03 |
| rectilinear-compensation08.3mf | generic compensation control | 851a673b2a5db02a5f184fcf6c6f5053eb0ca262b78bee68a31f801b34715a03 |

The new scenario contains only data and eight named test wrappers. It calls
shared `anchoring_zaa/object_bounds.rs::check`, extracted without assertion
or helper-body changes from the existing PlanePath scenario. The original
PlanePath case table, tests and independent Native compensation-geometry
assertion are unchanged; all19 prior shared cases remain passing controls.
No copied harness, observer, process runner or comparator.

The shared harness checks pinned SHA, public loaded pattern/multiline/top/ZAA,
Native footer/bridge/Sparse activity, two fresh Native references and stable
API repeats, raw API/CLI identity and complete strict Native output. The
extracted object-frame helper additionally checks typed public compensation,
Native compensation footer, positive Native/API Sparse extrusion and unchanged
initial solid-only G0/G1 movement. Geometry observations are extra activity
assertions, never acceptance filtering. M73/other progress, E/wipe/retraction,
feedrates and every command remain under the unchanged strict comparator.

Expected RED each mode: three targets FAIL, five controls PASS, existing19
PASS (27 selected tests,24 PASS/3 FAIL). Each new case has two fresh Native/
API/CLI records. Existing19 contribute40 records including the original
PlanePath target's two extra baseline records; total56 records per mode,
112 both modes. Verify real target Sparse geometry differences externally
against canonical references, not by inverted Ares-divergence assertions.

No production change. Independent RED approval precedes bounded GREEN;
any strict residual after that frame-only port means STOP/report. No graph,
phase, locking, E/M73, kernel or generic compensation expansion. Queue remains
W1 Voronoi -> W4 ironing -> other reviewed RRs. This RED requests no FULL or
merge window.
