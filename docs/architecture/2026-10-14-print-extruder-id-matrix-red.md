# Slice 3: public matrix accounting RED

Approved SPEC:0efc202d4. Base main96328f269; clean git rebase was up-to-date.
No implementation changed before these runs. Only the public regression in
option_coverage/sweep_tests.rs was added; the six identity tests and all
existing assertions remain unchanged.

Upstream: PrintConfig.cpp:5405,9004-9055,9634-9683 and
PrintApply.cpp::Print::apply:1165-1172; the generated materialized config is
emitted by GCode.cpp:5591-5644. Scope remains finite identity accounting.

| Mode | Fresh external root | Result |
|---|---|---|
| official | /tmp/worker2-s3-red-official.YtKBkT | 7run/6PASS/1FAIL, exit100 |
| seeded | /tmp/worker2-s3-red-seeded.Z7lbtb | 7run/6PASS/1FAIL, exit100 |

Each leg used a fresh target/cache/artifact root, outer taskset CPU13 or14,
CARGO_BUILD_JOBS=2, NEXTEST_TEST_THREADS=2 and ARES_PARITY_THREADS=2.
Command driver:/tmp/worker2-s3-red-leg.sh; existing directed control wrapper
/tmp/worker2-bridge-directed-control.sh. Selection was
`test(print_process_identity_) | test(orca_parity_print_extruder_id_accounts_six_materialized_identity_cases)`.
No full/workspace, no unpinned Native probes. START/END posted to inbox.

Both modes: six existing standalone materialization/full-stream witnesses
PASS. New public matrix regression executes the live baseline and writes
option-coverage-summary.md/option-domain-plans.json before asserting complete
finite scope. Native/Ares baseline comparison passes, but the summary row
has one generated/compared case, value=None and INCOMPLETE. The assertion
fails at sweep_tests.rs:165: left INCOMPLETE, expected PASS; detail is
`required legal cases not all compared; `. This is the approved accounting
RED, not a slicing discrepancy, rejected Native fixture or build failure.

Logs, retained inputs, outputs and exact Native command receipts stay in the
fresh external roots. No Native products are committed. GREEN will reuse the
same six requests, preserve their assertions, add bounded plan/executor and
per-case comparison receipts, then rerun this unchanged regression and the
six controls in both modes before review. No production fix is authorized.
