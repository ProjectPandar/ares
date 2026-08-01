# Task 22O Package A0 Qualification Recovery Attempt 7 Plan

## Goal

Preserve the whole terminal Attempt-6 campaign, repair the adjacent runtime
closure and publication state machine test-first, prepare a fresh build and
candidate before final authorization, execute one new 71-role campaign, and
publish only independently approved staged corpus/manifest bytes.

## Non-negotiable boundaries

- Do not alter the fixed Orca commit/tree, Package-0 patch, A0 source patch, or
  twelve fixed derivative source files.
- Do not add, remove, rename, or change anything beneath failed candidate
  `8a5aab7461b877b1`.
- Do not resume ordinal 70, execute ordinal 71 alone, or reuse any old leaf.
- Do not launch a formal process before final Attempt-7 authorization and an
  atomic create-once launch claim.
- Keep runtime loading adjacent to each probe. Do not add a formal PATH search
  fallback or copy a DLL at role time.
- For CTest only, pass a complete closed environment with no `PATH` key; reject
  literal empty, nonempty, inherited, or generated test-local PATH values. Keep
  the formal 71-role runner's separately recorded `PATH=""` contract unchanged.
  The exact CTest keys are `SystemRoot`, `TEMP`, `TMP`, `LANG`, `LC_ALL`, and
  `LC_CTYPE`; construct them without ambient merge, require absolute nonempty
  path values and exact `C` locale values, and reject case variants or extras.
- A new campaign runs all 71 roles exactly once. Any formal failure consumes
  Attempt 7 and requires Attempt 8.
- Stage and review corpus/manifest bytes before publishing final paths.
- Keep every edited source and test module below 400 physical lines.

## A0R7.1: persist document iteration 0001

1. Preserve the exact first reviewed specification and plan at:

   ```text
   coverage-repair/tooling-review/attempt-7/document-reviews/iteration-0001/
     specification.md
     plan.md
   ```

2. Persist the governance and technical `REVISE` reports beside them.
3. Build canonical `review-envelope.json` with the exact seven-key schema,
   both subject identities, both reviewer identities, both literal verdicts,
   `state="revise"`, and `implementation_authorized=false`.
4. Validate proposed envelope bytes twice before create-once publication.

Verification: the copied document identities are exactly
`ed011adaa68621af9b3bcc41fe72b7f9cfb73adc6e14c4e050a6a49a34096a9d`
and
`7a7ed57f6d5fadc863a726a0b8acfac8c1c648e7ca841c951b574c908933be2c`.

## A0R7.2: approve the repaired document frame

1. Freeze the revised specification and plan identities.
2. Allocate the next consecutive create-once document-review directory.
3. Send identical document bytes to the governance and technical reviewers.
4. Persist both reports and the per-iteration envelope.
5. If either verdict is `REVISE`, apply only its concrete repairs and repeat
   with another directory. Never mutate an old iteration.
6. If a reviewer explicitly emits terminal `REJECT`, stop Attempt 7.
7. When both final reports say `APPROVE`, publish
   `document-reviews/registry-v1.json` over the consecutive five-file
   iterations. Each ordered registry row binds the current envelope and the
   preceding envelope identity (null for `0001`) without mutating the exact
   seven-field iteration envelopes. Then publish
   `document-reviews/final-envelope.json` over that registry and the one
   commonly approved document pair.

Verification: the final envelope validates canonical bytes, consecutive
iterations, unchanged prior iterations, two approvals over the same subjects,
distinct governance/technical reviewer identities, and
`implementation_authorized=true`. No tooling source edit precedes it.

