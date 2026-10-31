# #30: eight runtime stock early CLI rejection cases

## ARD: phase-owned error classification

This is the remaining eight-case slice of approved residual spec `02ffadb9c`
and [the preparation spec](../superpowers/specs/2026-10-09-stock-early-cli-eight.md),
not an Ares-owned pipeline feature.

- `OrcaSlicer.cpp:5710–5724` rejects zero eligible instances with
  `CLI_NO_SUITABLE_OBJECTS (-50)` before `Print::apply`.
  `Model.cpp:3423–3469::ModelInstance::calc_print_volume_state`,
  `Model::update_print_volume_state/count_inside_instances` and `BuildVolume`
  remain the owning eligibility boundary.
- Rust destination: the existing `project_slice/state.rs` producer in
  `prepare_loaded_project_slice_with_validation`. Change only its existing
  `count_inside_instances == 0` error variant from `InvalidInput` to the
  existing `NoSuitableObjects`. Condition, error text, phase, geometry and
  placement are unchanged. No new Core process/filesystem contract is added.
- Existing `ares-cli/src/failure.rs` already transports `NoSuitableObjects` as
  exit `206`, upstream code `-50`, empty stderr and the upstream result bytes.
  Do not add CLI string matching or duplicate transport.
- `OrcaSlicer.cpp:1682–1690` owns loaded embedded-postprocess rejection `-19`
  before slicing. Four Prusa cases match the existing implementation without
  a production change.

Non-negotiable: unchanged runtime stock selection/export/cache; identical
runtime-exported input bytes for both consumers; no committed Native input or
output products; full exit/stdout/stderr/result comparison. Only the shared
`native_wall_clock::result_comparison_bytes` exempts `prepare_time` and
`export_time`. G-code comparison exempts only the entire generator line,
never result clocks. No repair, placement rewrite, guard weakening, retry to
PASS or expected-output change. Worker4's `plate_selection.rs` origin work and
`cache/cli_result.rs` recipe are untouched. Twelve Native crashes, SLA/timelapse,
profile admission, general slicing and adjacent geometry behavior are deferred.

## Cases and unchanged RED → GREEN

Anycubic Kobra 3 `0.2/0.4/0.6/0.8 nozzle`: eligibility `206/-50`.
Prusa MK3S `0.25/0.4/0.6/0.8 nozzle`: embedded-postprocess `237/-19`.
Own tests: `orca_parity/stock_early_cli.rs`, selector table `stock_early_cli/cases.rs`.
The existing `VendorProfiles/select_printer`, flattened presets, empty overrides,
`cube10.stl` and `OrcaRunner::export_case` produce the original stock inputs at
runtime in external artifact/cache directories. `early_cli_reference` is called
twice with unchanged `EmptyPlate`/`Postprocess` recipes; cached directory,
stdout/stderr/result/G-code inventory must match. Real Native exit comes from
its receipt, not an invented expected process status. Optional Native prints
visible SKIPPED before artifact-root access; required absence fails closed.

Unchanged RED `34b08e089`, run `c7355b45-9d44-4721-8b5c-0432a4a44928`:
4 MK3S PASS, 4 Kobra FAIL, exit100. Native Kobra exits206, stdout contains the
`-50` exit line, stderr is empty, result.json is complete. Ares exits1, stdout
is empty, stderr wraps the same text in `InvalidInput`, and result.json is
absent. The test fails reading the missing file; recorded real channels and
Native receipts establish the production mismatch. It is not an input mismatch.
No failing input or assertion was changed to obtain GREEN.

GREEN actual `bfcbbd6732fbdd6074063f21272f20511a79a38f`, run
`0f3ba103-390e-4806-a266-e839c212869a`: 10 PASS, exit0, comprising eight new
CLI cases plus two existing empty-selected-plate controls. The populated plate2
control retains complete Native G-code equality with generator-only exemption.

Exactly one existing Native-derived variant assertion changed, before the
production commit:

- `empty_selected_plate_controls::empty_selected_plate_control_nonempty_outside_volume_keeps_later_error`:
  `Err(InvalidInput(ref text)) if text == OUTSIDE` →
  `Err(NoSuitableObjects(ref text)) if text == OUTSIDE`. Native exit206,
  empty stderr, return_code/error_string and exact payload assertions remain.

No assertions were deleted. The four O6 volume-rejection tests assert Display
text and are unchanged. Other `InvalidInput` checks concern invalid options,
geometry or missing plate identities and are unchanged.

## Full gates: actual trees, not rebased execution claims

Actual BASE `d96fe98ce81dca9919f877c010d653a2a021da5e`; actual CAND
`bfcbbd6732fbdd6074063f21272f20511a79a38f`. One heavy job at a time,
16 nextest threads, 8 Cargo build jobs. Both use the same immutable upstream
source/profile tree and the unchanged coordinator control-set-v2 script.

- Strict workspace clippy/all-targets/all-features, fmt check: exit0.
- WASM core/adapter compilation: exit0. Browser gate waived by coordinator.
- Default no-ARES workspace: `4468800e-dee0-44cb-bcb9-e204a7e4d43f`,
  8222 PASS / 6 existing nextest skips, exit0. Eight new cases visibly Native
  SKIPPED; no artifact-root panic. These skips are not Native execution proof.
- Seeded BASE: `99cba255-c8cc-4d65-9bfd-c4fd41952508`, 848/848 PASS, exit0.
- Real isolated-target seeded CAND: `b49c45d6-e848-42b6-999a-e2d06cd3ef0d`,
  856/856 PASS, exit0. Preflight list contains 856 tests, including eight new
  cases, candidate package/cwd and compiled manifest literal. The
  `initialized_loop_chain` provenance guard PASS proves the candidate wrapper
  matches the compiled candidate worktree.
