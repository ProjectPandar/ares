# Plan: Task 22O248 rectilinear offset coordinate conversion parity

1. Replace the fractional-offset test with a focused assertion for OrcaSlicer's positive and negative `coord_t` truncation and run it red.
2. Convert the scaled rectilinear offsets to integral coordinate values at `scaled_offsets`, preserving the existing public module seam and error handling.
3. Run the focused conversion test and the first-layer project geometry test.
4. Generate the KSR fixture through the CLI and compare its executable body, excluding only progress commands and dynamic object IDs, to locate the next geometry divergence.
5. Run rustfmt and clippy, then commit and push this source-cited slice.
