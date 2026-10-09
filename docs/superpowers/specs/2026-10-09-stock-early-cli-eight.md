# Follow-up: eight stock early CLI rejection cases (#30)

Preparation only. This is the remaining eight-case partition of approved spec
02ffadb9c; the five -100 cases merged as95eb301e4. No new Native execution or
production RED is claimed. Wait for worker4's cli-exit merge before registering
and executing the test skeleton; no heavy job during this preparation.

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

## Recipe handoff and admission

Worker4's in-flight cache/cli_result.rs contains public EarlyExit::code() and
OrcaRunner::early_cli_reference. Do not edit that file or cli_exit_channels while
worker4 owns it. After merge, inspect actual variants and recipe contract, then
map each case to the existing -50/-19 variant. If an empty-plate variant assumes
semantics or product inventory that exclude Kobra3's eligibility outcome, report
this to the sole coordinator before any cache changes; never disguise the input
as an empty model or rewrite the Native recipe.

Own new test file: orca_parity/stock_early_cli.rs, case data prepared under
stock_early_cli/cases.rs. The unregistered skeleton contains selectors/codes
only, no ignored/fake-PASS tests or expected Unsupported guard assertions.

Inputs must be generated at runtime with the unchanged stock sweep selection:
VendorProfiles + select_printer, flattened selected machine/process/filaments,
empty overrides, cube10.stl and the existing runner.export_case/cache recipe.
Native3MF/public exported bytes stay/tmp and feed both consumers unchanged.
No Native-generated input/output may be committed; no repaired presets, altered
placement, introduced defaults or frozen-output fallback. Record producer and
input SHA/selection/argv/cache/commit provenance for each attempt. Historical
stock receipts under worker3-stock-full-current-* are leads only.

Call the merged early_cli_reference on that same project and verify a repeat
cache hit. Compare actual public Ares CLI POSIX exit206/237 with the receipt-
verified Native process status, complete stdout/stderr and result bytes. Use
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
source-owned behavior with sole-coordinator/owner coordination. No retry-to-PASS
or expected-output/comparator change. All equal means tests/docs-only closure.

## Deferred behavior and acceptance

12 recorded Native crashes remain excluded: no imitation or guard. Already-
merged five template cases, #7 loaded/width cohort, SLA/timelapse, vendor catalog
selection repairs and general slicing changes are not this slice.

After worker4 merge, latest-main execution and gates are coordinator-assigned;
proposed baseline is strict workspace clippy/fmt, default no-ARES workspace
with eight visible skips/no panic, required seeded/official eight-case directed
runs. One heavy job,16threads/build8. Preserve all failed attempts and exact
execution bindings; docs-only carry is not execution on rebased HEAD.

Eight admitted matching CLI channel outcomes (or honest failed/inconclusive
records), source-owned conclusion, external review receipt → REVIEW-REQUEST →
sole APPROVED → fetch/rebase and ordinary ff-only push. No execution yet.
