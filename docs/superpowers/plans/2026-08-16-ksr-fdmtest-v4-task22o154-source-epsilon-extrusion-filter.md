# Plan: Task 22O.154 source-epsilon extrusion filtering

1. Add a failing KSR assertion that rejects the observed zero-extrusion fitted segment.
2. Introduce the upstream `1e-4` mm threshold in the motion emitter.
3. Skip sub-epsilon fixed lines and fitted arcs without moving emitter state.
4. Skip sub-epsilon variable segments while retaining their wipe points and prior emitted endpoint.
5. Run focused and complete motion contracts, generate the CLI slice, and identify the next exact divergence.
6. Run rustfmt and Clippy, record the roadmap milestone, then commit and push independently.
