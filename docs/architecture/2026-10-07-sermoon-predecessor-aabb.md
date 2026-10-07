# Sermoon predecessor AABB parity

## Evidence and non-negotiable boundary

Port `AABBTreeLines.hpp::LinesDistancer<ExtrusionLine>` and
`AABBTreeIndirect.hpp::build_recursive/partition_input` plus
`squared_distance_to_indexed_primitives_recursive` as used by
`SupportSpotsGenerator.cpp::estimate_malformations` (lines 170–175).
Destination: existing `geometry/line_distance_tree/floating` and the
previous-layer query in `gcode_emit/motion/overhang/curl.rs`.

The linear predecessor scan is a temporary compatibility shell. Replace
it with the equivalent balanced AABB build and containing-box traversal.
Never implement an input-index tie preference, last-match rule, or
Sermoon-specific corner exception. Strict improvement preserves the
first visited hit, not the first input line. Keep native f32 bounds,
centroids and query precision for upstream `ExtrusionLine::Scalar`;
retain native f64 for the existing `Linef` callers. Share the existing
build/traversal algorithm rather than duplicate trees or annotation.
Return the primitive index so its chained height is read from the
original previous-layer line set. Build once on layer rollover.

Included: native float precision, primitive index, AABB predecessor
selection and existing height decay. Deferred: other annotation slices,
progress-line attribution and adjacent slicing behavior. No new Ares
pipeline or legacy fallback is introduced.

## Authoritative probe

Recipe: `tools/orca-probes/bottomline-src.nix`, exact seed-replay producer
source `/nix/store/g3wj93lyhjsvc32fmgvw0cdbd6191q55-orca-slicer-2.4.2-seed-replay-source`.
Probe binary `/nix/store/4zs39jdm6cg6xzrnlpwk3nmfm031pcxs-orca-slicer-2.4.2`;
`bin/.orca-slicer-wrapped` SHA256
`52c9d8e2ffabca89472c67fbb019cc9e2486a6e402587668ee522097667a1873`.

Run with `ORCA_REPLAY_SEED=0`, CPU 3 and staged input
`/tmp/ares-orca-cache/defb366df2562374/3d02bff68752da9a-c07acddfbf16cc60.3mf`.
`/tmp/bottomline-replay-recheck/plate_1.gcode` equals
`case-6qvP0t/orca.gcode` byte-for-byte after removing generated-by only.
Probe `/tmp/bottomline-replay-recheck/lines.txt`:

| index | line | squared distance | height |
|---|---|---|---|
| 0 | repeated (+4.645000935,+4.645000935) corner | 0 | 0.476121098 |
| 1 | top edge, right to left (**selected**) | 0 | 0.316996664 |
| 2 | left edge | 86.3041347 | 0.312911779 |
| 3 | bottom edge | 86.3041347 | 0.312911779 |
| 4 | right edge, bottom to top | 0 | 0.300293654 |

Previous plain-source probe x42gflf2 is quarantined: its full output did
not match the seeded case even with logging disabled. It omitted the
seed-replay producer's concentric initialization patch. Do not use its
output as parity evidence.

Core RED committed separately as `7baa7837`: native nextest observed
`0.3261211 != oracle 0.166997`. Whole-file GREEN remains mandatory before
review; M73 omission in the attribution test does not prove M73 parity.
