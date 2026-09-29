# ARD-0030: Preserve seam sample scalar conversion

`TriangleSetSampling.cpp::sample_its_uniform_parallel` multiplies Vec3f
vertices by double-computed barycentric weights. Eigen narrows each scalar
weight before the float multiplication; the Rust port must do likewise.
Do not retain extra precision across that source boundary: changed sample
positions can select another seam and change both movement and progress.

Keep the two visibility-weight distance contributions separately evaluated
before adding them, as `SeamPlacer.cpp:341-345` does. Reassociation across
that addition is not equivalent for near-equal candidate scores.

Do not change vector reduction, compensate through M73 placement, or
introduce an epsilon tie-breaker.
Validate the source-cited correction through complete public slicing output;
native diagnostic builds do not replace the official acceptance executable.
