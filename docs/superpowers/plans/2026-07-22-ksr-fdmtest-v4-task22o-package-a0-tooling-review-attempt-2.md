# Task 22O Package A0 Tooling Review Attempt 2 Plan

## Objective

Implement only the repair defined by the matching attempt-2 specification.
Preserve the two create-once attempt-1 evidence files, retain the exact first
review rejection, repair the four blocked public boundaries test-first, and
resubmit exact attempt-2 bytes to the same independent reviewer. Do not run a
formal Orca process until the full attempt-2 approval validator passes.

## Working rules

- Treat the approved coverage-repair specification, plan, document envelope,
  fixed source, Package 0, 71-role order, and seven excluded residue files as
  immutable prerequisites.
- Do not delete, overwrite, append to, rename, or substitute the unversioned
  attempt-1 ledger or mock transcript.
- Publish every new review/evidence path create-once. A rejected attempt 2
  requires a separate attempt-3 amendment.
- Document-review attempts are independently create-once: rejected reports and
  envelope stay under `document-reviews/attempt-1/`; only the revised review is
  published under `document-reviews/attempt-2/`.
- Follow RED, GREEN, refactor at each public seam. A RED must fail for the
  intended missing behavior before production edits.
- Keep every source and test ASCII and below 400 physical lines. Do not use
  `include!` or `include_bytes!` to split Rust.
- Do not edit tracked Rust production, Cargo metadata, fixed Orca source,
  Package 0, architecture, roadmap, workflow, or unrelated `main.obj`.

## A0R2.1: freeze the rejection and amendment

1. Rehash the approved parent repair documents and envelope.
2. Rehash the unversioned attempt-1 ledger and transcript at their frozen
   byte lengths and SHA-256 identities.
3. Persist the complete first six-axis response create-once at
   `attempt-1/six-axis-review.md` with one terminal `VERDICT: REJECT`.
4. Generate canonical `attempt-1/review-envelope.json` with the exact rejected
   schema, identities, four blocking IDs, seven repair IDs, six rejected axes,
   and formal authorization false.
5. Write this spec/plan pair, rehash it, and verify the exact closed-set delta:
   13 parent subjects, 88 additions, four repair documents, seven exclusions,
   seven new source/test paths, and a 58-path final source ledger.
6. Preserve both first-round document rejections and their rejected envelope at
   exact `document-reviews/attempt-1/` paths. Start two fresh read-only reviews
   of the revised frame and publish only under `document-reviews/attempt-2/`.
   Any second rejection requires attempt 3.
7. Create the canonical document approval envelope binding 31 frozen
   prerequisites and two literal approvals.

Verification: all immutable hashes match, the attempt-1 report/envelope are
create-once and internally exact, both amendment reviews say `APPROVE`, and no
source/test or formal-run path changed.

## A0R2.2: write approval-gate REDs

Add `a0_tooling_approval.py` and `test_tooling_approval.py` only after REDs at
the public validation seam cover:

- missing, extra, unsorted, duplicate, malformed, aliased, or symlinked ledger
  entries;
- wrong SHA, LOC, ASCII, path count, path set, or physical EOF;
- source mutation after ledger creation;
- absent or failed unittest, CTest, AST, no-`$args`, ASCII/LOC, forbidden macro,
  prerequisite, integration-name, or terminal result evidence;
- `FORMAL_ORCA_EXECUTED=true` and wrong ledger SHA;
- noncanonical or open envelope fields, wrong attempt/reviewer/path/root,
  removal, substitution, alias, one-byte mutation, duplicate subject, stale
  identity, missing/multiple/conflicting verdicts, `REJECT`, unresolved repair,
  non-approved axis, or post-review mutation.

Implement two APIs: pre-review validation of the exact 58-file ledger and mock
transcript, and full attempt-2 approval validation of report and envelope.
Implement a CLI used by the formal runner. Add the same full validation to
assembly, subject verification, and deep verification.

