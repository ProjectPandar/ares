# Task 22O Package A0 Tooling Review Attempt 3 Specification

## Status and scope

This amendment repairs only the two P1 approval-gate defects returned by the
independent Task 22O Package A0 tooling review attempt 2. It supplements, and
does not rewrite, the approved attempt-2 specification and plan at SHA-256
`9bc0b15c1ba0e3cb70e2db7b295178aeac5c671f08cad9a8e972e0a71fbc7b2c`
and
`fd238b39d6d63f65e3bdd544dfec5e18741cf1a35da9ce6c365ea3a8ed487463`.
Their document approval envelope remains
`ad8f614569d203f2d46cfb930c26cabf19c433d9a86fb8e398bc6518f13cfb06`.

The amendment changes ignored Package A0 approval tooling, tests, manifests,
mock fixtures, and create-once review evidence only. It does not change
tracked Rust production code, Cargo metadata, the fixed Orca derivative,
Package 0, the 71-role order, architecture, roadmap, workflows, or `main.obj`.
No formal Orca build, input-derived qualification process, or formal
`run_one` may start until attempt 3 has an independent approval and its
full approval CLI passes at the formal-runner boundary. Development-mock
`run_one` tests and the fixed-probe CTest contract may produce review evidence;
they are not formal qualification. A development test may create only its
asserted ephemeral candidate, must remove it before completion, and leaves
`runs/qualified` empty before evidence freeze.
The upstream rewrite boundary remains OrcaSlicer tag `v2.4.2`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, specifically
`src/libslic3r/PerimeterGenerator.cpp`,
`PerimeterGenerator::process_classic()` at fixed-source lines 1144-1692, and
the fixed Voronoi dependencies named by the Package A0 amendment. Attempt 3
adds no Ares-owned slicing behavior and changes no upstream source boundary.

## Immutable attempt-2 rejection

Attempt 2 is complete rejected history. These four create-once subjects must
never be deleted, renamed, appended to, overwritten, or substituted:

| Subject | Bytes | SHA-256 |
| --- | ---: | --- |
| `coverage-repair/tooling-review/attempt-2/source-files.sha256` | 5,978 | `47baf70c599daf794b92857bc2404acb5433436d263da6ea3d8cb4d3203872b4` |
| `coverage-repair/tooling-review/attempt-2/mock-test-results.txt` | 19,242 | `344708cdedad36ec6b5e6d3d463ada7e9b5587a4e40c0b996ae4f32753b97e77` |
| `coverage-repair/tooling-review/attempt-2/six-axis-review.md` | 6,116 | `af8f870ac829a960411dbcc066e34af785a96e80db7dea824d71d1a562ce76ca` |
| `coverage-repair/tooling-review/attempt-2/review-envelope.json` | 4,077 | `278fca5cedd3ec961edbd53da572ea97b53581222457a1e8c0ecf5afb47e5b16` |

The attempt-2 blocker IDs, in reviewer order, are:

```text
unclosed-unittest-transcript-validation
incomplete-formal-approval-prerequisite-rehash
```

The rejected envelope semantics are ratified exactly as confirmed by the
reviewer: the first five axes are `REJECT`,
`actual_execution_results=APPROVE`, the resolved-repair list contains the
first three and final two attempt-2 repair IDs in their original order, the
six subject keys are the attempt-2 approval subject keys, state is
`rejected`, verdict is `REJECT`, and formal authorization is false.

## Attempt-3 document gate

This specification and its matching implementation plan must be reviewed
read-only before any source or test edit. The same independent reviewer used
for attempt 2 must publish one complete create-once response at:

```text
coverage-repair/tooling-review/attempt-3/document-review.md
```

The response is ASCII/LF with one final newline and these exact leading and
semantic lines:

