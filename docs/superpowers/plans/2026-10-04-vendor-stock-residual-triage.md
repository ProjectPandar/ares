# Vendor stock residual triage (2026-10-04)

## Execution and ownership

Worktree `/tmp/ares-pi-matrix-vendors`, branch `pi/matrix-vendors`, starting
main `7ac55555131fbfe96ee5f8e3f0caa88773e5fd10`; independent
`CARGO_TARGET_DIR=/tmp/ares-pi-matrix-vendors-target`, Cargo build jobs2.
Read-only baseline `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock.log` and
`stock/stock-printer-summary.json`/`stock/case-*` belong to the frozen9a7b46fc
producer. Never modify them or claim their results came from latest main.

This worker owns Snapmaker, Qidi, re3D, Raise3D, Flashforge, Anycubic, Prusa
and Creality DIVERGENT/ARES_ERROR. BBL/Option sweep belongs to pi-arachne;
timelapse/wipe tower/AdaptivePA/bridge to pi-tower; lint to w25:p3.
Record and route shared-owner findings, do not modify those regions.
The latest user instruction replaces external review with coordinator Claude:
full green feature branch + push, then terminal `REVIEW-REQUEST <hash> <summary>`
and stop; after explicit APPROVED, rebase latest origin/main, rerun workspace
Nextest/Clippy/fmt/WASM and fast-forward-only push HEAD:main. Never force push.

## First-difference clustering (diagnostic only)

81 owned residuals:79 DIVERGENT and2 ARES_ERROR. Grouping below is a hypothesis,
not a common-cause proof or permission to weaken the generator-only comparator.
The diagnostic view temporarily omits M73 and time comments to expose earlier
geometry/order causes; acceptance always compares the complete ordered bytes.
External inventory `/tmp/ares-vendor-triage-inventory.json`, diagnostic first
movement contexts `/tmp/ares-vendor-triage-diagnostic.json`, clusters
`/tmp/ares-vendor-triage-clusters.json` retain all case paths and exact labels.

| Candidate cluster | Rows | Evidence / next step |
| --- | ---: | --- |
| Snapmaker cubic ordering | 60 | 0.2/0.4 first speed differences, 0.6/0.8 first infill travel differences; same-layer cubic paths differ after cooling/M73. Priority tracer A250 BKit0.4 `case-2ZmY3O`. |
| Qidi grid ordering | 8 | Q1 Pro and X-Max4, 0.2–0.8. Similar symptom but different fill pattern; do not assume same fix. |
| Start-template random coordinates | 6 | Flashforge Guider3 Ultra0.4/0.6/0.8 and Raise3D Pro3 Dual/Left/Right; first purge coordinates differ. Separate PlaceholderParser RNG boundary candidate. |
| re3D travel/ordering | 3 | GigabotX2, XLT, TerabotX2 1.75 nozzle; travel changes after wipe. Exact owner unproven. |
| Extra overhang perimeters | 2 | Anycubic Kobra3 Max0.6/0.8 existing UnsupportedProjectFeature guard. PerimeterGenerator.cpp helper depends on bridge detection; coordinate with pi-tower, do not merely delete guard. |
| Terminal wipe duplicate | 1 | Prusa XL0.25 missing repeated endpoint before WIPE_END; separate GCode wipe candidate, not proven wipe-tower ownership. |
| Solid infill cooling | 1 | Creality SermoonV1 first F2304 versus F2318; requires tracing underlying geometry/time before cooling changes. |

## Selected tracer and source candidates

Copy the immutable A250 BKit0.4 input/reference to
`/tmp/ares-vendors-snapmaker-replay`; replay through existing public
`replay::orca_parity_replay_sweep` on latest main and CPU0 before edits.
Original official2.4.2 CPU0 slicing of this saved input is retained under
`/tmp/ares-vendors-snapmaker-official`; it matches the frozen seeded reference
when only the generator line is normalized (diagnostic identity check).

Owning candidates: OrcaSlicer v2.4.2 commit8500fcdccaa10b5099ac20d252af3a7c560046f1
`FillRectilinear.cpp::FillCubic::fill_surface`,
`FillRectilinear::fill_surface_by_multilines`,
`FillBase.cpp::create_boundary_infill_graph`/`Fill::connect_infill`, and
`ShortestPath.cpp` chaining if initial fill output matches. Rust destinations
are existing `fill/multiline`, `fill/connect`, and `project_slice` path chaining;
these are compatibility shells around those source concepts, not a new pipeline.
After focused RED, compare equivalent boundaries to identify the exact owner,
then author a bounded repair spec/ARD before changing production. Preserve
all earlier green controls. No tests or production changes yet.
