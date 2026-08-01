# Task 22O Package A0 Attempt-11 Diagnostic Tooling Successor 2 Plan

## Goal

Recover the terminally failed `a22or16` qualification namespace without
replaying it. Build one independently reviewed successor around an ordinary,
complete PowerShell 7.6.4 runtime, prove the repair with TDD, obtain six-axis
approval, then publish one source ledger and invoke one matrix exactly once.

## Non-negotiable boundaries

- Keep `C:\a22or16-driver` and `C:\a22or16-evi` immutable.
- Never rerun `C:\a22or16-driver\run_attempt11_test_matrix.py`.
- Never create or backfill the predecessor `test-results-v1.json`.
- Do not execute from `WindowsApps`.
- Do not warm up, retry, increase 30/120/900000 timeouts, skip AST/compile, or
  substitute Windows PowerShell 5.1.
- Do not execute OrcaSlicer, Attempt-10 watchdog/formal/collector, or any A0
  formal path during successor qualification.
- Do not change Ares slicing logic, KSR inputs, expected G-code, or A0 source.
- Use TDD and keep every reviewed source/test file below 400 physical lines.
- Use real modules; no `include!` or `include_bytes!` source splitting.
- Preserve create-once, no-follow, closed-set, exact-case, and no-retry rules.

## S2.1: Freeze the predecessor terminal failure

1. Rehash the approved 46-file predecessor snapshot and compare it with
   `21d6cf365683f64f43a5b526a1f4da15d0d172cdf6d318f9a58a1bb81df0c298`.
2. Rehash the predecessor spec, plan, document envelope, schema, TDD history,
   source-ledger builder, matrix runner, Store `pwsh.exe`, and published ledger.
3. Verify the published ledger remains 7798 bytes with SHA-256
   `5d6f57e09866a875d92c7243ace460cd87ac56cba074cbb82f58e53b22589345`.
4. Verify predecessor test results, bindings, reports/finals, reservation, and
   receipt remain absent.
5. Verify Attempt-10 formal, claim, candidate, campaign, collector, and
   publication outputs remain absent.
6. Record the exact known failure chain and mark environment/timing/raw-output
   fields unavailable because the predecessor runner did not persist them.
7. Record suite state: first source-ledger process entered; all other fresh
   processes and suites not entered.
8. Record clean process/cache/TEMP post-state.

Verification: a closed failure model exists without changing any predecessor
byte or inventing any omitted evidence.

## S2.2: Approve successor documents

1. Freeze this specification and matching plan identities.
2. Send identical bytes to one independent technical reviewer and one
   independent governance reviewer.
3. Require the technical reviewer to verify runtime closure, path selection,
   environment, timeout, cleanup, matrix transaction, and TDD design.
4. Require the governance reviewer to verify predecessor immutability,
   namespace separation, one-shot semantics, unavailable fields, review
   independence, and terminal failure handling.
5. If either returns `REVISE`, edit only these documents and repeat both
   reviews on a new document-review iteration.
6. After dual `APPROVE`, create
   `C:\a22or17-evi\document-reviews` through ordinary non-reparse parents.
7. Create/read back one document-publication reservation.
8. Persist both verbatim verdicts and one canonical final envelope binding the
   exact documents and reviewers.
9. Validate the closed document DAG and create/read back one pass/fail terminal
   receipt.
10. Treat any reservation, partial output, failed receipt, or missing receipt
    as terminal for these successor roots.

Verification: the final envelope approves identical bytes before
`C:\a22or17-driver` or `C:\a22or17-runtime` exists.

## S2.3: Freeze the predecessor failure-packet contract

1. Freeze the exact closed keys, identities, argv, exception chain, suite
   state, post-state absences, and unavailable-field reasons in the approved
   documents.
2. Define the canonical target path as
   `C:\a22or17-evi\failure\predecessor-matrix-failure-v1.json`.
3. Do not publish the packet until its schema, renderer, validator, and fault
   tests reach GREEN.
4. Use only temporary test fixtures before production publication.

Verification: RED/GREEN can implement a closed packet without inventing or
mutating predecessor evidence.

## S2.4: Create the mechanical successor driver

1. Confirm `C:\a22or17-driver` is absent under follow and no-follow checks.
2. Mechanically copy the exact 46-file predecessor driver without edits.
3. Rehash the copy against the predecessor ordered snapshot digest.
4. Verify no missing, extra, reparse, alias, case, text-hygiene, or LOC drift.
5. Do not create the portable runtime or apply any production repair yet.

Verification: RED has an exact isolated driver in which to fail, while every
predecessor byte remains immutable.