```text
TASK22O A0 TOOLING REVIEW ATTEMPT 3 DOCUMENT REVIEW
REVIEWER: /root/task22o_a0r_tooling_six_axis_review
SPEC_SHA256: <exact specification SHA-256>
PLAN_SHA256: <exact implementation-plan SHA-256>
DOCUMENT: specification: APPROVE
DOCUMENT: implementation_plan: APPROVE
CHECK: immutable_attempt_2_history: APPROVE
CHECK: public_tdd_seams: APPROVE
CHECK: closed_transcript_grammar: APPROVE
CHECK: no_follow_prerequisite_gate: APPROVE
VERDICT: APPROVE
```

Supporting proof may occur between the check lines and verdict, but no other
line begins `DOCUMENT:`, `CHECK:`, or `VERDICT:`. The sole final
nonempty line is `VERDICT: APPROVE`. A canonical create-once envelope at:

```text
coverage-repair/tooling-review/attempt-3/document-approval-envelope.json
```

has exactly these keys:

```text
schema_version
kind
state
reviewer
documents
review_report
immutable_attempt_2
subjects_mutated_after_review
implementation_authorized
formal_orca_execution_authorized
verdict
```

It uses `schema_version=1`,
`kind=task22o-a0-tooling-review-document-approval`, `state=approved`,
the exact reviewer identity, `subjects_mutated_after_review=false`,
`implementation_authorized=true`,
`formal_orca_execution_authorized=false`, and `verdict=APPROVE`.
`documents` has exactly `specification` and `implementation_plan`
repository-root identities. `review_report` is the exact A0-root review
identity. `immutable_attempt_2` has exactly `review_report` and
`review_envelope` with the fixed attempt-2 identities above. Every identity
has exactly `root`, `path`, `bytes`, and `sha256`. The JSON is
ASCII, sorted-key, two-space indented, with one final newline.

A document rejection uses `state=rejected`, false implementation and formal
authorization, `verdict=REJECT`, consumes both review paths, and requires an
attempt-4 amendment.

## Public seams and TDD

The pre-agreed observable seams are:

1. `a0_tooling_approval.validate_pre_review()` and its `pre-review` CLI;
2. `a0_tooling_approval.validate_full_approval()` and its `approve` CLI;
3. `run_fixed_qualification.ps1` before qualification-input access or
   candidate creation; and
4. existing assembly, subject-verification, and deep-verification public
   entry points that call the full approval gate.

Every repair starts with a RED test through one of these seams. Tests may
prepare filesystem fixtures but may not assert private helper call order.
Production code changes only after the intended RED is observed. Each slice
then goes GREEN before the next mutation class is added.

## Closed transcript grammar

Attempt 3 replaces extraction-based unittest validation with one closed
grammar. Given independently discovered IDs, the unittest section is valid
only when its complete body is exactly:

```text
<method> (<fully-qualified-test-id>) ... ok
... exactly one line for every discovered ID ...

----------------------------------------------------------------------
Ran <N> tests in <nonnegative numeric duration>s

OK
```

Result IDs are unique and equal the discovered set; `method` equals the final
ID component. There are no lines before, between, or after this sequence
other than the shown result lines and blanks. The grammar rejects skipped and
multi-token skipped statuses, expected failures, failures, errors, traceback
blocks, `FAILED` summaries, conflicting terminals, duplicate outcomes, wrong
method names, and every unclassified result-like or terminal line.

The CTest section is also closed. It contains exactly the two fixed build
directory lines, the single `Start 1` line, exactly one recognized 1/1 passed
row for `ares22o_voronoi_fixed_probe_contract`, one blank line, the exact
100-percent/zero-failed summary, one blank line, and one numeric total-time
line. `Errors while running CTest`, `The following tests FAILED:`,
extra rows, extra summaries, zero-test text, and arbitrary extra lines fail.

Transcript mutation tests exercise every rejected class above through
`validate_pre_review()`. The valid fixture is independently generated from a
known literal grammar, not by calling the production parser.

## No-follow frozen prerequisites