The late CTest launch correction uses final document iteration `0011`;
iteration `0007` remains immutable with its technical `REVISE`, and iteration
`0008` and `0009` remain immutable with their two approvals. Revision 9 closed
the repository-document hash fixture; the first fresh build then proved that
CTest 3.31.6-msvc6 cannot launch its first absolute test executable when the
closed process environment contains literal `PATH=""`. Iteration `0010` is
retained with governance and technical `REVISE` because it omitted the exact
environment closure, formal RED/GREEN and child-observation binding, complete
mutations, and child-observation producer authorization. Iteration `0011`
authorizes the corrected CTest PATH-absence, existing-path fixture/validator,
one-for-one required-ID, six-to-eleven history, and final document-hash updates
described below. The rebuilt document registry has exactly eleven consecutive
rows, and its final envelope binds the iteration-0011 documents and two
independent approvals without mutating iterations `0001`-`0010`.

## A0R7.3: freeze the complete Attempt-6 failure

1. Snapshot the external Attempt-6 qualification input and build fixed probe
   into the exact failure-handoff paths.
2. Generate PE imports with a fixed absolute tool identity and retain the text.
3. Generate a no-follow whole-tree ledger for the failed candidate, including
   every directory, file, and reparse entry.
4. Validate exactly 70 specs, 70 run directories, 69 passed CLI leaves, one
   failed direct leaf at ordinal 70, no ordinal 71, root `FAILED.json`, and no
   root `result.json`.
5. Build `failure-handoff-v1.json` over the complete ledger, snapshots,
   failure artifacts, process/tree/residue facts, and Attempt-6 approvals.
6. Label the loader-path conclusion as an inference because historical
   inherited PATH was not recorded.

Verification: regenerate both ledger and handoff twice in memory; require byte
identity after every later gate and immediately before formal launch.

## A0R7.4: RED runtime, candidate, and recovery contracts

Add one vertical public test at a time and retain one focused RED transcript.
The seams are the build-result validator, qualification-input builder,
candidate preflight, failure-handoff validator, and runner launch preflight.

Required RED sequence:

1. whole-tree failure validation detects arbitrary file mutation, deletion,
   addition, result injection, or ordinal-71 injection;
2. build/install runtime sets reject missing, extra, renamed, mutated, or
   reparse DLLs;
3. CTest evidence rejects any PATH key, generated environment modification, or
   non-adjacent loading;
4. build-result v2 rejects a loose/build-tree direct probe or unequal probe;
5. framed ID changes for every payload/domain/order/boundary mutation but not
   for root-directory renaming;
6. qualification v2 selects only `installed.fixed_probe`;
7. candidate parent accepts exactly the bound failed child and an absent new
   child; and
8. runner rejects absent/changed approval, an existing launch claim, candidate
   collision, or any Attempt-6 leaf reuse. The existing 69+2 mock campaign test
   also requires the bound absolute PowerShell launcher, `PATH=""` for every
   runner/formal child, only O/V per-role deltas, and rejection of ambient
   lookup or nonempty base PATH.

Verification: each new test fails for the intended public reason before its
production edit; no test mocks an internal validator.

## A0R7.5: GREEN runtime, candidate, and recovery contracts

The exact allowed source topology is:

New source modules:

```text
tooling/a0_candidate_v2.py
tooling/a0_candidate_v2.psm1
tooling/a0_failure_handoff.py
tooling/a0_runtime_closure.py
tooling/a0_publication.py
```

Modified source modules:

```text
fixed-probe/CMakeLists.txt
tooling/a0_cases.py
tooling/a0_common.py
tooling/a0_control_validation.py
tooling/a0_corpus.py
tooling/a0_qualified.py
tooling/a0_repair_contract.py
tooling/a0_run_validation.py
tooling/a0_source_build.py
tooling/a0_tooling_approval.py
tooling/a0_tooling_evidence.py
tooling/assemble_sidecar_manifest.py
tooling/verify_manifest.py
tooling/run_fixed_qualification.ps1
```

New test modules:

```text
tooling/tests/test_a0_candidate_v2.py
tooling/tests/test_a0_failure_handoff.py
tooling/tests/test_a0_runtime_closure.py
tooling/tests/test_a0_publication.py
```

