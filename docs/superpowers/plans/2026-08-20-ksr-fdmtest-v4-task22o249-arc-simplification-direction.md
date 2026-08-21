# Plan: Task 22O249 arc simplification coordinate and direction parity

1. Add a failing focused assertion for the wrapped-arc zero-polar middle point and extend the project seam assertion to the next source destination.
2. Run Douglas-Peucker point-to-segment calculations in scaled integral coordinates, preserving the existing iterative simplifier and module seam.
3. Match OrcaSlicer's strict polar-angle direction predicates so the zero-angle boundary rejects the candidate arc.
4. Run the focused arc tests and first-layer project seam test; generate the KSR fixture and locate the next executable-body divergence after excluding progress commands and dynamic object IDs.
5. Run rustfmt and clippy, then commit and push this source-cited slice.
