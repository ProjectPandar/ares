# Task 22O Package A0 Tooling Review Attempt 2 Specification

## Status and scope

This amendment repairs the four blockers returned by the first independent
six-axis tooling review of the approved Package A0 coverage-repair frame. It
supplements, and does not rewrite, the approved repair specification and plan
at SHA-256
`49461b81bcaf236dedd4cbeb7d8697f13b698dabe3eb281598e9ae2b75f9882b`
and
`b19177c7042c1e4ab1d5edc675136a6a4746db07b0a84eaca52f2621dd77238e`.
Their document approval envelope remains
`28eed19cacc9cb8514c3679f1a5aa14e43be742a3d005fad009b7b89c404a0ed`.

This amendment changes ignored Package A0 tooling, tests, mock fixtures, and
review evidence only. It does not change the fixed Orca derivative, Package 0,
the twelve frozen Orca source files, the 71-role order, tracked Rust production
code, Cargo metadata, architecture, roadmap, or workflow files. No formal Orca
build or qualification process may start until attempt 2 is independently
approved and its approval gate validates.

## Attempt 1 rejection

The first tooling source ledger and complete mock transcript are immutable:

| Subject | Bytes | SHA-256 |
| --- | ---: | --- |
| `coverage-repair/tooling-review/source-files.sha256` | 3,844 | `29c3664180329c373ccd0ff407af8904d308920c6014db05a439bc30d6dc67aa` |
| `coverage-repair/tooling-review/mock-test-results.txt` | 14,733 | `185b7e559f918240c23f6407fb89b10588993643ac617e6624b6b068df89242b` |

They may not be deleted, renamed, copied as a substitute, appended to, or
overwritten. The complete first reviewer response is retained create-once at
`coverage-repair/tooling-review/attempt-1/six-axis-review.md`, beginning with
`Outcome: REJECT. Four blocking issues prevent A0R.5 approval.` and ending with
the sole final nonempty line `VERDICT: REJECT`. A detached rejected envelope at
`coverage-repair/tooling-review/attempt-1/review-envelope.json` binds those
three exact subjects.

The four blocking issue identifiers are, in order:

```text
assembler-public-entry
invalid-control-collector
tooling-review-semantic-approval
deep-verification-positive-path
```

The required repair identifiers are, in order:

```text
assign-groups-before-validation-and-test-public-entry
bind-all-23-control-contracts
preserve-only-control-owned-preexisting-paths
validate-ledger-results-report-and-envelope
enforce-formal-runner-and-manifest-preflight
exercise-complete-positive-deep-path-and-mutations
rerun-refreeze-and-same-reviewer
```

## Exact amendment paths

Relative to `.superpowers/sdd/task22o-oracle/voronoi-a0/`, remove the two
authorized but never-created attempt-1 final destinations from the repair
closed set:

```text
coverage-repair/tooling-review/approval-envelope.json
coverage-repair/tooling-review/six-axis-review.md
```

Keep the two immutable unversioned attempt-1 subjects above and add exactly
these twelve evidence and document-review paths:

```text
coverage-repair/tooling-review/attempt-1/review-envelope.json
coverage-repair/tooling-review/attempt-1/six-axis-review.md
coverage-repair/tooling-review/attempt-2/document-reviews/attempt-1/approval-and-deep-verification.md
coverage-repair/tooling-review/attempt-2/document-reviews/attempt-1/control-and-collector.md
coverage-repair/tooling-review/attempt-2/document-reviews/attempt-1/review-envelope.json
coverage-repair/tooling-review/attempt-2/document-reviews/attempt-2/approval-and-deep-verification.md
coverage-repair/tooling-review/attempt-2/document-reviews/attempt-2/approval-envelope.json
coverage-repair/tooling-review/attempt-2/document-reviews/attempt-2/control-and-collector.md
coverage-repair/tooling-review/attempt-2/mock-test-results.txt
coverage-repair/tooling-review/attempt-2/review-envelope.json
coverage-repair/tooling-review/attempt-2/six-axis-review.md
coverage-repair/tooling-review/attempt-2/source-files.sha256
```

