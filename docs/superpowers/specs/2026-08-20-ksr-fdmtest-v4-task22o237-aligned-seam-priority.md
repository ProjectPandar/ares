# Spec: Task 22O237 aligned-seam source priority

## Observable contract

For `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf`, `seam_position = aligned` must start cross-layer seam alignment from the best generated candidates, independent of perimeter traversal order. Production behavior is derived from generated seam candidate visibility, overhang, embedding, and local-angle scores.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp:1259-1278`, which gathers each chosen perimeter seam and stable-sorts all seams through `SeamComparator::is_first_better`, into `crates/ares-core/src/project_slice/seam_placement/alignment.rs`.

Included: a source-equivalent comparator across candidates from different layers and stable priority ordering before seam-string construction. Deferred: unrelated seam modes, visibility sampling, candidate geometry, spline fitting, and downstream G-code differences.
