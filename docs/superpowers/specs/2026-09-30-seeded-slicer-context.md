# Seeded slicer Context and fuzzy-skin replay

Source/destination and invariants: ARD-0068.
Expose a per-job Context containing a random instance initialized by seed.
Exercise it through the real byte-input slicing interface: fresh equal seeds
produce identical full output; different seeds alter active fuzzy geometry;
interleaved independent jobs do not perturb one another.

Compare against an explicitly identified reproducible original build using
the same random-consumption contract. Preserve untouched official controls
and report their nondeterminism, not successful parity. Fixed seed does not
by itself prove thread-schedule independence.

Exit: interface/consumption design review, RED/GREEN behavioral tests,
actual original repeatability and complete paired-output comparison, full
checks, independent six-axis review/repair, then code+docs commit/push.
