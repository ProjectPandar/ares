# Task 22O Package A0 Tooling Review Attempt 4 Plan

## Objective

Repair the three attempt-3 document-review blockers without touching slicing
behavior. Preserve attempt-2 and attempt-3 rejection history, approve this
document pair before source edits, prove the complete 17/31/12 public mutation
matrices and both independent runner pins, then return fresh create-once
attempt-4 evidence to the same reviewer. Formal Orca remains prohibited until
full attempt-4 approval succeeds.

## Working rules

- Treat fixed Orca, Package 0, the 71-role order, prior repair documents, and
  all eight rejection-history subjects as immutable.
- Publish every attempt-4 document and evidence destination create-once. A
  document rejection requires attempt 5.
- Permit only development-mock runner tests and fixed CTest before approval.
  Each development candidate is ephemeral and removed; `runs/qualified` is
  empty at every freeze.
- Use only the public seams named in the specification. Do not assert private
  helper call order.
- Record each intended RED before its production change, then make that slice
  green before proceeding.
- Keep every source ASCII and below 400 physical lines. Add no tracked Rust,
  Cargo, architecture, roadmap, workflow, fixed Orca, or Package 0 change.
- Never touch `main.obj`; never use `include!` or `include_bytes!` to split Rust.

## A0R4.1: freeze history and approve documents

1. Rehash and semantically validate the four attempt-2 subjects and four
   attempt-3 document-rejection subjects from the specification.
2. Confirm attempt-3 final ledger, transcript, report, and envelope do not
   exist and are not represented as history.
3. Check this spec/plan pair for ASCII/LF, one final newline, non-reparse regular
   files, and fewer than 400 physical lines.
4. Send their exact identities to
   `/root/task22o_a0r_tooling_six_axis_review` for read-only review.
5. Persist the complete response and canonical 12-key envelope at the two
   attempt-4 document-gate paths.
6. Rehash all 12 document-gate subjects. Source work starts only after exact
   `VERDICT: APPROVE` and true implementation authorization.

Verification: immutable hashes unchanged, no Orca/formal runner process,
create-once approved document gate.

## A0R4.2: extract the shared test fixture without behavior change

Create `tooling/tests/a0_tooling_approval_fixture.py` by moving the existing
`ApprovalFixture`, identity/JSON helpers, and fixture-only constants from
`test_tooling_approval.py`. Update `test_tooling_approval.py`,
`test_qualification_contract.py`, `test_manifest.py`, and
`a0_manifest_fixture.py` to import the new module directly. Do not re-export
the old location.

Extend the fixture to copy all 27 A0 and four repository frozen prerequisites,
the 12 nested-envelope targets, and exact attempt-2/attempt-3/attempt-4 history
needed by later positive fixtures. Keep literal transcript generation
independent of production parsing.

Run the existing focused approval, formal-runner, manifest, assembly, and deep
tests before adding new assertions. They must remain green; this is a mechanical
test-support extraction, not a behavior change.

## A0R4.3: RED/GREEN the exact 17-class transcript grammar

Add
`test_transcript_rejects_all_17_closed_grammar_mutations` to
`test_tooling_approval.py`. Build one literal valid transcript and exactly the
17 mutation classes in specification order. Coherently update the mutable
transcript identity needed to reach `validate_pre_review()`.
Assert the literal table has total and unique cardinality 17.

Run one subtest at a time against current production. Record a RED where the
current extraction-based validator accepts footer/arbitrary-line mutations.
Record extra recognized row, duplicate summary, and zero-test as inherited-green
controls; they remain required regression cases in the same method.

Create `tooling/a0_tooling_evidence.py`. Move the complete unittest/CTest body
validation there while leaving discovery and `ApprovalError` at the public
approval boundary. Match every physical line against the closed grammar,
independently compare discovered/result sets, and permit variability only in
documented numeric timing fields and the fixed build-root rendering.
Derive that root only from the executing validator module's `A0_ROOT`, not the
caller-supplied evidence root.

Call it from `validate_pre_review()`. Run the focused test until the literal
positive and all 17 mutations pass. Re-run existing transcript tests.

## A0R4.4: RED the complete 31/12/path matrix

