# Lightning generator configuration owner plan

1. Cite PrintObject.cpp1010-1020/2494-2498 and Generator.cpp66-100, record
   owner/activation decisions in ARD-0091 and the included/deferred spec.
2. Reproduce both configuration contracts on clean45cfb1d4 through the existing
   bridge transaction and consume actual generated polylines. Do not widen
   fixture visibility or carry regional addressing/support WIP.
3. Port only positive-density activation and first global configuration owner;
   preserve bridge pattern-only flag and all generator arithmetic.
4. Verify focused and full nextest, fmt/Clippy, actual relevant Orca output
   controls and Tier1 compile targets. Independent six-axis review must check
   this clean closure rather than borrowing main-WIP results.
5. Commit/push code, tests and docs together only after independent acceptance;
   keep Normal/Tree and other regional WIP unsubmitted.

## Evidence

Clean candidate `/tmp/ares-lightning-owner-clean.BmhCOL` is detached45cfb1d4,
dedicated target `/tmp/ares-lightning-owner-clean-target`, incremental disabled.
No shared target cache or source/reference build changes. Existing upstream
source is linked read-only for repository test dependencies.

Test-only RED41395 in `/tmp/ares-lightning-owner-clean-red.log` terminal100,
14.578s2run/2FAIL7070skip. Width assertion fails at actual ordered polyline
equality. Zero density instead reaches a distance-field panic with the old
activating-region configuration; it is not an independent activation assertion
RED. Compilation succeeds, but do not claim both transactions complete.
Apply only owner first, keep old pattern-only activation and rerun both cases
to isolate the activation omission with nonzero global front density. Remaining
verification and independent closure review are pending; no submission or broad
parity claim.

Owner-only isolation99103 in
`/tmp/ares-lightning-owner-clean-owner-green-activation-red.log` terminal100,
14.542s2run1PASS/1FAIL7070skip. Width passes14.530s; zero-density fails7.226s
at the intended `None` assertion, not a constructor or slot panic. This proves
the second activation contract independently after repairing ownership.
Add positive-density generator activation; bridge flag and fixture visibility
stay unchanged. Final clean workspace, Clippy, public controls and platform
checks now required before independent submission review.

Final clean validation started against the frozen production SHA256
`4ad5a104e1375bc2c17b8f63efd1c3fa2b5b247ccb249fee0040d57eeeb3cdd7`
and test SHA256
`343c1e58ca5d5f4184f9c515570266784882d618b24ff241485d6e44f5593421`.
Logs use `/tmp/ares-lightning-owner-clean-` followed by `workspace.log`
(1182), `clippy.log` (37747), `seeded.log` (46283), `official.log` (92975),
`wasm.log` (76825), `macos.log` (85983), and `windows.log` (44986).
All were started with the dedicated clean target and incremental disabled.
Workspace uses `cargo nextest run --workspace --no-fail-fast`; Clippy uses
`cargo clippy --workspace --all-targets`. Platform checks are compilation only.
Both native controls enable the Lightning and emission-phase reference flags
and select the three complete-output Lightning/ironing/timelapse tests.
Fresh artifact roots are `/tmp/ares-lightning-owner-seeded.QRNu8v` and
`/tmp/ares-lightning-owner-official.G0dIMC`; neither cached WIP controls nor
disabled-test early returns count as evidence. Terminal results and independent
six-axis acceptance are still pending.

Terminal clean workspace1182 exits0:265.392s7473run7473PASS/9skip/36slow.
New width/zero-density cases pass33.943s/17.100s. Clippy37747 exits0,
5m39s; inherited252 core warnings and268 test warnings/252duplicates remain.
No warning in the new test; unchanged generator signature retains its existing
argument-count warning. WASM76825, macOS85983 and Windows44986 exit0 at
7m40s/7m59s/8m18s (including build-lock waits), compilation only.

