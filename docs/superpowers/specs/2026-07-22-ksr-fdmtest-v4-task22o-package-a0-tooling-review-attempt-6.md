# Task 22O Package A0 Tooling Review Attempt 6 Recovery Specification

## Status and scope

Attempt 6 recovers from the consumed Attempt-5 document gate. The Attempt-5
specification and plan received read-only review, but the create-once document
envelope was serialized in noncanonical top-level key order:
`implementation_authorized` appears before `immutable_attempt_2`,
`immutable_attempt_3`, and `immutable_attempt_4`. Its 13-key set, 19 nested
identities, and semantic values do not authorize implementation because exact
canonical JSON bytes are part of the gate.

No source or test edit, Attempt-5 final evidence publication, formal Orca build,
formal `run_one`, candidate creation, or 71-process execution occurred after the
invalid Attempt-5 gate. Attempt 5 is retained unchanged as consumed history.

Attempt 6 uses fresh document and final-evidence paths, validates proposed
document-envelope bytes in memory before create-once publication, and versions
the existing tooling contracts directly to Attempt 6. It adds no tooling source
module, test module, required test ID, tracked Rust source, Cargo metadata,
slicing behavior, architecture decision, roadmap feature, workflow, fixed Orca
source, Package 0 source, or 71-role definition. It does not touch `main.obj`.

No formal Orca execution may begin until the Attempt-6 final envelope authorizes
it and full approval, runner preflight, assembly, subject verification, and deep
verification pass on unchanged subjects.

## Unchanged source-cited Orca boundary

The upstream boundary remains OrcaSlicer tag `v2.4.2`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, specifically:

- `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp`;
- `PerimeterGenerator::process_classic()` at fixed-source lines 1144-1692;
- the fixed Voronoi dependencies already named by Package A0.

Attempt 6 adds no Ares-owned slicing pipeline and changes no upstream boundary,
fixed Orca derivative, Package 0 source, or 71-role order.

## Exact immutable prior subjects

The following 20 create-once subjects are immutable. They may not be deleted,
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
| `repository` | `docs/superpowers/specs/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-5.md` | 18,530 bytes / `499241c194f35ba2d1ce8db6549a9178a7c750fb5ac29afa8204f31edc10a12d` |
| `repository` | `docs/superpowers/plans/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-5.md` | 16,261 bytes / `48d3f13ffd651773e42b83b1715e85d796656784a80b692a08abd6e5e82cadf2` |
| `a0` | `coverage-repair/tooling-review/attempt-5/document-review.md` | 2,451 bytes / `0328ec37dc90b4fdaa8ecebd07594dfe34f8687e55e8e7804c338c9e1abdff46` |
| `a0` | `coverage-repair/tooling-review/attempt-5/document-approval-envelope.json` | 5,174 bytes / `fb9825145a5d2dcc5f048923c8c5a71866efe313b75b5824285e62508b147d2a` |

Attempt 4 remains rejected by `premature-final-source-ledger-publication`.
Attempt 5 is consumed by exactly:

```text
noncanonical-attempt-5-document-envelope-key-order
```

The Attempt-5 envelope has the correct closed 13-key set and 19 identity
bindings, but its bytes differ from sorted-key canonical serialization. Its
`implementation_authorized` value is ignored. No validator may normalize it in
place or reinterpret it as approval.

## Attempt-6 recovery and repair IDs

Attempt 6 must preserve the invalid Attempt-5 gate, use fresh paths, validate
the complete document envelope in memory before its create-once write, complete
all verification and two unpublished ledgers, publish only afterward, and
return unchanged subjects to the same reviewer.

The exact ordered repair IDs are:

```text
preserve-attempt-5-invalid-document-gate-unchanged
use-fresh-attempt-6-document-and-final-evidence-paths
validate-canonical-attempt-6-document-envelope-bytes-before-create-once
complete-full-suite-ctest-static-gates-and-two-unpublished-rehashes
publish-attempt-6-ledger-and-transcript-only-after-prerequisites
rerun-refreeze-and-same-reviewer
```

## Attempt-6 document gate

The fresh repository documents are:

```text
docs/superpowers/specs/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-6.md
docs/superpowers/plans/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-6.md
```

The same reviewer performs a read-only review before any source or test edit.
The response and envelope are create-once at:

