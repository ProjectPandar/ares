# Task 22O Package A0 Tooling Review Attempt 6 Recovery Plan

## Objective

Preserve the consumed Attempt-5 document gate unchanged, create and validate a
fresh canonical Attempt-6 document envelope before its create-once write,
version existing tooling directly to Attempt 6, and repeat final verification
with two unpublished ledger builds before publication. Add no source or test
module and start no formal Orca process before full approval.

## Working rules

- Preserve all 20 prior subjects byte-for-byte.
- Treat `noncanonical-attempt-5-document-envelope-key-order` as the exact
  Attempt-5 blocker.
- Do not normalize, replace, or authorize from the malformed Attempt-5 envelope.
- Use only fresh Attempt-6 document and final paths.
- Build and validate canonical document-envelope bytes in memory before writing.
- Add no source module, test module, required ID, Rust source, Cargo change,
  slicing behavior, architecture change, roadmap feature, or workflow change.
- Keep exact counts 60/41/7/24/107/11/31 and every source below 400 LOC.
- Require production discovery and `validate_test_sections` to use
  `count == 107`, never `count >= 107`.
- Keep final Attempt-6 evidence absent until full qualification and both
  unpublished ledger builds pass.
- Do not call real-root `validate_repair_closed_set()` until all four final
  Attempt-6 artifacts exist.
- Stage outside the repository and A0 closed set.
- Never touch `main.obj`.
- Keep formal Orca prohibited until complete Attempt-6 approval.

## A0R6.1: freeze Attempt-5 invalid history

1. Rehash the Attempt-2 four, Attempt-3 four, and Attempt-4 eight subjects.
2. Verify the Attempt-5 specification is 18,530 bytes with SHA-256
   `499241c194f35ba2d1ce8db6549a9178a7c750fb5ac29afa8204f31edc10a12d`.
3. Verify the Attempt-5 plan is 16,261 bytes with SHA-256
   `48d3f13ffd651773e42b83b1715e85d796656784a80b692a08abd6e5e82cadf2`.
4. Verify the Attempt-5 document review is 2,451 bytes with SHA-256
   `0328ec37dc90b4fdaa8ecebd07594dfe34f8687e55e8e7804c338c9e1abdff46`.
5. Verify the Attempt-5 envelope is 5,174 bytes with SHA-256
   `fb9825145a5d2dcc5f048923c8c5a71866efe313b75b5824285e62508b147d2a`.
6. Parse the malformed envelope and confirm its correct 13-key set and 19
   identities.
7. Require its bytes to differ from sorted-key canonical serialization.
8. Require `implementation_authorized` to precede the three immutable Attempt
   keys in its physical top-level order.
9. Confirm no Attempt-5 final ledger, transcript, review, or envelope exists.
10. Confirm no source edit or formal process followed the invalid gate.

Verification: exactly 20 immutable subjects exist and Attempt 5 authorizes
nothing.

## A0R6.2: create and review Attempt-6 documents

1. Create the Attempt-6 specification and plan at their fresh dated paths.
2. Require ASCII, LF-only, one final newline, regular non-reparse files, and
   fewer than 400 physical lines.
3. Confirm both documents retain the fixed Orca tag, commit, tree,
   `PerimeterGenerator.cpp`, `process_classic()` lines 1144-1692, and fixed
   Voronoi boundary.
4. Confirm all four Attempt-6 final evidence paths are absent.
5. Send both hashes and all 20 prior identities to
   `/root/task22o_a0r_tooling_six_axis_review`.
6. Require the exact two document lines, five approved checks, and sole final
   `VERDICT: APPROVE`.
7. Persist the response create-once at
   `coverage-repair/tooling-review/attempt-6/document-review.md`.
8. Do not edit source or tests yet.

Verification: the review is complete and the fresh envelope path is absent.

## A0R6.3: canonicalize before the document-envelope write

1. Construct one complete envelope object in memory with exactly 14 keys.
2. Bind current specification, plan, and review identities.
3. Bind four Attempt-2, four Attempt-3, eight Attempt-4, and four Attempt-5
   identities, for exactly 23 nested identities.
4. Set approved state, exact reviewer, no subject mutation, true implementation
   authorization, false formal Orca authorization, and approve verdict.
5. Serialize with sorted keys, two-space indentation, ASCII, and one newline.
6. Parse the proposed bytes and validate the closed schema, semantics, paths,
   identities, and exact nested counts.
7. Reserialize the parsed object and require exact byte equality.
8. Require the physical top-level order:
   - `documents`;
   - `formal_orca_execution_authorized`;
   - `immutable_attempt_2`;
   - `immutable_attempt_3`;
   - `immutable_attempt_4`;
   - `immutable_attempt_5`;
   - `implementation_authorized`;
   - `kind`;
   - `review_report`;
   - `reviewer`;
   - `schema_version`;
   - `state`;
   - `subjects_mutated_after_review`;
   - `verdict`.
