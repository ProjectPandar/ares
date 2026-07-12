# ARD-0014: Speed artifacts after extrusion artifacts

## Decision
Ares will attach speed/feedrate data as a structured stage after extrusion artifacts and before support, travel optimization, retraction, or full G-code parity work.

## Context
M14 established extrusion artifacts with absolute `E` positions. OrcaSlicer emits feedrates from speed options stored in mm/s and converted to mm/min for G-code. Ares needs these printer-relevant values attached to stable move/extrusion artifacts before later stages add more complex printer behavior.

## Consequences
- UI/API consumers can inspect speeds as structured data instead of parsing G-code text.
- The current `;SPEED -> ;EXTRUSION -> ;MOVE -> command` output preserves marker adjacency while allowing later stages to refine speeds.
- M15 only covers the three current roles/speeds needed by existing artifacts; first-layer, bridge, support, cooling, and volumetric speed behavior remain explicit later milestones.
