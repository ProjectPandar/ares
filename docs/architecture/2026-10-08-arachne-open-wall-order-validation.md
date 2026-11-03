# Arachne open-wall candidate ordering validation

Status: validation complete; REVIEW-REQUEST, awaiting coordinator approval for push.
Spec: [arachne-open-wall-candidate-order.md](arachne-open-wall-candidate-order.md),
approved `26da8d624`. Upstream pin
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Original RED `50e206ff8678ce40767061dc0f4502c137914610` and GREEN
`a0412b07483a13762ff9a96173d170f76a62a895` were based on main `113a351e3`
(Rust baseline `016456549`). Zero-conflict rebase onto main `bff9cb1b0`
produced RED `3c8c36bdd2ac5a10402495cff520201f6fd155fe` and GREEN
`0c2d27c8c3d82c9f529505743f5dd37dabb4eaeb`. The slice production, fixtures
and E2E test module are byte-identical across rebase; see
`/tmp/worker6-open-order-post-bff-slice-diff.log` (empty).

## Change and source attribution

`PerimeterGenerator::process_arachne` at
`OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:2326–2328` sorts available
walls by is_closed ascending. Open paths therefore participate before
closed paths. Ares `perimeters/arachne/order.rs::order_walls` used the inverted
key !is_closed. The one behavioral edit removes that negation; two directly
misleading comments were corrected. Constraints, starting position, strict
distance comparison, current-position update and traversal are unchanged.
This changes candidate scan order; it does not globally force open walls
before closed walls when dependencies or nearer candidates decide otherwise.

The six narrow-neck cases now emit the same complete output as Native.
This establishes the source discrepancy as the cause of this finding's
observed first-layer sequence and downstream M73 differences. No visibility,
signed-distance, flow, width generation, gap-fill or emission fix was added.
The earlier C/C1 arithmetic investigations remain bounded no-RED reports.

## Committed RED and whole-file GREEN

The RED commit contains six byte-identical copies of the frozen finding
inputs under `tests/parity/validation/arachne-open-wall-order/`, plus public
Native E2E tests and a wider-neck control using the existing unchanged
`seam-aligned-back-single-control.3mf`. The test module is
`crates/ares-cli/tests/orca_parity/runner/arachne_open_wall_order.rs`.
It pins input SHA256 and verifies Arachne, one wall, uniform0.2mm heights,
one object/volume and explicit current-object seam mode.

| Input | RED complete first difference | GREEN |
| --- | --- | --- |
| AlignedBack0° | byte847 line32, M73 P28 versus P29 | PASS |
| AlignedBack17° | byte1398 line63, travel X108.051 Y109.404 versus X106.713 Y112.854 | PASS |
| AlignedBack43° | byte1398 line63, travel X108.51 Y108.61 versus X105.794 Y111.124 | PASS |
| Nearest0° | byte847 line32, M73 P28 versus P29 | PASS |
| Nearest17° | byte847 line32, M73 P29 versus P28 | PASS |
| Nearest43° | byte1386 line62, travel X108.51 Y108.61 versus X110.827 Y105.726 | PASS |
| Wider-neck AlignedBack control | PASS | PASS |

Real fresh seeded Native directed RED:1 PASS /6 DIVERGENT, nextest4.652s.
GREEN:7/7 PASS, nextest1.672s. Every comparison is complete ordered bytes;
only generated-by is normalized. No timing, M73, width, travel or wipe line
is removed. The separate first-layer entity context is diagnostic only;
startup M73 remains the true first difference in the three affected cases.

`/tmp/worker6-open-order-red-record.json` and `green-record.json` retain all
seven input/output/executable hashes and matched fresh Native producer
receipts. `/tmp/worker6-open-order-directed-comparison.json` independently
compares both rounds under the existing generated-by rule: Native complete
normalized bytes are unchanged7/7, Ares changes exactly six failing inputs,
and the wider-neck Ares control is unchanged. Raw Native SHA differences
are retained and caused by generated-by timestamps, not silently discarded.
The comparison script reproduces the existing golden normalizer only.

## Fresh paired full control sets

Original base and candidate have the same674-test inventory, including the
seven new cases; the new-main pair has the same675-test inventory. Base is detached worktree `${TMP_ROOT}/ares-worker6-open-order-base` at RED;
candidate is worker6 at GREEN. Each round uses its own fresh Native cache.
Targets are independent worker6 directories: base
`${TMP_ROOT}/ares-worker6-seam-A-base-target`, candidate
`${TMP_ROOT}/ares-worker6-green-target`. Parallelism:16 nextest threads,
ARES_PARITY_THREADS=16, CARGO_BUILD_JOBS=8, CARGO_INCREMENTAL=0, one heavy task at a
time. Both producers and process outcomes must be retained for each change.

