# PlanePath object reference bounds — input-only RED

Approved SPEC: `ba1372792`,
`docs/architecture/2026-10-20-plane-path-object-bounds-spec.md`.
Upstream: `PrintObject.cpp:95-112` / `Print.hpp:347` model-size bbox,
`Fill.cpp:1224,1237,1398,1442` emitted/anchoring filler inputs and
`FillPlanePath.cpp:75-118` sparse reference-frame selection.

Exactly five frozen input-only3MFs, each nine ZIP members without G-code,
preview or stored Native output. Mesh, instances, profile, all ZIP members
except `Metadata/project_settings.config` are identical. Configurations
vary only `sparse_infill_pattern` and `xy_contour_compensation`. Common:
10mm authored single model/instance, classic, direction45, sparse20%,
multiline1, layer0.2, top6, ZAA0/0. Compared with the approved authored
Honeycomb input, only the common pattern/multiline/top/ZAA setup and XY
compensation differ; no geometry search or mesh replacement.

| File | Role | SHA256 |
|---|---|---|
| hilbert-compensation01.3mf | target | 2d59332f77beced29dddeff87eded5d8fb6d980042c8566cd46c36de630644b3 |
| hilbert-compensation0.3mf | phase control | 50db1ed174c0914cbfbe0b36ca4d4b21e22d36acca3125e05b343c3db2af92f2 |
| rectilinear-compensation01.3mf | general compensation control | d57952e651047eabddf0afb6ce97805ea3b5ddfc08fbb67cbd01cd54728cc022 |
| archimedean-compensation01.3mf | shared PlanePath bbox control | 07f2127d3b9c85f3660da93f2732c554de1ccb1e558e2d6718ae36f56b2e3859 |
| octagram-compensation01.3mf | shared PlanePath bbox control | 12d4b5c3d98e26014e3166dfaf90a2dde8b57bb3a8e5120bd40393c3e2e7e970 |

The shared anchoring runner owns SHA/public settings checks, two fresh Native
runs and stable public API repeats, effective Native footers, active bridges,
raw API/CLI equality and unchanged strict whole-file comparison. Its repeated
run and strict acceptance stages are reused, not copied. All existing
Honeycomb/Line/TPMS-D cases still require their original ordinary Sparse
Native/API equality; only its assertion location moves to their verifier.
No old assertion or observer is removed/weakened. This PlanePath scenario
instead diagnoses the emitted Sparse delta and asserts whole-file equality.

The new checks also admit typed public XY compensation, Native compensation
footer, positive Native/API Sparse extrusion, unchanged initial solid-only
G0/G1 rows, and independent Native XY geometry change between Hilbert0 and
Hilbert+0.1. That activity comparison uses two additional fresh baseline
references inside the target test; it never asserts that Ares must differ.
Each mode therefore has12 PlanePath Native/API/CLI records (two per case,
plus two repeated baseline records) and28 existing anchoring records.

Expected RED in both modes: Hilbert+0.1 FAIL at the original strict whole-file
M73 difference, all four controls PASS and existing14 anchoring tests PASS.
M73, E/wipe/retraction, feedrates and every other command remain under the
unchanged strict comparator. Geometry observations are only additional
activity/control assertions, never acceptance normalization.

No production code is included. Independent RED approval must precede GREEN.
GREEN is restricted to PlanePath bbox provenance and source-cited raw model
bounds reuse, preserving Archimedean/Octagram and every passing control.
Any strict residual means STOP/report; do not widen into generation, generic
compensation, E/time/M73, bridge-angle or kernel changes.