- Official BASE: `49834a1f-2ab5-495c-a307-4fc5c8a389bb`, 839 PASS / 9 FAIL,
  exit100; official CAND `4929415b-eda5-4b26-be29-cea0bbc2cbde`,
  847 PASS / 9 FAIL, exit100. Eight new CLI cases PASS in each candidate mode;
  all 16 Native receipt/input/exit/channel records independently reread.

Complete common Ares G-code streams (generator line only): seeded649 unchanged,
0 changed/ambiguous/new; official608 unchanged, 0 changed/ambiguous/new.
Official3 BASE-only streams are later tip-domain cases not reached after the
candidate organic-angle assertion fails; they are not compared or promoted.

### Preserved infrastructure failure

Nominal candidate run `f54c1ba6-483e-4aae-8190-fcc3d5592960` executed a
BASE-compiled binary reused from a shared Cargo target: 848 rather than856
listed, with the initialized-loop-chain guard rejecting candidate wrapper vs
BASE compiled-manifest path. Exit100 and artifacts remain preserved. This is
**invalid candidate execution evidence**, not a candidate production RED.

Coordinator authorized fresh isolated targets, retaining the valid BASE seeded
run. CAND uses `/tmp/worker3-early-eight-cand-bfc-target`; official BASE uses
`/tmp/worker3-early-eight-base-d96-target`. No code, guard, expectation or Native
recipe was changed for the correction; failed attempts were not relabeled.

## Official exact-name failure sets and conditional ND adjudication

The following eight names fail on **both** sides due to the known existing
missing `ORCA_REPLAY_SEED` environment reads in the official control context:

1. `option_coverage::bookkeeping_export::bookkeeping_matches_complete_configured_filament_output`
2. `option_coverage::initialized_object_id::initialized_reference_repeats_complete_ksr_output`
3. `option_coverage::initialized_short_line::initialized_short_line_matches_complete_internal_fill_output`
4. `option_coverage::initialized_stitch_junction::initialized_stitch_junction_matches_complete_internal_fill`
5. `option_coverage::initialized_transition_snap::initialized_transition_snap_matches_complete_internal_fill`
6. `option_coverage::single_volume_offset::single_volume_offset_matches_complete_rotated_prism_output`
7. `runner::initialized_loop_chain::initialized_loop_chain_matches_complete_output`
8. `runner::initialized_overhang_intersections::initialized_overhang_intersections_match_complete_output`

BASE-only ninth failure:
`runner::tree_support::manual::styles::manual_strong_painted_overhang_preserves_complete_output`.
CAND-only ninth failure:
`option_coverage::tree_activation::organic_angle_and_normalized_grid_tip_preserve_complete_output`.
Thus the **raw candidate failure-name set is not a subset of BASE**. Matching
failure counts alone are not a passing gate.

Coordinator conditionally applies the previously adjudicated tree-style/organic
Native ND class to this two-way flip, only after proving equal **raw Ares SHA**
and differing Native outputs on both cases. Conditions are met without retries:

| Case | Raw Ares SHA256 (identical BASE and CAND) | First non-generator Native difference, original line |
| --- | --- | --- |
| organic angle-slow, organic/seeded | `b57beb6cf5de43fa2445b8ba8cd2a20e527862458a9658ac00b8cf1e531cdd05` | 359: BASE `G1 Z.2` then `M73 P5 R20`; CAND reverses these two lines |
| manual strong painted-overhang | `ec6c7805da23c18dd143c279922510df6ee76536831e1a302fdbe579d7e1226e` | 1620: BASE travel `X113.388 Y113.135` / later WIDTH0.437637; CAND `X113.244 Y113.297` / WIDTH0.46426 |

Native raw first difference is generator timestamp at line2; the substantive
non-generator differences above independently establish ND, not just a header
change. Both pairs have identical ordered uncompressed input ZIP members;
actual ZIP hashes remain recorded separately. Raw Ares SHA equality uses no
normalization at all. After only these expressly authorized ND classifications,
remaining official candidate failures are the same eight names as BASE and
satisfy the exact-name subset rule. Raw exit100/failures remain failures; no
assertion, comparator, output or result-clock exemption was broadened.

## External durable evidence and carry

- `/tmp/worker3-early-eight-review.json`: actual RED/GREEN/gates, runtime16,
  raw failure-name sets, conditional ND proof and execution bindings.
- `/tmp/worker3-stock-early-eight-v1*`, `...-green*`: unchanged RED and directed GREEN.
- `/tmp/worker3-early-eight-full-*`: lint/default/valid BASE seeded and invalid
  shared-target nominal-candidate attempt, with launch receipts/logs/exits.
- `/tmp/worker3-early-eight-isolated-*`: preflight and isolated full executions.
- `/tmp/worker3-early-eight-official-failure-subset.json`: both full raw lists.
- `/tmp/worker3-early-eight-official-nd-proof.json`: raw hashes, input members,
  original Native line numbers/context and source manifests.
- `/tmp/worker3-early-eight-{seeded,official}-common-byte-record.json`: complete
  common streams and unmatched identities, without G-code clock exclusions.

Coordinator explicitly permits Rust evidence carry over subsequent test/docs-only
main `b3f8894fd`, `cc92cfa9b`, `151660d1e` (including browser spec and deletion
of Native KSR products), and `4cfdb2be8` roadmap-only change. A git-only rebase
is not execution at the rebased HEAD; actual execution references remain those
above. Production main changes require refreshed production gates. No browser
Playwright configuration/`ARES_E2E_CHROME` was used for this slice.

Gate evidence is ready for REVIEW under the coordinator's conditional ND policy,
not a claim that the raw official suite is all green. No push before sole APPROVED.