Modified test/fixture modules:

```text
fixed-probe/tests/test_probe.py
tooling/tests/a0_manifest_fixture.py
tooling/tests/a0_tooling_approval_fixture.py
tooling/tests/qualification_fixture.py
tooling/tests/runner_fixtures.py
tooling/tests/test_a0_cases.py
tooling/tests/test_a0_corpus.py
tooling/tests/test_a0_source_build.py
tooling/tests/test_assembly.py
tooling/tests/test_assembly_integration.py
tooling/tests/test_manifest.py
tooling/tests/test_manifest_deep.py
tooling/tests/test_qualification_contract.py
tooling/tests/test_run_catalog.py
tooling/tests/test_run_one.py
tooling/tests/test_tooling_approval.py
```

Keep the two newly authorized existing tests below 400 LOC by moving obsolete
assembly/candidate/closed-set cases from `test_assembly.py` into the new
candidate/publication test modules, removing the closed-set mock from
`test_manifest.py`, and moving reusable manifest setup into the already
authorized `a0_manifest_fixture.py`. Moves are one-for-one: they add no path,
discovered test, or required-ID count.

Any additional source/test path requires another approved document iteration.
The revision-11 CTest repair adds no path or test. It replaces
`test_ctest_uses_adjacent_runtime_and_absolute_git_with_empty_path` with
`test_ctest_uses_adjacent_runtime_and_absolute_git_with_path_absent`, requires the
recorded CTest environment to omit `PATH`, rejects any generated `ENVIRONMENT`
or `ENVIRONMENT_MODIFICATION`, and updates existing CTest build-result fixtures.
The canonical build-result CTest record binds a literal-`PATH=""` RED, the
PATH-absent GREEN, and a create-once child-environment observation. The wrapper
validates the effective nine-key CTest child environment before any subprocess
and writes that observation to the exact build-root path passed by CMake.
Mutations cover every required parent/child key and value, extras, missing
entries, duplicate/case-variant PATH aliases, both generated properties, RED
status/transcript drift, and real wrapper injection.
The `test_run_one.py` edit is limited to launching the literal runner through
the already bound absolute PowerShell, setting the child process environment
to `PATH=""`, executing `$env:PATH=''` inside PowerShell before the literal
runner call, propagating `$LASTEXITCODE`, and asserting retained
`process.json["exit_code"] == 9`. It adds no test ID and does not modify
`run_one.ps1`.

The existing `runner_fixtures.py` authorization also covers one isolation
repair: create a `runner-evidence` sibling for derived build/input JSON rather
than overwrite the shared `formal_bundle()` evidence. The fresh `source`,
`runner-build`, `runner-install`, and `runner-evidence` roots remain pairwise
tree-disjoint.

The existing authorizations for `a0_qualified.py`,
`fixed-probe/CMakeLists.txt`, `fixed-probe/tests/test_probe.py`,
`a0_tooling_approval.py`, `a0_runtime_closure.py`,
`a0_repair_contract.py`, `tooling/tests/a0_manifest_fixture.py`,
`tooling/tests/a0_tooling_approval_fixture.py`,
`tooling/tests/test_a0_corpus.py`,
`tooling/tests/test_a0_runtime_closure.py`,
`tooling/tests/test_a0_source_build.py`, `tooling/tests/test_manifest.py`,
`tooling/tests/test_manifest_deep.py`, and
`tooling/tests/test_tooling_approval.py` cover only the literal six-to-eleven
document iteration cardinality/range, generated-scope, final revision-11
repository document identities, CTest RED/GREEN/child-observation
producer/fixture/validator, required-ID replacement, and mutation expectation
updates required by the retained revision-7 review, immutable
revision-8/revision-9 approvals, and retained revision-10 review. Preserve
consecutive five-file
iterations, closed schemas, predecessor envelope hash chaining, 69 source
roles, 28 test roles, 122 discovered tests, and 26 required IDs.