Add exactly these seven authored source and test paths:

```text
tooling/a0_control_validation.py
tooling/a0_repair_contract.py
tooling/a0_tooling_approval.py
tooling/tests/a0_manifest_fixture.py
tooling/tests/test_assembly_integration.py
tooling/tests/test_manifest_deep.py
tooling/tests/test_tooling_approval.py
```

The exact new tracked amendment documents are this specification and:

```text
docs/superpowers/plans/2026-07-22-ksr-fdmtest-v4-task22o-package-a0-tooling-review-attempt-2.md
```

The final repair closed set therefore has 13 parent subjects, 88 repair
additions, four tracked repair documents, and the unchanged seven exact
excluded residue files. The approved attempt-2 source ledger has exactly 58
paths: the 38 literal paths in the immutable attempt-1 ledger, the seven source
additions above, and these 13 fixed-probe paths: `fixed-probe/CMakeLists.txt`,
both JSON files under `fixed-probe/corpus/`, `fixed-probe/main.cpp`,
`fixed-probe/src/fixed_cases.cpp`, `fixed_cases.hpp`, `fixed_cases_basic.cpp`,
`fixed_cases_edge_collapse.cpp`, `fixed_cases_internal.hpp`,
`fixed_cases_regressions.cpp`, `medial_cases.cpp`, `medial_cases.hpp`, and
`fixed-probe/tests/test_probe.py`. Every shortened source name is relative to
`fixed-probe/src/` and no other fixed-probe path enters the ledger.

Generated build trees and `__pycache__` remain non-subjects. No wildcard
authorizes any other path.

`a0_repair_contract.py` owns the frozen repair identities, exact path sets,
closed-set checks, and repair group bindings currently in the near-limit
assembler. `a0_control_validation.py` owns independent invalid-control
regeneration and evidence checks. `a0_tooling_approval.py` owns pre-review and
approved-review validation. Every Python, PowerShell, C++, and Rust source or
test remains ASCII and below 400 physical lines. Rust source splitting still
forbids `include!` and `include_bytes!`.

## Amendment document gate

Document-review attempt 1 is immutable rejected history at the three exact
`document-reviews/attempt-1/` paths. Its canonical envelope binds the reviewed
document byte identities, both reports, both reviewer identities, rejection,
and `superseded_by_attempt=2`; it is not overwritten or treated as approval.
Before source or test edits, two fresh read-only reviewers inspect the revised
pair and publish only at the three `document-reviews/attempt-2/` paths:

1. `control-and-collector.md` checks all 23 control contracts, the independent
   derivation boundary, retained pre-existing identities, and closed topology.
2. `approval-and-deep-verification.md` checks create-once attempt history,
   review-envelope semantics, runner/assembler/verifier gates, the successful
   public entry paths, and the complete positive deep fixture.

The final document approval envelope binds the two current documents, both
attempt-2 reports, the attempt-1 document-review envelope, all four tooling
attempt-1 subjects, and the approved parent repair frame. It records two
literal approvals and `formal_orca_execution_authorized=false`. After this gate
the frozen prerequisite count is 31. A second document rejection consumes the
three attempt-2 document paths and requires an attempt-3 amendment.

## Exact amended manifest groups

The closed `documents` group keeps every existing field and adds exactly
`tooling_repair_specification_role`, `tooling_repair_plan_role`,
`tooling_repair_rejected_document_review_roles`,
`tooling_repair_rejected_document_review_envelope_role`,
`tooling_repair_document_review_roles`, and
`tooling_repair_document_approval_envelope_role`. The two review-role lists are
ordered control then approval/deep and bind document-review attempts 1 and 2.

