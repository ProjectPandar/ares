# Upward result metadata — independent CustomGCode RED

Approved SPEC `d8b155d79`; owner subsequently required upward first in its own
merge, no worker3 unmerged grouping layer, downward later separately. Upstream
`OrcaSlicer.cpp:1769,2517–2540,2656,428–432` owns this metadata boundary.

Test-first commit `e6433b2e6` is based on main
`d4efbaad0a273422169f34cd147510967746b5e3`; it changes only test registration
and `cli_upward_machine_result.rs`. No production/core/CLI writer changes.

The fixed source machine is **Bambu Lab H2D 0.4 nozzle**; **H2D Pro0.4 is its
upward target**, not its input machine. Flattened stock machine JSON is retained
in runtime artifacts. Existing preset normalization/smoke/cube10 export recipe
is reused; the sole dedicated error input is machine_start_gcode
`{ares_upward_result_missing_variable}\n`. No grouping/TPU changes, repeated
sample search, frozen Native result or new probe tool is used.

Completed archive build exit0, jobs2/incremental0, before actual Native admission.
Required fresh seeded run `2d5b70b6-2c4b-48b4-8be6-362388e31d2f`, two nextest
threads, actual launch load<60 recorded externally; 1test FAIL/exit100,
918filtered tests (not Native execution/skips), no timeout. Test frame proves:

- Native runtime result return_code **-100**, nonempty upward vector exactly
  `["Bambu Lab H2D Pro 0.4 nozzle"]`; default downward header absent.
- Public current-main Ares CLI exit156/-100, complete stdout and stderr exact
  to live Native; no final G-code.
- Every non-time result field matches except **missing upward_compatible_machine**.
  Sole failed assertion is complete result bytes at109; existing integer
  prepare_time/export_time adaptation only. Neither field is removed from the
  comparison; no additional normalization or relaxed assertion.

Exact runtime input SHA:
`023008f8fb816bf57abe76c5421eb9ad63850c51af1a8276d18150c5358ce593`.
Authoritative log/admission/archive:
`${TMP_ROOT}/ares-cli-upward/red-d4/`.
Raw input/stock source/Native producer receipt and full channels:
`${TMP_ROOT}/ares-cli-upward/red-d4/seeded/artifacts/cli-upward-machines-Z0lz6P/`.
Machine audit `${TMP_ROOT}/ares-cli-upward/red-d4/seeded/audit.json` records exact
channel matches, metadata differences, input hash and exit. Original worker3
A1 handoff remains distinct historical evidence; it was not rerun or relabeled
as this test. Native production source read/fetch provenance must still be
proved in the later source-selection GREEN controls; a nonempty late report
alone does not certify every up-to-date/system-refresh branch.

This is genuine metadata RED, not a missing Native/channel/admission failure.
No GREEN or downward implementation, no push, and no current heavy execution
follows automatically from recording it.
