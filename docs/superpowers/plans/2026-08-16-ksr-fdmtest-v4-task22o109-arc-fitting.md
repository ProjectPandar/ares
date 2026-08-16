# Plan: KSR FDM v4 task22o109 arc fitting

1. Add a failing focused emitter assertion using a three-point circular path and the typed `enable_arc_fitting` gate.
2. Add a private arc-fitting module with source-cited geometry and segment emission; keep motion.rs below 400 LOC by moving formatting/arc logic into normal modules.
3. Wire the option-driven fitter into perimeter and fill path emission, preserving current G1 behavior for disabled/unsupported paths.
4. Run the focused core test, KSR fixture test to the next independent diff, formatter/clippy checks, and LOC/macro checks.
5. Commit and push this vertical slice before starting retraction/wipe and timing slices.