Add
`test_full_approval_rejects_all_31_prerequisites_and_12_nested_identities`.
Use a fresh positive fixture for each mutation or restore every changed byte
and path before the next subtest.

First call `validate_full_approval()` for each of the 31 fixed subjects after:

1. removing that one subject; and
2. appending one byte to that one subject.

Do not update its expected frozen identity. Require 62 rejections. Record the
current acceptance of an unvalidated subject as the public full-gate RED.
The fixture supplies an independent literal case table and asserts both total
and unique cardinality are 31; it is not generated from production maps.

For each of the 12 stable nested selectors, flip one SHA nibble and serialize
the envelope canonically without changing a frozen expected identity. Call
public full approval and require a stable diagnostic naming that nested subject.
The current generic outer-hash diagnostic is the RED because it cannot prove
that the individual nested entry was validated. No private helper is called.
Assert the independent literal selector table has total and unique cardinality
12. The identity matrix therefore has exactly 74 cases.

Finally, for one A0 subject and one repository subject, exercise exact-case
mismatch, root junction alias, and parent junction/reparse substitution. Call
direct full approval for all six cases. Defer the two public-runner
representatives until A0R4.7 has finalized both source pins. Require cleaned
aliases and no residue.

## A0R4.5: GREEN the strict prerequisite and nested gate

Move the fixed 27-A0 and four-repository maps from `a0_repair_contract.py` into
`a0_tooling_evidence.py`; import them at their actual consumers without a
compatibility shim.

Implement exact-case, component-by-component `lstat` traversal. Reject root and
component aliases/reparse entries, missing/non-regular targets, and byte/hash
drift without resolving an unvalidated path.

Validate the attempt-2 document approval envelope's canonical closed schema and
fixed semantics, then rebind all 12 nested identities in stable order with the
same traversal before comparing and accepting its frozen outer identity. Keep
this implementation private to the evidence module; authorization remains at
the public full-approval boundary.

Within the strict 31-subject validator, special-dispatch that envelope to its
nested-first validation and only then compare the frozen outer identity; send
the other 30 subjects through the generic identity path. Call this validator
from `validate_full_approval()` before attempt-4 report/envelope checks. Keep
assembly, subject verification, and deep verification routed through that same
full gate. Run the focused direct test until all 62 + 12 + six both-root path
cases pass.

## A0R4.6: version approval and manifest contracts

Before versioning, add the complete
`test_formal_runner_rejects_both_pinned_sources_before_inputs_candidates_or_children`
method specified in A0R4.7. This closes real discovery at 107 IDs before the
contract requires it. Any stale approval-pin failure during setup is not the
behavioral RED and must not be recorded as one.

Update the approval contract to attempt 4:

- attempt-4 document gate, ledger, transcript, report, and envelope;
- immutable attempt-2 and attempt-3 rejected histories;
- exact 60 source, 41 Python, seven PowerShell, and 24 test-role paths;
- 11 required test IDs and at least 107 discovered tests;
- the five attempt-4 repair IDs and eight final envelope subject keys; and
- exact final approval summary with attempt 4.

Update `a0_repair_contract.py`, manifest/group fixtures, assembly, manifest
tests, and deep mutation tests. Require exactly 13 parent subjects, 98 A0
additions, eight repository documents, and seven exclusions. Bind the eight
new attempt-3/attempt-4 document roles and five attempt-4 tooling fields in
specification order. Do not create attempt-3 final tooling fields.

Require `source_roles=60`, `test_roles=24`, seven fixed evidence roles, and
`active_review_attempt=4`. Require the exact report header, axis/repair lines,
sole verdict, 13-key final envelope, and eight-key subject object before formal
authorization.

Finish every attempt-4 edit to `a0_tooling_approval.py` and
`a0_tooling_evidence.py`. Then derive the existing approval-tool path as a
sibling of `$tooling`, embed its exact final byte length and SHA-256, switch the
runner's expected stdout to the exact attempt-4 summary, and run existing
formal-runner tests green. Do not add the helper pin yet and do not edit either
pinned Python source in any later step.

Now run the complete two-target method. Require the approval-tool case to be an
inherited green control. Record the intended helper RED only when the copied
approval CLI first succeeds and the unpinned helper then starts the sentinel
child or reaches qualification-input access.

