# Task 22O Package A0 Tooling Review Attempt 4 Specification

## Status and scope
This amendment repairs only the three implementation-plan blockers returned by
the independent attempt-3 document gate. It supplements the approved attempt-2
specification and plan and the rejected attempt-3 document pair; it does not
rewrite their history. Attempt 3 made no source or test edit and published no
ledger, transcript, six-axis report, or final review envelope.

The amendment changes ignored Package A0 approval tooling, tests, manifests,
mock fixtures, and create-once review evidence only. It does not change tracked
Rust production code, Cargo metadata, the fixed Orca derivative, Package 0, the
71-role order, architecture, roadmap, workflows, or `main.obj`. No formal Orca
build, input-derived qualification, or formal `run_one` may start until attempt
4 has independent six-axis approval and its full approval CLI passes at the
formal-runner boundary. Development-mock runner tests and the fixed-probe CTest
contract are not formal qualification.

The upstream boundary remains OrcaSlicer tag `v2.4.2`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, specifically
`src/libslic3r/PerimeterGenerator.cpp`,
`PerimeterGenerator::process_classic()` at fixed-source lines 1144-1692, and
the fixed Voronoi dependencies named by Package A0. This amendment adds no
Ares-owned slicing behavior and changes no upstream source boundary.

## Immutable rejection history
The following eight create-once subjects must never be deleted, renamed,
appended to, overwritten, or substituted.

| Subject | Bytes | SHA-256 |
| --- | ---: | --- |
| `coverage-repair/tooling-review/attempt-2/source-files.sha256` | 5,978 | `47baf70c599daf794b92857bc2404acb5433436d263da6ea3d8cb4d3203872b4` |
| `coverage-repair/tooling-review/attempt-2/mock-test-results.txt` | 19,242 | `344708cdedad36ec6b5e6d3d463ada7e9b5587a4e40c0b996ae4f32753b97e77` |
| `coverage-repair/tooling-review/attempt-2/six-axis-review.md` | 6,116 | `af8f870ac829a960411dbcc066e34af785a96e80db7dea824d71d1a562ce76ca` |
| `coverage-repair/tooling-review/attempt-2/review-envelope.json` | 4,077 | `278fca5cedd3ec961edbd53da572ea97b53581222457a1e8c0ecf5afb47e5b16` |
| `docs/superpowers/specs/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-3.md` | 15,922 | `1b720c6a4737c046bf27fbc866b5a53b97f5da2e2d9ca65900fc484b842f00e8` |
| `docs/superpowers/plans/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-3.md` | 11,303 | `3bc53640cb0aad2a570a95319da846d193a6feb66db36ba0e7d85542795153bc` |
| `coverage-repair/tooling-review/attempt-3/document-review.md` | 5,091 | `795a80cc2ef6ba1381c6c7a848d4b2a51e57d8c374736e984aa39b75809c6fbf` |
| `coverage-repair/tooling-review/attempt-3/document-approval-envelope.json` | 1,608 | `73abd4108ac0fe52e4fecc7841312728e6d0bde7a50e01a7e45173889ae3faea` |

The attempt-3 document envelope is canonical, rejected, binds the exact
attempt-3 documents and review, has false implementation and formal
authorization, and ends the attempt-3 document gate. The three attempt-3
blockers are named here for the attempt-4 machine contract, in reviewer order:

```text
incomplete-ctest-red-matrix
missing-independent-helper-pin-red
incomplete-31-and-12-mutation-proof
```
The repairs required by that report are exact: expand the transcript mutation
matrix from 14 to 17, prove both runner source pins independently before any
child or input access, and make every one of the 31 frozen subjects and 12
nested identities observable through public validation seams.

## Attempt-4 document gate
This specification and its matching plan must receive read-only approval from
the same reviewer before any source or test edit. The complete ASCII/LF response
is published create-once at:
```text
coverage-repair/tooling-review/attempt-4/document-review.md
```
Its exact leading semantic lines are:
```text
TASK22O A0 TOOLING REVIEW ATTEMPT 4 DOCUMENT REVIEW
REVIEWER: /root/task22o_a0r_tooling_six_axis_review
SPEC_SHA256: <exact specification SHA-256>
PLAN_SHA256: <exact implementation-plan SHA-256>
DOCUMENT: specification: APPROVE
DOCUMENT: implementation_plan: APPROVE
CHECK: immutable_attempt_2_and_3_history: APPROVE
CHECK: public_tdd_seams: APPROVE
CHECK: exact_17_mutation_matrix: APPROVE
CHECK: exact_31_and_12_matrix: APPROVE
CHECK: dual_runner_pin_red: APPROVE
VERDICT: APPROVE
```

