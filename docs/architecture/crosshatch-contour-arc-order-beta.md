# Contour arc order selection — slice B

Authority: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`Fill/FillBase.cpp:1822–1898`: `Arc`, `count_joins`, and the forward/backward
`along_contour` sorting candidates. Rust destination:
`fill/connect/order.rs`, called from `apply::apply_connections` after
collecting eligible arcs and before applying sorted arcs. Existing length-only
sorting is a temporary compatibility shell, not an independent ordering policy.

## Included and non-negotiable behavior

Retain the original length sort as the first candidate. Simulate full joins
with private consumed flags and DSU roots, including the remaining-endpoint
pass, without mutating the real graph. Multiline arcs below spacing*multiline
are skipped. Full joins use strict length < join_length in the sorted pass;
the remaining pass includes length == join_length. Both simulation thresholds
use `FillBase.cpp:1731`: when anchor_length > scaled epsilon, join_length is
max(anchor_length_max, 2*(anchor_length + 0.5*scaled spacing)); otherwise it
is anchor_length_max. The actual connector-application correction at
`FillBase.cpp:1919,1981` is retained as a separate implementation commit.
Correction to that commit's message: anchor_length_max contributes to
join_length only; it is not a hook-fallback threshold. Its other upstream
use is inside the disabled `#if 0` block at 1733–1806. Hooks use
anchor_length, with consumption gated by anchor_length > scaled epsilon. Compare both contour directions in upstream
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

## Qualification

The new ordered crosshatch witness fails on A1+A2 and passes with this port;
all five directed witnesses pass. The unchanged 421-test inventory moves from
204 PASS / 217 FAIL to 211 PASS / 210 FAIL: seven FAIL-to-PASS and zero
PASS-to-FAIL. Improvements are the by-object/mirror crosshatch targets, one
TPMS-D anchoring control, three honeycomb direction controls, and the support
touching-grid public API/CLI case. The new witness is additional to that
inventory. Remaining failures are not claimed byte-identical or fixed.
Rustfmt and all-target Clippy pass with zero warnings; no lint suppressions
were added. This qualification is local, before the pending main rebase.