The closed `tooling` group keeps `python_role`, `wire_role`, `source_roles`, and
`test_roles`; removes the old `test_result_role` and three unversioned
`tooling_review_*` fields; and adds exactly `attempt_1_test_result_role`,
`attempt_1_source_ledger_role`, `attempt_1_review_report_role`,
`attempt_1_review_envelope_role`, the corresponding four `attempt_2_*` roles,
`attempt_2_fixed_probe_evidence_roles`, and `active_review_attempt`.

`active_review_attempt` is integer 2. `source_roles` binds all 58 ledger paths
in ledger order. Attempt-1 fields bind the two immutable base paths then the two
versioned paths. Attempt-2 fields bind ledger, transcript, report, and envelope.
`attempt_2_fixed_probe_evidence_roles` binds, in order, the five exact
`coverage-repair/direct-probe/` subjects followed by
`fixed-probe/evidence/post-review-final/run-1.orca22v` and `run-2.orca22v`.
Every role remains reachable exactly by the artifact registry.

Repair/approval APIs accept explicit resolved `evidence_root` and
`repository_root` parameters, defaulting to the production roots. The assembler
CLI exposes the same two absolute-root options. The manifest destination must
be exactly `<evidence_root>/sidecar-manifest-v1.json`; aliases and symlinks fail.
Tests use this public seam rather than patching globals. Subject and deep
verification receive the same roots and run repair validation for that exact
destination, including temporary roots.

## Tooling review envelope

Every subject identity is a closed object with exactly `root`, `path`, `bytes`,
and `sha256`. `root` is `a0` for ignored evidence and `repository` only for the
two tracked amendment documents. Paths are normalized forward-slash relative
paths beneath the selected root. Absolute paths, `..`, aliases, symlinks,
wrong case, extra keys, duplicate paths, and wrong roots fail.

Both attempt envelopes are canonical ASCII JSON with sorted keys, two-space
indentation, and one final newline. They have exactly:

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

`schema_version` is 1, `kind` is `task22o-a0-tooling-review`, and `reviewer` is
exactly `/root/task22o_a0r_tooling_six_axis_review`. `axis_verdicts` is a closed
object with these six keys:

```text
requirements_completeness
logical_correctness
boundary_cases
code_quality
test_coverage
actual_execution_results
```

Attempt 1 has `attempt=1`, `state=rejected`, `verdict=REJECT`, the four blocking
IDs, all seven required repairs, no resolved repairs, six `REJECT` axes,
`subjects_mutated_after_review=false`, and formal authorization false. Its
subjects are exactly the immutable source ledger, mock transcript, and
attempt-1 report.

Attempt 2 has `attempt=2`. Approval requires `state=approved`,
`verdict=APPROVE`, no blocking IDs, all seven required and resolved repair IDs
in fixed order, six `APPROVE` axes, `subjects_mutated_after_review=false`, and
formal authorization true. Its subjects are exactly the attempt-2 source
ledger, mock transcript, review report, attempt-1 review envelope, attempt-2
document approval envelope, and a seven-entry fixed-probe evidence list in the
manifest order above. Every identity is rehashed by each approval preflight. A
rejection is retained create-once with state and
verdict `rejected`/`REJECT`, formal authorization false, and requires a separate
attempt-3 amendment; attempt-2 files are never overwritten.

## Attempt 2 ledger, transcript, and report

`attempt-2/source-files.sha256` is canonical ASCII/LF with one final newline.
It has exactly 58 lexicographically path-sorted unique rows. Each row is
`64-lowercase-hex`, two spaces, a positive physical LOC, two spaces, and the
exact relative path. Validation recomputes the SHA-256 and LOC of every current
regular non-symlink source and test, requires the exact path set, ASCII bytes,
and LOC below 400.

`attempt-2/mock-test-results.txt` begins exactly:

```text
TASK22O A0 TOOLING REVIEW ATTEMPT 2 MOCK TEST RESULTS
FORMAL_ORCA_EXECUTED=false
```

