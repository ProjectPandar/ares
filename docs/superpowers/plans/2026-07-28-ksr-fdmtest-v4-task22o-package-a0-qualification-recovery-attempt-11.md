# Task 22O Package A0 Qualification Recovery Attempt 11 Plan

> Historical/non-blocking record. Task 22O.1 supersedes this external recovery
> campaign as a production gate. Do not retry it; retain it only as audit
> evidence.

## Goal

Recover Package A0 after the terminal Attempt-10 generated-scope validator
failure without retrying or mutating Attempt 10. First obtain a durable,
independently reviewed diagnostic; then repair only the proven boundary with
TDD, rebuild the current-attempt evidence as Attempt 11, and complete one fresh
atomic qualification and publication path.

## Non-negotiable boundaries

- Never rerun or modify the Attempt-10 watchdog.
- Never overwrite its failed receipt.
- Never create the Attempt-10 claim, candidate, formal transcript, or collector
  outputs.
- Do not invoke the live validator until the matching diagnostic phase has
  independent document and tooling approval.
- Keep the frozen Orca commit/tree unchanged.
- Bind the governing Task 22O specification at 29472 bytes /
  `78c44972e284eb615bf96228cbc5d0fe3a5c731a853c3b1cf518f92219b95674`
  and plan at 35729 bytes /
  `94c361d0d4c89eb5019f07f3a3e4101b8d89857d02c06629e3c794920f645e80`.
- Keep Ares production behavior unchanged during Package A0 recovery.
- Preserve exact scope, no-follow, identity, reviewer independence, cache,
  process, parent, create-once, and no-retry contracts.
- Use TDD for every repair.
- Keep every source file below 400 physical lines.
- Use real modules; do not use source-splicing macros or generated code to evade
  the LOC boundary.
- Do not hardcode an Attempt-11 candidate ID or generated-scope path set.

## A0R11.1: freeze the Attempt-10 terminal failure

1. Rehash the failed receipt, frozen watchdog wrapper/test, iteration-0013
   ledger/results/review, tooling final, prelaunch, launch, static topology,
   build result, and qualification inputs.
2. Validate the receipt as canonical ASCII/LF JSON and record its exact failed
   state, 310-path scope, and clean pre/post state.
3. Verify the Attempt-10 claim, candidate, formal transcript, and both collector
   pairs are absent.
4. Verify the qualified parent contains exactly the two retained candidates.
5. Verify all launch subjects, 180 A0 topology records, 22 repository documents,
   and 312 prelaunch records still match.
6. Verify the exact 490-file non-retained physical closed set, all 26 absolute
   build/input artifacts, 12 source-status records, both 39-DLL runtime
   closures, and probe closure.
7. Reproduce the prospective passed-receipt identity by changing only
   `state`/`failure` in memory; do not write the result.
8. Record that child stderr/stdout and elapsed/process details are unavailable
   because the consumed wrapper omitted them.
9. Record the three remaining live-only candidate boundaries without probing
   them separately: test-ID discovery, PowerShell AST validation, and
   build-regeneration Git under the restricted environment.

Verification: one immutable failure table exists and no Attempt-10 artifact was
created, replaced, or executed.

## A0R11.2: approve the recovery documents

1. Freeze identical copies of this specification and plan under
   `C:\a22or16-evi\document-reviews\iteration-0001`.
2. Obtain independent technical and governance reviews.
3. Require both reviewers to rehash the frozen Attempt-10 evidence and confirm
   that the proposed diagnostic is non-authorizing.
4. If either reviewer returns `REVISE`, create a new review iteration with
   revised exact document copies and an explicit predecessor link.
5. Publish a final document envelope only after both reviewers approve identical
   bytes.

Verification: the final document envelope authorizes A0R11.3-A0R11.4 and
conditionally authorizes only the read-only part of A0R11.5 after a complete
validator-result receipt passes independent validation.

## A0R11.3: TDD the external diagnostic runner

1. Create sources under `C:\a22or16-driver`.
2. Start with synthetic RED tests for:
   - success with exact stdout and empty stderr;
   - nonzero exit with stderr that must survive in the receipt;
   - nonzero exit with empty stderr;
   - arbitrary stdout/stderr bytes;
   - success, nonzero, and timeout with both streams above pipe capacity;
   - arbitrary bytes and descendant-held inherited pipe handles;
   - empty, valid, truncated, and invalid strict-ASCII stderr diagnostics;
   - timeout with child and grandchild;
   - deadline completion with a final authoritative job query;
   - GQCS error;
   - QueryInformationJobObject error;
   - a two-process reservation collision proving one child start;
   - input drift and pre-cleanliness failure after reservation;
   - process-start and incomplete-drain failures;
   - residue;
   - cleanup exception;
   - serializer, write, flush, and readback faults;
   - inherited-environment leakage;
   - missing/extra/order/count/hash/base64/fallback schema mutations;
   - missing/extra/substituted/alias/case-drift working-directory mutations.
3. Implement a suspended-create, assign-before-resume Job Object runner.
4. Start binary asynchronous drains for both streams before resume, keep them
   concurrent, and await both to EOF after authoritative job-zero cleanup.
