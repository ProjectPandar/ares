# ARD-0081: Promote KSR native/browser checks to ordered acceptance

Source boundary is `libslic3r/Print.cpp:2618-2638::Print::export_gcode`,
`GCode.cpp:2030::GCode::do_export`, `GCodeWriter` movement/control output and
`GCodeProcessor::run_post_process`. This slice verifies the existing Rust
rewrite's public bytes, not a new slicing pipeline or a production algorithm
change. Destinations are `ares-cli/tests/ksr_fdmtest_v4.rs`, its separate test
modules, and the existing WASM browser test's callback into that comparator.

ARD-0080 established full KSR equality against the separately approved
seed/fill/object-ID initialized reference. Its committed capture and identity
already exist in `tests/ksr_fdmtest_v4/initialized-reference.json`; preserve
those bytes and the historical untouched official capture. Do not regenerate
the expected output from Ares or silently relabel initialized data as official.

The existing CLI/browser `compare_actual` still calls a tolerant semantic
diagnostic and rewrites object IDs. Another public core test accepts 90% XY
multiset coverage. Replace these temporary diagnostic acceptance shells with
the unchanged ordered generator-line-only comparator. Retain semantic parser
tests strictly as diagnostic tools, not a fallback on strict failure.

Include native CLI and actual browser WASM output at the user-approved
project-input/complete-output seam. Preserve full input, native capture,
actual output and reference identity before comparison; print the external
artifact directory on success and failure. Do not normalize IDs, M73,
coordinates, path order, timing, configuration or statistics.
Prove sensitivity by changing object IDs in actual public CLI output:
the old semantic gate masks that change, the strict gate must reject it. This is a
comparator mutation, not a claim of a producer bug. Keep all changed Rust
files below400LOC and tests in separate modules.

Delete the now-redundant 90% structural golden and the object-ID rewrite
helper made orphaned by this change. Browser caller and test names must say
ordered parity. Defer unrelated semantic diagnostic cleanup, printer-default
coverage, support/SLA behavior and all-option domain completion.

Exit requires witnessed RED/GREEN, native full output PASS, real browser run
(any remaining mismatch is a genuine open gate), workspace Nextest, fmt/clippy,
Tier-1 compile/build evidence in proportion to test-only scope, and independent
six-axis review/fix/re-review before code/docs commit and push together.
The immutable009d printer/option scans continue in their original processes;
their modified classic baseline does not prove untouched default parity.

Initial extra-comment mutation is already rejected by the old parser;
run53569 passes and is not a valid RED witness. Replace that probe with
changed object IDs, which the current acceptance shell explicitly rewrites.

## Execution evidence

The object-ID mutation gives witnessed RED (Nextest session95900, exit100,
`${TMP_ROOT}/ares-ard81-id-sensitive-red.log`) then GREEN after replacing the gate
(57187, exit0, `${TMP_ROOT}/ares-ard81-sensitive-green.log`). Final exact-code workspace
run83361 passes7459 tests with9 skips in262.197s; one obsolete structural
golden was deleted. All-target workspace Clippy, fmt and diff checks pass;
Clippy retains existing warnings. Windows/macOS/Linux CLI tests compile via
`cargo check`; these are checks, not cross-platform runtime executions.
Release WASM build and wasm-bindgen generation both succeed.

Actual Chromium runs18686 and51986 both fail the ordered browser gate at
byte241011, line8134 column28: initialized Orca emits
`G1 X132.718 Y95.588 E-.02097`, WASM emits `E-.02098`.
Retained full browser files are `${TMP_ROOT}/ares-wasm-gcode-DKUMMk/browser.gcode`
and `${TMP_ROOT}/ares-wasm-gcode-tGJ9LK/browser.gcode`; all non-generator bytes
replay identically between them. Final paired input/reference/output/identity
are in `${TMP_ROOT}/ares-ksr-ordered-eahjbW`. This is a genuine browser parity RED,
not an accepted tolerance, and later path/control differences also remain.
The passing native CLI gate does not establish browser or global completion.

Independent preliminary review found output retention occurred after CLI
timestamp and browser structural assertions. Both callers now save/compare
complete output before those assertions; the final real browser failure
confirms paired artifacts survive the failure. Independent six-axis re-review
finds no remaining required code fixes and approves this finite acceptance-shell
promotion, explicitly not browser producer parity. Final immutable patch
confirmation is required before commit. No production numerical repair or
causal claim is part of this slice.
