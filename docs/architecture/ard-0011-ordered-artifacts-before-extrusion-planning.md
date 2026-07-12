# ARD-0011: Ordered artifacts before extrusion planning

## Decision
Ares will introduce ordered print path artifacts that combine existing perimeter and infill artifacts before porting OrcaSlicer extrusion planning, support, bridge, skirt/brim, and full G-code emission behavior.

## Context
OrcaSlicer orders perimeters and infills during G-code generation, with `is_infill_first` changing wall/infill order. Ares currently has geometry artifacts but no extrusion roles beyond metadata, extruders, islands, or travel planner.

## Consequences
- The core pipeline gains a stable handoff point for later G-code parity work.
- M12 can type and validate the first print-order option without claiming full extrusion behavior.
- Later milestones must replace or enrich these artifacts with island grouping, tool ordering, support/bridge/skirt/brim handling, extrusion values, and travel optimization.
