# Task 22O Package A0 Qualification Recovery Attempt 8 Specification

## Status

Proposed, revision 6. Before final document approval, only create-once
document-review iteration artifacts are authorized; all other Attempt-8
evidence publication, tooling source edit, fresh build, launch claim, and formal
process remain forbidden. One reviewer owns governance and requirement
completeness; the other owns technical correctness and executable verification.
Reviewer feedback is iterative.

Document iterations preserve every reviewed frame:

- iteration `0001` binds revision-1 spec
  `272a116dd3af48ae70e77d081f91720bd499c2f2f94fd3d6bbd7fc48534d67a3`
  and plan
  `3c9b519997439b5d6c679164a7e9dc719251a97299d8362c1b9fc810eff13ccc`,
  plus both initial `REVISE` reports;
- iteration `0002` binds revision-2 spec
  `4fac70f0622f3dc384c229d4ff1d3217f0d6ccf1c967dbc16efd0d8a51f196f6`
  and plan
  `0593227ac84736cf776938bd20b396fbbe39be6d6416f6161f0254b90dec0d43`,
  plus both second `REVISE` reports; and
- iteration `0003` binds revision-3 spec
  `4a01248e37f504a84bf10889199301f812727fdf93f7128f0b542ddd9d07ba91`
  and plan
  `ff9adef3aab72339f7212a4b0869b4dd0b173d366b9b1dcedc5ad57304a8329e`,
  plus governance `REVISE` and technical `APPROVE` reports; and
- iteration `0004` binds revision-4 spec
  `9f9335174c1144ebf776de921205da145010e4fd442c912f60e6289ec35e4a6a`
  and plan
  `9a44d080fecd0dd2e97a964d6635d9331fb285bab24afd0eea16927236d1e2ce`,
  plus governance `APPROVE` and technical `REVISE` reports; and
- iteration `0005` binds revision-5 spec
  `50c770d32ad1f4599f2f01126b2d544afb86e3e6ac880efa87b2672d5bd5ab2e`
  and plan
  `875448c0b3d45c68db47400128c8f8b40bd04aed37e8ee8787c6bbbe42d41627`,
  plus governance `REVISE` and technical `APPROVE` reports; and
- iteration `0006` binds this revision-6 pair and its two review reports.

All reviewed five-file iterations are created once and retained. A
registry/final envelope is published only when the latest consecutive iteration
(currently `0006`) receives two `APPROVE` verdicts.

## Purpose

Attempt 7 was invoked exactly once and failed before its atomic claim because
the first production `generated-scope` validator exceeded its fixed 180-second
deadline. The validator repeatedly rehashed the same retained 3.42 GB failed
candidate tree during one public validation call. Attempt 8 repairs that
preclaim validation path test-first, prepares wholly fresh build and
qualification subjects, and authorizes one new 71-role campaign.

This is an ignored Package A0 oracle-tooling repair. It does not change Ares
production behavior, the fixed Orca derivative behavior, the 11-case matrix,
the 23 invalid controls, the 69 CLI plus two direct role order, or any expected
KSR G-code bytes.

## Fixed upstream and Ares boundaries

The Task 22O rewrite boundary remains OrcaSlicer tag `v2.4.2`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, specifically:

- `src/libslic3r/PerimeterGenerator.cpp`,
  `PerimeterGenerator::process_classic()` at fixed-source lines 1144-1692;
- `src/libslic3r/LayerRegion.cpp:82-142` as the caller/output seam; and
- the fixed Voronoi dependencies already cited by Package A0.

The eventual Ares destination remains `crates/ares-core` over
`PreparedPostPerimeterInputs`, producing ordered Classic perimeter islands,
loop/path metadata, gap-fill entities, `fill_surfaces`, and
`fill_no_overlap`. Existing rectangle-oriented Ares code is not a fallback.
Attempt 8 changes only ignored qualification tooling and evidence.

## Consumed Attempt-7 launch

The following failure is immutable:

- transcript:
  `C:\a22or11-driver\attempt7-campaign-formal-run-v1.txt`;
- byte count: `234`;
- SHA-256:
  `37f5c17ad0e06247f03737be25be8c087964a40ae6530c3598a5350fb60e462a`;
- created: `2026-07-24T14:44:00.0169290Z`;
- finalized: `2026-07-24T14:47:02.8639345Z`;
- wrapper exit code: `1`;
- exact terminal error: `Attempt7 Python validator timed out`.

