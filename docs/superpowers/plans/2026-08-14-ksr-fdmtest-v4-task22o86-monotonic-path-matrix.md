# Task 22O.86 implementation plan

1. Add RED orientation-addressing and pheromone-reset tests.
2. Add O80 endpoint/cost accessors.
3. Port dense lazy path matrix with source f32/epsilon order.
4. Run O80-O85 regressions, strict Clippy/rustfmt/LOC, update docs, commit/push.

## Completed evidence

Compile RED proved the missing matrix module. GREEN passes 2/2 O86 and both O85
regressions. Strict core Clippy, rustfmt, diff, and LOC gates pass.

No ant simulation/RNG, entity output, lifecycle, fallback, or G-code.
