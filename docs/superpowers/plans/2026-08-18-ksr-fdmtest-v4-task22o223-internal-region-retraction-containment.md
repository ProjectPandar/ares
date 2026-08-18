# Plan: KSR FDM Test V4 task223 internal-region retraction containment

1. Add a failing KSR observable check showing that fill-preparation surfaces produce excess wipe/retraction sequences.
2. Follow the prepared perimeter predecessor chain back to the original layer-region slices and pass those slices to the G-code motion seam without cloning geometry.
3. Port Orca's role gates: only contained travel to a non-perimeter may skip retraction, and departure from an external or overhang perimeter forces retraction.
4. Add focused decision and geometric-containment tests, regenerate the complete KSR output, and record line, G1, arc, and wipe counts.
5. Run formatting, focused Clippy, and file-size checks; commit and push this source-cited slice independently.