The original-main fresh official pair is complete: base658 PASS /16 FAIL
(162.815s), candidate664 PASS /10 FAIL (139.891s), inventory674 each.
Exactly the six listed narrow-neck cases change FAIL→PASS. Candidate-only
failures are empty; all other statuses/reasons are unchanged. Full names
are retained in `/tmp/worker6-open-order-pre-bff-control-status-changes.json`.

Eight common official failures are missing reference environment variables
(`MISSING_REFERENCE_ENV`), not passes. Two common X2D failures are
`INCONCLUSIVE-oracle-UB`: `bridge_over_infill_multi_filament_x2d` and
`narrowed_x2d_whole_file`. Their Ares complete SHA256 values remain identical
between base and candidate, respectively
`331827159281658d6baac34427836ee19fe89242c123fcb32ae3757ccd4ed79d` and
`ff469e046fbd6f49e361bed645f184e5daed08366a9ce02d0f48a0dc26574543`.
Native differs in the uninitialized object-id field (`Print.hpp:582 m_id`),
as already classified in
[the X2D report](2026-10-08-x2d-official-first-difference-classification.md).
Decoded assertion bytes, first lines and both-side hashes are retained in
`/tmp/worker6-open-order-x2d-pair-record.json`; no UB is ported. Neither
official round contains a timeout.

The original seeded results below are preserved evidence, superseded for
merge by a new fresh paired seeded run on `bff9cb1b0`: the worker5 change
affects BBL multi-extruder placeholders, and this inventory includes H2D.
Official remains the original same-main pair under the coordinator
broadcast's seeded rerun requirement; it is not represented as a new-main
official run. Old seeded logs have the `-pre-bff-main.log` suffix; original
cache/artifact roots and receipt paths are preserved.
Fresh seeded: base668 PASS /6 FAIL (231.253s) → candidate674 PASS /0 FAIL
(125.461s). Exactly six new narrow-neck REDs change FAIL→PASS; all other
statuses and reasons are unchanged. Candidate-only failures are empty.

The unchanged official missing-environment cases are:

- `option_coverage::bookkeeping_export::bookkeeping_matches_complete_configured_filament_output`
- `option_coverage::initialized_object_id::initialized_reference_repeats_complete_ksr_output`
- `option_coverage::initialized_short_line::initialized_short_line_matches_complete_internal_fill_output`
- `option_coverage::initialized_stitch_junction::initialized_stitch_junction_matches_complete_internal_fill`
- `option_coverage::initialized_transition_snap::initialized_transition_snap_matches_complete_internal_fill`
- `option_coverage::single_volume_offset::single_volume_offset_matches_complete_rotated_prism_output`
- `runner::initialized_loop_chain::initialized_loop_chain_matches_complete_output`
- `runner::initialized_overhang_intersections::initialized_overhang_intersections_match_complete_output`

### Preserved invalid initial base round

The first fresh seeded-base attempt lacked the untracked OrcaSlicer upstream
symlink in the new worktree. Tests loading upstream preset resources failed
with No such file or directory. This is **INCONCLUSIVE-infrastructure**;
it is excluded from the valid base/candidate comparison, not counted PASS.
The original log/cache/artifacts remain at their original evidence paths;
the log is `/tmp/worker6-open-order-seeded-base-missing-upstream.log`.
After restoring the same read-only link to `OrcaSlicer`,
the entire base round was repeated with independent `seeded-base-v2-cache`
and `seeded-base-v2-artifacts` roots. No assertions, fixtures or production
code changed. `/tmp/worker6-open-order-infra-retry.json` records this repair.

### New-main seeded pair and exact case changes

On main `bff9cb1b0`, fresh base669 PASS /6 FAIL (148.767s) → candidate
675/675 PASS (181.245s). Base uses `seeded-base-v3-cache/artifacts`, candidate
uses `seeded-cand-v2-cache/artifacts`, all prefixed `/tmp/worker6-open-order-`.
The seven directed tests are included and pass on the rebased candidate.
`/tmp/worker6-open-order-control-status-changes.json` records the exact
new-main seeded and original-main official status/reason comparisons.

The following six names under `runner::arachne_open_wall_order::` are the
**only status/reason changes in each pair**, all DIVERGENT/FAIL→PASS:

| Case suffix after `arachne_open_wall_` | Seeded | Official |
| --- | --- | --- |
| `c0_neck0_25_r0` | FAIL→PASS | FAIL→PASS |
| `c0_neck0_25_r17` | FAIL→PASS | FAIL→PASS |
| `c0_neck0_25_r43` | FAIL→PASS | FAIL→PASS |
| `nearest_control_neck0_25_r0` | FAIL→PASS | FAIL→PASS |
| `nearest_control_neck0_25_r17` | FAIL→PASS | FAIL→PASS |
| `nearest_control_neck0_25_r43` | FAIL→PASS | FAIL→PASS |

No other status or reason changes; candidate failures are a strict subset
of base failures in both pairs. The wider-neck control remains PASS.

## Quality gates and limitations

Workspace `cargo nextest run --workspace --no-fail-fast --test-threads 16`:
8074 PASS,9 skipped,622.339s on main `bff9cb1b0` + GREEN. No failure.
The initial fmt check required only alphabetical module registration order;
its log is preserved as `fmt-before-registration-order.log`. After rustfmt,
fmt PASS; the order-only change is committed separately.

A subsequent zero-conflict rebase onto `dc8d7f00c` (typed template error/CLI
exit code, no G-code changes) produced RED `df133fbaf`, GREEN `de87a6fab`,
and formatting commit `30a107581`. Coordinator explicitly authorized
seeded/official/workspace evidence reuse with post-rebase strict clippy and
directed checks. Strict clippy `--workspace --all-targets --all-features
-- -D warnings` PASS (1m05s). Fresh seeded directed seven-case whole-file
Native comparison PASS7/7 (2.273s), independent `post-dc8-directed-cache`
and `post-dc8-directed-artifacts`. WASM `cargo check -p ares-core -p ares-wasm
--target wasm32-unknown-unknown` PASS (25.80s). fmt PASS.

Final zero-conflict rebase onto test-only main `92e9573df`, evidence reuse
explicitly authorized: RED `df9d67093`, GREEN `696617582`, formatting
`5eee457cf`. Production, input fixtures and directed module are unchanged;
`/tmp/worker6-open-order-post-test-only-slice-diff.log` is empty. Final fmt
check PASS. Report-only changes do not alter compiled behavior.

The approved >16-candidate introsort/equivalent-key permutation port and
large-candidate Native witness remain deferred. Rust stable boolean sorting
matches the identified libstdc++ small-set insertion-sort behavior when the
same candidate input has at most16 elements. General std::sort tie parity is
not claimed. No Native candidate cardinality instrumentation was introduced;
emitted entity counts alone do not prove the pre-materialization count.
Any new full-control failure must be reported, including large-set cases;
this scope limit cannot excuse an unexplained candidate-only regression.

## Producer evidence and reproduction

`/tmp/worker6-open-order-evidence-index.json` joins all63 directed/changed/
control outputs from9 rounds to Native input/output hashes and retained
process evidence (1141 indexed files). Frozen index SHA256:
`e064d40d27f7e6694d910a351a6b5241db26117ad40ef572e664404216207981`.
It hashes manifests, inputs, outputs, command/status/
lifecycle/stdout/stderr, receipts, comparison scripts and gate logs. Full
control statuses are retained independently of the changed-case join.
Some full rounds concurrently reuse the wide-control input in another test:
where a cache receipt was replaced by the equivalent second invocation,
the index explicitly marks raw-output SHA mismatch, verifies only the
generated-by timestamp differs, and points to the exact original Native
process output matching the asserted raw SHA. No fabricated raw receipt
match or relaxed test assertion is claimed.

Seeded Native executable SHA256:
`59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`;
resources `286fab66becc4c87ebb72dc8e66ec9d515e7f440d530bacebeb1f51407bb8a69`.
Official executable SHA256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Official uses `/tmp/orca-wrapper-inherit-cpu.sh`, inheriting the runner CPU
after the coordinator's 16:03 fix; it does not force all jobs onto CPU1.
Receipts retain exact producer components/resources/environment per round.

Reproduce full rounds with `/tmp/worker6-open-order-controls.sh` arguments
`seeded-base`, `seeded-cand`, `official-base`, `official-cand`; use a new
cache/artifact root for every repeat. Quality commands are retained in
`/tmp/worker6-open-order-quality.sh`. The original official pair belongs
to original RED/GREEN, not the rebased checkout used by this script now.