## S2.5: Write deterministic RED tests

1. Add a runtime-manifest contract test that rejects the existing
   `WindowsApps` executable path.
2. Add complete closure tests for missing, extra, mutated, reparse, alias,
   directory-drift, case-drift, and executable-only bindings.
3. Add cross-entrypoint tests requiring builder, matrix, bridge, PowerShell
   tests, production binding, DAG, bootstrap, and receipt verifier to select
   one canonical runtime and environment.
4. Add a deterministic injected AST timeout test.
5. Prove timeout invokes one child, no compile, no retry, no later suite, no
   test-results publication, and complete process cleanup.
6. Add matrix reservation and terminal-receipt lifecycle tests for success,
   nonzero, timeout, interruption, write/flush/readback faults, collision, and
   stale partial output.
7. Add exact-environment tests for missing, extra, reordered, value-drifted,
   case-drifted, and ordinal-ignore-case duplicate pairs at every direct and
   nested child boundary; poison the parent environment and inspect each child.
8. Add source-review lifecycle tests proving no terminal receipt can precede
   the create-once external verdict and complete-iteration validation.
9. Add prospective source-ledger tests proving reviewed bytes are the only
   publishable bytes, exclude every per-run field, and fail on later
   non-publishing builder drift.
10. Add matrix-parent tests proving the reservation and terminal receipt bind
    exact argv-literal identities and revalidate the passed
    qualification-publication transaction only after reservation durability.
11. Add pre-launch raw-stream/journal collision and I/O fault tests proving zero
    children start, same-handle readback rejects path substitution, and later
    terminal-publication faults preserve raw bytes.
12. Prove every terminal state rejects a second invocation.
13. Prove all predecessor subjects remain unchanged.
14. Record exact failing test IDs and outputs in successor TDD history.

Verification: RED fails for the current Store path, inherited environment,
missing runtime closure, and missing durable matrix terminal evidence. The RED
uses injected process boundaries and does not invoke the live matrix.

## S2.6: Implement complete successor GREEN

1. Apply only the changes required by the approved successor documents and RED.
2. Add one immutable manifest owning every PowerShell and environment path.
3. Add a no-follow runtime installer/validator that enumerates the complete
   source and destination closures.
4. Reach synthetic GREEN for installer reservation, copy, readback, cleanup,
   terminal receipt, collision, and fault behavior.
5. Implement failure-packet, source-ledger, source-review, qualification
   publication, matrix reservation/receipt, raw streams, nested journals,
   production binding, DAG, bootstrap, and receipt-verifier changes.
6. Reach synthetic GREEN for all successor code before live runtime install.
7. Clear every direct and nested child environment and use the specification's
   exact ordered eight key/value pairs byte-for-byte.
8. Use the one literal PATH value with portable PowerShell first, exact Git
   second, and `C:\Windows\System32` third.
9. Keep child timeout 30 seconds, matrix suite timeout 120 seconds, diagnostic
   timeout 900000 ms, no retry, and production native-job semantics.
10. Split files before 400 lines and keep tests in separate modules.

The immutable ordered environment for steps 7 and 8 is exactly:

```text
PATH=C:\a22or17-runtime\PowerShell-7.6.4;C:\Program Files\Git\cmd;C:\Windows\System32
PYTHONDONTWRITEBYTECODE=1
SystemRoot=C:\Windows
TEMP=C:\a22or17-evi\successor-temp
TMP=C:\a22or17-evi\successor-temp
WINDIR=C:\Windows
POWERSHELL_UPDATECHECK=Off
POWERSHELL_TELEMETRY_OPTOUT=1
```

Verification: all synthetic GREEN tests pass, the closed source set contains
no `WindowsApps` literal outside frozen failure fixtures, and no live runtime
or production artifact exists.

## S2.7: Approve and install the successor runtime

1. Freeze the exact installer-owned immutable set listed in the specification.
2. Persist an independent focused technical review at
   `C:\a22or17-evi\runtime-install\focused-review.md`.
3. Publish a canonical focused envelope at
   `C:\a22or17-evi\runtime-install\focused-review-envelope-v1.json`.
4. Require the envelope to bind every source/test identity, synthetic TDD
   result, expected closure, reviewer identity, and `VERDICT: APPROVE`.
5. Confirm `C:\a22or17-runtime` and the runtime destination are absent under
   follow and no-follow checks.
6. Create/read back
   `C:\a22or17-evi\runtime-install\reservation-v1.json` before source validation
   or copy.
7. Require the reservation to recursively bind the focused envelope, immutable
   set, argv/cwd/environment, roots, closure, and output paths.