Implementation order:

1. add whole-tree handoff generation/validation without following reparse
   points;
2. add canonical build/install runtime-closure generation and comparison;
3. install the fixed probe at the install root and remove every CTest
   `ENVIRONMENT` or `ENVIRONMENT_MODIFICATION`;
4. add build-result v2 and recorded closed-environment CTest evidence;
5. add root-relative qualification-input v2;
6. add identical labeled/length-delimited ID implementations in Python and
   PowerShell;
7. update runner/collector/deep verification to consume only build-result v2
   installed-probe identity; and
8. add final-envelope and atomic launch-claim validation without launching any
   formal role.

Verification: focused RED tests turn green; the mock catalog remains exactly
69 CLI plus two direct roles and stops at the first failure.

## A0R7.6: RED/GREEN publication state machine

The public seam is
`validate_repair_closed_set(root, repository, phase, publication_root)`.
Closed-set validation is never mocked.

1. Remove prefix exclusions for `fixed-probe/build`,
   `runs/qualified`, and `__pycache__`.
2. Require repository-local `fixed-probe/build` and every `__pycache__`
   absent. Validate qualified candidates through their own complete scope.
3. Add `static-topology-v1.json` for the fixed source/test/doc scope only.
   Freeze 135 A0 paths and 14 repository documents; 69 source roles split into
   49 Python, eight PowerShell, and twelve other paths; 28 test-role paths;
   exactly 122 discovered tests; exactly 26 required IDs; and 31 immutable
   historical prerequisites.
4. Add hash-chained registry validators. Document iterations contain five
   files, tooling iterations four, and publication iterations six. Require
   consecutive literal child names and exact closed schemas; do not
   pre-enumerate future evidence iterations in the static topology.
5. Add `publication/policy-v1.json`, which freezes publication iteration and
   root-relative binding schemas before launch.
6. Add real stage corpus files at the planned literal iteration path.
7. Reproduce the current closed-set rejection through the real assembler.
8. Implement `stage-prepublish`: exactly two corpus files, manifest/reviews
   absent. `assemble_sidecar_manifest.py` invokes this before writing.
9. Implement `stage-published`: corpus plus manifest, reviews absent.
   `verify_manifest.py --phase stage-published` invokes this.
10. Implement `stage-revise`: corpus, manifest, two ordered reports, and a
    rejected envelope. Accept only `APPROVE/REVISE`, `REVISE/APPROVE`, or
    `REVISE/REVISE`.
11. Implement terminal `stage-reject` for either ordered verdict containing
    `REJECT`; preserve the six files and forbid another stage.
12. Implement `stage-approved`: corpus, manifest, two reports, and envelope.
   `verify_manifest.py --phase stage-approved --require-approval` invokes it.
13. Implement `final-published`: final corpus/manifest match the approved stage
   and final reviews are absent. `a0_publication.py verify-final` invokes it.
14. Implement `approved`: add exactly the final two reports and envelope.
    `verify_manifest.py --phase approved --require-approval --deep` invokes it.
15. Make all publication artifact records canonical relative paths resolved
    only beneath an explicit `publication_root`; reject absolute, dot-dot,
    alias, and owner-relative fallback paths.
16. Reject missing, swapped, substituted, unbound, adjacent, reordered, or
    byte-mutated corpus/review evidence.
17. Reject equal governance/technical reviewers at document finalization, a
    tooling reviewer equal to either document reviewer at launch, and equal
    sidecar reviewers or a sidecar reviewer equal to the tooling reviewer at
    every sidecar phase.

Verification: focused RED/GREEN transcript proves create-once assembly and
subject/deep approval through the real closed-set validator.

## A0R7.7: full tooling verification before fresh build

1. Run the complete Python tooling suite.
2. Run static/source/ASCII/LOC checks; every edited source/test file is below
   400 physical lines.
