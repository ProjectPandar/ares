# ARD-0016: Brim artifacts after skirts and before object paths

## Decision
Ares will model brims as structured layer artifacts after skirt generation and before ordered print paths, then merge brim paths after skirts and before perimeter/infill paths for downstream move, extrusion, speed, and G-code stages.

## Context
M16 introduced skirts as inspectable adhesion artifacts before object paths. Brims are also adhesion geometry, but unlike skirts they are first-layer object-adjacent paths. Keeping brims as a separate pipeline artifact preserves API visibility and prevents G-code formatting from owning geometry decisions.

## Consequences
- UI/API consumers can inspect brim geometry before it becomes G-code.
- Existing downstream path machinery can reuse `PrintPathRole::Brim` once brims are merged into ordered print paths.
- M17 uses rectangular outer loops around contour bounds as a deterministic scaffold; exact Orca offsets, inner brims, mouse ears, painted brims, support brims, and automatic brim analysis remain later milestones.
