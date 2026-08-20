# Plan: KSR FDM Test V4 fitted-circle center quantization

1. Add a focused scaled-coordinate boundary test for a fitted-circle center.
2. Convert the calculated scaled center using truncation before returning to millimeters, matching the loaded geometry coordinate domain.
3. Verify focused arc tests and regenerate the KSR slice, then run formatting, lint, and workspace tests before committing and pushing the slice.
