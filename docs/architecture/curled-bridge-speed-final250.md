# Fully unsupported bridge speed with curled-perimeter slowdown

## Upstream and Rust boundary

Port OrcaSlicer commit `44f792dbc3cd94b927ad770ae482a99e17db4d25`, present at release `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`. The owner is `src/libslic3r/GCode.cpp::GCode::_extrude`: final source lines9693-9718 select the role/reference speed and overlap levels; lines9719-9743 build the curled-slowdown speed table; lines9744-9763 contain the unchanged non-curled table.

Rust destination: `crates/ares-core/src/project_slice/gcode_emit/motion/overhang.rs::estimate` and `motion/overhang/sections.rs::speed_sections`. The existing overhang estimator is a temporary compatibility shell around upstream extrusion-quality estimation, not a new speed planner.

## Included

- Carry the existing path's bridge-infill identity into speed-table construction. The `Bridge` feature corresponds to `erBridgeInfill`; `Internal Bridge` does not.
- When `slowdown_for_curled_perimeters` is enabled, the final zero-overlap table entry for bridge infill uses `bridge_speed`, normalized through the existing reference-speed percentage and float narrowing. Partially supported entries retain their existing overhang bands.
- Other roles retain the fourth overhang band at zero overlap with curled slowdown enabled. The non-curled table continues to use bridge speed as before.
- Preserve the existing overlap sampling, rounding, segmentation and adjacent-span merging. A changed speed table may naturally remove a slow split inside an anchored bridge; no new geometry or merging rule is introduced.

## Deferred / unchanged

Do not change curled-line estimation, interior sampling, first-layer/raft exclusions, overhang fan handling, reference-speed/volumetric/scarf limits, internal-bridge classification, support/ironing algorithms, filament grouping, wipe towers, or configuration defaults. This slice adds no options, role vocabulary or alternative path generator.

## Validation seam and failure modes

Use existing runtime Native2.5 versus Rust complete-output E2E cases across physical-layer brims, tree-support controls, rear seam, single-nozzle ironing patterns, top-surface expansion, project-setting substitutions, grid bridge control and layer-change/seam travel. The35 cases exercise actual extrusion and downstream timing, not only table values. Existing authored inputs and comparison rules remain unchanged; no new test interface is needed.

Check the Native bridge-command sequence as well as complete output: the physical-layer brim witness changes a slow `G1 F600` split with intermediate `Y158` to a continuous `G1 F2850` strand ending at `Y156.421`, with extrusion `.10757 + .08084` becoming `.18841`. Runtime comparison is the authority rather than a stored G-code baseline.

Potential mismatches include selecting bridge speed for every curled path, treating internal bridges as bridge infill, changing partially supported entries, bypassing the percentage/float conversion, or updating speed without allowing the existing segmentation decisions to consume the new table. Other pre-existing complete-output differences are not grounds to relax comparison or expand this source boundary.
