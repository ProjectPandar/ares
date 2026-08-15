# Task 22O.102: Arachne transition middle generation

## Source boundary

Port OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Arachne/SkeletalTrapezoidation.cpp:788-850`,
`SkeletalTrapezoidation::generateTransitionMids`, and use the existing
`SkeletalTrapezoidationEdge` transition payload vocabulary.

## Requirements

- Keep the implementation inactive and crate-private.
- Visit only central edges whose radius increases from `from` to `to` and whose
  bead count increases.
- Generate one middle for every lower bead-count transition using the pinned
  `BeadingStrategy::getTransitionThickness` threshold.
- Clamp transition radius to the edge interval and calculate the integer edge
  position with wide intermediates.
- Keep transition storage alive for weak references held by graph edges.
- Do not implement filtering, transition ends, ribs, toolpaths, or G-code in
  this slice.
- Use a separate test module and ordinary Rust modules below 400 LOC; do not
  add source-splitting macros or fixture-specific branches.

## Acceptance

- The focused O102 test proves ordered transitions, lower bead counts, and
  interval-bounded radii.
- `cargo fmt --all`, focused nextest, strict workspace Clippy, and diff audits
  pass.
- The change is committed and pushed before dependent work begins.