Make the approval CLI the runner's first child process before qualification
input parsing or any input-derived executable. Bootstrap it with the exact
bundled Python path/size/SHA from the specification and a frozen approval-tool
path/SHA constant, then require exact argv, working directory, canonical stdout,
empty stderr, and exit 0.

Add a formal-runner mutation test: every missing/rejected/aliased/mutated gate
fails before candidate-root creation, leaves before/after filesystem snapshots
identical, and launches no Orca/fixed-probe process. Development mocks remain
explicitly non-formal and cannot claim the formal approval result.

Verification: focused RED/GREEN tests, exact schema/path checks, and zero
mutation before the runner gate.

## A0R2.3: repair all 23 control contracts

Add `a0_control_validation.py`. Move independent control regeneration and
evidence validation out of the 395-LOC run validator. Extend the real PowerShell
generator tests and realistic qualification fixture before changing the
collector.

Required REDs include:

1. one discriminator mutation for every ordered control ID;
2. coordinated complete run-spec/definition/environment/result mutation;
3. poison missing, changed, duplicated, or moved from the final argument;
4. final/temp observed path, expected-state, and poll mutation;
5. retained file deletion, content mutation, identity mutation, or retention by
   a non-owning control;
6. unexpected final or temp output;
7. extra control, file, directory, reparse entry, or changed alias target;
8. command executable, arguments, workspace, clone, timeout, or environment
   mutation;
9. process-tree root, PID, executable, command line, ancestry, sample, cycle,
   orphan, timeout, or residue mutation; and
10. coordinated current/before/after/TSV/JSON datadir forgery or nonempty diff;
11. candidate `STARTED.json` or `8dot3-preflight.json` schema/binding/handle
    mutation; and
12. an immediate-exit process that the producer fails to retain as the root.

Implement the exact derived contract from the specification. Generic temp
absence remains for non-invalid modes only. Invalid observations are checked by
the control validator so the exact O/V temp-preexists controls pass with their
original identities while all unrelated O/V final/temp residue fails.

Make `collect_candidate_runs` consume the validator's exact closed control file
list rather than recursively binding arbitrary contents. Realistic fixtures
write full run specs, definitions, commands, process trees, environment,
observations, retained bytes, and topology. Do not weaken a runtime evidence
gate to accommodate a fixture; fix the fixture or producer.

Freshly rerun the frozen no-follow hasher and require current datadir bytes and
metadata to equal base, before, after, TSV, JSON, and artifact identities with
an empty diff. Repair `run_process.psm1` so a first-statement exit still records
the exact root member and final empty sample.

Verification: all 23 controls pass independently; every mutation fails at its
own boundary; collector binds no unclassified control file; all touched files
remain below 400 LOC.

## A0R2.4: repair public assembly

First add `test_assembly_integration.py` and a fixture in
`a0_manifest_fixture.py` that calls the real `build_outputs()` path. Record the
expected RED `UnboundLocalError` caused by using `groups` before assignment.

Move the repair constants, exact path sets, frozen-prerequisite checks,
closed-set validation, and group binding into `a0_repair_contract.py`. Keep the
assembler public API small. Assign `groups` immediately after validating the
closed assembly input and before calling group validation.

Implement the exact replacement `documents`/`tooling` fields and path orders
from the specification. Pass explicit resolved evidence/repository roots through
assembler, repair validators, CLI, subject verifier, and deep verifier; tests
must not patch globals.

The GREEN path must use a temporary exact repair root, current attempt-2
approval, a valid candidate identity/root, every group and artifact role, and
an absent exact destination. Require deterministic canonical output twice.
Invoke the CLI to publish once, then invoke it again and prove byte-for-byte no
mutation. Independently mutate candidate ID/root, groups, review approval,
artifact bindings, assembly input/destination alias, pre-existing output, and
an adjacent authored path; every case fails before publication.