```text
coverage-repair/tooling-review/attempt-6/document-review.md
coverage-repair/tooling-review/attempt-6/document-approval-envelope.json
```

The review has exactly two document lines, five check lines, and one terminal
verdict:

```text
TASK22O A0 TOOLING REVIEW ATTEMPT 6 DOCUMENT REVIEW
REVIEWER: /root/task22o_a0r_tooling_six_axis_review
SPEC_SHA256: {64 lowercase hexadecimal characters}
PLAN_SHA256: {64 lowercase hexadecimal characters}
DOCUMENT: specification: APPROVE
DOCUMENT: implementation_plan: APPROVE
CHECK: immutable_attempt_2_3_4_5_history: APPROVE
CHECK: attempt_5_noncanonical_envelope_rejection: APPROVE
CHECK: canonical_attempt_6_envelope_preflight: APPROVE
CHECK: minimal_attempt_6_versioning_arithmetic: APPROVE
CHECK: unpublished_double_rehash_publication_order: APPROVE
VERDICT: APPROVE
```

No other line begins `DOCUMENT:`, `CHECK:`, or `VERDICT:`. The sole final
nonempty line is `VERDICT: APPROVE`.

The document envelope has exactly 14 top-level keys:

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
immutable_attempt_5
subjects_mutated_after_review
implementation_authorized
formal_orca_execution_authorized
verdict
```

It binds the current specification and plan, current review report, four
Attempt-2 identities, four Attempt-3 identities, eight Attempt-4 identities,
and four Attempt-5 identities: exactly 23 nested identities. Every identity has
exactly `root`, `path`, `bytes`, and `sha256`.

Before publication, construct the complete envelope in memory and produce bytes
equivalent to:

```text
(json.dumps(value, sort_keys=True, indent=2) + "\n").encode("ascii")
```

Parse those proposed bytes, require the closed schema and 23 identities,
reserialize with the same expression, and require byte equality. The serialized
top-level key order is exactly:

```text
documents
formal_orca_execution_authorized
immutable_attempt_2
immutable_attempt_3
immutable_attempt_4
immutable_attempt_5
implementation_authorized
kind
review_report
reviewer
schema_version
state
subjects_mutated_after_review
verdict
```

Only validated bytes may be written with exclusive create semantics. Readback
must equal the proposed bytes. Approval uses schema version 1, kind
`task22o-a0-tooling-review-document-approval`, approved state, the exact
reviewer, no subject mutation, true implementation authorization, false formal
Orca authorization, and `verdict=APPROVE`.

A rejected review or canonical preflight failure authorizes no source edit. If
either Attempt-6 document-gate path is created, Attempt 6 is consumed and a
later repair requires Attempt 7.

## Public seams and versioning-only TDD

The public seams remain pre-review, full approval, the formal runner before any
input/candidate/child action, and assembly/subject/deep verification through
full approval.

Attempt 6 adds no test module or required test ID. Existing fixtures and public
tests change to Attempt-6 expectations before production. Their RED must be
current Attempt-4 behavior where Attempt 6 is required, not stale pin or fixture
failure. Production versions existing contracts directly to Attempt 6 without
an Attempt-5 final approval fallback.

Production discovery and `validate_test_sections` must each require
`count == 107`; `count >= 107` is not accepted.

## Source, test, and manifest topology

The source and test topology remains exactly:

```text
source roles: 60
Python sources: 41
PowerShell sources: 7
test roles: 24
discovered tests: 107
required test IDs: 11
frozen prerequisites: 31
```

No source path is added. Both runner-pinned Python sources receive final byte
and SHA-256 pins only after their last Attempt-6 edit.

The closed-set arithmetic is exactly:

```text
parent subjects: 13
repair additions: 106
repository repair documents: 12
excluded development residue: 7
coverage-repair/tooling-review prefix subjects: 30
```

The 106 additions are the Attempt-4 baseline 98, the two actual Attempt-5
document-gate artifacts, and six Attempt-6 A0 artifacts: document review,
document envelope, ledger, transcript, final review, and final envelope. There
is no Attempt-5 final ledger, transcript, review, or envelope.

Before all four Attempt-6 final artifacts exist, the real A0 root is
intentionally four paths short. Do not call real-root
`validate_repair_closed_set()` during that interval. Verify 13/106/12/7 and
prefix 30 through constant arithmetic and complete synthetic fixtures. Run the
real-root closed-set validator only after all four final artifacts are present.

The `documents` group retains prior fields and adds, in order:

```text
tooling_attempt_5_specification_role
tooling_attempt_5_plan_role
tooling_attempt_5_document_review_role
tooling_attempt_5_document_approval_envelope_role
tooling_attempt_6_specification_role
tooling_attempt_6_plan_role
tooling_attempt_6_document_review_role
tooling_attempt_6_document_approval_envelope_role
```

The `tooling` group retains Attempt-1, Attempt-2, and Attempt-4 fields and adds
only:

```text
attempt_6_test_result_role
attempt_6_source_ledger_role
attempt_6_review_report_role
attempt_6_review_envelope_role
attempt_6_fixed_probe_evidence_roles
```

The fixed list reuses seven evidence roles. `active_review_attempt=6`. Every
source is ASCII and below 400 physical lines. No Rust source uses `include!` or
`include_bytes!`.

## Unpublished qualification and publication

The fresh final paths are:

```text
coverage-repair/tooling-review/attempt-6/source-files.sha256
coverage-repair/tooling-review/attempt-6/mock-test-results.txt
coverage-repair/tooling-review/attempt-6/six-axis-review.md
coverage-repair/tooling-review/attempt-6/review-envelope.json
```

All four remain absent during source edits, tests, CTest, static gates, staging,
mirror validation, and both unpublished ledger builds. Staging occurs outside
the repository and A0 closed set.

After the final source edit and runner pin:

1. require no formal process active and run exactly 107 tests;
2. run fixed CTest 1/1 and the 60/41/7/31 gates;
3. verify closed-set constants and synthetic fixtures without a real-root call;
4. build unpublished ledger A from fresh reads;
5. construct a complete non-reparse A0 and repository mirror;
6. copy the immutable Attempt-2 base ledger, all exact 60 source-relative files,
   every required imported tooling module, and required current/prior evidence;
7. place the proposed Attempt-6 ledger and transcript only after that complete
   mirror exists, then execute the mirrored production validator;
8. build unpublished ledger B from independent fresh reads immediately before
   publication;
9. require A and B byte-identical, every source unchanged, and all 24
   document-gate subjects exact;
10. reconfirm all four final paths absent;
11. publish ledger and transcript create-once and run real-root pre-review.

A mirror containing only proposed ledger and transcript is invalid. Any edit or
failure discards staging and restarts the complete sequence. The transcript uses
attempt 6, false formal Orca, exactly 107 passing tests, fixed CTest 1/1, and
the exact 41/7/60/31 static markers.

## Same-reviewer final approval

The same reviewer assesses the established six axes. Approved report bytes are
ASCII, LF-only, and have one final newline. Approval is exactly 18 nonempty
physical lines in order: five exact header lines, six ordered
`AXIS: <axis>: APPROVE` lines, six ordered
`REPAIR: <repair>: RESOLVED` lines, and sole final `VERDICT: APPROVE`. No blank,
proof, blocker, duplicate, or extra line is permitted. The approval parser
compares the complete physical-line list exactly. Rejection grammar is separate
and may contain only its explicitly defined blocker and open-repair lines.

The canonical 13-key final envelope has exactly 11 subjects:

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
attempt_6_document_approval_envelope
fixed_probe_evidence
```

Approval uses attempt 6, all six repairs required and resolved, six approved
axes, no blocker or subject mutation, true formal Orca authorization, and
`verdict=APPROVE`.

A rejection is preserved create-once and requires Attempt 7. After the final
review and envelope exist, run real-root `validate_repair_closed_set()` and
require 13/106/12/7 before full approval, runner no-launch preflight, assembly,
subject verification, and deep verification. Only then may formal Orca begin.

## Exit criteria

Attempt 6 completes only when all 20 prior and four current document-gate
subjects remain exact; the Attempt-5 blocker remains exact; the document
envelope was canonical before exclusive write; topology is
60/41/7/24/107/11/31; closed-set arithmetic is 13/106/12/7 with prefix 30; no
Attempt-5 final role exists; exactly 107 tests, CTest, static gates, a complete
mirror, and two unpublished ledgers pass; all four final artifacts exist before
real-root closed-set validation; the same reviewer approves; post-review gates
pass unchanged; and no formal Orca ran before full approval.
