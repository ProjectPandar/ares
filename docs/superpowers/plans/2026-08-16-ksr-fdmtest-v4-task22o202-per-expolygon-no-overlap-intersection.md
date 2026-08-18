# Plan: KSR FDM Test V4 task202 per-expolygon no-overlap intersection

1. Add a failing focused test proving a small fill ExPolygon restricts a larger no-overlap domain.
2. Port the source per-ExPolygon intersection before Arachne concentric generation and map Clipper errors through the fill boundary.
3. Remove all `[DEBUG-a201]` instrumentation and re-run the original fixture differential loop.
4. Verify focused concentric/Clipper tests; record the fixture's newly reached wall-outline preprocessing assertion for the next source slice; run line counts, formatting, and workspace Clippy.
5. Record the confirmed root cause in `docs/roadmap.md`, commit, and push independently.
