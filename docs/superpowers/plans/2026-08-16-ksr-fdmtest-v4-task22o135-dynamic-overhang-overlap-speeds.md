# Plan: Task 220.135 dynamic overhang overlap speeds

1. Add a red KSR output assertion for the first source-inserted overhang boundary point and the 50/33 mm/s speed transition on both sides of the dedicated overhang fragment.
2. Expose the prior layer's union boundary to the emitter and resolve the loaded overhang-band options with the role and volumetric base speed.
3. Port signed boundary distances, crossing insertion, long-segment sampling, band interpolation, variable-speed linear emission, and G-code coordinate quantization into sub-400-line motion modules.
4. Regenerate KSR output and verify the exact transition, then run focused motion tests, rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the slice.
