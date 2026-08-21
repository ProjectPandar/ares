# Spec: Task 22O.254 precise seam and variable-loop wipe state

## Observable contract

After the KSR Z0.4 variable-speed perimeter ending at `G1 X133.83 Y89.362 E.00989`, Ares emits the loop's forward seam-gap wipe as `G1 X133.669 Y89.214 E-.08742`, `G1 X133.055 Y88.777 E-.30159`, and `G1 X133.03 Y88.765 E-.01099`. The path and retraction values derive from f64 planned layer heights, aligned seam geometry, the processed path's source-precision endpoint, project placement, and typed wipe options.

## Upstream boundary

- `OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp:1019-1031` — seam candidate Z uses `Layer::slice_z`.
- `OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp:1323-1392` — aligned seam spline observation and final-position evaluation use f32 source arithmetic.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5978-5991, 6538-6566, 7350-7448` — loop wipe geometry remains in the source coordinate domain while writer XY is formatted independently.

## Deferred behavior

The following spiral lift, later executable-body divergences, timing, progress, and metadata remain outside this slice.
