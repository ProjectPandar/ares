# Stock printer inventory implementation plan

Source citation, included/deferred scope and temporary scaffold limitations:
[spec](../specs/2026-10-04-stock-printer-sweep.md), ARD-0096.

1. Add a separate acceptance/report regression: errors remain in the report
   and the nonempty all-PASS requirement cannot excuse them. Show RED.
2. Connect complete vendor/machine enumeration to the existing stock export,
   with no preset rewrites or filters. Make the gate regression GREEN.
3. Start the full seeded CPU0 native run; keep paired whole-file artifacts
   and every failure. Preserve the already live eb65bca3 option scan unchanged.
4. Verify the existing stock control, workspace, fmt/Clippy and independent
   six-axis review. Record live runs as live, never as completed acceptance.
5. Commit/push the tested harness and docs; repair actual upstream-owned
   stock output divergences as subsequent TDD slices. Do not wait for unrelated
   domain fixes to commit the verified acceptance entrypoint.

## Verification record

- RED: `/tmp/ares-stock-sweep-red.log`, Nextest exit100. The persisted
  report retained ORCA_ERROR, but the incomplete result still returned success.
- GREEN: `/tmp/ares-stock-sweep-green.log`, exit0, two report/acceptance
  regressions pass (errors remain visible, empty/unknown/error rows fail).
- Fresh existing stock Ender-3 complete-output control:
  `/tmp/ares-stock-sweep-control.log`, exit0, 1/1 PASS against seed0/CPU0.
- Full unfiltered stock run started on base `eb65bca3` plus this test-only
  patch: `/tmp/ares-stock-matrix-eb65bca3-wip-seeded.log`, external artifacts
  `/tmp/ares-stock-matrix-eb65bca3-wip-seeded`. It is currently live, not a
  completed inventory or parity claim. Generic cached artifact provenance
  remains unknown; the explicitly configured seeded executable is the same
  ELF identified in the preceding stock-preset plan. No reference binary edits.
- The previous full 650-domain option scan remains on its original live
  process, immutable loaded test binary and artifact root
  `/tmp/ares-option-matrix-eb65bca3-seeded`. It uses the existing modified
  option baseline; do not relabel it as stock-printer evidence.
- Initial verification predates the discovery repair; use the final
  repaired-code receipts below for checkpoint acceptance.

## Independent review repair

The reviewer found possible false GREEN on partially failed discovery:
legacy `vendors` silently flattened directory-entry failures and treated
metadata errors as absent vendors. Stock discovery now checks those external
I/O boundaries; absent nonmachine collections alone are omitted. A dangling
machine symlink is an error, not an absent collection. Shared preset indexing
also propagates nested metadata errors rather than skipping broken entries.
The original modified-matrix vendor helper remains unchanged.

- Discovery RED: `/tmp/ares-stock-discovery-red.log`, two tests fail against
  the legacy discovery adapter; initial GREEN four tests pass in
  `/tmp/ares-stock-discovery-green.log`.
- Nested indexing RED: `/tmp/ares-stock-nested-discovery-red.log`, the
  broken nested entry was silently ignored alongside a valid preset.
- Report counters are now `outcome_rows`/`passed_rows`; vendor-error sentinels
  are not fabricated printer counts. Recursive raw profile enumeration
  currently finds1001 instantiated machines (flat-directory glob finds only913).
- The first live stock scan uses the pre-review loaded test binary and report
  schema, not the repaired acceptance entrypoint. It remains useful output
  evidence, but cannot validate the new discovery gates. Final fixed-code
  gate tests, full workspace, control and review are required independently.
- Real live output residuals already preserved: Anycubic Kobra 3 Max0.4 first
  bounding-box difference at byte758/line26; native min203.951,200.951 versus
  Ares203.929,200.929. Kobra3 Max0.6/0.8 hit the existing
  `extra_perimeters_on_overhangs` rejection. These are future source-owned
  fixes, not excuses or completion successes.

## Repaired-code verification

- `/tmp/ares-stock-sweep-fixed-green.log`: exit0, 7/7 acceptance/discovery
  regressions PASS. The two symlink-specific checks are Unix-gated; the
  remaining external filesystem tests are portable. No new core I/O.
- `/tmp/ares-stock-sweep-fixed-control.log`: exit0, stock Ender-3 native
  seed0/CPU0 complete-output control PASS after shared indexing changes.
- `/tmp/ares-stock-sweep-fixed-workspace.log`: exit0, 7600/7600 PASS,
  9 skipped, 265.024 seconds. The earlier pre-review 7595 PASS receipt is
  not substituted for this final snapshot.
- `/tmp/ares-stock-sweep-fixed-{clippy,fmt}.log`: exit0; unchanged warnings
  253 core library /269 library tests /31 CLI parity.
- Independent six-axis final review read back terminal receipts and staged
  eleven-file patch, closed the discovery omission and denominator findings,
  and approved the finite matrix-branch checkpoint. Both native full scans
  are still live and incomplete; no global completion approval.
