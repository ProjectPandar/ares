# Spec: Task 22O.156 standard monotonic queue distribution

## Observable contract

Monotonic ant traversal chooses its first ready region with the same unbiased multiply-and-reject integer distribution as the C++ standard distribution used by the reference slicer. Selection consumes one MT19937-64 word even for a singleton queue and remains deterministic on every Tier-1 target.

The old exact three-region order assertion is removed: it pinned Ares's modulo sampler rather than an output contract. The replacement verifies repeatability, completeness, and precedence.

## Upstream boundary

This slice corrects the OrcaSlicer 2.4.2 `src/libslic3r/Fill/FillRectilinear.cpp:2423-2429` port. `std::uniform_int_distribution<>(0, queue.size() - 1)(rng)` is represented by a 64x64-to-128-bit product with low-word rejection and high-word selection, while the existing default `std::mt19937_64` stream remains unchanged.

Region costs, emitted path orientation, flow width, cooling, timing, and later exact G-code differences are deferred.
