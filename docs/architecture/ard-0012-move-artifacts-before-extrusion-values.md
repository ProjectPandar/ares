# ARD-0012: Move artifacts before extrusion values

## Decision
Ares will translate ordered print path artifacts into travel/print move artifacts before adding extrusion E values, speeds, retraction, or full OrcaSlicer G-code writer behavior.

## Context
OrcaSlicer travels to path starts and then emits extrusion moves through `GCode` and `GCodeWriter`. Ares currently has ordered print path artifacts but no extrusion planner, flow model, speed planner, or travel optimizer.

## Consequences
- The core API starts emitting path-following `G0`/`G1` XY commands while keeping extrusion semantics explicit non-goals.
- Later milestones can attach extrusion, speed, retraction, and travel optimization to stable move artifacts.
- Output remains deterministic and WASM-safe because all move generation is in-memory.