Supporting proof may occur before the verdict, but no other line begins
`DOCUMENT:`, `CHECK:`, or `VERDICT:`. The sole final nonempty line is
`VERDICT: APPROVE`. A rejection uses the same header/hashes, exactly two
`DOCUMENT:` and five `CHECK:` lines with values `APPROVE` or `REJECT`, at least
one `REJECT`, no duplicate semantic lines, and sole final `VERDICT: REJECT`.
A canonical create-once envelope at:
```text
coverage-repair/tooling-review/attempt-4/document-approval-envelope.json
```
has exactly `schema_version`, `kind`, `state`, `reviewer`, `documents`,
`review_report`, `immutable_attempt_2`, `immutable_attempt_3`,
`subjects_mutated_after_review`, `implementation_authorized`,
`formal_orca_execution_authorized`, and `verdict`.

It uses `schema_version=1`,
`kind=task22o-a0-tooling-review-document-approval`, `state=approved`, the
exact reviewer identity, no post-review mutation, true implementation
authorization, false formal authorization, and `verdict=APPROVE`. `documents`
has exactly `specification` and `implementation_plan`; `review_report` binds
the attempt-4 document review. `immutable_attempt_2` has exactly
`source_ledger`, `mock_test_results`, `review_report`, and `review_envelope`;
`immutable_attempt_3` has exactly `specification`, `implementation_plan`,
`document_review`, and `document_approval_envelope`. Every identity has exactly
`root`, `path`, `bytes`, and `sha256`. JSON is ASCII, sorted-key, two-space
indented, and has one final newline. Rejection retains the same schema with
`state=rejected`, `subjects_mutated_after_review=false`, false authorizations,
and `verdict=REJECT`, consumes both
attempt-4 document paths, and requires attempt 5.

## Public seams and TDD
The pre-agreed observable seams are:

1. `a0_tooling_approval.validate_pre_review()` and its `pre-review` CLI;
2. `a0_tooling_approval.validate_full_approval()` and its `approve` CLI;
3. `run_fixed_qualification.ps1` before any child launch, qualification-input
   access, or candidate creation; and
4. existing assembly, subject-verification, and deep-verification entry points
   that call the full approval gate.
Every behavior change begins with a failing test through one of these seams.
Filesystem setup may be factored into a dedicated test fixture but tests do not
import or call private production validators. Each intended acceptance or
launch bug is observed before production changes, then made green.

## Exact closed transcript grammar
The valid unittest body is exactly one `ok` result line for every independently
discovered unique ID, followed by the standard separator, exact numeric count,
nonnegative numeric duration, and sole terminal `OK`, with only the standard
two blank lines. Result method names equal the final ID components. No line is
extracted and ignored.

The CTest body is exactly the fixed `Internal ctest changing into directory`
and `Test project` lines derived from the build root, one `Start 1` line, one
1/1 passed row for
`ares22o_voronoi_fixed_probe_contract`, one blank, the exact 100-percent and
zero-failed summary, one blank, and one numeric total-time line. No other line
is accepted. The build root is `A0_ROOT/fixed-probe/build`, where `A0_ROOT` is
derived from the executing validator module, never caller-supplied
`evidence_root`. A production transcript therefore remains valid when the
production validator checks mirrored evidence.

The literal positive fixture is independent of the production parser. The
single public `validate_pre_review()` test contains exactly these 17 mutation
classes:

1. multi-token skipped result;
2. expected-failure result;
3. `FAIL` result;
4. `ERROR` result;
5. traceback block;
6. `FAILED (failures=1)` terminal;
7. duplicate outcome;
8. conflicting `OK` and `FAILED` terminals;
9. wrong method name;
10. arbitrary unparsed result-like line;
11. arbitrary terminal line;
12. `Errors while running CTest` footer;
13. `The following tests FAILED:` footer;
14. arbitrary extra CTest line;
15. an extra recognized `Start` plus result row;
16. a duplicate success summary; and
17. zero-test output including `No tests were found`.
Each mutation reaches transcript parsing with coherent outer fixture identities
and must fail. The literal table asserts total and unique cardinality 17. Cases
15-17 are inherited-green controls in current production; the method-level RED
comes from currently accepted footer/arbitrary-line cases. The positive literal
transcript must pass.

## Exact 31-subject and 12-identity proof
The original frozen prerequisite set remains exactly the 27 A0 subjects and
four repository documents already defined by the attempt-2 repair contract.
Every full approval call traverses and rehashes all 31 before report/envelope
authorization. Traversal is literal, exact-case, component-by-component, and
uses `lstat`; it rejects aliases, symlinks, junctions, mount-point/reparse
entries, missing or non-regular subjects, and wrong-case components without
calling `resolve()` on an unvalidated subject.

The public full-approval test has, for every one of the 31 subjects, one missing
case and one one-byte mutation case. It restores or recreates a positive fixture
between subtests and never recomputes the immutable expected identity. This is
exactly 62 direct full-approval rejection cases.

