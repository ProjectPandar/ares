# ARD-0007: JSON profile composition before typed option registry

## Status
Accepted for M8.

## Context
OrcaSlicer composes full FFF configs in `PresetBundle::construct_full_config` and `PresetBundle::full_fff_config` by applying machine, process, project, and filament presets over `FullPrintConfig::defaults()`, then writing profile IDs and cumulative metadata groups. Orca can normalize vector lengths and extruder variants because it has a complete typed option registry.

Ares currently has a WASM-safe byte-in/byte-out `slice` API, dynamic `SliceOptions`, a small set of typed accessors, and M7 same-kind profile inheritance. It does not yet have the complete Orca option registry or filesystem profile loaders.

## Decision
M8 composes resolved process, machine, and filament profiles at the JSON map level inside `ares-core`. The core API remains in-memory and filesystem-free. The composition function will remove profile-local conflict keys, add profile ID/group metadata, preserve unknown keys, and return `SliceOptions` for the current slicer.

Multi-filament composition will use deterministic JSON-level rules: it visits the union of selected filament keys, collects present values in selection order, flattens arrays, and appends scalar values as single entries. The default `filament_map` mirrors Orca's resized default by assigning each selected filament to extruder `1` until a later explicit filament-map milestone adds caller-provided routing. This is intentionally smaller than Orca's typed vector/extruder normalization and creates a stable bridge for future typed option milestones.

## Consequences
- The current CLI and WASM-safe core API can consume composed profile options without introducing profile directory loading.
- Future typed option milestones can replace JSON-level multi-filament approximations with registry-aware normalization without changing the high-level composition entry point.
- Compatibility expressions, substitutions, aliases, vendor bundles, and project config remain explicit later milestones rather than hidden fallback behavior.
