# Printer sweep completion gate

The source boundary and non-negotiable acceptance policy are in ARD-0042.
This repairs the CLI acceptance harness for the `PrintConfig.cpp`,
`Print::process` and `GCode::do_export` rewrite, not slicing behavior.

Include rejection of empty matrices and every non-PASS row, including known
ORCA_ERROR and VENDOR_INCOMPLETE outcomes. Keep reporting before failure and
retain complete output artifacts when a reference is available. Preserve
failure classification so missing evidence is not mislabeled as an Ares bug.

Defer changes to printer enumeration, smoke overrides, vendor profiles and
the underlying slicing implementations. The modified baseline is not proof
of untouched-default or all-option parity; missing upstream behavior remains
temporary scaffold, not an alternative Ares pipeline.

Acceptance: sensitive RED/GREEN for formerly exempted statuses, existing
failure and empty cases, full real-Orca sweep with truthful terminal status,
workspace Nextest, fmt/Clippy and independent six-axis review. Commit code
and documentation together only after review; a correctly failing coverage
run must not be reported as global completion.
