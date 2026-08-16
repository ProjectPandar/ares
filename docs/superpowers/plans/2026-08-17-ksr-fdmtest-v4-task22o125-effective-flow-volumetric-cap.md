# Plan: Task 220.125 effective-flow volumetric speed cap

1. Extend the existing KSR `slice_project` motion test with the OrcaSlicer regular-inner-wall feedrate `F15791.926`; run it to record the current `F15476.087` failure.
2. Include the selected 3MF `filament_flow_ratio` in the effective `mm3_per_mm` denominator used by the G-code motion emitter's volumetric cap, matching `GCode::_extrude`.
3. Re-run the focused test, core rustfmt, and strict `ares-core` Clippy; commit and push this isolated option-driven slice.