The fixture owns an independent literal `(root, path)` table and asserts that
its cardinality and unique cardinality are both 31 before the matrix runs. It
does not derive its cases from the production maps under test.

The attempt-2 document approval envelope contains these 12 nested identity
entries in stable order:

```text
immutable_parent_repair.document_approval_envelope
immutable_parent_repair.implementation_plan
immutable_parent_repair.specification
immutable_tooling_attempt_1.mock_test_results
immutable_tooling_attempt_1.review_envelope
immutable_tooling_attempt_1.review_report
immutable_tooling_attempt_1.source_ledger
reviewed_documents.implementation_plan
reviewed_documents.specification
reviews[0].subject
reviews[1].subject
supersedes_document_review
```

For each entry, the test flips one SHA-256 nibble, serializes the envelope
canonically, and calls public full approval without changing any frozen expected
identity. Full approval parses and rebinds these entries before accepting the
outer envelope. All 12 cases must fail with a stable public diagnostic naming
the nested subject, not only a generic outer-hash error.

The 31-subject loop special-dispatches this envelope: it validates all 12
nested identities first, then compares its frozen outer identity. The other 30
subjects use the generic exact identity path.

The nested selectors are an independent literal table whose cardinality and
unique cardinality are both asserted as 12. Together with the 62 subject cases,
this is exactly 74 identity mutations plus six both-root topology cases.

Separate path cases use representative A0 and repository subjects. Each root
must reject an exact-case mismatch, a root junction alias, and a parent
junction/reparse substitution. Direct full approval and a representative formal
runner invocation must reject before input access or candidate creation. Tests
restore their fixture and leave no alias or candidate residue. Runner topology
cases execute only after both final source pins are current and must not accept
a source-pin identity-drift diagnostic as topology evidence.

## Independent formal-runner pins
Before its first child process, the formal runner computes and compares the
literal path, exact embedded byte length, and SHA-256 of both:

```text
tooling/a0_tooling_approval.py
tooling/a0_tooling_evidence.py
```

Only then may it launch bundled Python 3.12.13 as the first child and require
the exact attempt-4 approval summary on stdout, empty stderr, and exit zero.
It does not resolve or read qualification inputs or create a candidate first.

The public runner test uses an unmodified runner in a temporary A0 mirror. Pin
paths are exact siblings of that runner's `$tooling` directory, so the embedded
production pre-mutation hashes remain meaningful without rewriting the script.
Before adding the mutation test, a behavior-preserving refactor replaces the
old absolute approval-tool path with that sibling path while retaining the
existing approval-tool hash pin; existing runner tests must stay green.
For each pinned Python file, the test injects an ASCII, valid-AST,
environment-driven child sentinel and coherently rebuilds the mutable ledger,
transcript, final report, and final envelope. Direct invocation of the mutated
copied approval CLI must return exact approval and write the sentinel.

The sentinel is then removed and the copied runner is invoked. It must reject
the mutated pin with no sentinel, no qualification-input path in diagnostics,
no candidate, byte-identical fixture state, and no Orca, fixed-probe, or
`run_one` process. The exact diagnostics are `Tooling approval validator
identity drift` and `Tooling evidence validator identity drift`. This proves
both pins separately before child launch.

Before the helper pin is added, the approval-tool mutation is an inherited
green control and the evidence-helper mutation is the required RED: the latter
starts the approval child or reaches input access. Adding the helper pin makes
both cases green. Final pin bytes/hashes are computed only after every
attempt-4 change to both pinned sources; no later step edits them.

## Source, test, and manifest topology
The attempt-4 source ledger is the immutable 58 attempt-2 paths plus exactly:

```text
tooling/a0_tooling_evidence.py
tooling/tests/a0_tooling_approval_fixture.py
```

The final set is 60 sorted unique paths: 41 Python and seven PowerShell. The
production helper owns closed transcript parsing, strict traversal, the 31
fixed maps, and document-envelope validation. The test fixture owns literal
positive evidence generation and is imported by existing test modules. The
existing `ApprovalFixture` and its fixture-only constants move out of
`test_tooling_approval.py`; no compatibility re-export is added, and all
consumers import the new module directly.

The exact closed-set arithmetic is:

```text
parent subjects: 13
repair additions: 98
repository repair documents: 8
excluded development residue: 7
```

The 98 additions are the immutable attempt-2 88, the two rejected attempt-3
document artifacts, and eight attempt-4 A0 artifacts: document review,
document envelope, ledger, transcript, final review, final envelope, production
helper, and fixture module. The eight repository documents are the attempt-2
four plus the attempt-3 and attempt-4 spec/plan pairs.

The `documents` group retains all prior fields and adds exactly these eight:

