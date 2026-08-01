# Task 22O Package A0 Tooling Review Attempt 3 Plan

## Objective

Implement only the two approval-gate repairs defined by the matching attempt-3
specification. Preserve all attempt-1 and attempt-2 evidence, approve this
document pair before source edits, close the transcript grammar, rebind all 31
frozen prerequisites without following aliases, and return fresh create-once
attempt-3 evidence to the same independent reviewer. Do not run formal Orca
until full attempt-3 approval succeeds.

## Working rules

- Treat the fixed Orca source, Package 0, 71-role order, prior repair
  documents, attempt-1 history, and all four attempt-2 subjects as immutable.
- Publish every attempt-3 document, transcript, ledger, report, and envelope
  create-once. A rejected attempt requires an attempt-4 amendment.
- Before approval, prohibit formal/input-derived Orca and formal `run_one`
  execution. Permit development-mock tests and the fixed-probe CTest needed
  for review evidence. Development candidates are ephemeral and cleaned by
  each test; require empty `runs/qualified` before evidence freeze.
- Use the public seams fixed by the specification. Do not test private helper
  call order.
- Work one RED/GREEN slice at a time. Record the intended failure before each
  production change.
- Keep each source and test ASCII and below 400 physical lines.
- Add no tracked Rust, Cargo, architecture, roadmap, workflow, fixed Orca, or
  Package 0 changes. Never touch unrelated `main.obj`.
- Do not use `include!` or `include_bytes!` for Rust source splitting.

## A0R3.1: freeze rejection history and approve documents

1. Verify attempt-2 ledger, transcript, report, and rejected envelope at the
   exact identities in the specification.
2. Verify the rejected envelope is canonical and its blocker IDs, axes,
   resolved repairs, subject set, state, verdict, and false formal flag match
   the same reviewer's clarification.
3. Write this spec/plan pair without changing any ledger-bound source.
4. Check both files are ASCII/LF, have one final newline, and are below 400
   physical lines.
5. Send both exact identities to
   `/root/task22o_a0r_tooling_six_axis_review` for read-only document
   review.
6. Persist the complete response at
   `attempt-3/document-review.md` and its canonical detached envelope at
   `attempt-3/document-approval-envelope.json`.
7. Rehash all six subjects. Source work is authorized only when the report has
   one final `VERDICT: APPROVE` and the envelope validates.

Verification: no source/test hash changes, no Orca process, both document
identities bound, document approval create-once.

## A0R3.2: RED for closed transcript output

Add
`tooling/tests/test_tooling_evidence.py` with
`test_transcript_requires_closed_success_grammar`. Construct each mutation
from the positive `ApprovalFixture`, update any outer ledger/transcript
identity needed to reach the public parser, and call
`validate_pre_review()`.

Run one subtest at a time and require the current implementation to accept the
first forbidden mutation before production edits. Cover:

1. multi-token skipped;
2. expected failure;
3. `FAIL` result;
4. `ERROR` result;
5. traceback block;
6. `FAILED (failures=1)` summary;
7. duplicate outcome;
8. conflicting `OK`/`FAILED` terminals;
9. wrong method name;
10. arbitrary unparsed result-like line;
11. arbitrary terminal line;
12. CTest error footer;
13. CTest failed-test footer; and
14. arbitrary extra CTest line.

Verification: focused test is RED for the intended acceptance bug, not fixture
setup or a stale identity.

## A0R3.3: GREEN for closed transcript output

Create `tooling/a0_tooling_evidence.py`. Move the complete unittest and
CTest body validation there. Leave test discovery and the public error type at
the approval API boundary.

The unittest validator compares every physical body line against the exact
result-plus-terminal grammar. It independently checks the result set,
cardinality, method/ID agreement, required IDs, numeric duration, and sole
terminal `OK`.

The CTest validator compares against the exact build-root-derived grammar and
permits only numeric timing fields. It rejects every other line.

Call the helper from `validate_pre_review()` and preserve the existing
canonical section framing. Update `ApprovalFixture` to emit the literal valid
grammar independently.

Verification:

```powershell
python -m unittest -v test_tooling_evidence.ToolingEvidenceTests.test_transcript_requires_closed_success_grammar
```

All fourteen mutation classes and the positive fixture must pass.

## A0R3.4: RED for the 31-subject full gate

Add
`test_full_approval_and_runner_reject_prerequisite_mutations_before_inputs`
through the two public boundaries.

Populate the positive fixture with exact copies of all 27 A0 and four
repository prerequisite subjects. For each direct full-approval mutation,
rebuild only the outer attempt-3 approval identity necessary to reach the
prerequisite validator; never recompute the expected frozen subject identity.

Exercise:

1. missing repository document;
2. one-byte repository document mutation;
3. wrong-case repository path;
4. repository-root junction/symlink alias;
5. parent-directory junction/symlink/reparse;
6. file symlink/reparse; and
7. stale or wrong nested document-envelope identity.

For the formal runner, use the same cases and an intentionally absent
qualification-input path. Snapshot the fixture before invocation. Require
nonzero exit, tooling-approval diagnostic, no qualification-input path in the
input-access diagnostic, absent candidate root, byte-identical snapshot, and
no Orca/fixed-probe process.

