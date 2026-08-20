# Spec: KSR FDM Test V4 task229 project print-flow ratio

## Observable contract

G-code extrusion distances multiply each path's geometric volumetric flow by the effective project `print_flow_ratio` before converting volume to filament length. A project value of `0.5` therefore halves every role's emitted relative-E distance while preserving geometry, role ordering, and motion rates.

The ratio comes only from the loaded typed 3MF region configuration. Production code does not inspect fixture identity, reference G-code, or known coordinates.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:6467-6513`: `_extrude` multiplies `path.mm3_per_mm` by `PrintRegionConfig::print_flow_ratio`, then the selected filament flow ratio, before converting effective volume through `Extruder::e_per_mm3()`. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/motion/options.rs::MotionOptions` and `motion/path.rs::extrusion_for_length`.

Included: typed project ratio resolution, source-order arithmetic for linear and fitted-arc extrusion, and an option-driven complete-slice test. Deferred: role-specific optional flow multipliers, remaining path-coordinate numeric differences, object labels, travel/retraction, cooling, timing/M73, and later exact G-code differences.
