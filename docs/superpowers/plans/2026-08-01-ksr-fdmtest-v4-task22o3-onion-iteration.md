# Task 22O.3 execution plan

1. Freeze the fixed OrcaSlicer commit and loop-back boundary
   `PerimeterGenerator.cpp:1304-1387`; keep Task 22O.2 immutable and stop before
   hierarchy line 1388.
2. Add a typed successor that nests each Task 22O.2 object and carries aligned
   records of ordered raw depths, final `last`, gaps, and effective count.
3. Validate typed effective `sparse_infill_density` for every record, convert it
   to the source `int` local, then consume the predecessor; source all spacing
   and gap-enable values from the reviewed predecessor.
4. Seed depth zero and `last` from their distinct Task 22O.2 outputs, then port
   internal `offset2_ex`, gap append, collapse, extra-pass, and converted-int
   zero gates with source-exact f64-to-f32 cast points and fixed coordinates.
5. Add separate direct-geometry/config tests and typed KSR option-mutation tests.
6. Advance the lifecycle through Task 22O.3 while retaining
   `ProjectSlicingIncomplete`.
7. Update the architecture ledger and roadmap without claiming hierarchy,
   extrusion, G-code, or complete Task 22O.
8. Run focused and workspace nextest, warning-denying Clippy, rustfmt, native and
   WASM checks; verify touched Rust files are below 400 lines and scan for
   forbidden runtime/oracle/filesystem/source-pinning constructs.

Acceptance requires deterministic source ordering, no edits under
`classic/top_split*`, unchanged Task 22O.1/22O.2 behavior, whole-stage failure
on invalid density, and review of the bounded diff.