9. Only after all checks, write the exact bytes with exclusive create semantics
   to the Attempt-6 document-envelope path.
10. Read back and require exact proposed bytes and hash.
11. Rehash the 20 prior plus four current document-gate subjects.
12. Require all 24 unchanged.

Any failure before the write leaves the envelope path absent. Any failure after
exclusive creation consumes Attempt 6 and requires Attempt 7.

Verification: implementation starts only from a canonical approved envelope.

## A0R6.4: RED existing public Attempt-6 expectations

Update existing tests and fixtures before production to require:

1. Attempt-6 current evidence paths and approval summary;
2. the invalid Attempt-5 envelope as immutable rejected history;
3. the approved Attempt-6 document envelope;
4. attempt 6 and six ordered repair IDs;
5. the exact 11-key final subject set;
6. `active_review_attempt=6`;
7. Attempt-5 and Attempt-6 document roles;
8. Attempt-6 final tooling roles only;
9. no Attempt-5 final tooling roles;
10. exact 13/106/12/7 closed-set arithmetic and tooling prefix 30;
11. unchanged 60/41/7/24/107/11/31 topology;
12. exact `count == 107` in production discovery and transcript validation.

Use existing approval, runner, manifest, assembly, integration, and deep tests.
Add no test module or required test ID. Record only clean versioning REDs, not
stale source-pin or fixture setup failures.

## A0R6.5: GREEN approval and frozen-history contracts

1. Version current ledger, transcript, report, envelope, attempt, repair IDs,
   subject keys, and approval summary directly to Attempt 6.
2. Add all four frozen Attempt-5 identities.
3. Validate the malformed Attempt-5 envelope by exact bytes, exact parsed
   semantics, and exact noncanonical-order defect.
4. Reject any Attempt-5 authorization or normalized substitute.
5. Add the approved Attempt-6 document-gate identity and canonical semantics.
6. Make discovery and `validate_test_sections` require exactly 107 tests.
7. Keep the existing 31 prerequisite and 12 nested-identity no-follow gates.
8. Keep pre-review and full approval at their public seams.
9. Run focused approval, history, transcript, prerequisite, and envelope tests
   green.

Keep existing sources below 400 LOC by replacing versioned definitions and
sharing current closed-schema helpers. Add no compatibility fallback.

## A0R6.6: GREEN manifest and assembly contracts

1. Extend repository documents with the Attempt-5 and Attempt-6 pairs.
2. Extend repair additions from 98 with the two actual Attempt-5 document
   artifacts and six Attempt-6 artifacts.
3. Require exact constant arithmetic and synthetic-fixture counts:
   - parent subjects 13;
   - repair additions 106;
   - repository documents 12;
   - exclusions seven;
   - tooling-review prefix 30.
4. Add four ordered Attempt-5 document roles.
5. Add four ordered Attempt-6 document roles.
6. Retain Attempt-1, Attempt-2, and Attempt-4 tooling fields.
7. Add only the five Attempt-6 tooling fields.
8. Add no Attempt-5 test-result, ledger, final-review, final-envelope, or
   fixed-evidence field.
9. Reuse the existing seven fixed evidence roles.
10. Set `active_review_attempt=6`.
11. Retain source roles 60 and test roles 24.
12. Run manifest, assembly, integration, subject, and deep synthetic tests
    green without calling real-root `validate_repair_closed_set()`.

Verification: every actual or planned artifact is classified, no nonexistent
Attempt-5 final artifact is claimed, and the intentionally incomplete real root
is not treated as a failure.

## A0R6.7: finalize both runner pins

1. Finish all Attempt-6 edits to the two pinned Python sources.
2. Compute their final exact bytes and SHA-256 values.
3. Update the existing two-entry runner preflight table.
4. Switch runner stdout to the exact Attempt-6 approval summary.
5. Require both checks before process construction, input access, candidate
   creation, or child launch.
6. Run both independent mutation cases and topology representatives green.
7. Make no later edit to either pinned source or the runner.

Verification: both identities reject independently with no downstream effect.

## A0R6.8: focused and static preflight

1. Confirm all four Attempt-6 final paths remain absent.
2. Require 60 sorted unique sources: 41 Python and seven PowerShell.
3. Require 24 test-role paths, exactly 107 unique tests, and all 11 required IDs.
4. Parse all Python and PowerShell sources.
5. Require ASCII and one to 399 lines for every source.
6. Run the Rust-suffix-only forbidden split-macro scan.
7. Validate all 31 prerequisites and 12 nested identities.
8. Run all affected approval, runner, manifest, assembly, and deep modules.
9. Rehash all 24 immutable document-gate subjects.
10. Verify 13/106/12/7 and prefix 30 with constants and synthetic fixtures.
11. Do not call real-root `validate_repair_closed_set()`.
12. Confirm no formal process or staging residue.

