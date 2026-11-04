# Follow-up: eight stock early CLI rejection cases (#30)

Historical preparation record for the remaining eight-case partition of spec
02ffadb9c; the five -100 cases are merged separately. Runtime execution
found and corrected one phase-owned variant mismatch; see the actual RED/GREEN,
full gates, failed-attempt preservation and conditional official ND ruling in
[the verification/ARD](../../architecture/2026-10-09-stock-early-cli-eight-verification.md).
The preparation requirements below remain the original admission contract.

## Exact stock selectors and upstream boundary

- Vendor Anycubic: `Anycubic Kobra 3 0.2 nozzle`, `0.4 nozzle`, `0.6 nozzle`,
  `0.8 nozzle`; -50, POSIX206. OrcaSlicer.cpp:5717-5724 constructs BuildVolume,
  calls Model::update_print_volume_state and rejects count0 before Print::apply.
  Model.cpp:3423-3469::ModelInstance::calc_print_volume_state owns transformed
  model-part volume eligibility; Model::update_print_volume_state counts eligible
  printable instances. This is not necessarily an empty-model case. Exact
  cli_errors text at OrcaSlicer.cpp:137 must retain the whole empty-or-no-object-
  fully-inside wording and the native first-error phase/metadata.
- Vendor Prusa: `Prusa MK3S 0.25 nozzle`, `0.4 nozzle`, `0.6 nozzle`,
  `0.8 nozzle`; -19, POSIX237. OrcaSlicer.cpp:1682-1690 examines loaded embedded
  post_process values during normative_check and rejects any nonempty vector,
  including a vector with an empty string. Exact text at:131 is
  `post_process is not supported under CLI.`; record_exit_reson uses plate0 here.

Rust destinations remain existing project/BuildVolume eligibility and validated
embedded-postprocess owners, with ares-cli failure.rs responsible for upstream
process-channel/result transport. No new Ares pipeline, parallel validation
registry, signal guard or lower-level process exit convention in Core.

## Recipe and admission

cache/cli_result.rs contains public EarlyExit::code() and
OrcaRunner::early_cli_reference; this slice does not edit that file or
cli_exit_channels. Inspect actual variants and recipe contract, then map each
case to the existing -50/-19 variant. If an empty-plate variant assumes
semantics or product inventory that exclude Kobra3's eligibility outcome, stop
before any cache changes; never disguise the input as an empty model or rewrite
the Native recipe.

Own new test file: orca_parity/stock_early_cli.rs, case data prepared under
stock_early_cli/cases.rs. The unregistered skeleton contains selectors/codes
only, no ignored/fake-PASS tests or expected Unsupported guard assertions.

Inputs must be generated at runtime with the unchanged stock sweep selection:
VendorProfiles + select_printer, flattened selected machine/process/filaments,
empty overrides, cube10.stl and the existing runner.export_case/cache recipe.
Native3MF/public exported bytes stay outside the repository and feed both
consumers unchanged.
No Native-generated input/output may be committed; no repaired presets, altered
placement, introduced defaults or frozen-output fallback. Record producer and
input SHA/selection/argv/cache/commit provenance for each attempt. Historical
stock sweep records are leads only.

Call the merged early_cli_reference on that same project and verify a repeat
cache hit. Compare actual public Ares CLI POSIX exit206/237 with the recorded
Native process status, complete stdout/stderr and result bytes. Use
only shared native_wall_clock::result_comparison_bytes for prepare_time and
export_time; all other bytes/field ordering remain exact. Native output-product
inventory is recipe-owned; assert Ares has no final exported G-code if Native
has none. Public Core typed error checks, if needed, are separate from CLI exit
claims and must preserve the owning early phase rather than run a later guard.

Missing optional Native must print visible SKIPPED before artifact-root access;
required missing Native fails closed through the existing shared guard/runner.
Wrong Native error, read/export/cache failure or unequal inputs is inconclusive,
not a production RED. Real same-input CLI channel mismatch is production RED:
retain unchanged failing E2E and runtime artifacts first, then port the minimum
source-owned behavior. No retry-to-PASS
or expected-output/comparator change. All equal means tests/docs-only closure.

## Deferred behavior and acceptance

12 recorded Native crashes remain excluded: no imitation or guard. Already-
merged five template cases, #7 loaded/width cohort, SLA/timelapse, vendor catalog
selection repairs and general slicing changes are not this slice.

Gates are strict workspace clippy/fmt, default no-ARES workspace with eight
visible skips/no panic, and required seeded/official eight-case directed runs.
Preserve all failed attempts.

Acceptance: eight admitted matching CLI channel outcomes (or honest
failed/inconclusive records) and a source-owned conclusion. Actual executions
are recorded in the verification.
