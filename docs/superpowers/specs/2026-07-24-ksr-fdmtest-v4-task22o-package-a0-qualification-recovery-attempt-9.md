# Task 22O Package A0 Qualification Recovery Attempt 9 Specification

## Status

Proposed, revision 2. Before two independent document reviewers approve the
same exact specification and plan bytes, only create-once document-review
artifacts are authorized. Attempt-9 tooling source edits, failure-handoff
publication, fresh build/input preparation, tooling-review evidence, launch
approval, claim creation, and formal processes remain forbidden.

One document reviewer owns governance and requirement completeness. The other
owns technical correctness and executable verification. Review feedback is
iterative: each reviewed document pair and both reports are retained in a new
consecutive create-once iteration. A registry and final document envelope are
published only after the latest iteration receives two `APPROVE` verdicts.

Document iteration `0001` binds revision-1 spec
`9c1c37bf310b848b31b0bd3748b28b9acfaad83d5e6bbe073e92fa8f36d9f7d0`
and plan
`aa8a75f4d537cb391d8dec4242ebb953483f2c8316691bec5f1aa48021228245`,
plus governance and technical `REVISE` reports. Iteration `0002` binds this
revision-2 pair and its two new reports. Both five-file iterations remain
immutable.

## Purpose

Attempt 8 executed its one authorized campaign through all 71 roles, but its
candidate is not publication-eligible. The strict postcampaign collector found
that the standalone datadir JSON and the same data embedded in each CLI
`result.json` were not byte-semantically identical after JSON parsing:

```text
standalone earliest_last_write_utc = 2026-07-06T14:20:20.0000000Z
embedded   earliest_last_write_utc = 2026-07-06T14:20:20Z
```

The first rejection is ordinal `00001`, role `env-off`, with
`datadir JSON file/result drift`. The same representation drift is present in
all 69 CLI leaves. The two direct fixed-probe roles correctly have no datadir
evidence.

The source hash-tree record is correct. The central PowerShell evidence reader
uses default `ConvertFrom-Json`, which converts the ISO string to
`System.DateTime`; later `ConvertTo-Json` removes the seven zero fractional
digits. The collector's exact equality check is correct and must not be
relaxed. Attempt 9 repairs only the producer-side string preservation,
test-first, then authorizes one wholly fresh qualification campaign.

This remains ignored Package A0 oracle-tooling work. It does not change Ares
production behavior, fixed Orca behavior, the 11-case matrix, the 23 invalid
controls, the 69 CLI plus two direct role order, or expected KSR G-code bytes.

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
Attempt 9 changes only ignored qualification tooling and evidence.

## Consumed Attempt-8 campaign

The following subjects are immutable:

- formal transcript
  `C:\a22or13-driver\attempt8-campaign-formal-run-v1.txt`, zero bytes,
  SHA-256
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`;
- claim
  `coverage-repair/tooling-review/attempt-8/launch-claim-v1.json`,
  989 bytes, SHA-256
  `84825676dee4c7637870ae1113f0be9dc23d88fdba6236bb48e061af5b9c5a46`;
- candidate `11d07a6fd071c211` at exact root
  `C:\Users\Indexyz\Projects\Ares\.superpowers\sdd\task22o-oracle\voronoi-a0\runs\qualified\11d07a6fd071c211`;
- candidate `STARTED.json`, SHA-256
  `d8a0f60584c843ddb3e95bc2fe686854697aad1d27110c4274beac446b623634`;
- candidate `result.json`, SHA-256
  `e8e94b392a99a0338f3f6710e328c5837e7bc0289354886919e03aa04be0ead9`;
- terminal tooling final
  `coverage-repair/tooling-review/attempt-8/tooling-reviews/final-envelope.json`,
  SHA-256
  `38ef08936f25d5d54c28ec22cd7c63c0129784583c61aed98ebb2e7d763e0d21`;
- launch approval, SHA-256
  `09c23db0022b4b353ae7ba84c28e2504e0bcc4ed4a4b67e3f04c8879768644d6`;
- prelaunch evidence, SHA-256
  `154f5142dcf033a8fc0b7ac099a404f8572f3b72b7206a3277d7fe6661960aa6`;
- generated-scope watchdog
  `C:\a22or13-driver\attempt8-generated-scope-watchdog-v1.txt`,
  11,579 bytes, SHA-256
  `42d252108006948197ef27ef6cf9e91c0e58f775e7eb3d4b5562da9d795ba142`;
- build result, SHA-256
  `ba58c34673f0e16c4c28e2ed34aede5c7f89b4fc3f31f835ff2bc04699bbaa18`;
  and
- qualification inputs, SHA-256
  `97be6fefd917b569bc9826406f5944c588c1dfab85b00c86287e1e262776a2eb`.

The result reports 71 passed processes, 69 CLI roles, two direct roles, no
retry, and no selected old run. Exactly 71 role specifications and 71 run
directories exist, with no `FAILED.json`. These execution facts do not
override the failed evidence contract.

The qualified parent contains exactly the retained failed-history children.
Their canonical logical order is retained-attempt order:

```text
8a5aab7461b877b1
11d07a6fd071c211
```

Physical directory membership is compared separately as a lexically sorted
set, which is `["11d07a6fd071c211","8a5aab7461b877b1"]`. The two order
contracts are never conflated.

No whole-candidate-tree SHA-256 was computed before this recovery. The
zero-byte formal transcript cannot prove the wrapper's final return code and
cannot contain the collector traceback. Attempt 9 must not invent either fact.
It relies on independently rehashed immutable leaves, the bound root/results,
and a fresh read-only reproduction of the exact collector rejection.

Attempt 8 is consumed. Never invoke its formal wrapper, production runner, or
campaign again. Never overwrite its transcript, claim, candidate, run leaves,
review evidence, launch subjects, build/input evidence, or watchdog evidence.
Never normalize or repair the 69 embedded timestamp strings. Never create an
Attempt-8 replacement claim, candidate, transcript, retry marker, selected-run
marker, stage, publication review, or final sidecar.

## Attempt-9 failure handoff

After document approval and before source repair, publish create-once:

```text
coverage-repair/tooling-review/attempt-9/prior-attempt/attempt8-runner.ps1
coverage-repair/tooling-review/attempt-9/prior-attempt-v1.json
```

The runner copy has independent rooted identity. The prior-attempt record binds
and rehashes:

- the Attempt-8 formal transcript, claim, launch approval, watchdog transcript,
  build result, and qualification inputs;
- the immutable Attempt-8 runner copy and the terminal Attempt-8 source-ledger
  row for `tooling/run_fixed_qualification.ps1`;
- candidate `STARTED.json`, `result.json`, all 71 role specs, and all 71 run
  roots;
- exact parent children `["8a5aab7461b877b1","11d07a6fd071c211"]`;
- the first failing ordinal, role, field, standalone value, embedded value, and
  exact collector error;
- the all-69 CLI blast radius and the two direct-role absence contract;
- the successful underlying execution counts and absent retry/selection facts;
  and
- an empty matching-process set plus no recursive `__pycache__`, `.pyc`, or
  `.pyo` residue, captured immediately before handoff publication.

The public Attempt-9 validator independently rehashes these subjects and
recomputes the mismatch from the immutable leaf evidence. The `prior_attempt`
launch subject transitively binds a complete no-follow snapshot of the entire
Attempt-8 candidate tree, so no twelfth launch subject is permitted. It also
preserves the nested immutable Attempt-7-to-Attempt-8 handoff. The validator
rejects a changed or aliased root/path pair, a different logical order or
physical parent set, a mutated Attempt-8 leaf, or a collector relaxation.
Later source edits cannot change the copied runner or canonical prior-attempt
record.

## Test-driven producer repair

Keep exactly 69 source roles, 28 test-role paths, 122 discovered test IDs, and
26 required test IDs. Extend an existing test method; do not add a new test ID.

The RED phase extends the existing successful run-one lifecycle test. Its real
datadir fixture must contain a file whose last-write time is exactly on a
second, producing an `earliest_last_write_utc` value ending in
`.0000000Z`. Through the real PowerShell leaf runner, assert:

- `datadir-before.json` parsed JSON exactly equals
  `result.json["datadir_before"]`;
- `datadir-after.json` parsed JSON exactly equals
  `result.json["datadir_after"]`;
- `datadir-diff.json` parsed JSON exactly equals
  `result.json["datadir_diff"]`; and
- the before and after embedded timestamps retain the exact seven fractional
  zero digits.

The focused test must fail on the frozen pre-repair source because the embedded
before/after values end in `Z` without `.0000000`. Retain the RED transcript.
It may not mock the JSON reader, bypass the real leaf runner, or weaken the
collector.

The GREEN production repair is limited to the central PowerShell JSON reader:
parse evidence JSON with `ConvertFrom-Json -DateKind String`, supported by the
launch-bound PowerShell 7.6.4 runtime. No validator, hash-tree producer,
timestamp formatter, result schema, or expected value changes.

After GREEN, run the strict candidate collector against a fresh small
filesystem fixture and prove the standalone and embedded objects compare
equal. Run all 122 tests and every static check. Every edited source/test file
must be ASCII and below 400 physical lines. Rust source splitting with
`include!` or `include_bytes!` remains forbidden.

The tooling-review source ledger and transcript bind the exact focused RED
transcript, focused GREEN transcript, and a provenance record naming the
pre-repair source identity, failing assertion, repaired source identity, and
passing assertion. These subjects are required in the first tooling-review
iteration and every later iteration; they are not temporary logs.

## Attempt-9 current-attempt adaptation

All current-attempt source and fixture adaptations occur after the authentic
RED and before complete verification:

- Python and PowerShell candidate domains use a new framed Attempt-9 domain;
- current schema kinds, attempt numbers, runner messages, literal review,
  launch, claim, and publication paths say Attempt 9;
- historical Attempt-7 and Attempt-8 paths, kinds, numbers, claims, candidates,
  and evidence identities remain unchanged;
- prior-attempt validation accepts only the canonical Attempt-8 failure
  handoff;
- launch validation requires exactly eleven subjects; and
- static topology names exactly 135 A0 paths, 69 source paths, 28 test paths,
  122 discovered tests, 26 required tests, 31 legacy frozen prerequisites, and
  20 repository documents.

The 20 documents are the frozen Attempt-8 set plus exactly:

```text
docs/superpowers/specs/2026-07-24-ksr-fdmtest-v4-task22o-package-a0-qualification-recovery-attempt-9.md
docs/superpowers/plans/2026-07-24-ksr-fdmtest-v4-task22o-package-a0-qualification-recovery-attempt-9.md
```

The Attempt-9 launch envelope has exactly:

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

`prior_attempt` binds the Attempt-8 handoff. The other roles bind fresh
current-attempt review/build identities or inherited immutable prerequisites.
Any source, test, CMake, runner, candidate-domain, build-result,
qualification-input, runtime-closure, policy, topology, or document-final
change invalidates all prepared current roots and requires new roots before
launch.

Candidate-parent validation has two exact states. Before claim/candidate
creation, the logical binding is
`["8a5aab7461b877b1","11d07a6fd071c211"]`. After the atomic claim and candidate
creation, it is
`["8a5aab7461b877b1","11d07a6fd071c211","<attempt9_candidate_id>"]`.
In both states, physical membership equals the lexically sorted set of the
applicable logical binding. Python validation and
`run_fixed_qualification.ps1` enforce both order and set semantics; tests
reject missing, extra, reordered logical, or physically aliased children.
Current document iteration count is always read from and matched to the
Attempt-9 document registry, never copied from Attempt 8.

## Fresh A22OR14 build and launch

Use new literal, ordinary, canonical, pairwise tree-disjoint roots under the
A22OR14 family for source, build, install, and evidence. They must not reuse
A22OR8 through A22OR13 or any Attempt-6 root. Recreate and independently
validate:

- fixed source commit/tree and patch identities;
- configure, build, install, CTest RED/GREEN, and child-environment evidence;
- all four adjacent-DLL runtime closures;
- build-result v2 and deterministic qualification-inputs v2;
- Python/PowerShell agreement on the fresh Attempt-9 candidate ID; and
- candidate absence with the qualified parent containing exactly the two
  retained failed-history children in the prelaunch logical and physical
  contracts.

After a six-axis tooling reviewer, distinct from both document reviewers,
approves the exact final source ledger, RED/GREEN and 122-test transcripts,
fresh build/input evidence, prior-attempt handoff, static topology,
publication policy, and launch contract, publish the sole Attempt-9 launch
approval create-once.

Before formal invocation, run the live `generated-scope` validator under the
same independent 180-second non-formal watchdog contract. It must use bundled
Python with `-B`, `PYTHONDONTWRITEBYTECODE=1`, the bound tooling working
directory, and a PATH limited to bound PowerShell and Git directories. It must
not invoke the production runner or create a claim/candidate. Timeout handling
recursively terminates and waits for the process tree. Require no matching
process or cache residue before and after.

The sole formal transcript target is:

```text
C:\a22or14-driver\attempt9-campaign-formal-run-v1.txt
```

Immediately before invocation require the transcript, Attempt-9 claim, fresh
candidate, and repository-root runner temporaries to be absent. Invoke the
formal wrapper exactly once and execute all 71 roles from ordinal 1 without
resume, retry, overwrite, or old-leaf reuse.

The runner atomically creates the claim before the candidate and requires the
post-claim logical binding to append exactly the computed Attempt-9 candidate
ID. Every subsequent validator requires that logical order and the equal sorted
physical set.

Immediately after successful execution, run the strict existing collector over
the fresh candidate before any stage is created. It must validate every leaf,
including standalone/embedded datadir equality for all 69 CLI roles, and return
the complete ordered corpus. A collector rejection consumes Attempt 9. Do not
repair or replace Attempt-9 evidence in place; recovery would require a
separately reviewed Attempt 10.

## Postcampaign publication

Only a fresh candidate that passes both 71/71 execution and the strict
collector may enter the existing stage/review/publish state machine below the
Attempt-9 publication root. Two independent sidecar reviewers inspect identical
staged corpus/manifest bytes. They differ from each other and from the tooling
reviewer.

Publish final corpus, manifest, reports, registry, and final sidecar envelope
create-once only after dual approval. Make the stage unavailable and run deep
approved verification proving all final identities are final-root-relative.
Any sidecar `REVISE` or `REJECT`, reviewer disagreement, or launch-bound byte
drift retains the stage/reports immutably, consumes Attempt 9, forbids further
Attempt-9 stage/publication/final-sidecar creation, and requires separately
reviewed Attempt 10. Final paths are never overwritten.

## Approval gates and exit

Attempt 9 has three independent gates:

1. governance and technical document reviewers approve the same spec/plan;
2. a distinct six-axis tooling reviewer approves repaired source, full tests,
   fresh build/input/candidate evidence, and launch contract; and
3. two distinct sidecar reviewers approve the same staged corpus/manifest.

Attempt 9 exits only when the Attempt-8 candidate remains frozen, exact JSON
string preservation passes the real public seam, one fresh campaign passes all
71 roles and strict collection, approved staged bytes are published once, deep
verification passes, and Package A0 releases the separately reviewed Task 22O
adapter/engine work.

After tracked Rust work reaches exact KSR output, one reviewer independent of
the implementing main thread reviews requirement completeness, logical
correctness, boundary cases, code quality, test coverage, and actual execution
results. It returns an ordered fix list to the main thread. The main thread
repairs findings and the same reviewer identity revalidates repeatedly until
all six axes approve or a precise blocker is recorded.

## Inherited KSR requirements

The governing Task 22O specification remains 29,472 bytes with SHA-256
`78c44972e284eb615bf96228cbc5d0fe3a5c731a853c3b1cf518f92219b95674`.
The governing plan remains 35,729 bytes with SHA-256
`94c361d0d4c89eb5019f07f3a3e4101b8d89857d02c06629e3c794920f645e80`.

After Package A0, tracked implementation still must derive every Option only
from the supplied 3MF; must not inspect fixture names or reference G-code; must
remove obsolete source-pinning tests in touched scope; must use real Rust
modules and separate test modules below 400 physical lines without `include!`
or `include_bytes!` splitting; must provide no legacy fallback; and must return
every final six-axis finding to the main thread and the same reviewer until
approval.
