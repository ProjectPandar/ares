# FlushMode remembered-candidate map pair

## Source and destination

OrcaSlicer 2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/FilamentGroup.cpp:427–566`, owns remembered-candidate retention
and AMS postselection. The private public-API regression is
`crates/ares-cli/tests/orca_parity/runner/config_block_roundtrip/nullable_vectors/filament_group_flush_remembered_pair.rs`.
The production destination remains the engine implementation plan's private
`tool_ordering/filament_group/{remembered,ams}.rs`; this test adds no solver.

## Non-negotiable test boundary

- Reuse the existing fixed small-gap and outside-gap archives without edits.
  They share model/plate/preset payloads and differ only in flush-matrix
  nonzero entries, `3` versus `4`. No new fixture or expected-output golden.
- Verify both Native endpoints with the existing double-replay harness.
  Literal Native maps are premise checks; Ares maps are compared against
  those runtime Native results, never against stored expectations alone.
- Complete successful public API and CLI slicing for both endpoints before
  target map equality. Compare complete API/CLI bytes and retain strict
  whole-Native comparison diagnostics without adding JSON artifact writes.
- Preserve every existing mechanism test and its complete field assertion
  order. A shape failure in the original small-gap test is not scoring credit.
  The additional pair observes map behavior without replacing that test.

## Included and deferred verification

Native changes from `1,1,1` to `1,2,1`; the current all-master compatibility
shell does not change. The pair distinguishes the composed remembered-gap
and empty-inventory selection behavior. It does not independently isolate
every heap tie, nonzero relative-gap branch or individual score contribution.

Synthetic CLI AMS records have capacity but no `tray_name`; usable inventory
is empty. Nonempty inventory, colour matching, initial loaded state and
multi-nozzle contexts remain deferred. No new inventory producer, shared API,
configuration adapter or geometry borrow belongs to this test slice.
