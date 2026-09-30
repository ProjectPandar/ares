# Locked Zag width range probes

Implement ARD-0045's source-cited range evidence at the existing CLI
option-plan and public project-to-G-code boundaries.

Replace the two authored interior-only probes per key with literal auto,
context upper endpoint, seeded interior and near-lower-edge values;
percent cases cover their upper endpoint, seeded interior and near-lower
edge. Verify requested width tags and nozzle/layer/activation dependencies.

For every successful value, require original-Orca output with actual
positive sparse extrusion at the resolved width and complete strict Ares
output equality. Separately exercise the zero-percent and out-of-context
values, requiring the expected reference stage, process status and
diagnostic rather than accepting any failure.

Retain INCOMPLETE scope: this verifies a fixed context, not global range
extremes or all printers. No new pipeline, core algorithm changes,
source-layout pinning or relaxed comparison.

Acceptance: sensitive RED/GREEN of missing domain endpoints; full live
positive/rejection tests; selected actual sweep with honest incomplete
exit; workspace Nextest, fmt/Clippy, Tier 1 compilation and independent
six-axis review. Commit code and docs together after review.
