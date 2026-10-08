# Stage1 object range-height verification

Status: numerical GREEN, fresh full seeded gates and authorized post-rebase
checks complete. Coordinator review required before Rust push.
Approved source spec and runtime RED remain in this branch's two-commit chain.

## Source slice and retained boundaries

Port Slicing.cpp:234-305 `layer_height_profile_from_ranges` into profile.rs;
planning passes source-object data with SlicingParameters into existing layer
pair interpolation (Slicing.cpp:807-870). Respect Slicing.hpp fixed-first-layer
predicate and relative uncompensated top. Fixed-only shell no longer supplies
production profiles. No new Ares adaptive algorithm.

Height-only ranges do not split region candidates: PrintApply.cpp:958+ creates
PrintRegion configurations per range and :1000-1011 reuses equal configurations.
A height-only range produces the same PrintRegion; Rust preserves that equivalence.
Raw range gate admits explicit height and no region overrides only. Mixed missing
height is upstream missing-option dereference UB; existing unsupported boundary
remains, without fabricated defaults or crash imitation. Pure None/region overrides
retain previous rejection. Painted profiles, automatic SlicingAdaptive and active
ZAA remain deferred; this does not port region-range slicing.

## Test-first evidence

Runtime RED (before production) run2929f3f7-8af6-465c-8364-a9b68c151954:
Native succeeded, Ares rejected `layer_height`, exit100. First attempted GREEN
revealed the second raw-range scaffold rejection; the coordinator approved its
height-only equivalent mapping before implementation. The failure is retained,
not counted as GREEN. See the RED report and committed fixture/producer receipts.

Primary: original seam overhang geometry with only range XML appended. Nominal
height0.2, first0.3, range0.3..0.6 at0.1. Native actual (z,h):
(0.3,0.3),(0.4,0.1),(0.5,0.1),(0.6,0.1). Thus range height changes actual layers
and exercises worker6 seam-current-layer-height path.

Additional independent single-attempt Native probes:
- overlap: reversed XML order, first-layer clipping, sorted overlapping intervals;
  (0.3,0.3),(0.4,0.1),(0.5,0.1),(0.65,0.15).
- gap-top: fixed first-layer override, default gap, top clamp;
  (0.3,0.3),(0.5,0.2),(0.65,0.15).
- epsilon: sub-EPSILON range ignored;
  same schedule as gap-top, independently run (no deduplicated execution).

All four complete Native/Ares outputs compare with only the entire generator
line removed. Raw bytes and input/Native receipt hashes are committed alongside
fixtures. Mixed-height/region-only rejection is an E2E control, never invoked as
Native numerical oracle. Existing seamD fixed-height control also matches.

## Gate rounds (each fresh, no retry-to-PASS)

| Round | Base / candidate | Seeded | Other |
|---|---|---|---|
| first same-main |1a8c92f55 /2d3609786|660/660 /663/663 PASS|clippy -D warnings, fmt, WASM, targeted4 PASS|
| after seamA/scarf |016456549 /283023e6c|667/667 /670/670 PASS|clippy -D warnings, fmt, WASM, targeted4 PASS|
| after BBL physical tool placeholder |bff9cb1b0 /5526e4b12|668/668 /671/671 PASS|clippy -D warnings, fmt, WASM, targeted4 PASS|

Base lacks the three newly introduced range test functions; candidate adds them
(one function visits three independent boundary inputs). Do not count missing
base tests as coverage. All common tests pass. Every round uses16 parity threads,
CARGO_BUILD_JOBS8, independent base/candidate targets, fresh per-stage Native
cache/artifact roots. Only one heavy command chain active per worker.

Workspace (before final rebases) run0048cbc6-94e6-4444-a24d-08cc5708ba9c:
**8053/8053 PASS**,9 existing skips,750.815s. No new ignored test. Later main
CLI/test-only changes require targeted and clippy after rebase per coordinator.

Final bff seeded audit: **552 common complete Ares raw hashes preserved**;
candidate556 outputs adds exactly four range witness streams (two happen to
have identical bytes). No base-only raw or generator-normalized hash. Earlier
1a8 seeded similarly preserved543 common raw outputs. These audits retain full
raw streams and do not normalize motion or M73.

## Official gate — explicit non-PASS carry-forward

Same-round1a8 official: base650 PASS/10 FAIL of660; candidate653 PASS/10 FAIL
of663, identical failure names. Coordinator permits carrying official evidence
through subsequent upstream changes; it is not a fresh bff official run.
-8 `NotPresent` environment initialization errors, not numerical PASS.
-2 X2D full-output assertion divergences, **not timeouts**. First differing line
  is Native's `printing object ... id` versus Ares stable id0. Bridge Native ID
  varies4575657223555907584→9223372037920129024; narrowed Native ID
  2147483648→1065204784. Native hashes therefore differ between attempts.
  Both base/candidate Ares complete assertion streams have identical hashes
  (bridge331827159..., narrowedff469e046...). This establishes no candidate-only
  regression, not official parity PASS or new stack-proved UB classification.

Official normal-output audit preserves525 common raw Ares hashes, candidate
529 includes four new ranged witnesses. Complete source-linked first differences
and failure names retained in the machine record. No rerun to hide these failures.
An initial audit assumption that Native hashes would match failed; corrected
read-only audit retained differing Native hashes instead of retrying Native.

## Authorized evidence carry-forward through later main changes

Coordinator authorizes retaining the completed bff fresh seeded round and1a8
same-round official evidence for this slice after later rebase, with clippy
`-D warnings` and this slice's targeted tests rerun. `3c1a79941` Arachne open-wall
change affects only six narrow-neck cases (FAIL→PASS); no claim this report's
bff full round scanned those new cases. `de57be8b0` J1 affects only six J1 cases
(tool-use mask/initial template current_extruder/final fill-surface Top input).
This slice does not edit avoid_crossing, and coordinator permits carry-forward
for it. `dc8d7f00c` error/CLI typing, `92e9573df` test-only, `8264d38ab` web-only
changes do not require new full numerical rounds under the same instruction.

On3c1, clippy -D warnings/fmt exit0 and targeted4/4 PASS (5.405s) were verified.
Candidate rebased on exactde57be8b00fb0e3028acb725cfdc85e74b2831e8:
03922a6d387dff7eaad2c0ef3eb52a22603631b2 (before this final docs commit).
Post-rebase clippy -D warnings/fmt exit0 and fresh targeted4/4 PASS (3.103s),
681 other tests filtered, no ignored tests. These are slice-targeted receipts,
not a fresh full de57 seeded or workspace claim.

## Durable receipts

[Gate record](2026-10-08-object-layer-height-range-gate-record.json) binds full
byte-audit rows, producer/build identity, failure/first-difference receipts and
log SHA256s. Original logs/artifacts `/tmp/worker3-layer-range-{1a8,016,bff}-*`.
Original runtime RED `/tmp/worker3-layer-range-runtime-red.*` and independent
Native single attempts `/tmp/worker3-layer-range-{native-red,boundary-native}/`.

Submitting coordinator REVIEW-REQUEST; no Stage2 approval or implementation
implied, and no Rust push before approval. Browser pauses prevented launching
new heavy jobs; existing commands completed and all evidence stayed single-attempt.
