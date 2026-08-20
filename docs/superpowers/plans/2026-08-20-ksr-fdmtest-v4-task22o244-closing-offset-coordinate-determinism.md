# Plan: Task 22O.244 closing offset coordinate determinism

1. Add a failing KSR `slice_project` assertion for the adjacent reference E words `.02865` and `.02866`.
2. Bisect geometry changes with isolated project slices and identify grouped `PolyTree` offset execution as the first divergence.
3. Apply signed per-path raw offsets before the final tree union and remove the obsolete grouped-root-order pinning test.
4. Run the focused behavior test and geometry clipper suite; record the next exact divergence.
5. Commit and push this source-cited slice independently.