It binds the ledger SHA and contains closed `SECTION`, exact `COMMAND`, complete
captured output, `EXIT_CODE=0`, and `SECTION_END` records for unittest, CTest,
PowerShell AST/no-`$args`, Python AST, ASCII/LOC, forbidden macros, and 31 frozen
prerequisites. Unittest discovery is independently enumerated; the verbose
result IDs must equal that complete discovered set, every result is `ok`, no
skip/expected-failure is allowed, and `Ran N tests` equals the set size. CTest
is exactly 1/1 and 100 percent. Static results are exactly Python AST 39,
PowerShell AST/no-`$args` 7, and ASCII/LOC 58.

The transcript must contain these exact successful fully qualified tests:

```text
test_assembly_integration.SidecarAssemblyIntegrationTests.test_build_outputs_public_entry_succeeds_and_cli_is_create_once
test_assembly_integration.SidecarAssemblyIntegrationTests.test_build_outputs_rejects_mutations_without_publication
test_qualification_contract.PathControlTests.test_all_23_invalid_controls_bind_specs_definitions_commands_trees_and_outputs
test_run_catalog.RunCatalogTests.test_preexisting_control_paths_are_preserved_and_other_residue_rejected
test_tooling_approval.ToolingApprovalTests.test_review_gate_rejects_remove_substitute_mutation_and_bad_verdict
test_qualification_contract.FixedQualificationOrchestratorTests.test_formal_runner_blocks_before_candidate_creation_without_approval
test_manifest_deep.DeepManifestTests.test_deep_verification_positive_fixture_reaches_every_boundary
test_manifest_deep.DeepManifestTests.test_deep_verification_boundary_mutations_fail
```

The sole terminal line is `FULL_RESULT=PASS` at physical EOF.

The attempt-2 report identifies the reviewer, attempt, source-ledger SHA, and
mock-results SHA. It contains exactly six `AXIS: <id>: APPROVE` lines and seven
`REPAIR: <id>: RESOLVED` lines for approval. Its sole final nonempty verdict
line is `VERDICT: APPROVE`. Missing, repeated, conflicting, or rejected verdicts
fail.

Pre-review validation checks the ledger and transcript without requiring the
not-yet-created report/envelope. Full approval validation additionally checks
the report and envelope. Assembly, subject verification, and deep verification
independently run full approval validation. The formal PowerShell runner's first
child process, before reading qualification inputs or starting any input-derived
tool, is exactly the approval CLI under bundled Python 3.12.13 at
`C:\Users\Indexyz\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe`,
91,648 bytes, SHA-256
`3c6a206b7d93cca823934a83732220dcffd413fd1036d9fb82eebb64599cf7f3`.
The runner also freezes and rehashes the approval-tool path/SHA after its bytes
stabilize. It checks exact argv, tooling working directory, exit 0, canonical
approval-summary stdout, and empty stderr before `Assert-A0CanonicalRolePlan`.
Failure starts no input-derived process and leaves a full preflight snapshot
byte-identical.

## Exact invalid-control contract

Invalid roles remain ordinals 25 through 47 in the frozen role order. The
validator derives each expected definition from `control_id`; it never accepts
coordinated agreement among mutable JSON files as proof.

The exact categories are:

- activation 25-29: `v-with-o-absent`, `v-empty-with-o-absent`,
  `both-present-empty`, `v-empty-with-o-valid`, `o-empty-with-v-valid`;
- relative 30-32: `o-relative`, `v-relative`, `both-relative`;
- freshness 33-36: O final, O temp, V final, V temp pre-exists;
- alias 37-43: final/final, casefold, O-final/V-temp, O-temp/V-final,
  dot-parent, junction/real-parent, and short/long-parent;
- invalid Windows names 44-47: trailing dot, trailing space, O `NUL`, V `NUL`.

