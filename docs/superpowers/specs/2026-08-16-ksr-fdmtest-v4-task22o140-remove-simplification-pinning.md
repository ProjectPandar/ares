# Spec: Task 220.140 remove simplification checkpoint pinning

## Observable contract

Project simplification remains exercised transitively through public `slice_project` behavior and the KSR G-code assertions. Tests must not require private ARES22H/ARES22I checkpoint bytes, fixture digests, record lengths, contour counts, or the exact internal placement and tolerance representation of the simplification stage.

## Removed pinning boundary

The deleted Task 22I tests encoded intermediate `PostClosingPrintObject` state and compared source-stage magic bytes and SHA-256 digests. Those assertions constrained Ares to an internal OrcaSlicer-shaped checkpoint without defending emitted slicing behavior. The production simplification implementation and downstream observable tests remain unchanged.

Tests must also stop asserting the obsolete `ProjectSlicingIncomplete` result now
that `slice_project` emits G-code. Synthetic printer-model mutation coverage
that combines a generic printer identity with Bambu-only templates is removed;
it no longer represents a valid external project contract. The retained
MT19937-64 reference test uses the canonical published sequence rather than
three transcribed incorrect constants.
