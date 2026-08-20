# Plan: KSR FDM Test V4 dynamic overhang fan transitions

1. Add a failing project-slice behavior test for the configured fan command inside a variable-speed inner-wall region.
2. Preserve OrcaSlicer processed-point overlap, resolve `enable_overhang_bridge_fan`, `overhang_fan_speed`, and `overhang_fan_threshold` from the loaded typed project configuration, and emit role transitions through a dedicated motion fan module.
3. Verify the focused test and KSR slice, then run rustfmt, clippy, and the workspace nextest suite before committing and pushing this slice.
