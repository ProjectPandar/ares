# ARD-0015: Skirt artifacts before ordered print paths

## Decision
Ares will model skirts as structured layer artifacts before ordered print paths, then merge skirt paths ahead of perimeter and infill paths for downstream move, extrusion, speed, and G-code stages.

## Context
The current pipeline orders object perimeter and sparse infill paths before converting them into moves. OrcaSlicer treats skirts as printable adhesion/priming paths around objects. Adding skirts as a separate pre-print-path artifact keeps the generation step inspectable and avoids hiding adhesion behavior inside G-code formatting.

## Consequences
- UI/API consumers can inspect skirt geometry before it becomes G-code.
- Existing downstream stages can reuse the same move/extrusion/speed machinery once `PrintPathRole::Skirt` is added.
- M16 uses simple rectangular loops around contour bounds as a deterministic scaffold; exact polygon offsets, brims, support, and multi-extruder behavior remain later milestones.