5. Capture raw bytes and serialize byte count, SHA-256, and base64 only after
   both drains complete.
6. Add a non-authoritative stderr diagnostic derived with strict ASCII from at
   most the first 8192 raw bytes; record truncation or the first invalid byte
   offset without replacement decoding.
7. Record child/wall timing, exit code, timeout, cleanup, exact
   argv/environment/working directory, bound subjects, and pre/post cleanliness.
8. Freeze a reviewed closed receipt-schema artifact with exact ordered keys,
   terminal variants, reservation/fallback grammar, exact working-directory
   field, and coherence rules.
9. Acquire a separate create-once reservation before all failure-capable
   preflight; write/flush/read back that canonical reservation through its
   exclusive handle, then create/write/flush/read back the receipt with another
   create-new exclusive handle and a cleared exact child environment.
10. Keep tests in separate modules and all files below 400 LOC.
11. Run at least four fresh PowerShell processes after the first green run.
12. Parse every PowerShell and Python source and compile every C# source.

Verification: all synthetic cases pass with no TEMP, cache, or process residue.

## A0R11.4: review and run one diagnostic

1. Send the exact diagnostic source ledger, closed schema, RED/GREEN results,
   frozen inputs, and deterministic synthetic receipt identities to an
   independent six-axis reviewer.
2. Return every finding to the main thread, fix it, and repeat review until all
   six axes approve.
3. Rehash all approved subjects and verify production diagnostic output is
   absent.
4. Verify both production reservation and receipt are absent, then invoke the
   diagnostic runner exactly once with a 900000 ms hard timeout and the frozen
   Attempt-10 validator command/environment.
5. Do not invoke the Attempt-10 watchdog wrapper.
6. Validate reservation ownership and the closed canonical receipt
   independently without rerunning the child.
7. Require `validator-result`, complete EOF drains, clean cleanup/poststate,
   coherent raw streams, exact environment, and the ordinary non-reparse,
   non-aliased ordinal working directory before classification.
8. If any infrastructure stage fails, consume Attempt 11 and start a separately
   reviewed recovery attempt; do not classify, repair, or rerun.

Verification: a complete terminal diagnostic receipt exists at
`C:\a22or16-evi\diagnostic\attempt10-validator-replay-v1.json`.

## A0R11.5: classify and reproduce the failure

1. Confirm the independently validated receipt is `validator-result`; otherwise
   stop under A0R11.4's infrastructure-failure rule.
2. Classify the diagnostic as contract, process/cache/environment, timeout, or
   nondeterministic completion.
3. Map the first exact failure to its owning module and function.
4. Record the source-cited diagnosis, rejected alternatives, proposed RED, and
   exact edit boundary in an architecture review note.
5. Obtain two independent approvals of a diagnosis envelope binding that note,
   the complete receipt, exact source identity, and proposed RED.
6. Only then write the smallest deterministic RED test that reproduces that
   boundary without executing Orca or the live validator.
7. For a diagnostic exit zero, audit all live-only boundaries and inject the
   lost-state scenario before selecting a repair.
8. Do not change exact-scope or identity rules to make the test pass.

Verification: an independently readable RED test fails for the diagnosed
reason, not for fixture setup.

## A0R11.6: implement the focused repair

1. Change only the owning validation boundary.
2. Split new behavior into a normal Python module because
   `a0_tooling_approval.py` is already 399 LOC.
3. Preserve existing public entry points unless the diagnosis proves a contract
   change is required.
4. Make the focused RED test green.
5. Run all prior Attempt-7 through Attempt-10 tooling tests plus the new
   diagnostic regressions.
6. Run AST, canonical JSON, ASCII/LF, no-reparse, no-cache, process, and LOC
   checks.
7. Obtain independent focused review of the exact diff and test evidence.

Verification: the diagnosed case and all inherited tests pass, with no unrelated
validator optimization.

## A0R11.7: create the Attempt-11 domain

1. Create `coverage-repair/tooling-review/attempt-11`.
2. Publish a create-once prior-attempt handoff binding the failed receipt,
   absent formal state, diagnostic receipt, and frozen Attempt-10 subjects.
3. Copy the exact Attempt-10 runner into the historical handoff before adapting
   the live runner.
4. Add Attempt 10 to historical readers without weakening their exact schemas.
5. Change current schemas, paths, kinds, process terms, and tests to Attempt 11.
6. Preserve the logical retained candidate order and exact two-child physical
   parent.
7. Derive current generated-scope paths and static topology from actual records.
8. Regenerate publication policy from final source semantics.
9. Preserve every prior test ID and add all new diagnostic/repair IDs.

Verification: no current Attempt-10 literal remains in an Attempt-11-owned
contract except explicit historical references.

## A0R11.8: run complete tooling verification

1. Run all Python tooling tests in deterministic lexical order with `-B`.
2. Run all PowerShell tooling tests in fresh processes.
3. Validate every source file is ASCII/LF, has one final LF, and is below 400
   physical lines.
