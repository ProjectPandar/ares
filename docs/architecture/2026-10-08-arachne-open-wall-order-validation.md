# Arachne open-wall candidate ordering validation

Spec: [arachne-open-wall-candidate-order.md](arachne-open-wall-candidate-order.md),
`26da8d624`. Upstream pin
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Original RED `50e206ff8678ce40767061dc0f4502c137914610` and GREEN
`a0412b07483a13762ff9a96173d170f76a62a895` were based on main `113a351e3`
(Rust baseline `016456549`). Zero-conflict rebase onto main `bff9cb1b0`
produced RED `3c8c36bdd2ac5a10402495cff520201f6fd155fe` and GREEN
`0c2d27c8c3d82c9f529505743f5dd37dabb4eaeb`. The slice production, fixtures
and E2E test module are byte-identical across rebase.

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

Under the existing generated-by rule, Native complete normalized bytes are
unchanged7/7, Ares changes exactly six failing inputs, and the wider-neck Ares
control is unchanged. Raw Native SHA differences are caused by generated-by
timestamps.

## Fresh paired full control sets

Original base and candidate have the same674-test inventory, including the
seven new cases; the new-main pair has the same675-test inventory. Base is RED; candidate is GREEN.
Each round uses its own fresh Native cache. Both producers and process outcomes must be retained for each change.

The original-main fresh official pair is complete: base658 PASS /16 FAIL
(162.815s), candidate664 PASS /10 FAIL (139.891s), inventory674 each.
Exactly the six listed narrow-neck cases change FAIL→PASS. Candidate-only
failures are empty; all other statuses/reasons are unchanged.

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
No UB is ported. Neither
official round contains a timeout.

The original seeded results below are preserved evidence, superseded for
merge by a new fresh paired seeded run on `bff9cb1b0`: an intervening main change
affects BBL multi-extruder placeholders, and this inventory includes H2D.
Official remains the original same-main pair; it is not represented as a
new-main official run.
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
After restoring the same read-only link to `OrcaSlicer`,
the entire base round was repeated with independent `seeded-base-v2-cache`
and `seeded-base-v2-artifacts` roots. No assertions, fixtures or production
code changed.

### New-main seeded pair and exact case changes

On main `bff9cb1b0`, fresh base669 PASS /6 FAIL (148.767s) → candidate
675/675 PASS (181.245s).
The seven directed tests are included and pass on the rebased candidate.

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

Workspace `cargo nextest run --workspace --no-fail-fast`:
8074 PASS,9 skipped,622.339s on main `bff9cb1b0` + GREEN. No failure.
The initial fmt check required only alphabetical module registration order.
After rustfmt,
fmt PASS; the order-only change is committed separately.

A subsequent zero-conflict rebase onto `dc8d7f00c` (typed template error/CLI
exit code, no G-code changes) produced RED `df133fbaf`, GREEN `de87a6fab`,
and formatting commit `30a107581`.
Seeded/official/workspace evidence was reused with post-rebase strict clippy and
directed checks. Strict clippy `--workspace --all-targets --all-features
-- -D warnings` PASS (1m05s). Fresh seeded directed seven-case whole-file
Native comparison PASS7/7 (2.273s). WASM `cargo check -p ares-core -p ares-wasm
--target wasm32-unknown-unknown` PASS (25.80s). fmt PASS.

Final zero-conflict rebase onto test-only main `92e9573df`, evidence
reused: RED `df9d67093`, GREEN `696617582`, formatting
`5eee457cf`. Production, input fixtures and directed module are unchanged. Final fmt
check PASS. Report-only changes do not alter compiled behavior.

The >16-candidate introsort/equivalent-key permutation port and
large-candidate Native witness remain deferred. Rust stable boolean sorting
matches the identified libstdc++ small-set insertion-sort behavior when the
same candidate input has at most16 elements. General std::sort tie parity is
not claimed. No Native candidate cardinality instrumentation was introduced;
emitted entity counts alone do not prove the pre-materialization count.
Any new full-control failure must be reported, including large-set cases;
this scope limit cannot excuse an unexplained candidate-only regression.

## Producer executables

Seeded Native executable SHA256:
`59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`;
resources `286fab66becc4c87ebb72dc8e66ec9d515e7f440d530bacebeb1f51407bb8a69`.
Official executable SHA256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