The complete original frozen prerequisite set remains exactly 31 subjects:
the 27 A0 entries and four repository documents currently defined by the
attempt-2 repair contract. The full approval API and CLI must rehash every
subject against the supplied evidence and repository roots on every call.

Traversal is literal and component-by-component. It must:

- require absolute, existing, exact-case roots;
- reject a symlink, junction, mount-point/reparse entry, alias, or wrong-case
  component at the root or below it;
- reject missing subjects and non-regular subjects;
- read only the exact literal path after all components pass; and
- compare exact byte length and SHA-256 without resolving through an alias.

The attempt-2 document approval envelope is parsed as canonical ASCII JSON.
Its closed top-level schema and fixed semantic fields are validated. All 12
nested identity objects are rebound to their exact declared root, relative
path, bytes, and SHA-256 using the same no-follow traversal. Agreement among
mutable fixture files is not evidence.

The existing assembly and deep-verification prerequisite call uses this same
strict validator. The full approval CLI invokes it before validating the
attempt-3 report/envelope. Any prerequisite error is a failed approval.

## Formal-runner gate

The formal runner continues to invoke bundled Python 3.12.13 as its first
child process, before resolving or reading qualification inputs. It pins and
rehashes both executable pre-approval Python subjects:

```text
tooling/a0_tooling_approval.py
tooling/a0_tooling_evidence.py
```

It passes the literal evidence and repository roots and accepts only exit 0,
empty stderr, and the exact canonical attempt-3 approval summary. Missing,
byte-mutated, wrong-case, root-aliased, parent-reparse, or file-reparse
repository documents must fail before qualification-input access, candidate
creation, Orca/fixed-probe launch, or any input-derived child process.

Tests snapshot the entire fixture before the runner invocation and require
byte-identical state after every rejected gate.

## Source and test topology

The attempt-3 source ledger is the immutable 58 paths from the attempt-2
ledger plus exactly these two additions:

```text
tooling/a0_tooling_evidence.py
tooling/tests/test_tooling_evidence.py
```

The final source set is 60 unique lexicographically sorted paths: 41 Python
and seven PowerShell subjects, with the remaining fixed-probe C++/JSON/CMake
subjects unchanged. `a0_tooling_evidence.py` owns the closed transcript
grammar, strict subject traversal, 31-subject identities, and nested document
envelope validation. `a0_tooling_approval.py` remains the public API/CLI.

The exact repair closed-set arithmetic is:

```text
parent subjects: 13
repair additions: 96
repository repair documents: 6
excluded development residue: 7
```

The 96 additions are the immutable attempt-2 set of 88 plus exactly the
attempt-3 document review, document approval envelope, ledger, transcript,
review report, review envelope, production helper, and test module. The six
repository repair documents are the immutable four attempt-2 repair documents
plus this specification and plan.

The closed `documents` group retains every attempt-2 field and adds exactly:

```text
tooling_attempt_3_specification_role
tooling_attempt_3_plan_role
tooling_attempt_3_document_review_role
tooling_attempt_3_document_approval_envelope_role
```

They bind, in that order, this specification, this plan,
`coverage-repair/tooling-review/attempt-3/document-review.md`, and
`coverage-repair/tooling-review/attempt-3/document-approval-envelope.json`.

The closed `tooling` group retains `python_role`, `wire_role`,
`source_roles`, `test_roles`, all attempt-1 fields, all attempt-2
fields, and adds exactly:

```text
attempt_3_test_result_role
attempt_3_source_ledger_role
attempt_3_review_report_role
attempt_3_review_envelope_role
attempt_3_fixed_probe_evidence_roles
```

The first four bind, in that order, the attempt-3 transcript, ledger, report,
and envelope paths above. The fixed-evidence list has exactly seven roles in
the unchanged attempt-2 fixed-evidence order. `source_roles` has 60 roles,
`test_roles` has 24 roles, and `active_review_attempt` is exactly 3.

Tests stay in test modules. Every source is ASCII and below 400 physical
lines. No Rust source splitting uses `include!` or `include_bytes!`; this
amendment adds no Rust source.