Verification: the current full approval or runner accepts at least the
reviewer's missing/mutated-document case before production edits.

## A0R3.5: GREEN for no-follow prerequisite validation

Move the fixed 27-A0/four-repository maps from
`a0_repair_contract.py` into `a0_tooling_evidence.py` and re-export or
import them where the existing repair contract needs them.

Implement literal component traversal using directory enumeration and
`lstat`. Reject symlinks, junctions, reparse attributes, aliases, wrong
case, missing components, and non-regular final subjects before reading bytes.
Do not call `resolve()` on an unvalidated subject.

Parse the frozen attempt-2 document approval envelope canonically. Validate
its closed schema and fixed semantic fields, then rebind all 12 nested
identities through the literal traversal.

Invoke this strict 31-subject check from `validate_full_approval()` before
report/envelope authorization. Keep assembly and deep verification on the same
function.

Verification: the direct full-approval half of the focused test is GREEN.

## A0R3.6: pin both pre-approval Python subjects

Update `run_fixed_qualification.ps1` so the first child remains the bundled
Python approval CLI, but preflight freezes and rehashes both
`a0_tooling_approval.py` and `a0_tooling_evidence.py`. Pass exact
argv and working directory, require the canonical attempt-3 approval summary,
and require empty stderr.

Do not resolve qualification inputs or create the candidate until both hashes
and the full approval CLI succeed.

Verification: the runner half of the focused prerequisite test is GREEN for
all mutation classes and preserves the filesystem snapshot.

## A0R3.7: version approval and manifest contracts

Update the approval public contract from attempt 2 to attempt 3:

- attempt-3 ledger, transcript, report, and envelope paths;
- the exact attempt-2 rejected-history validator;
- the approved attempt-3 document gate;
- 60 source paths, 41 Python paths, seven PowerShell paths;
- ten required test IDs and at least 106 discovered tests;
- the four attempt-3 repair IDs;
- the attempt-3 approval summary and envelope subjects.

Update `a0_repair_contract.py`, group bindings, fixtures, assembly, manifest
tests, and static transcript expectations so
`active_review_attempt=3` and all attempt-3 evidence paths are closed-set
members. Preserve the exact attempt-1 and attempt-2 roles as immutable history.

Require exact closed-set counts of 13 parent subjects, 96 A0 repair additions,
six repository repair documents, and seven exclusions. Add only the four
`tooling_attempt_3_*` document fields and five `attempt_3_*` tooling
fields named by the specification. Bind them in the specified order, require
60 source roles, 24 test roles, seven attempt-3 fixed-evidence roles, and leave
every prior role present.

Require the exact five-line final-review header and the reused 13-key
review-envelope schema with the seven exact attempt-3 subject keys before any
approval can authorize formal execution.

Verification:

1. focused approval tests;
2. focused formal-runner tests;
3. assembly/integration tests;
4. manifest subject tests; and
5. deep positive/mutation tests.

## A0R3.8: static and focused verification

Build the exact source-path set as the 58 immutable attempt-2 ledger paths plus
the two specified additions. Before a full run, require:

- 60 sorted unique paths;
- 41 Python and seven PowerShell paths;
- every file ASCII and below 400 physical lines;
- all Python ASTs parse;
- all PowerShell ASTs parse and contain no literal `$args` token;
- Rust-suffix-only forbidden split-macro scan passes;
- all ten required tests are discovered exactly once; and
- all 31 prerequisites and 12 nested identities pass strict validation.

Verification: a source ledger built twice from fresh reads is byte-identical.
Do not publish it yet.

## A0R3.9: fresh final run and create-once publication

1. Recheck that no Orca, formal runner, `run_one`, or fixed-probe process is
   running.
2. Run bundled-Python discovery and the complete unittest suite.
3. Run the fixed CTest contract once.
4. Run all static gates against the exact 60-path candidate ledger.
5. Rehash all 60 sources after the run and require no mutation.
6. Assemble the complete attempt-3 transcript with
   `FORMAL_ORCA_EXECUTED=false` and the exact closed section grammar.
7. Validate proposed ledger/transcript bytes in a fresh non-reparse temporary
   mirror.
8. Publish both destinations create-once, then run production
   `validate_pre_review()`.
9. Rehash attempt-1, attempt-2, and document-gate history.

Verification: complete suite has no failures, errors, skips, or expected
failures; CTest is exactly 1/1; production pre-review reports 60 sources and
the exact discovered test count; no temporary publication file remains.

## A0R3.10: same-reviewer six-axis loop

Send the exact attempt-3 ledger and transcript identities plus all immutable
history to `/root/task22o_a0r_tooling_six_axis_review`. Require read-only
review of:

1. requirements completeness;
2. logical correctness;
3. boundary cases;
4. code quality and LOC;
5. test coverage and mutation sensitivity; and
6. actual execution results.

Persist the complete response and canonical envelope create-once. If the
review rejects, preserve attempt 3 and write an attempt-4 amendment before any
source edit. If it approves, run full approval directly, through a formal
runner no-launch preflight, through assembly, through subject verification,
and through deep verification.

Only after every path approves may the fresh detached Orca build and 71 formal
qualification processes begin.
