# ARD-0090: Single-region ironing emission phases

## Upstream boundary and destination

Rewrite OrcaSlicer v2.4.2 `src/libslic3r/GCode.cpp:5445-5468`'s island
dispatch and `GCode::extrude_infill` at 6150-6178 in
`ares-core/src/project_slice/gcode_emit/motion.rs`.
`GCode::retract` at 7654-7704 owns the associated traditional-timelapse
eager lift in `gcode_emit/motion/travel.rs::retract_for_timelapse`: completed
retraction does not suppress the separate `needs_lift && can_lift` branch.
The existing ordered island representation is a temporary single-region
compatibility shell, not a new Ares pipeline.

## Decisions

Ordinary infill, infill-first walls and ironing are distinct emission phases.
Ironing runs after the walls and chains from that phase's current cursor.
Move entities without copying geometry; stable partitioning must retain
source order within each phase. Preserve original infill existence for the
traditional timelapse insertion between the initial walls and ordinary infill;
ironing alone still counts as infill for that upstream gate.

## Included and deferred

Include single-region phase dispatch, ironing chaining and traditional
timelapse insertion in both wall-first and infill-first branches. Preserve
first-layer walls and non-ironing output. Verify at the public exported
3MF-to-complete-G-code seam using the unchanged generator-only comparator.
Defer regional ownership/configuration switching, compatible perimeter groups,
multiple tools/wiping, Normal/Tree support and global option acceptance. This
slice cannot replace those ports or authorize a support guard/skip.