Seeded46283 exits100:3run0PASS/3FAIL280skip, before native export because the
launch omitted mandatory ORCA_REPLAY_SEED/CPU. Preserve this setup failure.
Corrected44505 explicitly supplies seed0/CPU0 and a new empty artifact root
`/tmp/ares-lightning-owner-seeded-fixed.cD9Mo5`; its log
`/tmp/ares-lightning-owner-clean-seeded-fixed.log` exits0:3.013s3run3PASS280skip.
Official92975 exits100:1.178s3run2PASS1LightningFAIL280skip. First difference
is M73 placement at byte1808/line83; it is not filtered or reclassified PASS.
Separate clean45cfb1d4 baseline `/tmp/ares-emission-phase-clean.AVW3Ou` now
runs the same three activated controls with its own dedicated target
`/tmp/ares-lightning-owner-baseline-target` and fresh artifact root
`/tmp/ares-lightning-owner-baseline-official.3O60nc`. Baseline terminal pending.

Fresh native execution records retain Export/Slice command, status and output
under each orca-runner directory. Harness manifest producer identity remains
unknown by its current schema; do not relabel that cached-byte field. External
run identity is the unchanged official ELF SHA256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`,
or approved initialized replay wrapped ELF SHA256
`8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d`
(launcher SHA256
`3d05d4ccab740ca5e13aa94d0d00867a75f867ba2159b9f5b024dc0fcaa3bfaa`).
No source/build-policy/wrapper changes. Comparator still drops only generated-by
lines. Baseline comparison and final independent review remain required.

Baseline68461 exits100:0.980s3run2PASS1LightningFAIL280skip (build1m15s).
Its newly exported input alone does not exclude a regression. Therefore both
actual CLIs additionally slice the candidate failure's exact retained input
`/tmp/ares-lightning-owner-official.G0dIMC/case-ENU77T/input.3mf`, SHA256
`7e1f536baf3d24b06b7b8ad7b92be76e65108f85888c8a936f9fa06950f21a24`.
Baseline CLI SHA256
`39bc0a43387ee1c85b123b57d4026e4a704c12e258c6230495c83863a47c3465`
and candidate CLI SHA256
`f34cafdf4b41f198cbc2bf36434c8b3a9556c84983bdbed0f5bde556b5b1d525`
both exit0. Retained outputs `/tmp/ares-lightning-owner-baseline-same-input.gcode`
and `/tmp/ares-lightning-owner-candidate-same-input.gcode` compare exactly
with generated-by lines alone removed (actual cmp exit0); candidate also
matches the original failed-case Ares artifact (cmp exit0). This establishes
unchanged complete Ares output for the official counterexample, not merely
matching failure statuses on different exports.

That same retained input was sliced once more by unchanged official and once
by approved seed0/CPU0 initialized replay (both exit0), retained respectively
under `/tmp/ares-lightning-owner-same-input-official.SB4Jhm` and
`/tmp/ares-lightning-owner-same-input-seeded.redf16`. Candidate/replay cmp exits0;
candidate/official-repeat cmp exits1 (byte1764/line82). Original official and
official-repeat streams themselves differ (cmp exit1 byte1818/line84).
No selection of a passing official run, extra filter or changed assertion.
Baseline, candidate and initialized native complete streams have generator-only
normalized SHA256
`614920bd711b967acbdcbaa4e4ccc989e6ef8ab61e5675d47be291df2fbd204f`.
The original official Lightning counter remains RED; independent acceptance
must decide this finite source fix separately from global official parity.

Final independent six-axis review recomputes the exact retained input and
baseline/candidate/original-case/initialized-native normalized hashes, accepts
only this six-file owner/activation closure under the independent pre-Tree
exception, and finds no remaining mandatory finding. Official Lightning RED
stays recorded; its varying output is not a proven root-cause diagnosis.
No Normal/Tree, complete regional mapping, platform runtime or global approval.
Modified Rust files are297/294/80LOC. Freeze/check the exact staged six-file
patch and submit code, behavioral tests, spec, plan and ARD together.
