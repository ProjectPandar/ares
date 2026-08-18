# Plan: KSR FDM Test V4 infill entity chaining

1. Add a failing first-layer KSR assertion at the `slice_project` seam for the reference first infill travel and first extrusion, plus a focused gap-loop reversibility assertion.
2. Expose the existing source-derived entity chaining module to production and make island emission consume each infill phase from the live `EmitState` XY cursor.
3. Chain eligible collections before emission; after selecting a collection, chain its sortable paths from the then-current cursor while preserving `no_sort` path order.
4. Remove or replace exact assertions in the obsolete motion pinning test that encode the pre-chaining entity order.
5. Run the focused KSR test and regenerate the CLI output to verify the first infill block. Then run nextest, clippy, rustfmt, commit, and push.
