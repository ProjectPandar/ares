# Plan: Task 22O.246 arc-center coordinate conversion

1. Add a focused circle-center assertion whose scaled coordinates distinguish truncation from nearest-coordinate rounding.
2. Compare OrcaSlicer’s `Circle::try_create_circle` assignment into integral `Point` with Ares’s scaled center conversion.
3. Truncate each finite scaled center coordinate toward zero before unscaling it for retained arc emission.
4. Run the focused arc-fitting tests, clippy, and rustfmt checks.
5. Commit and push this source-cited slice independently, then regenerate the KSR output and record the next divergence.