The failure occurred in the first `a0_tooling_approval.py generated-scope`
call, before hydration, claim creation, candidate creation, or any Orca role.
Attempt-7 candidate `0897d8a2b652de82` and
`coverage-repair/tooling-review/attempt-7/launch-claim-v1.json` remain absent.
The qualified parent contains only retained failed candidate
`8a5aab7461b877b1`.

Attempt 7 is nevertheless consumed. Its formal wrapper and production runner
must never be invoked again. Preserve byte-for-byte:

- the failure transcript;
- every Attempt-7 document, tooling, prelaunch, launch, and publication-policy
  subject;
- A22OR11 build-result and qualification-input evidence;
- the complete Attempt-6 failure handoff and failed candidate tree; and
- the qualified-parent child set.

Do not create an Attempt-7 claim, placeholder candidate, retry marker, or
replacement transcript.

## Attempt-8 failure handoff

After document approval and before source repair, create these immutable
subjects with create-new semantics:

```text
coverage-repair/tooling-review/attempt-8/prior-attempt/attempt7-runner.ps1
coverage-repair/tooling-review/attempt-8/prior-attempt-v1.json
```

The runner copy has independent rooted identity
`{root, path, bytes, sha256}` and must be exactly 19,865 bytes with SHA-256
`301ef651ebf5a1b0c296c8815dfbedbdeb1a31d14b7f50c75c70389e45f86ba4`,
at its new immutable path. Separately, the validator uniquely selects terminal
Attempt-7 source-ledger row `tooling/run_fixed_qualification.ps1` and requires
that row to have the same SHA-256 and exactly 390 physical lines. Paths and
schemas are not compared as equal. Later source edits never change this copy.

The canonical prior-attempt record binds:

- the external failure transcript identity and timestamps;
- the Attempt-7 launch-approval identity;
- the A22OR11 build-result and qualification-input identities;
- the immutable runner copy and terminal Attempt-7 source-ledger identities;
- wrapper exit code and exact error;
- absent Attempt-7 claim and candidate;
- qualified-parent children exactly `["8a5aab7461b877b1"]`; and
- an empty matching-process set recorded after document approval, immediately
  before handoff publication and source repair.

Its `launch_subjects` object contains exactly the ten subjects from the frozen
Attempt-7 launch envelope:

```text
document_final
tooling_final
prelaunch_evidence
build_result
qualification_inputs
powershell
failure_handoff
failed_candidate_tree
static_topology
publication_policy
```

The public Attempt-8 validator rehashes the transcript, runner copy, source
ledger, launch envelope, and each of those ten nested subjects; requires the
nested identities to equal the frozen launch bytes; rechecks the absence facts
and parent children; and rejects any changed or aliased root/path pair. The
handoff is a reviewed launch subject.

## Single failed-tree scan contract

`validate_generated_scope_and_binding`, `validate_generated_scope`,
`validate_launch_binding`, `validate_full_approval`, and
`validate_pre_review` each create one fresh no-follow snapshot of the A0 tree
per top-level call. Wrapper functions share the snapshot with the delegated
implementation. Within that one call:

1. failed-candidate ordinary-file content is physically hashed no more than
   once through the expensive tree snapshot;
2. failed-candidate rows are derived from the snapshot by exact path-prefix
   projection and path rebasing;
3. the failed-candidate canonical snapshot, topology, transcript invariant
   scope, review-ready scope, and prelaunch exact scope consume the same rows;
4. no nested validator walks or hashes the failed candidate again; and
5. no module-global, process-global, persisted, or cross-call cache exists.

A later public call always takes a new snapshot. Mutating, adding, deleting,
renaming, or reparse-substituting any failed-candidate entry between calls must
be rejected. Existing same-byte hardlink topology semantics are unchanged and
are not expanded by this performance repair because current canonical rows do
not encode link equivalence. Small approval and metadata subjects may still be
independently rehashed for their explicit identity bindings. The optimization
may reuse immutable row data only inside one synchronous public call; it may
not weaken no-follow, case-fold-alias, canonical-path, topology, or file-content
identity checks.

The fixed 180-second production deadline remains. Increasing it is not the
repair. `Invoke-A0Python` may expose a test-only timeout parameter with the same
180000 ms default solely so an existing integration test can exercise the exact
kill-and-error branch quickly.

## Test-driven repair