The two new required test IDs are:

```text
test_tooling_evidence.ToolingEvidenceTests.test_transcript_requires_closed_success_grammar
test_tooling_evidence.ToolingEvidenceTests.test_full_approval_and_runner_reject_prerequisite_mutations_before_inputs
```

They join the eight immutable attempt-2 required IDs. Discovery must contain
at least 106 unique tests and the transcript must contain exactly the same IDs
once each with `ok`.

## Attempt-3 evidence and approval

All attempt-3 evidence is create-once under:

```text
coverage-repair/tooling-review/attempt-3/source-files.sha256
coverage-repair/tooling-review/attempt-3/mock-test-results.txt
coverage-repair/tooling-review/attempt-3/six-axis-review.md
coverage-repair/tooling-review/attempt-3/review-envelope.json
```

The ledger row grammar, ASCII/LF rules, complete section framing, fixed-probe
evidence, and no-formal-Orca marker remain the attempt-2 contract, with the
attempt number and static counts changed to 3, 60, and 41. The transcript
records fresh full unittest and CTest output plus actual static checks. It is
published only after a fresh final run and a second complete rehash of all 60
source subjects.

The same reviewer identity is
`/root/task22o_a0r_tooling_six_axis_review`. Approval requires exactly six
`AXIS: <id>: APPROVE` lines in the existing order and these four repairs in
order:

```text
close-unittest-and-ctest-transcript-grammar
enforce-no-follow-31-prerequisite-formal-gate
exercise-direct-and-runner-prerequisite-mutations
rerun-refreeze-and-same-reviewer
```

The report begins with exactly:

```text
TASK22O A0 TOOLING REVIEW ATTEMPT 3 SIX-AXIS REVIEW
REVIEWER: /root/task22o_a0r_tooling_six_axis_review
ATTEMPT: 3
SOURCE_LEDGER_SHA256: <exact hash>
MOCK_RESULTS_SHA256: <exact hash>
```

The sole final nonempty report line is `VERDICT: APPROVE`. The canonical
attempt-3 envelope reuses exactly the 13 review-envelope keys:

```text
schema_version
kind
attempt
state
reviewer
subjects
blocking_issue_ids
required_repair_ids
resolved_repair_ids
axis_verdicts
subjects_mutated_after_review
formal_orca_execution_authorized
verdict
```

Its closed `subjects` object has exactly:

```text
source_ledger
mock_test_results
review_report
attempt_1_review_envelope
attempt_2_review_envelope
document_approval_envelope
fixed_probe_evidence
```

These bind the new ledger, transcript, report, immutable attempt-1 rejected
envelope, immutable attempt-2 rejected envelope, attempt-3 document approval
envelope, and seven fixed-probe evidence entries in the unchanged order.
Approval has `schema_version=1`,
`kind=task22o-a0-tooling-review`, `attempt=3`, `state=approved`,
no blocker IDs, all four required and resolved repair IDs, six approved axes,
no post-review mutation, formal authorization true, and
`verdict=APPROVE`.

A rejected attempt 3 is retained create-once and requires an attempt-4
amendment. No attempt-3 file is overwritten.

## Exit criteria

Attempt 3 is complete only when:

1. the attempt-2 four-subject rejection history and attempt-3 document gate
   remain byte-identical;
2. every transcript mutation class fails through the public pre-review API;
3. all 31 prerequisites and 12 nested identities are strictly rebound;
4. every direct/full-approval and formal-runner document mutation fails
   before input access or candidate creation;
5. the exact 60-source static gate, fresh 106+ unittest suite, and fixed CTest
   pass;
6. attempt-3 ledger and transcript are published create-once and validate;
7. the same independent reviewer returns six-axis `VERDICT: APPROVE`; and
8. full approval, assembly, subject verification, and deep verification all
   accept the same unchanged subjects.

Only then may the fresh detached Orca build and 71 formal qualification
processes begin.
