# Plan: Task 220.136 quantized dynamic-segment extrusion

1. Add a red KSR assertion for the first post-overhang inner-wall endpoint and relative-E value.
2. Calculate each variable segment length from consecutive quantized processed endpoints instead of the preceding emitter state's unquantized coordinate.
3. Run the focused assertion, motion tests, rustfmt, strict `ares-core` clippy, and LOC checks; commit and push the slice.