For every invalid leaf, validation regenerates the entire closed formal run
spec, including executable, run/workspace/root-log paths, archive and tokens,
argument prefix/argv, datadir/hash tool, postprocess, development flag, timeout,
and poll fields. It also requires the exact role and control ID; a
closed `control-definition.json`; independently derived environment, observe,
expected-path, pre-existing, unsafe-name, and alias fields; a closed run spec
bound by `STARTED.json`; one terminal poison argument; exact command and
environment expansion; process and non-dummy process-tree evidence; four exact
O/O-temp/V/V-temp observations repeated unchanged across all polls; no G-code,
wire, parser, comparison, or tool artifact; and an unchanged datadir. The
validator freshly reruns the frozen no-follow hasher, requires current,
before, after, TSV, JSON, artifact, and base identities to agree, and requires
an exactly empty diff even under coordinated evidence mutation.

Only its owning freshness control may retain one regular file. Before/after
identity and bytes must equal exactly:

| Control | Bytes | SHA-256 |
| --- | ---: | --- |
| O final `preexisting O final\n` | 20 | `8095077fd2d9297587dff97c76f2efce0ba68944bc3be0d2c74ccdfff956fa59` |
| O temp `preexisting O temp\n` | 19 | `2c22a6357830c4d8198821386ece75ad770511023d9bd6946001b3ded078945e` |
| V final `preexisting V final\n` | 20 | `3d205cf1e9f5ef8680e164862551dde567bf5c39368a27bc77352fd568118433` |
| V temp `preexisting V temp\n` | 19 | `0072884717349486d02467ba5a5b3070c15ff0c3bba805bee77e6a6a82783147` |

All other final and temp observations must be truly absent under the no-follow
probe. Trailing-dot, trailing-space, and DOS-device raw names are not probed;
only their safe observation sentinels are checked. Junction and short aliases
are revalidated by current handle identity rather than trusted booleans.

Candidate `STARTED.json` and `8dot3-preflight.json` have closed schemas and bind
the exact qualification-input identity, candidate ID, retry/overwrite policy,
controls root, and current freshly recomputed distinct short/long handle
identity. The controls root contains exactly the 23 fixture directories. Each contains
only its definition except the four exact retained files and the exact
dot/junction topology. Additional files, directories, reparse entries, unsafe
device spellings, aliases, or control IDs fail. Command/process-tree evidence
must bind the installed executable and root PID, prove an acyclic descendant
tree and bounded empty final sample, and show the poison argument once at the
end. A real immediate-exit PowerShell RED requires the producer to retain the
root member, executable/command, ancestry, and final empty sample even when the
guard exits before the first CIM snapshot; validation is not weakened.

## Public assembly and complete deep verification

`build_outputs()` must assign the nine manifest groups before group validation.
A successful public-entry test uses the real validators and assembler against a
temporary exact closed-set root through explicit root parameters, validates canonical bytes, invokes the CLI to
publish once, and proves a second invocation cannot mutate the destination.
Candidate, group, review, artifact, and destination mutations fail before
publication.

The complete positive deep fixture calls the real public assembler and
`verify_manifest(..., require_approval=False, deep=True)` without stubbing a
production validator. It reaches formal build-result validation, deterministic
qualification-input regeneration, all 71 leaves, all 23 controls, coverage,
exact corpus regeneration, workspace integrity, repair group/closed-set checks,
and manifest role reachability. Independent mutations at each boundary fail.
Dummy run specs, control definitions, commands, process trees, or environment
values are forbidden.

## Exit criteria

This amendment is complete only when:

1. both amendment document reviews and their detached envelope approve;
2. attempt-1 subjects and rejection remain byte-identical and bound;
3. every required RED fails for the intended reason before its repair;
4. all public entry, 23-control, review-gate, and positive deep tests pass;
5. all 58 ledger subjects are ASCII and below 400 LOC and all static gates pass;
6. the complete attempt-2 ledger and mock transcript are published once;
7. the same independent reviewer approves all six axes and seven repairs;
8. the attempt-2 envelope passes runner, assembler, subject, and deep gates;
9. no formal Orca process occurred before item 8; and
10. no out-of-scope tracked or fixed-source bytes changed.