Verification: focused approval tests, assembly integration, manifest subject
tests, deep positive/mutation tests, and existing runner tests pass; only the
documented helper-pin subtest remains intentionally RED.

## A0R4.7: RED/GREEN both pre-child runner pins

Use the complete method already added in A0R4.6. Both Python source identities
are now frozen, and the sibling path makes an unmodified runner mirror exercise
the same public boundary.

For each of `a0_tooling_approval.py` and `a0_tooling_evidence.py`:

1. create a temporary A0/tooling mirror with the runner and required modules;
2. inject an ASCII, valid-AST, environment-driven sentinel write into the
   target copy without changing the runner's embedded pre-mutation pin;
3. rebuild mutable fixture ledger, transcript, report, and envelope identities;
4. invoke the mutated copied approval CLI directly and require exact approval
   plus the sentinel, then remove the sentinel;
5. invoke the unmodified copied runner with absent qualification input; and
6. require rejection with no sentinel, input-path diagnostic, candidate,
   filesystem mutation, Orca, fixed probe, or `run_one` process.

Generalize `Invoke-A0ToolingApproval` to preflight an exact two-entry table of
paths derived as siblings of `$tooling`, literal expected byte lengths, and
SHA-256 values. Require equality for both bytes and hashes before
constructing/starting the Python process. Preserve bundled Python pinning,
exact argv/working directory, empty stderr, and exact attempt-4 stdout. Keep
`run_fixed_qualification.ps1` below 400 LOC by replacing the one-tool check,
not by appending a second duplicated branch. Run both source cases green.
Require the exact source-specific identity-drift diagnostics named by the
specification.

Finally extend the 31/12 test with the deferred A0 and repository runner
topology representatives. Require the topology-specific approval diagnostic;
explicitly reject either source-pin identity-drift diagnostic. Also require no
qualification-input access, candidate creation, downstream child, filesystem
mutation, Orca, fixed probe, or `run_one` process.

## A0R4.8: static and focused verification

Build the source set as the 58 immutable attempt-2 paths plus the two specified
additions. Require 60 sorted unique paths, 41 Python, seven PowerShell, ASCII,
one to 399 physical lines each, valid Python and PowerShell ASTs, no literal
PowerShell `$args`, and a Rust-suffix-only forbidden split-macro scan.

Discover at least 107 unique tests; require all 11 IDs exactly once. Run the
three new tests individually, then all affected approval, runner, manifest,
assembly, and deep modules. Validate all 31 subjects and 12 nested identities.
Build the ledger twice from fresh reads and require identical bytes, but do not
publish yet.

## A0R4.9: fresh final run and create-once publication

1. Confirm no Orca, formal runner, `run_one`, or fixed-probe process is active.
2. Run bundled-Python full discovery and the complete unittest suite.
3. Run the fixed CTest contract once.
4. Run all static gates against the exact candidate ledger.
5. Rehash all 60 sources after tests and require no mutation.
6. Assemble the attempt-4 transcript with `FORMAL_ORCA_EXECUTED=false` and the
   exact closed grammar and counts.
7. Use the production validator module to validate proposed ledger/transcript
   in a fresh non-reparse evidence mirror; its module-derived CTest root must
   match the captured production transcript.
8. Publish both destinations create-once; run production `validate_pre_review`.
9. Rehash attempt-1, attempt-2, attempt-3, and attempt-4 document history.

Verification: no failures, errors, skips, or expected failures; fixed CTest is
exactly 1/1; production pre-review reports 60 sources and the exact test count;
no temporary publication path remains.

## A0R4.10: same-reviewer six-axis loop

Send exact attempt-4 ledger/transcript identities and all immutable history to
`/root/task22o_a0r_tooling_six_axis_review`. Require read-only review of
requirements completeness, logical correctness, boundary cases, code quality,
test coverage, and actual execution results.

Persist the complete response and canonical final envelope create-once. A
rejection envelope must derive its axes, blockers, and resolved-repair subset
exactly from the rejection report grammar; it is preserved and requires
attempt 5 before source edits. Approval is
followed by full approval directly, a formal-runner no-launch preflight,
assembly, subject verification, and deep verification on unchanged subjects.

Only after every path approves may the detached Orca build and 71 formal
qualification processes begin.
