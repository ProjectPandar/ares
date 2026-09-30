# Printer sweep completion plan

Source and destination boundaries are defined by ARD-0042 and the matching spec.

1. Extract the existing final gate without changing behavior; add separate
   behavioral tests for known missing evidence, empty input and all-PASS.
   Verify the exempted-status tests fail before repair.
2. Require every row to be PASS; retain report-before-failure and all status
   details. Verify focused tests turn green without comparator changes.
3. Run the full official printer matrix in an isolated checkout, retain all
   available output artifacts and classify terminal failures honestly.
4. Run workspace Nextest, fmt/Clippy and relevant compilation checks.
   Independent six-axis review, repair/review, then code-plus-doc commit/push.

The historical tracked matrix is 973/1001 PASS with 18 ORCA_ERROR and
10 VENDOR_INCOMPLETE. It is not current-tree evidence and cannot prove
completion under the requested all-printer requirement.

## Verification

Isolated checkout: `/tmp/ares-printer-gate-isolated.ScNamF`, HEAD 79dd7414,
with only this gate/test/document slice. RED:
`/tmp/ares-printer-gate-red.log`, 3 tests fail, Nextest exit 100. GREEN:
`/tmp/ares-printer-gate-green.log`, 3 tests pass (0.010 s). The new tests
reject both historical exemptions and unknown statuses; existing failures,
empty matrices and all-PASS success remain covered. Fmt check passes.

The live matrix completed with a fresh artifact root
`/tmp/ares-printer-gate-live.tydga2`; log `/tmp/ares-printer-gate-live.log`.
The wrapper is `scripts/orca-parity.sh`, with
`ORCA_APPDIR=/tmp/orca-official-242/squashfs-root`. The original binary hash
is 64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60.
Per-command records preserve invocation/environment and immutable stdout,
stderr and status. The generic paired manifest says runner_cache/unknown
producer; producer evidence here comes from this fresh run's retained
commands and explicitly checked binary, not from that generic label alone.

Workspace Nextest completed with 7378 passed, 9 skipped in 260.844 s
(`/tmp/ares-printer-gate-workspace.log`). Clippy completed successfully in
28.11 s (`/tmp/ares-printer-gate-clippy.log`). WASM, Windows and macOS
compiler checks also completed successfully; logs use the same prefix and
`wasm/windows/macos` suffixes. These are compiler checks, not cross-platform
runtime parity claims. A final fmt check passes.

The fresh report contains all 1001 printers: 979 PASS, 12 ORCA_ERROR and
10 VENDOR_INCOMPLETE, with no Ares or comparison failures. The sweep took
780.649 s and correctly failed with Nextest exit 100, reporting 22/1001
incomplete printers after writing the summary. This verifies the gate,
not global printer parity. The historical 18 oracle failures are not a
stable count; this fresh run has 12, without an oracle or profile change.

Independent six-axis review approved this limited slice without blocking
findings after checking requirements, logic, boundaries, code quality,
test coverage and actual results. It independently counted the report's
1001 rows and verified the main/isolated reports match. This is approval
of the completion-gate repair, not completion of all-printer parity.
