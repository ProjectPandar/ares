# Contour arc order selection — slice B

Authority: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`Fill/FillBase.cpp:1809–1897`: `Arc`, `count_joins`, and the forward/backward
`along_contour` sorting candidates. Rust destination:
`fill/connect/order.rs`, called from `apply::apply_connections` after
collecting eligible arcs and before applying sorted arcs. Existing length-only
sorting is a temporary compatibility shell, not an independent ordering policy.

## Included and non-negotiable behavior

Retain the original length sort as the first candidate. Simulate full joins
with private consumed flags and DSU roots, including the remaining-endpoint
pass, without mutating the real graph. Multiline arcs below spacing*multiline
are skipped. Full joins use strict length < join_length in the sorted pass;
the remaining pass includes length == join_length. Hook consumption uses
anchor_length > scaled epsilon. Compare both contour directions in upstream
order, grouping by llround(length / (0.5*scaled spacing)), then contour index,
then ascending or descending contour parameter. Replace only when the join
count strictly improves; equal scores keep the incumbent. dont_sort still
bypasses this pass. This is shared connector behavior, not a crosshatch special
case. Use the existing fixed GCC sorting kernel for upstream comparator ties.

## Deferred and verification

A1 close-end materialization and A2 entity-chain improvement remain unchanged.
Do not change touching, trimming, hook thresholds, cooling, travel emission or
complete-output comparators. Existing crosshatch final ordered Native records
at layer six/Z1.2 provide the public byte-input/API-output RED. No Native
outputs are checked into the repository. Confirm RED on A1+A2 before porting,
then exact ordered GREEN plus the four existing directed witnesses. Compare
the same 421 fill/support parity selectors to the A1+A2 baseline; report
PASS/FAIL transitions without claiming that unchanged failures have identical
bytes. Clippy must have zero warnings and no added lint suppressions.
External evidence retained outside the repository.
