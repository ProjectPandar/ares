# ARD-0005: Type hardware options in core before profile inheritance

## Status
Accepted for M6.

## Decision
Ares will type the first machine/filament hardware options directly in `ares-core::SliceOptions` before adding a separate preset/profile crate. The dynamic option map remains intact, and typed accessors are added only for options needed by near-term slicing stages.

## OrcaSlicer structure evidence
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp` defines nozzle diameter, filament diameter, and machine layer-height bounds as core print configuration options.
- `OrcaSlicer/resources/profiles/**` stores these options in scalar and vector JSON forms, including arrays of strings and semicolon-separated strings.
- Orca validates filament and nozzle diameter thresholds before using them for extrusion and G-code analysis.

## Consequences
- Later extrusion/flow milestones can depend on typed, validated hardware values without losing unknown Orca options.
- The core remains usable from WASM and CLI because option parsing is byte/value based and has no filesystem dependency.
- Full preset inheritance and profile discovery remain explicit later milestones instead of being hidden in this small option-typing step.