4. Validate Python AST, PowerShell AST, C# compilation, JSON canonicality,
   no-follow topology, source ledger, historical identities, process absence,
   cache absence, and qualified-parent membership.
5. Run exact-scope validation multiple times from fresh processes and require
   identical output.
6. Do not run pre-review or any live validator in this step.

Verification: one frozen test transcript ends in the sole terminal `OK` marker.

## A0R11.9: prepare fresh A22OR16 build and inputs

1. Build the fixed probe once from the frozen Orca commit/tree and final
   Attempt-11 source ledger.
2. Generate fresh build result and qualification inputs under
   `C:\a22or16-evi`.
3. Validate all four runtime closures, imports, hashes, candidate derivation,
   parent topology, and absence state.
4. Obtain independent build/input review from a reviewer not used for document
   review.
5. Freeze the exact approved build and input identities.

Verification: the candidate ID is derived from fresh evidence and its candidate
directory remains absent.

## A0R11.10: tooling review and launch approval

1. Allocate one create-once tooling review iteration.
2. Bind source ledger, test transcript, focused evidence, diagnostic receipt,
   diagnosis envelope, repair provenance, fresh build/input evidence,
   PowerShell, static topology, publication policy, and prior handoff.
3. Send identical subjects to an independent six-axis reviewer.
4. Return ordered findings to the main thread and repeat until every axis
   approves.
5. Publish final tooling, prelaunch evidence, and launch approval once.
6. Validate exact generated-scope count/hash and candidate absence from a fresh
   process.

Verification: final tooling and launch semantics agree, no formal artifact
exists, and only the exact launch approval authorizes A0R11.11.

## A0R11.11: one watchdog and atomic campaign

1. Rehash every launch subject and approved external watchdog source.
2. Verify watchdog receipt, claim, candidate, formal transcript, collector
   outputs, TEMP roots, caches, and relevant processes are absent.
3. Invoke the live Attempt-11 generated-scope watchdog exactly once.
4. Require exit zero, empty stderr, exact sorted output, exact count/hash, clean
   poststate, and a durable passed receipt.
5. Reserve the formal transcript once.
6. Invoke the Attempt-11 formal wrapper exactly once.
7. Run all 71 ordinals without resume, retry, overwrite, selection, old-leaf
   reuse, or fallback.
8. Require 69 CLI and two direct passes with paired equality and no residue.

Verification: one canonical transcript contains exactly reserved then passed,
and one claim/candidate is present.

## A0R11.12: collect, review, and publish

1. Validate launched generated scope in a fresh process.
2. Invoke the strict collector exactly once with explicit Attempt-11 claim and
   receipt paths.
3. Verify the complete ordered corpus and byte-identical candidate tree before
   and after collection.
4. Stage the publication once.
5. Obtain two independent sidecar approvals over identical stage bytes.
6. Publish final corpus, manifest, reviews, registry, and sidecar once.
7. Make the stage unavailable.
8. Run deep approved verification.

Verification: all published bytes resolve only relative to the frozen
Orca/Ares evidence roots and satisfy the full corpus contract.

## A0R11.13: release and resume Task 22O

1. Record Package A0 release in the Task 22O progress ledger.
2. Resume Package A output model/geometry/Voronoi implementation.
3. Continue Packages B-H in parent-plan order.
4. Keep option values sourced only from the 3MF.
5. Prohibit production/test hardcoding of fixture identity, option values,
   reference G-code bytes/lines/hashes, candidate IDs, or path sets; read the
   reference only at the final comparison boundary.
6. Port every slice from a cited Orca source boundary with no legacy fallback.
7. Remove obsolete source-pinning tests when their Ares behavior replacements
   are covered.
8. Keep real Rust source and separate test modules below 400 LOC; split with
   normal `mod`, never `include!` or `include_bytes!`.
9. Return ordered findings to the main thread and send identical repaired
   subjects back to the same independent six-axis review task until approval.
10. Run `cargo fmt`, `cargo clippy`, and `cargo nextest run --workspace`.
11. Compare Ares KSR G-code to the fixture with only the allowed generator/time
   metadata normalization.

Verification: the parent KSR task closes only on exact G-code parity.

## Checklist

- [ ] Attempt-10 terminal failure and absences are frozen.
- [ ] Two reviewers approved identical Attempt-11 spec/plan bytes.
- [ ] External diagnostic runner passed focused TDD and six-axis review.
- [ ] One diagnostic invocation produced a complete durable receipt.
- [ ] The exact failure boundary has a deterministic RED test.
- [ ] The focused repair and all inherited tooling tests pass.
- [ ] Attempt-11 current/historical topology is exact.
- [ ] Fresh A22OR16 build/input evidence is approved.
- [ ] Six-axis tooling review approved exact launch subjects.
- [ ] One watchdog and atomic campaign passed.
- [ ] Strict collection passed all 69 CLI and two direct leaves.
- [ ] Two sidecar reviewers approved identical stage bytes.
- [ ] Deep verification passed and Package A0 released.
- [ ] Task 22O Packages A-H resumed.
- [ ] Final KSR G-code parity and Rust workspace gates passed.
