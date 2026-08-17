# Plan: KSR FDM Test V4 task163 Arachne transition midpoint radius

1. Strengthen the focused upward-central-edge test with exact source-derived transition radii and positions; run it red against the current full-thickness calculation.
2. Convert the strategy's full transition thickness to radius before clamping and interpolation in `generate_transition_mids()`.
3. Run the focused Arachne tests, `cargo fmt --all -- --check`, and workspace Clippy.
4. Record the slice in `docs/roadmap.md`, commit, and push independently.