Verification: all focused and static gates pass before final publication.

## A0R6.9: complete two unpublished final builds

Use OS-temporary staging outside the repository and A0 closed set.

1. Confirm no formal process is active and all four final paths are absent.
2. Run exactly 107 tests with no failure, error, skip, or expected failure.
3. Run fixed CTest once and require exactly 1/1.
4. Run the complete 60/41/7/31 gates.
5. Verify 13/106/12/7 and prefix 30 by constants and synthetic fixtures only.
6. Build unpublished ledger A from fresh reads of all 60 final sources.
7. Assemble the proposed Attempt-6 transcript against ledger A.
8. Create complete non-reparse A0 and repository mirror roots.
9. Copy the immutable Attempt-2 base ledger into its exact relative path.
10. Copy all exact 60 source-relative files, including every tooling module
    imported by the mirrored validator.
11. Copy every required current and prior evidence/document subject into its
    exact relative path.
12. Only then place the proposed Attempt-6 ledger and transcript in the mirror.
13. Execute the mirrored production validator against the complete mirror and
    require source count 60 and test count exactly 107.
14. Reject a mirror containing only the proposed ledger and transcript.
15. Build unpublished ledger B from independent fresh reads immediately before
    publication.
16. Require A and B byte-identical and every current source unchanged.
17. Rehash all 24 document-gate subjects.
18. Reconfirm all four final paths absent.

Any edit or failure discards staging and restarts this complete step.

## A0R6.10: publish ledger and transcript create-once

1. Hold the final source set unchanged.
2. Precheck both destinations absent.
3. Publish ledger A create-once at the Attempt-6 ledger path.
4. Publish the validated transcript create-once at the Attempt-6 result path.
5. Read both back and require exact staged bytes and hashes.
6. Run production pre-review against the real roots.
7. Require source count 60 and test count exactly 107.
8. Rehash all current sources against the ledger.
9. Confirm final review and envelope paths remain absent.
10. Do not call real-root `validate_repair_closed_set()` because two final
    artifacts are still absent.
11. Confirm no formal Orca process started.

Verification: publication follows both unpublished builds and cannot overwrite.

## A0R6.11: same-reviewer six-axis loop

Send the exact Attempt-6 ledger and transcript, all 24 document-gate subjects,
the Attempt-5 blocker, and six repair IDs to the same reviewer.

Require review of requirements completeness, logical correctness, boundary
cases, code quality, test coverage, and actual execution results.

Approval output is ASCII, LF-only, and has one final newline. It has exactly 18
nonempty physical lines in this exact order:

1. five header lines;
2. six ordered `AXIS: <axis>: APPROVE` lines;
3. six ordered `REPAIR: <repair>: RESOLVED` lines;
4. sole final `VERDICT: APPROVE`.

Approval permits no blank, proof, blocker, duplicate, or extra line. The parser
must compare the complete physical-line list exactly. Rejection grammar is
separate and may contain its defined blocker and open-repair lines.

The approved canonical 13-key final envelope has exactly 11 subject keys,
attempt 6, all six repairs required and resolved, six approved axes, no blocker
or mutation, true formal Orca authorization, and approve verdict.

Persist review and envelope create-once under `attempt-6`. A rejection consumes
Attempt 6 and requires Attempt 7.

## A0R6.12: real-root closed set and unchanged-subject release gate

After both final review artifacts exist:

1. confirm all four Attempt-6 final artifacts are present;
2. run real-root `validate_repair_closed_set()` for the first time;
3. require exact 13/106/12/7 and tooling prefix 30;
4. run production full approval and require the exact Attempt-6 summary;
5. run formal-runner no-launch preflight;
6. run assembly, subject verification, and deep verification;
7. rehash all current and historical subjects;
8. require no mutation and empty `runs/qualified`.

Only then may the detached Orca build and 71 formal processes begin.

## Completion checklist

- [ ] All 20 prior subjects and the Attempt-5 blocker remain exact.
- [ ] Attempt-6 document-envelope bytes were validated before exclusive write.
- [ ] All 24 document-gate subjects remain exact.
- [ ] Counts are 60/41/7/24/107/11/31 and 13/106/12/7 with prefix 30.
- [ ] Discovery and transcript validation require exactly 107.
- [ ] No Attempt-5 final tooling field exists.
- [ ] Both pinned sources are final before the full run.
- [ ] The pre-publication mirror is complete, not a two-file mirror.
- [ ] Full tests, CTest, static gates, and two unpublished ledgers pass.
- [ ] Ledger and transcript are published only afterward.
- [ ] Final approval is exactly 18 nonempty lines with no extras.
- [ ] Real-root closed-set validation waits for all four final artifacts.
- [ ] The same reviewer approves all axes and repairs.
- [ ] Post-review gates pass without mutation.
- [ ] No formal Orca execution occurs before full approval.
