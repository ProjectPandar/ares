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

Run with `ORCA_REPLAY_SEED=0` on the staged input. The probe plate G-code
equals the seeded case oracle byte-for-byte after removing generated-by only.
Probe lines:

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
`0.3261211 != oracle 0.166997`. Whole-file GREEN remains mandatory; M73 omission in the attribution test does not prove M73 parity.

## Target GREEN

After the indexed predecessor rewrite, the corner regression passes.
Raw CLI output for the original staged case equals its saved oracle
including M73 (only generated-by normalized). Therefore the test
comparator has been tightened: M73 filtering removed from both operands
and from the native self-stability gate. Fresh-cache native nextest
`sermoon_v1_whole_file` passes under that strict comparison. Main stock M73 reorder is therefore
consistent with the fixed downstream motion timing, not a separately
proven insertion defect on this case. Full controls remain required.

## Correction: shared XYZ writer

The follow-up ports `GCodeWriter.cpp:975-1000::extrude_to_xyz` into
`motion/extrusion.rs`, shared by `scarf/emit.rs` and `path/variable.rs`.
Use a small segment command (XY endpoint, Z, delta E, path properties),
not a long positional argument list or lint suppression. Caller-specific
slope interpolation and flow calculations remain at the existing source
boundary. Bind slope and its total length into one optional tuple, so
there is no production unwrap or independently optional correlated state.

Compare Z at `GCodeWriter.hpp:226-228::quantize_xyzf` export precision;
remove the duplicated 1e-9 proximity check. The writer updates its raw
position on every XYZ extrusion even if quantized Z does not change.
The isolated RED fixture covers same-bin 0.40001 -> 0.40049, cross-bin
0.40049 -> 0.40051, and raw position tracking. Existing strict whole-file,
workspace and seeded controls remain unchanged acceptance gates.