8. Copy the reviewed 1007-file, 56-directory PowerShell 7.6.4 closure once to
   `C:\a22or17-runtime\PowerShell-7.6.4`.
9. Verify 297236691 total bytes, zero reparses, ordered file digest
   `c9b6ed7678c708cdc0ca00c99bc33dd5b2aca5afcaf1c2848c87c468f70f7ec0`,
   and ordered directory digest
   `942f5668ddceb1fed52b4d23d193cc2c3328e613a2eee2595212625f3b76cb93`
   using the specification's unsigned-ASCII-byte relative-path ordering and
   compact JSON serialization; reject any non-ASCII relative path.
10. Create/read back
    `C:\a22or17-evi\runtime-install\terminal-receipt-v1.json` after cleanup.
    A failed or missing receipt consumes the namespace.
11. Freeze in the install receipt every exact readback input needed to render
    one deterministic destination runtime-ledger candidate later; do not render
    or publish the production ledger before the source-review iteration.
12. Verify downstream renderers recursively bind focused review, installation
    reservation/receipt, and the runtime closure.

Verification: one approved installer transaction produced one exact portable
runtime, and every installer-owned byte remains frozen.

## S2.8: Run focused non-publishing verification

1. Run bundled Python with `-B` and `PYTHONDONTWRITEBYTECODE=1`.
2. Run all successor Python unit tests.
3. Run native and completion PowerShell suites only through the portable
   runtime and canonical environment.
4. Parse every Python source with `ast.parse`.
5. Parse every PowerShell source with the portable runtime.
6. Compile every C# source with a fresh portable-runtime process.
7. Run at least four fresh-process repetitions required by the existing
   qualification contract.
8. Validate JSON canonicality, LF-only text, ASCII source, closed source set,
   no reparses, and maximum 399 lines.
9. Rehash the complete runtime before and after.
10. Verify each direct suite child creates stdout/stderr files and opens their
    binary redirection handles before launch, then flushes, seeks, reads, and
    hashes through those same still-open handles before close.
11. Verify nested source-ledger AST/compile children use separate create-once
    journal reservations, terminal journal receipts, and raw streams, including
    timeout partial output, failed `child_started=false` receipts after
    post-reservation pre-launch faults, and zero child starts.
12. Render and validate candidate predecessor failure-packet, runtime-ledger,
    and prospective source-ledger bytes without a live production builder.
13. Bind the prospective source ledger's absolute target path, complete bytes,
    byte count, SHA-256, inputs, and recursive DAG; reject timestamps,
    durations, process IDs, reservation/stream/journal paths, or other per-run
    fields.
14. Create/read back a new source-review iteration reservation.
15. Persist candidate subjects with the exact source snapshot, schemas, TDD
    history, and test evidence. Leave the iteration open for external review;
    do not create its verdict or terminal receipt yet.
16. Rehash every predecessor subject.
17. Record RED/GREEN evidence and exact suite arithmetic.

Verification: all focused tests pass. The only durable successor outputs are
the approved runtime-install chain and the open source-review iteration;
production failure/runtime/source-ledger and matrix artifacts remain absent,
and process/cache/TEMP state returns to zero.

## S2.9: Obtain exact six-axis source approval

1. Freeze the complete successor source, schema, TDD, failure packet, document
   envelope, and runtime snapshot.
2. Compute the ordered source digest, file count, total bytes, and maximum LOC.
3. Send the exact subjects to the same user-visible independent six-axis review
   task used for Attempt 11.
4. Permit only explicitly listed read-only non-publishing tests, each at most
   once for that snapshot.
5. Require findings first across requirements completeness, logical
   correctness, boundary cases, code quality, test coverage, and actual
   execution results.
6. Persist the external verdict create-once in the open iteration, validate the
   now-complete iteration, then create/read back exactly one pass/fail terminal
   iteration receipt.
7. If the verdict is `REVISE`, return the concrete repair list to the main
   thread, preserve that transaction-valid passed receipt with
   `verdict=REVISE` and `authorizes_publication=false`, repair only those
   findings with TDD, freeze a new snapshot, and create a new ordinal iteration
   before asking the same task to review again.
8. Never mutate an earlier reservation, candidate, verdict, or receipt.
9. If a finding requires changing any installer-owned immutable file, focused
   approval, executed runtime-install transaction, installed runtime, or
   runtime contract, stop and start separately reviewed successor roots.
10. Continue repairable review loops until one complete iteration has an
    `APPROVE` verdict and passed terminal receipt.