Verification: successful API and CLI public paths, create-once destination,
strict mutation failures, and assembler/contract modules below 400 LOC.

## A0R2.5: exercise the complete deep path

Add a realistic reusable fixture in `a0_manifest_fixture.py` and successful
tests in `test_manifest_deep.py`. It may synthesize deterministic mock bytes,
but it may not stub production validators or insert dummy one-field evidence.

The positive path must:

1. validate a formal-shaped build result;
2. regenerate qualification inputs byte-for-byte;
3. validate all 71 ordered leaves and candidate result;
4. validate every independent control contract and exact closed topology;
5. observe all required coverage categories;
6. rebuild `corpus-v1.bin` and JSON exactly;
7. validate workspace integrity;
8. assemble the real manifest through `build_outputs()`; and
9. call `verify_manifest(..., require_approval=False, deep=True)` successfully.

Add independent mutations for build result, qualification input, candidate
identity, each run family, control evidence, coverage, corpus payload/parent,
workspace result, group/role binding, artifact identity, and physical EOF. Each
must reach and fail the intended boundary. Keep expensive fixture construction
shared without hiding assertions.

Verification: one complete positive deep traversal and mutation-sensitive
failures after each named stage.

## A0R2.6: rerun and freeze attempt 2

Run the bundled Python full verbose unittest suite, all PowerShell-backed tests,
and development fixed-probe CTest. Run Python AST, PowerShell AST/no literal
`$args`, ASCII/LOC, forbidden Rust split-macro, exact path-count, closed-set,
and 31-prerequisite checks. The exact ledger is the prior 38 tooling paths, the
seven additions, and the 13 fixed-probe source/test/input paths; it also drives
39 Python, seven PowerShell, and 58 ASCII/LOC checks.

Only after all gates pass:

1. build the exact 58-row source ledger twice in memory and require equality;
2. build the complete attempt-2 mock transcript from captured output;
3. run pre-review validation against both subjects;
4. publish both create-once under `attempt-2/`; and
5. verify no attempt-1 byte changed and no formal Orca process ran.

Verification: all results pass, transcript counts agree with the ledger, every
required integration test name is present, the two subjects validate, and no
temporary publication file remains.

## A0R2.7: same-thread six-axis revalidation

Reactivate `/root/task22o_a0r_tooling_six_axis_review`. Give it the exact
attempt-2 documents/envelope, attempt history, 58-row ledger, full transcript,
all source/tests, and fixed-probe evidence. It remains read-only and checks:

1. requirements completeness;
2. logical correctness;
3. boundary cases;
4. code quality and LOC;
5. test coverage and mutation sensitivity; and
6. actual mock execution results.

For approval, request the report format and seven `RESOLVED` lines defined by
the specification, ending in one `VERDICT: APPROVE`. Persist the response and
canonical review envelope create-once. If it rejects, preserve attempt 2 and
write an attempt-3 amendment before any further source edit.

Run full approval validation directly, through the formal-runner preflight
with a no-launch probe, through assembly, and through subject/deep verification.
Only then mark the tooling gate approved and proceed to the fresh detached Orca
build already defined by A0R.6.

## Focused commands

Use the frozen bundled Python. The minimum gates remain:

```text
bundled-python -m unittest discover -s .superpowers/sdd/task22o-oracle/voronoi-a0/tooling/tests -p "test_*.py" -v
ctest --test-dir .superpowers/sdd/task22o-oracle/voronoi-a0/fixed-probe/build -C Release -R ares22o_voronoi_fixed_probe_contract --output-on-failure
bundled-python .superpowers/sdd/task22o-oracle/voronoi-a0/tooling/a0_tooling_approval.py pre-review
bundled-python .superpowers/sdd/task22o-oracle/voronoi-a0/tooling/a0_tooling_approval.py approve
```

The exact command lines, exit codes, full outputs, static checks, ledger SHA,
and `FORMAL_ORCA_EXECUTED=false` are retained in the attempt-2 transcript.