3. Require `fixed-probe/build` and `__pycache__` absent after verification.
4. Freeze a provisional exact source ledger and complete transcript.
5. Revalidate the document registry/final envelope and failure handoff.

Verification: all public tests pass, all 15 new required test IDs are present once,
the full suite reports exactly 122 tests and 26 required IDs, and the static
topology matches all 69 source and 28 test-role paths.

## A0R7.8: prepare the fresh build and candidate

Use new short absolute source, build, install, and evidence roots. None may
alias or reuse the Attempt-6 roots.

1. materialize the fixed commit into a new source root;
2. apply and byte-verify the Package-0 and A0 patches;
3. hash the dependency tree before reuse;
4. configure and build all targets;
5. install Orca and the fixed probe;
6. enumerate the installed top-level DLL closure;
7. copy that exact DLL set beside the build probe;
8. generate build-before-CTest and installed-after-install closure records;
9. require identical basenames/bytes/hashes and equal probe bytes;
10. bind absolute Git and PowerShell executable identities;
11. pass Git to the probe wrapper through required `--git <absolute-path>`
    and use it for all wrapper Git subprocesses;
12. run and bind a canonical CTest RED showing literal `PATH=""` yields exit 8
    and `BAD_COMMAND`, then run fixed-probe CTest 1/1 through absolute commands
    with the exact six-key PATH-absent environment;
13. bind the wrapper's create-once canonical nine-key child-environment
    observation, prove no key case-folds to PATH, and prove generated CTest
    properties contain neither `ENVIRONMENT` nor `ENVIRONMENT_MODIFICATION`;
14. generate build-after-CTest and installed-prelaunch records and compare all
    four closures;
15. install completion must precede CTest, and no role-time copy is permitted;
16. hash the dependency tree after all work and require exact equality;
17. build and validate canonical `build-result-v2.json` twice;
18. build canonical root-relative `qualification-inputs-v2.json`;
19. independently compute the framed ID in Python and PowerShell; and
20. require the exact new candidate path and launch claim absent, with the
    qualified parent containing only the unchanged failed candidate.

Verification: source/status/patch identities remain frozen; build/install/CTest
pass; closure regeneration is exact; directory renaming leaves candidate ID
unchanged; any byte or qualification-plan mutation changes it.

## A0R7.9: final six-axis tooling review and launch envelope

1. Freeze the final source ledger and full transcript, including fresh build,
   CTest, closure, handoff, candidate, closed-set, and static results.
2. Allocate a create-once tooling-review iteration.
3. Send the exact subjects to a fresh read-only reviewer for all six axes.
4. Apply a concrete repair list through a new source/transcript/review
   iteration until `APPROVE`. Any source, test, CMake, runner, build-result,
   qualification-input, closure, or candidate change discards all fresh roots
   and repeats A0R7.7-A0R7.8 completely before that new review.
5. A terminal `REJECT` stops Attempt 7.
6. Hash-chain every four-file tooling iteration, then publish
   `tooling-reviews/registry-v1.json`.
7. Publish `tooling-reviews/final-envelope.json` with its exact ten-key closed
   schema and a reviewer distinct from both document reviewers.
8. Publish `prelaunch-evidence-v1.json`, then the sole runner-consumed
   `launch-approval-envelope.json` with its exact twelve-key candidate schema.
   It binds build/input/candidate/parent/handoff/static-topology/document and
   tooling registries/publication-policy identities.
9. Recompute installed closure, failed-candidate tree, parent children, and
   candidate absence immediately before launch.

Verification: full approval reports Attempt 7, approved state, one authorized
candidate, launch claim absent, no subject mutation, and formal execution
authorized.

## A0R7.10: one atomic fresh 71-role campaign

1. Invoke the production runner once with build-result v2, qualification-input
   v2, and the literal `launch-approval-envelope.json`; reject every alternate
   envelope path.
