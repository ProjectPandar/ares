# Task 22O Package A0 Tooling Review Attempt 5 Recovery Specification

## Status and scope

Attempt 5 is a minimal recovery amendment for the consumed Attempt-4 tooling review.
Attempt 4 completed its implementation behavior, full 107-test suite, fixed CTest,
and static gates, but its final source ledger was created before the full suite and
the two required unpublished source builds and rehashes. That create-once ordering
violation consumes Attempt 4 even though its final source bytes may match the premature ledger.

This amendment preserves Attempt 4 unchanged as rejected evidence, uses fresh
Attempt-5 document and final-evidence paths, versions the existing approval, runner,
manifest, fixture, assembly, and verification contracts to Attempt 5, and repeats
the final verification and review sequence in the correct order. It adds no tooling
source module, test module, tracked Rust source, Cargo metadata, slicing behavior,
architecture decision, roadmap feature, workflow, fixed Orca source, Package 0
source, or 71-role process definition. It does not touch `main.obj`.

No formal Orca build, input-derived qualification, formal `run_one`, or 71-process
execution may begin until the Attempt-5 final envelope authorizes formal Orca and
full approval, runner preflight, assembly, subject verification, and deep verification
all pass on unchanged subjects. Development-mock runner tests and the fixed-probe
CTest contract are not formal qualification.

## Unchanged source-cited Orca boundary

The upstream boundary remains OrcaSlicer tag `v2.4.2`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, specifically:

- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp`;
- `PerimeterGenerator::process_classic()` at fixed-source lines 1144-1692;
- the fixed Voronoi dependencies already named by Package A0.

Attempt 5 adds no Ares-owned slicing pipeline or replacement source boundary.
The fixed Orca derivative, Package 0, and the 71-role order remain immutable.

## Exact immutable prior subjects

The following 16 create-once subjects are immutable. They may not be deleted,
renamed, appended to, overwritten, regenerated, or substituted.

| Root | Subject | Exact identity |
| --- | --- | --- |
| `a0` | `coverage-repair/tooling-review/attempt-2/source-files.sha256` | 5,978 bytes / `47baf70c599daf794b92857bc2404acb5433436d263da6ea3d8cb4d3203872b4` |
| `a0` | `coverage-repair/tooling-review/attempt-2/mock-test-results.txt` | 19,242 bytes / `344708cdedad36ec6b5e6d3d463ada7e9b5587a4e40c0b996ae4f32753b97e77` |
| `a0` | `coverage-repair/tooling-review/attempt-2/six-axis-review.md` | 6,116 bytes / `af8f870ac829a960411dbcc066e34af785a96e80db7dea824d71d1a562ce76ca` |
| `a0` | `coverage-repair/tooling-review/attempt-2/review-envelope.json` | 4,077 bytes / `278fca5cedd3ec961edbd53da572ea97b53581222457a1e8c0ecf5afb47e5b16` |
| `repository` | `docs/superpowers/specs/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-3.md` | 15,922 bytes / `1b720c6a4737c046bf27fbc866b5a53b97f5da2e2d9ca65900fc484b842f00e8` |
| `repository` | `docs/superpowers/plans/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-3.md` | 11,303 bytes / `3bc53640cb0aad2a570a95319da846d193a6feb66db36ba0e7d85542795153bc` |
| `a0` | `coverage-repair/tooling-review/attempt-3/document-review.md` | 5,091 bytes / `795a80cc2ef6ba1381c6c7a848d4b2a51e57d8c374736e984aa39b75809c6fbf` |
| `a0` | `coverage-repair/tooling-review/attempt-3/document-approval-envelope.json` | 1,608 bytes / `73abd4108ac0fe52e4fecc7841312728e6d0bde7a50e01a7e45173889ae3faea` |
| `repository` | `docs/superpowers/specs/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-4.md` | 19,980 bytes / `5744ff3860d7c9558087801f7ccb92d18db451d8338ab8a7dce0468ec71cd166` |
| `repository` | `docs/superpowers/plans/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-4.md` | 14,121 bytes / `9adb68e20a3779eee42a107289f93086375fe35b42033f99dc419d7ac42cfc52` |
| `a0` | `coverage-repair/tooling-review/attempt-4/document-review.md` | 6,309 bytes / `d6201e7342e1a4dc92329dd2602bf0e1098fcaff98827ab86318033f81faead5` |
| `a0` | `coverage-repair/tooling-review/attempt-4/document-approval-envelope.json` | 3,158 bytes / `2f93cdbecc0a12270b88b3fd94a4190127923f60708137cdc8ecaa29c40ae6c0` |
| `a0` | `coverage-repair/tooling-review/attempt-4/source-files.sha256` | 6,197 bytes / `890b107e482ecccd3734b231e2f90c20c14ee9e40a6e98c4e0a39d4013151522` |
| `a0` | `coverage-repair/tooling-review/attempt-4/mock-test-results.txt` | 19,824 bytes / `9f0d52c35e42668db56295481dacb218f1d488c0afda407c5d85fd6abcb7a014` |
| `a0` | `coverage-repair/tooling-review/attempt-4/six-axis-review.md` | 1,657 bytes / `3f9275247a3c03ae3ca1cd14e81f00b67406b54e0629744ef511820810365219` |
| `a0` | `coverage-repair/tooling-review/attempt-4/review-envelope.json` | 4,480 bytes / `f4eeab42ba02a2a9bc5ccf3be22a93bec41fd52add181f8fe11871a3e2519ffc` |

The Attempt-4 transcript records all 107 tests passing in 2,328.553 seconds, fixed
CTest passing 1/1 in 0.42 seconds with 0.44 seconds total, and static counts of 60
sources, 41 Python sources, seven PowerShell sources, and 31 frozen prerequisites.
Those results prove implementation behavior and static quality, but they do not
repair the create-once publication-order violation.

The Attempt-4 final report is retained verbatim from
`/root/task22o_a0r_tooling_six_axis_review`. Its blocker is exactly:

```text
premature-final-source-ledger-publication
```

Its canonical 13-key envelope has attempt 4, `state=rejected`,
`formal_orca_execution_authorized=false`, `subjects_mutated_after_review=false`,
and `verdict=REJECT`. It binds the Attempt-4 ledger, transcript, review report,
prior review/document envelopes, and fixed-probe evidence. No Attempt-5 validator
may reinterpret that envelope as an approval.

## Exact Attempt-4 recovery obligations

Attempt 5 implements the reviewer's four ordered recovery requirements:

1. preserve Attempt 4 unchanged as rejected evidence;
2. use a fresh Attempt-5 directory;
3. complete the full test run, fixed CTest, static gates, and two identical
   unpublished ledger builds and rehashes;
4. only then publish the Attempt-5 ledger and transcript create-once.

The established same-reviewer refreeze requirement also remains mandatory.
These obligations are represented by the following exact ordered repair IDs:

```text
preserve-attempt-4-rejected-evidence-unchanged
use-fresh-attempt-5-evidence-directory
complete-full-suite-ctest-static-gates-and-two-unpublished-rehashes
publish-attempt-5-ledger-and-transcript-only-after-prerequisites
rerun-refreeze-and-same-reviewer
```

## Attempt-5 document gate

The Attempt-5 specification and plan paths are:

```text
docs/superpowers/specs/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-5.md
docs/superpowers/plans/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-5.md
```

Before any Attempt-5 source or test edit, the same reviewer performs a
read-only review. The response and envelope are published create-once at:

```text
coverage-repair/tooling-review/attempt-5/document-review.md
coverage-repair/tooling-review/attempt-5/document-approval-envelope.json
```

The review has this exact semantic prefix and sole terminal verdict:

```text
TASK22O A0 TOOLING REVIEW ATTEMPT 5 DOCUMENT REVIEW
REVIEWER: /root/task22o_a0r_tooling_six_axis_review
SPEC_SHA256: {64 lowercase hexadecimal characters}
PLAN_SHA256: {64 lowercase hexadecimal characters}
DOCUMENT: specification: APPROVE
DOCUMENT: implementation_plan: APPROVE
CHECK: immutable_attempt_2_3_and_4_history: APPROVE
CHECK: attempt_4_consumed_rejection_semantics: APPROVE
CHECK: fresh_attempt_5_destinations: APPROVE
CHECK: unpublished_double_rehash_publication_order: APPROVE
CHECK: minimal_versioning_and_manifest_arithmetic: APPROVE
VERDICT: APPROVE
```

Supporting proof may appear between the check lines and verdict, but no other line
begins `DOCUMENT:`, `CHECK:`, or `VERDICT:`. The sole final nonempty line is
`VERDICT: APPROVE`.

A rejection uses the same header and exact two document plus five check lines,
has at least one `REJECT`, ends with the sole final
`VERDICT: REJECT`, consumes both Attempt-5 document-gate paths, authorizes no
source edit, and requires Attempt 6.

The canonical document envelope has exactly:

```text
schema_version
kind
state
reviewer
documents
review_report
immutable_attempt_2
immutable_attempt_3
immutable_attempt_4
subjects_mutated_after_review
implementation_authorized
formal_orca_execution_authorized
verdict
```

Approval uses schema version 1, kind `task22o-a0-tooling-review-document-approval`,
`state=approved`, the exact reviewer, `subjects_mutated_after_review=false`,
`implementation_authorized=true`, `formal_orca_execution_authorized=false`, and
`verdict=APPROVE`.

`documents` has exactly `specification` and `implementation_plan`.
`immutable_attempt_2` has exactly four final Attempt-2 identities.
`immutable_attempt_3` has exactly specification, implementation plan,
document review, and document envelope.
`immutable_attempt_4` has exactly specification, implementation plan,
document review, document envelope, source ledger, transcript, final review,
and final envelope. Including the current document pair and review report, the
envelope contains exactly 19 nested identities. Every identity has exactly
`root`, `path`, `bytes`, and `sha256`.

JSON is ASCII, sorted-key, two-space indented, and has one final newline. After
publication, the 16 prior subjects plus the Attempt-5 specification, plan, document
review, and document envelope form 20 immutable document-gate subjects.

## Public seams and versioning-only TDD

The public seams remain:

1. `a0_tooling_approval.validate_pre_review()` and `pre-review`;
2. `a0_tooling_approval.validate_full_approval()` and `approve`;
3. `run_fixed_qualification.ps1` before child launch, qualification-input
   access, or candidate creation;
4. assembly, subject-verification, and deep-verification entry points that call
   full approval.

Attempt 5 adds no new test module or required test ID. Existing public fixture,
approval, runner, manifest, assembly, and deep-verification expectations are
changed to Attempt 5 before their production contracts. Their clean RED is
Attempt-4 path, attempt, subject-set, report, envelope, group, or runner-summary
behavior where Attempt 5 is required. Stale source-pin setup failure is not an
accepted behavioral RED.

Production changes then make the existing tests green by versioning existing
contracts only. There is no compatibility fallback, Attempt-4 approval alias,
private-validator test seam, or duplicate Attempt-5 implementation.

## Frozen history and no-follow validation

Full approval validates all 31 frozen prerequisites and their existing 12
nested identities through exact-case, component-by-component, no-follow
traversal before current evidence.

It also validates the exact 16 prior subjects and the approved Attempt-5
document gate. Attempt-4 final history is validated as rejected history:
the ledger and transcript use their frozen identities, the review report uses
its frozen verbatim identity, and the canonical envelope has the exact rejected
semantics and blocker. A generic outer hash is not a substitute for nested
identity validation.

The evidence and repository roots remain distinct. Root, parent, or subject
junctions, reparse points, aliases, case substitutions, missing paths,
non-regular files, byte drift, hash drift, schema drift, and nested-identity
drift are rejected before current approval.

## Exact source, test, and manifest topology

Attempt 5 adds no source path. The final source set remains exactly:

```text
source roles: 60
Python sources: 41
PowerShell sources: 7
test roles: 24
discovered tests: 107
required test IDs: 11
frozen prerequisites: 31
```

The 60 sorted unique paths are the same Attempt-4 source path set. Existing files may be edited only for Attempt-5 versioning.
Both runner-pinned Python sources receive final byte and SHA-256 pins only after their last edit, and no later step edits either pinned source.

The repair closed-set arithmetic becomes exactly:

```text
parent subjects: 13
repair additions: 104
repository repair documents: 10
excluded development residue: 7
coverage-repair/tooling-review prefix subjects: 28
```

The 104 additions are the existing 98 plus exactly these six Attempt-5 A0 artifacts:

```text
coverage-repair/tooling-review/attempt-5/document-review.md
coverage-repair/tooling-review/attempt-5/document-approval-envelope.json
coverage-repair/tooling-review/attempt-5/source-files.sha256
coverage-repair/tooling-review/attempt-5/mock-test-results.txt
coverage-repair/tooling-review/attempt-5/six-axis-review.md
coverage-repair/tooling-review/attempt-5/review-envelope.json
```

The ten repository documents are the existing eight plus the Attempt-5
specification and plan. The tooling-review prefix count is the existing 22 plus
the six Attempt-5 A0 artifacts.

The `documents` group retains every prior field and adds exactly, in order:

```text
tooling_attempt_5_specification_role
tooling_attempt_5_plan_role
tooling_attempt_5_document_review_role
tooling_attempt_5_document_approval_envelope_role
```

The `tooling` group retains every Attempt-1, Attempt-2, and Attempt-4 field and
adds exactly, in order:

```text
attempt_5_test_result_role
attempt_5_source_ledger_role
attempt_5_review_report_role
attempt_5_review_envelope_role
attempt_5_fixed_probe_evidence_roles
```

Attempt-4 tooling fields remain bound to rejected history. The Attempt-5 fixed probe list reuses the same seven fixed evidence roles.
`source_roles` remains 60, `test_roles` remains 24, and `active_review_attempt=5`.

Every source is ASCII, has one final newline where its format requires one, and has fewer than 400 physical lines.
Python and PowerShell AST checks pass. No Rust source uses `include!` or `include_bytes!`; Attempt 5 adds no Rust source.

## Attempt-5 unpublished qualification and publication

The four final create-once paths are:

```text
coverage-repair/tooling-review/attempt-5/source-files.sha256
coverage-repair/tooling-review/attempt-5/mock-test-results.txt
coverage-repair/tooling-review/attempt-5/six-axis-review.md
coverage-repair/tooling-review/attempt-5/review-envelope.json
```

All four remain absent during source edits, focused tests, full tests, CTest, static
gates, candidate construction, and the two unpublished source builds. Staging occurs
outside the repository and A0 closed set.

After the final source edit and runner pin:

1. confirm no Orca, formal runner, `run_one`, or fixed-probe process is active;
2. run bundled-Python discovery and all 107 tests;
3. run the fixed CTest contract and require exactly 1/1;
4. run the 60/41/7/31 static and frozen gates;
5. build unpublished ledger A from fresh reads of all 60 sources;
6. assemble the proposed Attempt-5 transcript against ledger A;
7. validate the proposed pair through production pre-review in a fresh,
   non-reparse evidence mirror;
8. build unpublished ledger B from independent fresh reads immediately before
   publication;
9. require ledger A and ledger B byte-identical and require every current
   source to match them;
10. reconfirm all four final Attempt-5 paths absent;
11. publish ledger and transcript create-once, read them back, and run
    production pre-review.

Any source edit, generated source mutation, test failure, CTest failure, static
failure, mirror-validation failure, or ledger mismatch discards unpublished staging
and restarts the complete sequence. It does not publish a partial Attempt-5 final subject.

The transcript has attempt 5, `FORMAL_ORCA_EXECUTED=false`, the exact closed
unittest and CTest grammar, all 107 unique tests exactly once with `ok`,
`PYTHON_AST_PASS=41`, `POWERSHELL_AST_PASS=7`,
`ASCII_AND_LOC_LT_400_PASS=60`, and `FROZEN_PREREQUISITES_PASS=31`.
Timing fields may vary; subjects and semantic markers may not.

## Same-reviewer final approval

The same reviewer assesses:

```text
requirements_completeness
logical_correctness
boundary_cases
code_quality
test_coverage
actual_execution_results
```

The approved report has the exact five-line Attempt-5 header with ledger and transcript hashes, exactly six ordered `AXIS:` lines ending in `APPROVE`,
exactly five ordered `REPAIR:` lines matching the repair IDs and ending in `RESOLVED`, no blocker, and the sole final line `VERDICT: APPROVE`.

The final canonical envelope retains the established 13 top-level keys. Its
closed `subjects` object has exactly:

```text
source_ledger
mock_test_results
review_report
attempt_1_review_envelope
attempt_2_review_envelope
attempt_3_document_approval_envelope
attempt_4_document_approval_envelope
attempt_4_review_envelope
attempt_5_document_approval_envelope
fixed_probe_evidence
```

Approval has attempt 5, all five repairs required and resolved, all six axes
approved, no blockers, no subject mutation, true formal Orca authorization,
and `verdict=APPROVE`.

A rejection retains the same header and closed schemas, has at least one
rejected axis, one unique kebab-case blocker per reviewer issue, an exact
resolved/open repair subset, `state=rejected`,
`formal_orca_execution_authorized=false`,
`subjects_mutated_after_review=false`, and `verdict=REJECT`. It consumes all
four Attempt-5 final destinations and requires Attempt 6 before source edits.

After approval, production full approval, the formal-runner no-launch preflight,
assembly, subject verification, and deep verification must pass on unchanged
subjects. Only then may the detached Orca build and 71 formal qualification
processes begin.

## Exit criteria

Attempt 5 is complete only when:

1. all 16 prior subjects and the approved Attempt-5 document gate remain exact;
2. Attempt-4 rejection semantics and blocker remain exact;
3. the unchanged 60/41/7/24/107/11/31 topology passes;
4. closed-set counts are exactly 13/104/10/7 with tooling prefix 28;
5. full unittest, fixed CTest, and every static gate pass before publication;
6. two independent unpublished ledgers are byte-identical;
7. ledger and transcript are then published create-once at fresh Attempt-5
   paths;
8. the same reviewer returns six-axis `VERDICT: APPROVE`;
9. full approval, runner preflight, assembly, subject verification, and deep
   verification pass without subject mutation;
10. no formal Orca execution occurred before complete approval.
