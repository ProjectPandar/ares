# Stock preset parity implementation plan

Upstream files/functions, destination, included/deferred scope, and scaffold
limits: [spec](../specs/2026-10-04-stock-preset-parity.md), ARD-0096.

1. Add the public native Ender-3 export/output tracer through the existing
   modified exporter; show RED when native saved settings lose thin walls.
2. Export stock machine/process/filaments with no smoke rewrites; prove the
   same tracer GREEN and full output parity, without comparator changes.
3. Run official and seeded single-CPU references, workspace Nextest,
   fmt/Clippy, then independent six-axis review. Fix and repeat if needed.
4. Commit and push code plus documentation on the new matrix branch.

Remaining global gates: current complete stock printer inventory, every
boolean/enum value, each range's min/max and seeded interior, feature-active
fixtures, source-pinning cleanup, and Tier-1 runtime coverage. Historical
971/1001 modified printers and 615/650 option domains do not close them.

## Verification record

- RED: `/tmp/ares-stock-ender3-red.log`, Nextest exit 100; native application
  check reports requested `detect_thin_wall=1`, effective `0`. This used
  the existing smoke exporter, before changing the stock path.
- GREEN official 2.4.2 single-CPU: `/tmp/ares-stock-ender3-official.log`,
  exit 0, 1/1 complete-output PASS. Saved 3MF and both complete outputs:
  `/tmp/ares-stock-ender3-official/case-A17oIX`.
- GREEN separate seed-0 single-CPU reference:
  `/tmp/ares-stock-ender3-seeded.log`, exit 0, 1/1 complete-output PASS;
  artifacts `/tmp/ares-stock-ender3-seeded/case-T2uow0`.
- GREEN official default-thread control: `/tmp/ares-stock-ender3-default.log`,
  exit 0, 1/1 complete-output PASS; artifacts
  `/tmp/ares-stock-ender3-default/case-ae6HWx`.
- Reference ELF identities checked again: official
  `64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`,
  separate seeded
  `8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d`.
  Generic artifact manifests still mark cached producer identity unknown;
  these explicitly configured live invocations and identities are the
  reference-provenance receipt, not a claim the manifest proves it alone.
- Final fmt and workspace/all-target Clippy exited 0:
  `/tmp/ares-stock-final-{fmt,clippy}.log`. Core warnings remain 253 library,
  269 library-test; CLI parity warnings remain 31.
- The cube tracer proves stock-settings preservation and full-file equality
  for this case, not thin-feature geometry coverage or the stock matrix.
- Final workspace Nextest exited 0: `/tmp/ares-stock-final-workspace.log`,
  7592/7592 PASS, 9 skipped, 257.284 seconds. The unprovisioned workspace
  invocation does not supply live-reference evidence; the three explicit
  native runs above do.
- Independent six-axis final review read back terminal receipts and the
  staged seven-path patch: no mandatory fixes, approved this finite stock
  tracer checkpoint for matrix-branch commit/push, not global completion.
