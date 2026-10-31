# PCP bounded embedded coStrings GREEN — final evidence

## Source boundary and limits

This is the bounded `print_compatible_printers` classification slice from
[the approved SPEC](option-print-compatible-printers-embedded-strings.md), not
an Ares pipeline feature or global string-domain completion. The upstream pin
is `8500fcdccaa10b5099ac20d252af3a7c560046f1`: `PrintConfig.cpp:1832–1834`,
`Config.hpp:1118–1158` (`ConfigOptionStrings`), `Config.cpp:72–120,146–218`
(serialization/escaping), JSON vector reading/writing, and
`Format/bbs_3mf.cpp:2632–2653,7722–7727` (embedded settings read/write).

Destination: only `ares-cli` parity-harness domain/classification/metadata and
key-local embedded archive witnesses. Existing public Ares load, typed preset,
Serde and config-export code supplies fidelity without production changes.

The seven canonical cases are absent, explicit empty array, current-machine
plus empty element, one label, ordered duplicates, escaped elements and
Unicode. Nonempty vectors contain the fixed source machine ID by the owner’s
admission ruling. The original source baseline is an eighth, separate control.
`missing-default`'s expected typed value `[]` means raw `remove_key`, not an
injection or promotion of a null baseline. Global null/unproven handling,
comparators, budgets and original assertions are unchanged. Unproven remains
INCOMPLETE. Only this key changes among 650 plans; all other 649 are exact.

Deferred: worker5’s real Native `-17` nonmatching-machine rejection, filtering,
load-settings/overlay semantics, lone-empty singleton without machine ID,
malformed/scalar/nested JSON, arbitrary vectors and other coStrings keys. The
original `-17` and erroneous footer-scalar Serde adapter attempt remain retained
as admission/harness failures, not classification RED. Semicolons demonstrate
that a footer token need not be reversible: complete-stream comparison covers
serialization bytes; actual Native exported 3MF JSON proves the typed vector.

## SPEC → RED → GREEN and gates

Product/test commit: `f45f5e824b737beaf8c93f5bbe6c12eef3e79ff8`.
Baseline: `f45b8194144615c5322361826554c31ffe2fffd5`.

Accepted classification RED was independently recorded at
`4f100a4c5d5e1c9d7eb84581e0e2988565385682` on production880: all eight fidelity
and full-output comparisons PASS; only INCOMPLETE versus PASS failed (exit100).
Evidence `/tmp/ares-pcp-embedded/red-880-official/`. Historical originals are
not rewritten by subsequent rebases.

GREEN promotes only the finite key-local plan, delegates that key’s sweep to
the existing eight-case fidelity routine (never generic load-settings export),
and exposes raw-action/typed-expectation/deferred-scope metadata. The touched
key-local Rust file is 303LOC. Official/seeded cold/warm directed tests each
PASS classification plus selected-key unfiltered-value sweep. Warm project
export cache does not turn the per-case footer/export witnesses into frozen
Native data. Evidence `/tmp/ares-pcp-embedded/green-{first-official,official-hot,seeded-cold,seeded-hot}/`.

Final production a539 fresh-target gates: workspace **8278 PASS, 5 configured
skips**, workspace/all-target/all-feature Clippy `-D warnings`, fmt and WASM
check all exit0. `/tmp/ares-pcp-embedded/gates-a539/`. Owner authorized carry
onto f45's tests/docs-only advance. Earlier880/626/711 gate attempts are
historical or explicitly VOID; no stale production-tree gate is substituted.

## Fresh full control pairs — retain original outcomes

Root `/tmp/ares-pcp-embedded/final-f45-isolated/`. Independent empty target
directories and completed archives, jobs2; distinct binary SHAs checked before
execution:

- BASE: `2e4a74eb8e3348496a4860fd21cef0477a80d7ed9a0dbb33c0a7c6ad497c7b8e`
- CAND: `d94b6212ba3df10501dca2110768dc0586116dc41489793c8612d749ece4ff6c`

Four unfiltered control legs used two nextest threads and per-leg actual
Native-start load admission. Archive adapter removed only incompatible
package/test selectors; no value/comparator filtering.

