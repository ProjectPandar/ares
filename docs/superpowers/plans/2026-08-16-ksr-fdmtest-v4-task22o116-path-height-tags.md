# Plan: Task 22o.116 per-path layer-height processor tags

1. Add a failing KSR output assertion for the thick internal-bridge height marker and its exact ordering.
2. Carry retained extrusion height through `PathProperties` and track the last processor height in motion state.
3. Reset processor height from each generated layer record and emit markers only on changes.
4. Run the focused output test, smoke-slice the KSR 3MF, then run rustfmt and clippy before committing and pushing.
