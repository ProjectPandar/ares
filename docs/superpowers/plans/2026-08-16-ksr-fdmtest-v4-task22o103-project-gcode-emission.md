# Task 22O.103 implementation plan

1. Add a source-cited concentric fill module and route
   `SurfaceFillPattern::ConcentricInternal` through it.
2. Add a crate-private project emitter that borrows resolved typed settings and
   ordered prepared entities, emits header/config/machine metadata and entity
   geometry, then lets the existing disposal chain release ownership.
3. Route CLI `.3mf` input through `slice_project` while retaining typed JSON
   options for `.stl` input.
4. Convert only valid-project lifecycle tests that asserted the obsolete
   terminal incomplete result into output assertions; preserve malformed-input
   precedence tests.
5. Run formatting, focused nextest, strict workspace Clippy, LOC/macro/diff
   audits, and the ignored KSR golden to record the real remaining first
   difference.
6. Run an independent read-only six-axis review, fix all findings, rerun the
   review, then commit and push this slice.
