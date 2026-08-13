# Task 22O.77 implementation plan

1. Add focused RED tests for the missing private vertical-segmentation seam,
   beginning with a rectangle and diagonal rational intersection.
2. Implement source intersection records and vertical-line allocation in
   `fill/rectilinear/segments.rs`.
3. Add source rotation/outer-inner offset preparation in a sibling bounded
   module; preserve contour identities and errors.
4. Add hole, endpoint/tangent, duplicate, empty, range, repeatability, and
   immutability tests one vertical slice at a time.
5. Run focused/dependent/workspace Nextest, strict workspace Clippy, rustfmt,
   static/LOC checks, update roadmap/parity evidence, commit, and push.

## Completed evidence

The missing-module compile failure supplied RED. GREEN passes 3/3 focused tests
covering source intersection kinds/order, holes and offset identities, rational
rounding, rotation, repeatability, immutability, and range error. Strict core
Clippy, rustfmt, diff, and sub-400-LOC gates pass.

No link graph, monotonic traversal, entity output, lifecycle, old infills reuse,
or G-code belongs in this task.
