# Plan: KSR FDM Test V4 task217 clipped-arc endpoint quantization

1. Add a retained-arc clipping test whose projected endpoint is not already on the integer coordinate grid; confirm the floating projection fails the source endpoint contract.
2. Pass the active coordinate scale into retained fitting clipping and quantize the normalized radius vector before adding the fitted center.
3. Run focused arc tests and regenerate the complete KSR fixture output; record normalized first divergence plus line, arc, and wipe counts.
4. Run formatting, focused Clippy, and file-size checks; commit and push this source-cited slice independently.