2. The runner validates all three and atomically creates
   `launch-claim-v1.json` using create-new semantics before candidate creation.
3. The runner creates only the exact approved candidate child.
4. Resolve the bound absolute PowerShell executable, set the base child PATH to
   empty, and execute all frozen roles from ordinal 1 through 71; ORCA O/V
   variables are the only per-role deltas.
5. Stop and retain the first failure. Never retry, overwrite, resume, or reuse.

Verification: success reports 71 passed processes, 69 CLI, two direct, paired
equality, no retry/selection, no residual process, and no root/temp residue.
Failure stops all formal work and requires Attempt 8.

## A0R7.11: stage, review, and publish corpus

1. Collect only the successful Attempt-7 candidate.
2. Validate semantic coverage and workspace integrity.
3. Allocate a create-once publication iteration.
4. Build the two corpus files twice in memory and publish them only to the
   stage. Run `stage-prepublish`.
5. Assemble the staged manifest once; run `stage-published` subject and deep
   verification.
6. Send the same staged corpus/manifest identities to the independent
   fixed-source and qualification reviewers.
7. All manifest corpus roles and review-envelope records use canonical paths
   relative to the explicit publication root; stage and final topology match.
8. A split or double `REVISE` result persists both ordered reports and its
   rejected envelope, validates `stage-revise`, retains the stage, and creates
   a new hash-chained iteration.
   Only unchanged launch-bound sources/candidate/evidence may be reused; a
   change to any launch-bound subject requires Attempt 8.
9. Either `REJECT` persists both ordered reports and terminal envelope, runs
   `stage-reject`, creates and validates `publication/registry-v1.json` over
   every consecutive retained stage, and only then stops Attempt 7 without
   final corpus publication or a final sidecar envelope.
10. Require sidecar reviewer identities to differ from each other and from the
    tooling reviewer.
11. After both approve, persist staged reports/envelope and run
   `stage-approved`.
12. Create `publication/registry-v1.json` over all consecutive stages.
13. Copy only those approved staged corpus/manifest bytes create-once to:

   ```text
   corpus/corpus-v1.bin
   corpus/corpus-v1.json
   sidecar-manifest-v1.json
   ```

14. Run `final-published`, then copy the identical approved report bytes to
    the two final report paths.
15. Generate a separate canonical final sidecar envelope that binds the
    postlaunch registry, approved stage envelope, final manifest, and final
    reports; publish it at the final approval-envelope path.
16. Make the stage unavailable and run full `approved` deep verification,
    proving only final-root-relative paths are resolved.

Verification: final corpus, manifest, and report bytes equal one approved stage
exactly; the final wrapper envelope binds that stage and registry. No final path
is overwritten. Any postpublication rejection or drift requires Attempt 8.

## A0R7.12: release gate

Only after approved deep verification:

1. update the Task22O progress ledger with exact identities and results;
2. release the separately reviewed adapter/engine-selection work;
3. keep tracked Rust production work blocked until its existing gate approves;
4. retain the complete failed Attempt-6 tree and all Attempt-7 evidence; and
5. continue Task22O packages test-first toward exact KSR G-code output.

## Checklist

- [ ] Complete Attempt-6 tree and external-only inputs are durably frozen.
- [ ] Both document reviewers approved the same repaired spec/plan bytes.
- [ ] Runtime, candidate, failure, and publication tests passed RED/GREEN.
- [ ] Full tooling, static, source, ASCII, and LOC gates passed.
- [ ] Fresh build-result v2, dual closure, and CTest evidence validated.
- [ ] Candidate ID matched independent framed Python/PowerShell computation.
- [ ] Six-axis tooling review and single-use launch envelope approved.
- [ ] Atomic launch claim preceded one full successful 71-role campaign.
- [ ] One staged corpus/manifest pair received both sidecar approvals.
- [ ] Final create-once publication passed approved deep verification.
- [ ] Rust production work remained blocked until every A0 gate passed.
