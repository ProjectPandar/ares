# Plan: sparse-anchoring reference center

Source boundary and deferred behavior:
[spec](../specs/2026-10-04-sparse-anchoring-reference.md),
[ARD](../../architecture/ard-sparse-anchoring-reference.md).

1. RED on main94a0a629, independent branch `pi/bridge-sparse-reference` in
   `/tmp/ares-pi-matrix-vendors`, isolated target
   `/tmp/ares-pi-matrix-vendors-target`. Run unchanged strict public replay for16
   residuals copied from frozen read-only stock artifacts. Inventory
   `/tmp/ares-sparse-reference-replay/inventory.json`; RED
   `/tmp/ares-sparse-reference-red/replay-summary.json`.
2. Replace the reference with upstream PrintObject-centered0,0, then remove
   only the newly unused slice-center helper/re-export. No bridge math changes.
3. Repeat16; repeat full68 and original52+823 green controls. Check8 previously
   freshly sliced official CPU0 references. Keep all failures and report actual
   outcomes; no weakened comparator, new skip or deleted assertion.
4. Run workspace Nextest, fmt, default Clippy and WASM build (core+adapter).
   Inspect probes, ownership boundaries and diff. Push only this bounded slice,
   print REVIEW-REQUEST and wait for coordinator APPROVED before latest-main
   rebase/revalidation and fast-forward-only integration.
5. Only after this slice completes proceed to #9 Qidi, then #11 extra-overhang
   perimeter upstream rewrite. The latter must port PerimeterGenerator behavior,
   never just delete the existing unsupported guard.

Bridge14 handoff requested by coordinator is independently available at
`/tmp/ares-bridge-residuals-for-tower.md` (case IDs and complete first differences).
If this reference correction changes those residuals, report the new evidence
before pi-tower spends effort on the old output. Baseline handoff is retained.

## Observed scope after the one-reference correction

The16-case RED is0/16PASS, all DIVERGENT on94a0a629. Candidate results:
14/16PASS, full68=66PASS/2DIVERGENT,823/823controlsPASS, official8=7PASS/1DIVERGENT.
All14 Snapmaker Internal Bridge cases now match complete ordered bytes; the
original52 successful labels remain PASS in full68. Both Qidi0.2 cases retain
identical first differences and are explicitly #9, not part of this #10 fix.
No test or expectation changed; residual16/full68/official8 still fail their
all-case assertion and their reports are preserved, never advertised all-green.
A separate14-case inventory consists exactly of the pre-existing bridge handoff,
not a new comparator or rewritten fixture.

The coordinator was notified to pause pi-tower investigation of the14 old E
mismatches. `/tmp/ares-bridge-residuals-for-tower.md` now has a candidate-result
update above the unchanged original case IDs and first differences.

## Verification receipt (base94a0a629)

- Exact pre-existing14 bridge cases:14/14PASS;
  `/tmp/ares-sparse-reference-bridge14-green/replay-summary.json`.
- Full16:14PASS/2QidiDIVERGENT;
  `/tmp/ares-sparse-reference-residual16/replay-summary.json`.
- Full68:66PASS/2QidiDIVERGENT;
  `/tmp/ares-sparse-reference-full68/replay-summary.json`.
- Prior-PASS stock controls:823/823PASS;
  `/tmp/ares-sparse-reference-controls823/replay-summary.json`.
- Official CPU0 references:7PASS/1QidiDIVERGENT;
  `/tmp/ares-sparse-reference-official8/replay-summary.json`.
- `cargo nextest run --workspace`:7594/7594PASS,9 pre-existing skips;
  `/tmp/ares-sparse-reference-workspace.log`.
- `cargo fmt --all -- --check`:exit0; `/tmp/ares-sparse-reference-fmt.log`.
- `cargo clippy`:exit0 with249 pre-existing core-library warnings;
  `/tmp/ares-sparse-reference-clippy.log`. Not a warning-free or strict-D claim.
- Actual `cargo build --target wasm32-unknown-unknown -p ares-core -p ares-wasm`:
  exit0; `/tmp/ares-sparse-reference-wasm.log`.
- Aggregate exits: `/tmp/ares-sparse-reference-exits.txt`; the16/full68/official8
  exit100 entries remain intentional evidence of unresolved #9, not successes.

Production delta: one sparse-anchoring reference plus comment; delete the28-line
now-unused object-center helper and its re-export. No new test code, skipped
fixture, suppressed warning or probe. Await coordinator approval before integration.


