# Plan: KSR FDM Test V4 task203 Arachne outline preparation

1. Add a failing regression test from the fixture domain that currently reaches the transition-rib assertion.
2. Port the source epsilon offset sequence and polygon simplification/degenerate cleanup into a child outline-preparation module; use the retained Clipper union seam for self-intersection normalization.
3. Feed only positive prepared geometry to raw trapezoidation and remove the caller-prepared naming assumption.
4. Re-run fixture slicing; record the newly reached missing `getOrCreateBeading` branch for the next source slice.
5. Run focused Arachne/Clipper tests, line-count checks, formatting, and workspace Clippy.
6. Record the source-cited slice and confirmed crash cause in `docs/roadmap.md`, commit, and push independently.
