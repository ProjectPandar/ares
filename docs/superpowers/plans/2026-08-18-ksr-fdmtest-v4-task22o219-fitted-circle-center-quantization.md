# Plan: KSR FDM Test V4 task219 fitted-circle center quantization

1. Add a circle-from-three test whose exact center has scaled-coordinate fractions above one half; confirm Ares rounds instead of applying the source integer conversion.
2. Replace nearest-grid center rounding with truncation toward zero while retaining the pre-conversion radius.
3. Run focused arc tests and regenerate the complete KSR fixture output; record normalized first divergence plus line, arc, and wipe counts.
4. Run formatting, focused Clippy, and file-size checks; commit and push this source-cited slice independently.
