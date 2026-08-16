# Plan: Task 22o.112 seam-gap loop clipping

1. Add a failing typed-option assertion that the KSR `10%` seam gap resolves against its `0.4 mm` nozzle to `0.04 mm`.
2. Add failing path tests for clipping one terminal segment and clipping across multiple short terminal segments; implement in a dedicated motion module.
3. Add a failing project-output assertion that the first KSR closed loop no longer extrudes back onto its travel start; route perimeter and closed gap loops through loop-only clipping.
4. Run the focused option, clipping, and project-output tests, smoke-slice the 3MF, then run rustfmt and clippy before committing and pushing this source slice.
