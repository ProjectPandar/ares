# Spec: Task 22o.114 internal-bridge processor role

## Observable contract

Ares emits `; FEATURE: Internal Bridge` for extrusion entities whose retained role is `InternalBridgeInfill`. Ordinary `BridgeInfill` entities continue to emit `; FEATURE: Bridge`. The KSR project output contains both processor roles.

## Upstream boundary

Port the role vocabulary from `OrcaSlicer/src/libslic3r/ExtrusionEntity.cpp:595-596`, where `erBridgeInfill` maps to `Bridge` and `erInternalBridgeInfill` maps to `Internal Bridge`. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/motion/features.rs::for_fill`.

## Included behavior

- Preserve the distinct retained extrusion roles at the G-code processor-tag seam.
- Leave role speed, flow, and fan control unchanged in this slice.

## Deferred behavior

`internal_bridge_speed`, `internal_bridge_flow`, and internal-bridge fan markers remain separate source-cited slices.
