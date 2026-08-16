# Spec: Task 220.130 internal seam corner projection

## Observable contract

For an aligned seam projected onto an internal perimeter, Ares uses the fitted external seam only to measure the inner-wall depth. Concave-corner overshoot starts at the selected external candidate, and applies only when OrcaSlicer's squared fitted displacement is smaller than that linear depth. A focused generated-loop contract distinguishes candidate-relative placement from the prior fitted-position-relative result; the bounded KSR seam result remains unchanged.

All inputs come from the generated seam candidate, fitted seam position, adjacent candidates, and generated extrusion loop. No fixture identity or expected coordinate enters production.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/GCode/SeamPlacer.cpp:1562-1599` into `project_slice::seam_placement::place_loop`. Include the fitted-position projection, squared-displacement guard, candidate-relative concave direction, literal `1.4142` overshoot, and final inner-loop projection. Staggered inner seams remain deferred because the KSR option disables them.