```text
tooling_attempt_3_specification_role
tooling_attempt_3_plan_role
tooling_attempt_3_document_review_role
tooling_attempt_3_document_approval_envelope_role
tooling_attempt_4_specification_role
tooling_attempt_4_plan_role
tooling_attempt_4_document_review_role
tooling_attempt_4_document_approval_envelope_role
```

They bind the two versioned document pairs and their versioned review/envelope
paths in the shown order. The `tooling` group retains all attempt-1 and
attempt-2 fields and adds exactly:

```text
attempt_4_test_result_role
attempt_4_source_ledger_role
attempt_4_review_report_role
attempt_4_review_envelope_role
attempt_4_fixed_probe_evidence_roles
```

Attempt 3 has no final tooling fields because those artifacts never existed.
The first four attempt-4 fields bind transcript, ledger, final report, and final
envelope; the fixed list retains the seven existing evidence roles.
`source_roles` has 60 roles, `test_roles` has 24 roles, and
`active_review_attempt=4`.

Tests remain in test modules. All sources are ASCII and below 400 physical
lines. No Rust source uses `include!` or `include_bytes!`; this amendment adds
no Rust source. The three new required test IDs are:

```text
test_tooling_approval.ToolingApprovalTests.test_transcript_rejects_all_17_closed_grammar_mutations
test_tooling_approval.ToolingApprovalTests.test_full_approval_rejects_all_31_prerequisites_and_12_nested_identities
test_tooling_approval.ToolingApprovalTests.test_formal_runner_rejects_both_pinned_sources_before_inputs_candidates_or_children
```

They join the eight immutable required IDs. Discovery contains at least 107
unique tests and the transcript contains the same IDs exactly once with `ok`.

## Attempt-4 evidence and approval
The final create-once paths are:

```text
coverage-repair/tooling-review/attempt-4/source-files.sha256
coverage-repair/tooling-review/attempt-4/mock-test-results.txt
coverage-repair/tooling-review/attempt-4/six-axis-review.md
coverage-repair/tooling-review/attempt-4/review-envelope.json
```

The transcript has attempt number 4, `FORMAL_ORCA_EXECUTED=false`, exact closed
section framing, `PYTHON_AST_PASS=41`, `ASCII_AND_LOC_LT_400_PASS=60`, and the
unchanged seven-PowerShell and 31-prerequisite markers. Ledger and transcript
are published only after a fresh full run, fixed CTest, static gates, and two
identical fresh 60-source rehashes.

The same reviewer assesses the six established axes. The exact repair IDs are:

```text
close-unittest-and-complete-ctest-transcript-grammar
enforce-no-follow-31-prerequisite-formal-gate
exercise-all-31-prerequisites-and-12-nested-identities
pin-both-formal-runner-python-subjects-before-input
rerun-refreeze-and-same-reviewer
```

The report has the established five-line header with attempt 4 and the new
ledger/transcript hashes, exactly six approved axis lines, exactly five resolved
repair lines, no `BLOCKER:` line, and sole final `VERDICT: APPROVE`.

The canonical final envelope retains the established 13 top-level keys. Its
closed `subjects` object has exactly `source_ledger`, `mock_test_results`,
`review_report`, `attempt_1_review_envelope`, `attempt_2_review_envelope`,
`attempt_3_document_approval_envelope`,
`attempt_4_document_approval_envelope`, and `fixed_probe_evidence`. Approval
uses attempt 4, all five repairs, six approved axes, no blockers or post-review
mutation, true formal authorization, and `verdict=APPROVE`.

A rejected report has the same five-line header, exactly six ordered `AXIS:`
lines valued `APPROVE` or `REJECT` with at least one rejection, one ordered
`REPAIR:` line for each required repair valued `RESOLVED` or `OPEN`, one unique
`BLOCKER: <kebab-case-id>` line per reviewer issue, and sole final
`VERDICT: REJECT`. Its canonical 13-key envelope retains the same subjects,
sets `state=rejected`, `formal_orca_execution_authorized=false`, and
`subjects_mutated_after_review=false`, and `verdict=REJECT`; `axis_verdicts`
mirrors the six lines,
`blocking_issue_ids` mirrors the blocker lines, `required_repair_ids` remains
all five, and `resolved_repair_ids` is their ordered `RESOLVED` subset.

## Exit criteria
Attempt 4 is complete only when the eight immutable rejection subjects and the
approved attempt-4 document gate remain exact; all 17 transcript mutations,
62 direct prerequisite cases, 12 nested-identity cases, both-root path cases,
and both runner-pin cases pass through their public seams; the exact 60-source
static gate, fresh 107+ suite, and fixed CTest pass; fresh evidence is published
create-once; and the same reviewer returns six-axis `VERDICT: APPROVE` followed
by successful full approval, assembly, subject verification, and deep
verification on unchanged subjects.

Only then may the fresh detached Orca build and 71 formal qualification
processes begin.