Use existing source and test paths. Keep exactly 69 source roles, 28 test-role
paths, 122 discovered tests, and the existing required-test cardinality.
Extend existing test IDs rather than adding a new test ID.

The RED phase must prove both:

- real `validate_generated_scope_and_binding`, `validate_generated_scope`,
  `validate_full_approval`, and `validate_pre_review` calls over small
  filesystem fixtures delegate failed-candidate content hashing no more than
  once per top-level call, while `validate_launch_binding` is covered through
  its shared generated-scope implementation;
  and
- the real PowerShell `Invoke-A0Python` function kills a sleeping child and
  emits the exact timeout error under a short injected test deadline, without
  creating a claim/candidate or leaving a child process.

The GREEN phase must additionally prove:

- two separate public calls perform two independent snapshots;
- a between-call mutation is rejected;
- transcript, review-ready, and prelaunch scope validation use the shared
  per-call rows;
- all 122 tests pass;
- the live production `generated-scope` validator completes below 180 seconds;
  and
- every edited source/test module is ASCII, below 400 physical lines, and
  contains no Rust `include!` or `include_bytes!` source-splitting macro.

Tests use public validators and real filesystem fixtures. The single-scan
assertion may instrument the hashing seam with a delegating spy, but it may not
mock closed-set validation or replace the public call with private helpers.

## Attempt-8 review and launch subjects

Attempt-8 evidence lives only below:

```text
coverage-repair/tooling-review/attempt-8/
```

It uses consecutive create-once document-review and tooling-review iterations,
hash-chained registries, a static-topology record, the prior-attempt handoff, a
publication policy, `prelaunch-evidence-v1.json`,
`launch-approval-envelope.json`, and `launch-claim-v1.json`.

The static topology retains the same source/test path sets and contains 18
repository document records: the prior 14 plus exactly these four:

```text
docs/superpowers/specs/2026-07-24-ksr-fdmtest-v4-task22o-package-a0-qualification-recovery-attempt-8.md
docs/superpowers/plans/2026-07-24-ksr-fdmtest-v4-task22o-package-a0-qualification-recovery-attempt-8.md
docs/superpowers/specs/2026-07-21-ksr-fdmtest-v4-task22o-classic-perimeter-generator.md
docs/superpowers/plans/2026-07-21-ksr-fdmtest-v4-task22o-classic-perimeter-generator.md
```

Static-topology and document-envelope validators rehash all 18 exact repository
paths. Every schema kind, candidate domain, runner message, literal launch path,
claim path, and attempt number used for the new campaign must say Attempt 8.
Historical Attempt-7 records retain Attempt 7.

The Attempt-8 launch envelope has exactly eleven subject keys:

```text
document_final
tooling_final
prelaunch_evidence
prior_attempt
build_result
qualification_inputs
powershell
failure_handoff
failed_candidate_tree
static_topology
publication_policy
```

`prior_attempt` binds `prior-attempt-v1.json`; the other ten preserve the
Attempt-7 launch subject roles with fresh current-attempt review/build
identities where applicable. Exact-key validation rejects omission,
substitution, or an extra subject.

The Attempt-8 candidate ID uses a new framed Attempt-8 domain in both Python and
PowerShell. It binds a fresh source ledger, installed Orca, installed probe,
installed runtime closure, role plan, and qualification inputs. Python and
PowerShell computations must agree.

Any source, test, CMake, runner, build-result, qualification-input, runtime
closure, or candidate-domain change invalidates all prepared fresh roots and
requires another fresh build/input preparation before tooling review.

All current-attempt source adaptations happen before complete tooling
verification and fresh input generation. This includes the Python/PowerShell
candidate domain; Attempt-8 paths, kinds, numbers, runner errors, literal launch
and claim paths; prior-attempt validation; eleven-subject launch validation;
and fixture/static-topology expectations. Focused REDs precede those production
changes. After they turn GREEN, run the complete 122-test/static verification,
then create the fresh build and compute qualification inputs with the already
final Attempt-8 domain. A later source repair repeats focused verification,
complete verification, fresh build, and input generation, but only before the
sole launch envelope is published.

## Fresh build and one-shot campaign

Use four new literal, ordinary, canonical, pairwise tree-disjoint roots for
source, build, install, and evidence. They must not reuse A22OR8 through
A22OR11 or any Attempt-6 root. Recreate and independently validate:

