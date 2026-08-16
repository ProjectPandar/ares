# Spec: Task 22o.115 internal-bridge motion options

## Observable contract

For retained `InternalBridgeInfill` paths, Ares resolves `internal_bridge_speed` from the effective region options and `bridge_acceleration` from the effective object options. Percent speed is relative to `bridge_speed`; percent acceleration is relative to `outer_wall_acceleration`. The KSR values resolve to `75 mm/s` and `2500 mm/s²`, producing `M204 S2500` and `G1 F4500` before internal-bridge extrusion.

Ordinary bridges use `bridge_speed` with the same bridge acceleration. First-layer speed and acceleration precedence remains unchanged.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/GCode.cpp:6423-6424` and `6528-6530`, where bridge roles select `bridge_acceleration` and `erInternalBridgeInfill` selects `internal_bridge_speed`. The Rust destination is `gcode_emit/motion/options.rs` and `gcode_emit/motion/features.rs`.

## Deferred behavior

Internal-bridge flow scaling and role-based cooling markers remain separate source-cited slices.
