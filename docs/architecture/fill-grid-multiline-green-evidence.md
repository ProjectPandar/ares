# FillGrid multiline mode0: bounded GREEN evidence

[Approved boundary](fill-grid-multiline-trapezoidal-spec.md), original SPEC
`44c55597e`; [accepted RED](fill-grid-multiline-native-red.md), original
`446cb48b4` (rebased evidence `211826964`, test `3ff9b763a`). Production
`814006f31` (initial trial tree `7328bba58`) starts on main `a120ebd5c`, after W2 C2-T merged.

## Separate mechanical extraction

`2005cf33c` extracts only the existing Stars post-row tail into
`fill/trapezoidal_tail.rs`. Reconstructing the old file before and after fmt
is byte-exact: SHA256
`2a26a7d0f3b83f2eb327ac3849f4ef3c686548318d53710a8b9b4207a4ec950d`.
Proof: `/tmp/worker6-grid-mechanical-text-audit.json`.

Fresh CPU11 official/seeded each 17/17 PASS, exit0:
`/tmp/worker6-grid-tail-mechanical.EfRSvh/{official,seeded}/{log,exit}`.
All 34 Stars/union captures preserve input ZIP members and Ares raw versus
main a120 canonical captures from `/tmp/worker2-c2t-full-a120-vxh3l5gx`.
`raw-invariance.json` records the exact paired paths and SHA256 values. This
baseline is an existing main capture, not a newly run worker6 base leg.

## Mode0 production and unchanged boundaries

`FillRectilinear.cpp:3048-3162` owns the new Grid setup and rows;
`:3344-3384` owns the shared tail. The generator retains float direction
promotion before double pi/4, integer spacing truncation before count,
floor-aligned bbox minimum, five-point cells, half-period alternating rows,
and XY transpose by layer_id/thickness_layers. `:3422-3446` confines the
ordinary two-family fill and odd-layer reversal to count1.

The normal Grid consumer and only the Grid anchoring arm dispatch count>1
to this generator. `/tmp/worker6-grid-arm-scope-audit.json` proves all text
outside that Grid arm unchanged. The approved bbox predicate extensions in
fill_entities.rs and transaction/candidate_expansion/layers.rs reuse the
existing actual object/source/transform; sparse_anchoring/mod.rs and all
bridge-anchor admission/projection code remain unchanged. No comparison,
request, fixture or assertion changes accompany production.

## Fresh finite controls

`/tmp/worker6-grid-mode0-trial.i9ctsZ/{official,seeded}/{log,exit}`:
26/26 PASS in each mode (Grid9, Stars7, union10), exit0. The trial started
from the working tree subsequently committed verbatim as `7328bba58`;
its recorded parent is mechanical commit `2005cf33c`. No source changed
while the batch ran. `affinity-snapshot.json` verifies the own script,
cargo and rustc process tree on CPU11; each mode also records parent affinity.

`control-audit.json` records 16 captures per mode: all seven accepted RED
cases now pass unchanged complete Native/API/CLI ordered-byte and line-count
comparisons; all nine paired count1 captures keep their Ares raw bytes from
RED. Every input retains the same ZIP member names/bytes as RED. Z/HEIGHT,
Native/API/CLI admission, sparse presence and combined-layer witness all
remain active assertions. `raw-invariance.json` separately proves all 34
Stars/union captures remain byte-exact versus main a120. Native products
remain outside the repository.

## Remaining acceptance

Official complete option_coverage: 220/220 PASS, exit0 at
`/tmp/worker6-grid-option-directed.FVi285/official/{log,exit}`. This includes
non-Native domain/inventory tests and existing visible seeded-only dependency
skips. The official-common-raw-audit.json matches 376 same-input keys to the
main a120 captures: all Ares raw bytes unchanged, no unmatched keys.

The seeded leg auto-started just before the between-leg deferral reached the
parent; it was stopped and marked INTERRUPTED/VOID in seeded/interrupted.json.
Its partial results are not evidence. A brief overlap with strict was
immediately corrected by stopping only the seeded option process tree.