- fixed-source commit/tree and patch identities;
- configure, build, install, CTest RED/GREEN, and child-environment evidence;
- all four adjacent-DLL runtime closures;
- build-result v2 and deterministic qualification inputs v2;
- the Python/PowerShell candidate ID; and
- candidate absence and exact qualified-parent children.

After an independent six-axis tooling reviewer, whose identity differs from
both document reviewers, approves the exact final source ledger, full 122-test
transcript, fresh build/input evidence, prior-attempt handoff, and launch
contract, publish the sole literal Attempt-8 launch envelope. The envelope
validator enforces that cross-gate identity inequality. The runner validates it
and atomically creates the Attempt-8 claim before candidate creation.

After the approved launch envelope is published and before formal invocation,
measure the live Python
`a0_tooling_approval.py generated-scope` CLI with a fresh non-formal external
watchdog. It uses the launch-bound bundled Python with `-B`, sets
`PYTHONDONTWRITEBYTECODE=1`, uses the bound tooling working directory, and
limits `PATH` to the bound PowerShell and Git directories. The watchdog must
recursively terminate and wait for the process tree on timeout. This measurement
must not invoke the PowerShell production runner or formal wrapper and must
prove the Attempt-8 claim/candidate remain absent before and after, so it does
not create a claim. Before and after both success and failure, require no
matching validator/formal-wrapper/production-runner/Orca/fixed-probe process
and no recursive `__pycache__`, `.pyc`, or `.pyo` residue under A0. If it fails
or exceeds 180 seconds, formal invocation is forbidden, all Attempt-8 bytes
remain immutable, Attempt 8 is consumed, and recovery requires a separately
reviewed Attempt 9. No Attempt-8 source repair or replacement launch envelope
is allowed afterward.

The formal wrapper's sole create-once transcript target is
`C:\a22or12-driver\attempt8-campaign-formal-run-v1.txt`. Immediately before
invocation require that transcript, the canonical Attempt-8 claim, candidate,
and runner temporaries `a0-mock-tree-<candidate-id>.json` and
`a0-mock-tree-<candidate-id>.tsv` at the repository root are absent. Invoke the
formal wrapper exactly once. Run all 71 roles from ordinal 1 with no resume,
retry, overwrite, or old-leaf reuse. Any failure consumes Attempt 8 and requires
a separately reviewed Attempt 9.

## Postcampaign publication

Only a successful 71/71 candidate may enter the existing stage/review/publish
state machine under the Attempt-8 publication root. Two independent sidecar
reviewers inspect the same staged corpus/manifest bytes. Reviewers differ from
each other and from the tooling reviewer. Publish final corpus, manifest,
reports, and final sidecar envelope create-once only after both approve.

After final publication, make the stage unavailable and run full deep approved
verification to prove all identities are final-root-relative. Any rejection or
launch-bound byte drift requires the next attempt; final paths are never
overwritten.

## Approval gates and exit

Attempt 8 has three independent gates:

1. governance and technical document reviewers approve the same exact spec and
   plan bytes;
2. a fresh six-axis tooling reviewer approves the repaired sources, complete
   tests, fresh build/input/candidate evidence, and launch contract; and
3. after a successful campaign, two sidecar reviewers approve the same staged
   corpus/manifest bytes.

Attempt 8 exits only when the prior failure remains frozen, single-scan
validation passes on fixtures and the live tree, one fresh campaign passes
71/71, approved staged bytes are published once, deep approved verification
passes, and Package A0 releases the separately reviewed Task 22O adapter/engine
work.

## Inherited KSR requirements

The governing Task 22O documents remain immutable inherited subjects:

- specification: 29,472 bytes, SHA-256
  `78c44972e284eb615bf96228cbc5d0fe3a5c731a853c3b1cf518f92219b95674`;
- plan: 35,729 bytes, SHA-256
  `94c361d0d4c89eb5019f07f3a3e4101b8d89857d02c06629e3c794920f645e80`.

Attempt-8 static topology, document final, and tooling evidence bind and rehash
those exact repository identities as part of the 18-record document set.
After Package A0, tracked work still must derive every Option from the supplied
3MF; must not inspect fixture names or reference G-code; must remove obsolete
source-pinning tests in touched scope; must use real Rust modules and separate
test modules below 400 physical LOC without `include!` or `include_bytes!`
splitting; must provide no legacy fallback; and must return every final
six-axis finding to the main thread and the same reviewer until approval.
