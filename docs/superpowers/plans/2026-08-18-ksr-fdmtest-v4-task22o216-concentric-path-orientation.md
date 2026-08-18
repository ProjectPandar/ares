# Plan: KSR FDM Test V4 concentric path orientation

1. Keep the failing `slice_project` assertion for the first internal-solid travel, first extrusion, and final endpoint; confirm the entity's generated source direction before live-cursor chaining.
2. Compare Ares' flattened variable-width entity seam with OrcaSlicer 2.4.2 concentric `no_sort` collection and path reversibility behavior.
3. Carry reversibility on materialized paths, mark flattened concentric paths non-reversible, and preserve the flag through seam splitting; cover constrained chaining with a focused oriented-path test.
4. Regenerate the CLI output, verify the complete first path orientation, record sub-micron arc fitting as the next divergence, then run nextest, clippy, rustfmt, commit, and push.
