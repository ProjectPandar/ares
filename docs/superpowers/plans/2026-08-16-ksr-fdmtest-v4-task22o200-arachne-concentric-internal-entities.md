# Plan: KSR FDM Test V4 task200 Arachne concentric-internal entities

1. Replace the offset-ring focused test with a failing variable-width concentric-internal behavior assertion.
2. Generalize the retained variable-width converter to accept an extrusion role while preserving the classic gap wrapper.
3. Build source wall parameters from flow/nozzle options, generate Arachne lines for no-overlap polygons, and append converted entities without overwriting existing thin fills.
4. Run focused fill, Arachne, and variable-width tests; check line counts, formatting, and workspace Clippy.
5. Record the source-cited slice in `docs/roadmap.md`, commit, and push independently.