| Tree | Seeded | Official |
| --- | --- | --- |
| BASE | 911 PASS, exit0 | 911 PASS, exit0 |
| CAND | 912 PASS, exit0 | **911 PASS, 1 FAIL, exit100** |

Native nonexecution is explicitly disclosed independently of nextest's zero
skip counts: each seeded tree has nine disabled Native families; each official
tree has those nine plus sixteen seeded-only guards (25). Full option sweep is
one of the nine disabled families; selected-key Native sweep evidence is the
separate directed runs. Exact skipped lines are in `final-owner-ruling-audit.json`.

## Owner-adjudicated Native nondeterminism — no full rerun

The sole original candidate-official failure is
`runner::tree_support::manual::vertical::manual_slim_nested_vertical_painting_preserves_complete_output`.
It is **not deleted, converted to PASS, retried until PASS or a new comparator
exception**. No blanket tree-support waiver follows from this ruling.

First difference: byte18553, line755, column9, internal-solid-infill travel:
Native `G1 X113.388 Y113.135 F9000`, Ares
`G1 X113.244 Y113.297 F9000`. Original candidate artifact
`candidate-official/artifacts/case-WmTwQ3/` is retained in full.

Input raw ZIP SHA is identical across original four legs and fresh BASE/CAND
directed runs: `6532c18bae59c9fdc4c37b8e4e144ccc258170dcfb26c33f4895609cc069bf66`;
every ZIP member name/length/full SHA is also exact. Ares raw SHA is constant
across all six Rust runs:
`a620a2f8c72f273951f917a18a9c0b02296543de229c24d5546fae009deb4483`.

Official BASE and CAND each had one authorized fresh-cache directed run,
both PASS. A fixed-count ten additional Native runs on that identical input,
original receipt CPU12/producer and fresh output directories all exit0. **The
ten alone show one variant**, not two. Combined fourteen official samples show
two whole-output variants, after only the existing generated-by identity rule:

- `fb9a8e80d9f2b2f125159ffc51e502e38042efb892b13134c157e882f7c81c8d`: 13;
  every one is whole-output exact to the stable Ares output.
- `2107796d56982ba5dd6bd224257b6a4c3d9f71d6e078ccd8b1abac9aef57f9dd`: 1;
  the original failed Native raw SHA is
  `19adcc1ebb7f148b3464f6f8ed9065965ff79494796d2250387a70690a2b0684`.

Authoritative machine evidence is
`/tmp/ares-pcp-embedded/final-f45-isolated/nested-vertical-diagnosis.json`;
`native-repeat-10/` retains every raw Native SHA, command, load admission,
stdout/stderr, exit and complete output. Owner explicitly ruled Native
nondeterministic from fourteen official samples/two variants, six stable Ares
outputs and identical input, and authorized disclosure/review without rerunning
full. This is a ruling about this original evidence, not global nondeterminism
coverage or new motion/timestamp filtering.

## Exhaustive changes and inventory

`pair-audit.json` and strict audit exit1 remain unchanged: one added PCP test,
eight new candidate-only PCP input cases per mode, and the original official
failure remain visibly recorded. `final-owner-ruling-audit.json` separately
checks the explicit addition ledger and named owner ruling; it exits0 without
rewriting raw outcomes.

All existing inputs have exact full member matches: official579 label groups/
628 streams, seeded590 groups/639 streams. **Zero shared fixed-input raw Ares
generator changes.** The eight added candidate cases each PASS complete output
against their own live Native; they are additions, not hidden missing pairs.
The complete 650-row plan audit changes only PCP, with other649 exact.

`all-raw-inventory.json` exhaustively retains every `.gcode`, not a filtered
motion inventory: seeded BASE1930/CAND1954 files, official BASE1846/CAND1870.
The eight added Ares generator outputs per mode are explicitly listed.
Nine Native-derived processor-Ares outputs per tree/mode change with their live
Native-derived input; all are byte-exact to their own live Native output.
`processor-raw-changes.json` distinguishes these from fixed-input generator
changes. All Native/input/processor/helper raw changes remain in the inventory.

No full rerun, unrelated core repair, new probe/API, frozen fallback, stash,
force push or relaxed comparator was used. Exact-hash review/integration is
still required; this record itself is not permission to push.
