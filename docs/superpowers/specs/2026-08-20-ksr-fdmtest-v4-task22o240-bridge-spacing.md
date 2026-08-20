# Spec: Task 22O240 internal-bridge spacing preservation

## Observable contract

For `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf`, every `Internal Bridge` extrusion keeps the configured thick-bridge circular flow at 0.4 mm width and 0.4 mm height. Solid-fill line fitting must not stretch bridge spacing. The now-identical consecutive bridge flow collapses one redundant processor transition, reducing the current total from 520 to 519 `; LAYER_HEIGHT:` lines. The remaining two-tag count delta is deferred with bridge collection geometry/order.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1339-1345` and `Fill/FillRectilinear.cpp:2787-2791`: bridge and internal-bridge fills set `FillParams::dont_adjust`, so full-density bounding-box fitting does not alter the bridge flow passed through `Flow::with_spacing`. Destination: `project_slice/fill_entities/monotonic.rs`. Deferred: bridge path geometry/order and later G-code differences.