Strict found excessive_nesting in the Grid arm. Final production `814006f31`
uses an exhaustive match to select the same calls and parameters before
extending the result; no other arm or assertions changed. The file is 398 LOC.
Tests-only `4b40fcd7c` fixes clippy duplicate_mod by importing the existing
multiline observation module instead of loading its file twice. Visibility
changes are restricted to the integration-test crate; helper bodies and
assertions are unchanged. Proof:
`/tmp/worker6-grid-observation-sharing-audit.json`.

Final-head strict/fmt/wasm32 each exit0:
`/tmp/worker6-grid-engineering.JbRFTF/{strict,fmt,wasm}.{log,exit}`.
The earlier strict failures are retained in engineering.21XPoh (nesting) and
engineering.a2biIs (duplicate module); neither is passing evidence.
Final-head finite controls, complete option_coverage both modes, allocated
workspace and explicitly authorized canonical full remain pending.
Finite GREEN is not a merge authorization. Existing count1 and non-RED
callers must remain raw-byte unchanged; any change stops for review.

## Final-head directed verification on C1 main

Clean rebase to `4ca7554c6` preserved all seven commits (`=` in every
range-diff row): `/tmp/worker6-grid-rebase-4ca.range-diff`. Exact tested HEAD
`3ca0838c8`; fresh CPU11 official/seeded results:
`/tmp/worker6-grid-final-directed.7oRuR5/{official,seeded}/{log,exit}`.
Each mode runs 229 tests (Grid9 plus complete option_coverage220), all PASS,
exit0. Grid7 RED cases retain strict Native/API/CLI ordered bytes and line
counts; the nine paired count1 captures retain their raw bytes from RED.
All 32 Grid input ZIP member sets remain unchanged from RED. No assertions
or comparator changes were required.

`control-audit.json` records those exact comparisons. `raw-invariance.json`
proves Stars7+union10 (34 captures across both modes) unchanged against current
main C1 full captures, not a new worker6 base run. The official and seeded
common-raw audits additionally match 376 and 384 same-input keys respectively,
all Ares raw unchanged with zero unmatched keys. Baseline:
`/tmp/worker3-centering-c1-full-4ca7554c6-vjqa0dvg/{official,seeded}/artifacts`.
The source-audit and arm-scope-audit in the directed root prove changed Rust
files <=400 LOC and only the Grid arm changed in arms.rs.

Canonical refs do not enable ARES_PARITY_OPTIONS. The existing sweep test
therefore explicitly reports SKIPPED: Native-not-executed. The coordinator
accepted this as the canonical-env policy used by full; no sweep Native
acceptance is claimed. Non-Native option_coverage tests and independent
Native controls were all selected, with existing seeded-only references
visibly skipped only where their canonical mode policy requires it.

`affinity-exe-audit.json` samples only the own process tree and classifies
Native by /proc/PID/exe readlink; all recorded masks are CPU11, zero violations.
The earlier generic affinity-audit is also retained. Its initial background
launch failed to persist; the premature path claim was corrected in the
inbox and sampling resumed in a managed session before Native began. The
02:12:04 unpinned process could not be worker6: its batch was still compiling
and had no test/Native execution at that time. No historical sample claim is
made for the period before the live sampler started.

Git-only rebase onto web/vgcode main `325ea6983` gives `91296c3fa`, seven
range-diff rows all `=`: `/tmp/worker6-grid-rebase-325.range-diff`. Per the
coordinator's carry ruling, directed results remain valid. Fresh strict
(workspace/all targets/all features, -D warnings), fmt and wasm32 checks all
exit0 on that HEAD: `/tmp/worker6-grid-engineering.iqdxPV`.
The local core-only gate on `91296c3fa` was explicitly stopped when the
coordinator moved that gate to CI self-hosted workspace on the exact candidate
hash. Its partial result is 5601/7177 PASS, zero FAIL/TIMEOUT; it is not
acceptance evidence. Logs and the interruption checkpoint are retained at
`/tmp/worker6-grid-core-gate.6VOW3y/{run.log,interruption.json}`. No test was
filtered or changed. CI workspace and canonical full have not been run for
this candidate; full remains queued after worker5 and needs FULL-GO.