11. Create/read back one final source-review envelope selecting that approved
    complete iteration and binding every prior iteration/residue.
12. Treat a missing or failed final envelope as terminal for the namespace.
13. Rehash every subject immediately after approval and reject drift.

Verification: one exact approved snapshot exists and production artifacts are
still absent.

## S2.10: Publish approved subjects and run one matrix

1. Pre-create only the approved ordinary evidence parents.
2. Construct the qualification-publication reservation from immutable
   manifest/review constants and create/read it back before any failure-capable
   validation.
3. Perform the no-follow root, source, runtime, environment, process, cache,
   TEMP, predecessor, and production-absence checks.
4. Publish the approved failure-packet review subject create-once.
5. Publish the approved runtime-ledger review subject create-once.
6. Publish the approved successor source ledger create-once.
7. Read all three back and recursively validate every row, runtime subject,
   post-state, and publication edge.
8. Construct a passed state only after complete validation; otherwise construct
   a failed state.
9. Create/read back one qualification-publication terminal receipt and
   independently verify it. Require a passed receipt to bind the exact matrix
   entrypoint identity, parent-identity argv contract, cwd, ordered environment,
   and three fixed output paths it authorizes.
10. Stop permanently on a reservation, partial output, failed receipt, or
    missing receipt. Do not invoke the matrix.
11. Freeze the exact qualification-publication reservation, unique passed
    receipt, three published output identities, and their complete DAG as direct
    matrix authorization parents.
12. Invoke the successor matrix publishing command exactly once only after an
    independently valid passed publication receipt. Pass every parent's absolute
    path, byte count, and SHA-256 plus the DAG-root identity as fixed argv
    literals; require the matrix reservation to copy those literals before
    opening any referenced artifact or launching a child.
13. Require the matrix to independently revalidate all authorization parents
    before its first suite and bind their before/after identities in its
    terminal receipt.
14. Do not run a production dry matrix before or after it.
15. Monitor the original process/session only; never restart after session-handle
    loss.
16. On success, validate reservation, test results, passed receipt, suite order,
    case/fresh-process arithmetic, raw-output identities, cleanup, and
    before/after source/runtime equality.
17. On failure, preserve reservation, failed receipt or partial bytes, verify
    cleanup, and stop for a separately reviewed recovery.

Verification: exactly one passed qualification-publication receipt, one matrix
reservation, and one passed matrix terminal receipt exist, with no predecessor
or A0 mutation.

## S2.11: Resume Attempt-11 diagnostic preparation

1. Publish one complete successor production binding.
2. Obtain independent tooling review and final envelope using the existing
   complete iteration mapping.
3. Recursively validate document, failure, source, runtime, tests, binding, and
   review subjects.
4. Bootstrap the Attempt-11 diagnostic reservation once.
5. Invoke the diagnostic child once with the frozen Attempt-10 validator
   command and 900000 ms hard timeout.
6. Independently verify the complete terminal diagnostic receipt.
7. Classify only from durable stdout/stderr, timing, exit/timeout, cleanup, and
   recursive DAG evidence.
8. Follow the approved Attempt-11 diagnosis/repair path.

Verification: the diagnostic, not qualification infrastructure, supplies the
next authoritative decision.

## Checklist

- [ ] Predecessor 46-file snapshot and source ledger remain exact.
- [ ] Predecessor matrix is never rerun or backfilled.
- [ ] Technical and governance reviewers approve identical successor docs.
- [ ] One final document envelope exists.
- [ ] One complete predecessor failure packet exists.
- [ ] Deterministic RED covers Store-path and terminal-receipt gaps.
- [ ] Complete portable PowerShell closure is installed and ledger-bound.
- [ ] Every direct/nested child uses the exact eight-pair cleared environment.
- [ ] Source-review verdict precedes its transaction receipt; `REVISE` is valid
      but non-authorizing.
- [ ] Prospective source-ledger bytes are approved and later published exactly.
- [ ] Child streams use pre-launch handles and split journal transactions.
- [ ] All focused GREEN tests and syntax/compile checks pass.
- [ ] All reviewed files are below 400 lines.
- [ ] Same independent six-axis task approves the exact successor snapshot.
- [ ] One qualification-publication transaction passes exactly once.
- [ ] Matrix reservation/receipt bind that publication as their direct parent.
- [ ] One successor source ledger is published.
- [ ] Exactly one successor matrix invocation yields a passed receipt.
- [ ] No forbidden A0, OrcaSlicer, formal, claim, candidate, or collector action
      occurs.
- [ ] Attempt-11 diagnostic flow resumes only after the passed matrix.
