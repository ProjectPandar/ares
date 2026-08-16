# Spec: Task 220.134 overhang-role base kinematics

## Observable contract

A non-first-layer path materialized with `ExtrusionRole::OverhangPerimeter` emits the effective project `bridge_speed` and `bridge_acceleration`, not the inner-wall/default pair. For the first KSR `Overhang wall` block this is `M204 S2500` followed by `G1 F3000`.

The values are resolved from the loaded 3MF options (`bridge_speed = 50`, `bridge_acceleration = 50%` of the effective outer-wall acceleration); production does not inspect fixture identity or reference G-code. Dynamic per-segment overhang overlap interpolation remains a separate source slice.

## Upstream boundary

OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:6415-6438` applies bridge acceleration to roles accepted by `is_bridge`, including `erOverhangPerimeter`. `GCode.cpp:6515-6533` selects `bridge_speed` as that role's base speed before the later `ExtrusionQualityEstimator` overlap-band processing at `GCode.cpp:6654-6715`.
