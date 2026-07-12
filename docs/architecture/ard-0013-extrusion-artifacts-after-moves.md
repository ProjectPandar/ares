# ARD-0013: Extrusion artifacts after move artifacts

## Decision
Ares will attach absolute extrusion `E` positions to deterministic print moves as a separate artifact stage after toolpath moves and before speed, retraction, travel optimization, support, or full G-code parity work.

## Context
M13 established explicit travel/print move artifacts. OrcaSlicer's writer emits extrusion by accumulating filament position and writing `E` on extrusion moves, while flow calculation depends on line width, layer height, nozzle diameter, and filament diameter. Ares needs this data attached to stable move artifacts before later printer-behavior milestones can add speeds, retractions, and parity checks.

## Consequences
- Extrusion math stays in `ares-core` and remains available to future UI/API consumers as structured data, not only as formatted G-code text.
- Travel moves stay distinct from print moves, so later retraction and travel optimization stages can operate without reverse-parsing G-code.
- M14 deliberately uses the first extruder/nozzle/filament only; multi-extruder behavior requires an explicit later milestone.
