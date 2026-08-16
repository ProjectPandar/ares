# Plan: Task 220.128 global seam visibility

1. Add focused red tests for deterministic area-weighted triangle samples and open-versus-occluded hemisphere visibility.
2. Port transformed model-part mesh assembly, MT19937-64 sampling, AABB ray queries, and nearby-sample interpolation into seam-placement modules below 400 LOC each.
3. Feed interpolated visibility into the OrcaSlicer aligned-seam comparator, associate each extrusion loop with its closest generated perimeter, and split it at the selected initial seam.
4. Run focused seam tests, generate the KSR G-code to inspect the first visibility-selected candidate, then run rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the isolated slice.
