# Task 22O.103: project G-code emission

## Requirements

- Route `.3mf` CLI input through `slice_project`; keep STL `--options`
  behavior unchanged.
- Preserve typed project settings and resolved layer/entity ownership through
  a crate-private emitter; do not read reference G-code or branch on fixture
  identity.
- Emit an Ares header, the existing typed config block, extrusion-width
  metadata, machine-envelope commands, ordered layer markers, and all prepared
  perimeter/fill/thin entity paths.
- Materialize `ConcentricInternal` groups through an ordinary module and the
  existing geometry offset kernel, retaining role/flow/width/height metadata.
- Keep source and test files below 400 LOC and use no `include!` or
  `include_bytes!` source splitting.
- Replace valid-project tests that asserted the obsolete terminal
  `ProjectSlicingIncomplete` boundary with output assertions; malformed input
  must continue returning its validation error.

## Deferred behavior

The cited Orca behavior not included here is full `WallToolPaths` Arachne
variable-width conversion, placeholder expression evaluation, machine start /
end expansion, seam placement, arc fitting, cooling/time estimation, and exact
motion/G-code formatting. These require later source-cited slices and are not
replaced by fixture-specific output.

## Acceptance

Focused typed-project tests and the concentric test pass. The CLI golden test
must run successfully far enough to report normalized first-difference output;
full byte parity remains a later acceptance gate until the deferred slices are
ported.
