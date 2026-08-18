# Plan: KSR FDM Test V4 task207 flat rectilinear offset contours

1. Freeze the fixture's post-E/I/J-normalization micron endpoint/path divergence and existing rectilinear contour tests.
2. Replace grouped `offset_expolygon`/`offset_expolygons` staging with source flat-path outer offset and inner shrink operations.
3. Preserve contour-before-hole and outer-before-inner ordering without new grouping.
4. Re-run focused rectilinear tests and the fixture normalized differential.
5. Run line-count checks, formatting, and workspace Clippy; record the source slice in `docs/roadmap.md`, commit, and push independently.
