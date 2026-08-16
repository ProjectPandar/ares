# Plan: Task 22O.124 monotonic configured direction

1. Add a focused failing test that evaluates the same monotonic surface with consecutive layer indices and requires identical polylines.
2. Remove generic odd-layer rotation from the monotonic direction resolver while preserving the configured and bridge-angle paths.
3. Run the focused direction test and the KSR motion smoke test; compare generated motion totals to the prior output.
4. Run rustfmt and strict `ares-core` Clippy, then commit and push the isolated slice.
