# Task 22O Package A0 Qualification Recovery Attempt 11 Specification

> Historical/non-blocking record. Task 22O.1 supersedes this external recovery
> campaign as a production gate. Do not retry it; retain it only as audit
> evidence.

## Status

Draft. This document and its matching plan must receive two independent,
read-only approvals over identical bytes before any Attempt-11 diagnostic,
validator mutation, build, qualification, OrcaSlicer execution, campaign,
collection, or publication may start.

Attempt 10 is terminally consumed. This document does not authorize an
Attempt-10 retry, receipt replacement, claim, candidate, formal transcript, or
collector.

## Purpose

Attempt 10 completed its reviewed tooling and launch preparation, but its sole
live generated-scope watchdog invocation ended with child exit code 1. The
create-once terminal receipt is:

```text
C:\a22or15-evi\watchdog\attempt10-generated-scope-v1.json
28746 bytes
SHA-256 5432e5db6ee41aeb57337abc8142a7cb7d46bffd416206bea89c1c593c423357
state = failed
failure = generated-scope validator exit 1
```

The receipt proves clean pre/post state and preserves the frozen 310-path
generated scope with normalized compact-JSON SHA-256
`f061992b3e47941575d7f2d0649870c896a325a5e76c9358c13482f312460d90`.
It does not preserve the child stderr, stdout, elapsed time, or the failing
validator stage. The exact validator diagnostic is therefore unknown.

Attempt 11 first closes that observability gap with one separately reviewed,
non-authorizing diagnostic execution. It then applies only the repair proven by
that diagnostic, rebuilds the current-attempt A0 domain, obtains fresh build and
input evidence, and authorizes one new atomic qualification campaign.

## Fixed upstream and Ares boundaries

The upstream source boundary remains OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`.

The governing Task 22O documents remain:

- specification: 29472 bytes, SHA-256
  `78c44972e284eb615bf96228cbc5d0fe3a5c731a853c3b1cf518f92219b95674`;
- plan: 35729 bytes, SHA-256
  `94c361d0d4c89eb5019f07f3a3e4101b8d89857d02c06629e3c794920f645e80`.

This recovery is ignored Package A0 oracle-tooling work. It does not change:

- Ares production slicing behavior;
- the fixed OrcaSlicer executable or source tree;
- the 11-case valid matrix or 23-case invalid matrix;
- the Task 22O classic perimeter generator contract;
- the 3MF-only option projection requirement;
- the no-hardcoding and no-legacy-fallback requirements; or
- the exact KSR G-code parity target.

The Rust and Orca source rewrite boundaries remain those in the parent Task 22O
specification. Attempt 11 may change only qualification tooling and evidence
needed to recover Package A0.

## Frozen Attempt-10 terminal evidence

The following identities are immutable Attempt-10 facts:

```text
failed watchdog receipt
  28746 bytes
  5432e5db6ee41aeb57337abc8142a7cb7d46bffd416206bea89c1c593c423357

watchdog production wrapper
  C:\a22or15i13-watchdog-driver\run_attempt10_generated_scope_watchdog_once.ps1
  7646 bytes
  b000397abb0d96bc601a40959fac492b400c3c788806988c89647ec3784ae02b

watchdog terminal test source
  C:\a22or15i13-watchdog-driver\tests\test_attempt10_watchdog.ps1
  11986 bytes
  bfd92267b91a071f5304e5eac1f601c654c1d393a80a6820f4c4c924d387cfdc

Attempt-10 iteration-0013 source ledger
  12015 bytes
  8dc9aca11b4e5cc6efcdcddb071ce0979c37b216886a7904e5ece924d397b8ec

Attempt-10 iteration-0013 test results
  77879 bytes
  26b48c8d266b07a1ba8f292e851d43533816b63ed267b1dd9d8bec2b64b6ba2d

Attempt-10 iteration-0013 six-axis review
  2029 bytes
  bf116046efc9af344362afb7e1ed3fa76b42468b46e6dc856fc36e3b50b6a925

Attempt-10 iteration-0013 review envelope
  4821 bytes
  0f7a34499501254fb9efd8eead5aaba8a6b1eabd772c12b7295a5d8f7dd5718b

Attempt-10 tooling final
  5040 bytes
  300898f2b96e04fc6d44bef24a05c3d677e8cbff5d4b4f4b6f70a7e09abf2bdf

Attempt-10 prelaunch
  105531 bytes
  8868ee948944609b0c9ddbb30fea2286ba4ed825c1776d2da6187fabac1dde83

Attempt-10 launch approval
  3525 bytes
  ba4df8e0696fa29e46ee3362260739d71b21533beecc3a27ed4702d47c1155bd

Attempt-10 static topology v13
  102088 bytes
  b3f6110e6d9770df780147d4b94f2c8b0893b39437028133ffc2e0e254e2b8d0

A22OR15 I13 build result
  80980 bytes
  54ee6fdfd2a3063461fa5b2c3d68cfc91704196f7bbf1ef1a2cdb23bc33f46ec

A22OR15 I13 qualification inputs
  61230 bytes
  1f232923c8490decb6da0864ce0c6bf6cf2c5bd3c860d3558450f3f3fb7e82be
```

The Attempt-10 claim, candidate `3754780b0fa4874d`, formal transcript, and both
collector pairs were absent when the watchdog reserved its receipt and remain
forbidden. The qualified parent contains only retained candidates
`11d07a6fd071c211` and `8a5aab7461b877b1`.

Never edit or rerun anything under `C:\a22or15i13-watchdog-driver`, overwrite
the failed receipt, invoke the Attempt-10 formal wrapper, create its claim or
candidate, or invoke either Attempt-10 collector.

## Confirmed failure boundary

Read-only postfailure validation established all of the following:

- all 12 launch subjects still match their approved bytes and hashes;
- the I13 source ledger still has 114 source rows;
- the static topology still binds 180 A0 records and 22 repository documents;
- the prelaunch evidence still has 312 exact records;
- the generated scope remains 310 paths with the frozen normalized hash;
- the no-follow physical closed set contains exactly 490 non-retained files,
  with no extra or missing path and no invalid directory or reparse point;
- all 26 absolute build/input artifacts, 12 source-status records, both
  39-DLL runtime closures, and probe closure still match;
- the launch candidate, claim, formal transcript, and collector artifacts are
  absent; and
- the qualified parent still has exactly the two retained children.

The failure is not attributed to those frozen inputs.

An in-memory-only substitution of the receipt's `state` from `failed` to
`passed` and `failure` from the generic exit message to `null` produces exactly
28716 bytes with SHA-256
`26cabda1230cdab2b7ccdddba983d84ecf47446383c0dae7a39a87f283e62105`,
the frozen prospective passed-receipt identity. No file was modified. This
confirms that the receipt's bound subjects, scope, timeout, and cleanliness
fields all reached their expected terminal values.

The external watchdog checks child exit code before stderr and constructs its
terminal receipt without process output fields. A nonzero child therefore
collapses every validator error to `generated-scope validator exit 1`. Attempt
11 must not guess the failing assertion and must not optimize unrelated
validator code before collecting a durable diagnostic.

The remaining live-only candidate boundaries are the 30-second
`discover_test_ids` subprocess contract, the 30-second PowerShell AST
subprocess contract, and build-regeneration Git execution under the restricted
six-variable environment. These are hypotheses for diagnosis, not permission
to probe them separately or change their contracts.

## Attempt-11 document gate

Document review evidence lives outside A0 under:

```text
C:\a22or16-evi\document-reviews\
```

Each review iteration contains exact copies of this specification and plan,
one technical review, one governance review, and a canonical envelope. Two
distinct independent reviewers must approve identical document bytes. The
final document envelope authorizes the diagnostic-tool TDD/review and one
diagnostic invocation. It conditionally authorizes read-only classification
only when a complete validator-result receipt passes independent validation.
It does not authorize a validator repair, Attempt-11 adaptation, build,
qualification, or publication.

Document review may read Attempt-10 evidence and source but may not execute the
live validator or mutate A0.

## One durable diagnostic execution

After document approval, create a new external diagnostic tool under:

```text
C:\a22or16-driver\
```

All source and test files must be ASCII, LF-only, end in exactly one LF, and
remain below 400 physical lines. PowerShell source must pass AST parsing,
Python source must pass `ast.parse`, and C# source must compile in every fresh
test process.

The diagnostic runner shall:

- create a Windows Job Object with `KILL_ON_JOB_CLOSE`;
- create the child suspended, assign it to the job, then resume it;
- open binary stdout and stderr drains before resume and drain both
  concurrently from process start without lossy decoding;
- after authoritative active-process count reaches zero, await both drains to
  EOF before hashing output or constructing the receipt;
- store byte count, SHA-256, and base64 for each stream as the authoritative
  output evidence;
- derive a non-authoritative, strict-ASCII stderr diagnostic from at most the
  first 8192 raw bytes, recording whether it was truncated or the first invalid
  byte offset instead of replacing or decoding invalid bytes;
- terminate and wait for the complete job on timeout or any failure;
- use active process count as the authoritative descendant-completion check;
- acquire a separate create-once invocation reservation before bound-input,
  cleanliness, receipt creation, or child-start work;
- prove that two racing launchers start at most one child;
- create the receipt with create-new semantics and one exclusive handle;
- force-flush and read back the complete terminal receipt through the same
  exclusive handle; and
- clear the child environment and supply only the reviewed exact allowlist.

Focused TDD must cover at least:

- success with exact stdout and empty stderr;
- nonzero exit with nonempty stderr retained byte-for-byte;
- nonzero exit with empty stderr;
- non-ASCII stdout and stderr byte preservation;
- success, nonzero, and timeout with both streams larger than pipe capacity,
  including arbitrary bytes and a descendant holding inherited pipe handles;
- empty, valid, truncated, and invalid strict-ASCII stderr diagnostics;
- timeout with child and grandchild cleanup before native return;
- deadline completion requiring one final authoritative job query;
- GQCS and QueryInformationJobObject error propagation;
- a two-process reservation collision proving at most one child starts;
- bound input drift and pre-cleanliness failure after reservation;
- process-start and incomplete-drain failures;
- pre/post residue;
- cleanup exception after reservation;
- serializer, write, flush, and readback faults;
- exact-environment allowlist with inherited-sentinel rejection; and
- closed-schema mutation cases for missing, extra, reordered, incoherent
  count/hash/base64, fallback fields, and working-directory substitution,
  alias, or case drift.

The reviewed diagnostic sources must freeze one closed receipt schema artifact
before live authorization. It enumerates exact ordered top-level and nested
keys, schema version, kind, state/result vocabulary, terminal variants,
stream-coherence rules, and the reservation/fallback grammar. Receipts are
compact ASCII JSON with no CR and exactly one final LF. Independent validation
rejects any missing, extra, reordered, incoherent, or noncanonical field.

The reservation is also closed canonical ASCII JSON. The winning launcher
writes, force-flushes, and fully reads it back through its create-new exclusive
handle before any further preflight. It permanently binds the reviewed sources,
documents, exact command/environment, exact working directory, and receipt
path. The working directory must be the reviewed ordinary, non-reparse,
non-aliased Attempt-10 tooling directory with exact ordinal spelling. Its
identity is validated before child start, after child completion, and during
independent receipt review. After that launcher returns, a reservation without
one complete independently valid receipt is terminal evidence of
`infrastructure-failed`.

The terminal decision table is closed:

1. `validator-result`: the reservation was acquired, the exact child started,
   its result is exit or timeout, both binary drains reached EOF, job cleanup
   completed, post-cleanliness is clean, and the canonical receipt passed
   same-handle flush/readback plus independent validation. Only this state may
   enter read-only diagnostic classification.
2. `infrastructure-failed`: any bound/pre-cleanliness, receipt creation,
   process start, pipe drain, job cleanup, serialization, write, flush, or
   readback failure. The reservation proves the consumed invocation even when
   the receipt is absent or invalid. This state consumes Attempt 11, forbids
   classification/repair and rerun, and requires a separately reviewed
   recovery attempt.
3. `not-invoked`: the reservation was not acquired. A collision leaves the
   winner's bytes untouched and starts no child in the losing process.

After independent six-axis approval of the exact external sources, run exactly
one diagnostic invocation of the frozen Attempt-10 validator command:

```text
bundled-python -B a0_tooling_approval.py generated-scope \
  --evidence-root <exact A0 root> \
  --repository-root <exact repository root>
```

Use the frozen tooling working directory and the same bound Python, PowerShell,
Git, `PYTHONDONTWRITEBYTECODE`, `SystemRoot`, `WINDIR`, `TEMP`, `TMP`, and
restricted `PATH`. The hard timeout is 900000 ms. The distinct create-once
reservation and receipt are:

```text
C:\a22or16-evi\diagnostic\attempt10-validator-replay-reservation-v1.json
C:\a22or16-evi\diagnostic\attempt10-validator-replay-v1.json
```

The diagnostic receipt binds this spec/plan approval, the failed Attempt-10
receipt, the frozen validator, launch, topology, prelaunch, tooling final,
source ledger, build result, qualification inputs, executable identities, exact
argv/environment/working directory, child and wall elapsed time, exit code,
timeout/cleanup state, raw stdout/stderr byte counts, SHA-256 values, base64
payloads, and pre/post cleanliness. The receipt also records the bounded
strict-ASCII stderr diagnostic, but the raw-byte fields remain authoritative.

This diagnostic is non-authorizing even if the child exits zero. It cannot
authorize its own classification, repair, A0 claim, candidate, formal
transcript, threshold, collector output, or publication.

Failure to create a complete diagnostic receipt consumes the Attempt-11
diagnostic authorization and requires a new reviewed recovery attempt.

## Diagnostic classification and repair

The exact diagnostic result controls the next step:

1. Contract failure: propose a focused test reproducing the exact validator
   message against a synthetic or copied tree and identify only the owning
   validator module.
2. Process/cache/environment failure: propose deterministic injected tests for
   the observed boundary and identify the owning observation/error contract.
3. Timeout: do not select a manual threshold. A separate reviewed threshold
   derivation is required from a successful bounded measurement.
4. Diagnostic exit zero: classify the Attempt-10 failure as nondeterministic,
   audit every live-only boundary not represented by frozen inputs, and propose
   a deterministic failure test before changing production validation.

Classification first produces a source-cited architecture note and proposed
deterministic RED test without mutating the validator. Two new independent
reviewers must approve a diagnosis envelope binding the complete receipt,
classification, exact owning function, proposed RED, and edit boundary. That
envelope conditionally authorizes the RED/repair and Attempt-11 preparation
through launch approval, but not the live watchdog or campaign.

Every authorized repair follows red-green-refactor. The RED result, GREEN
result, source ledger, and diagnostic receipt are review subjects. No repair
may weaken exact scope, identity, no-follow, reviewer independence, parent
membership, process absence, cache absence, create-once, or no-retry contracts.

`a0_tooling_approval.py` is already 399 physical lines. Any added behavior must
be placed in a real Python module imported normally. `exec`, generated source,
`include!`, `include_bytes!`, and text-spliced source are forbidden.

## Attempt-11 current-attempt adaptation

After the focused repair passes review, create a new current-attempt domain:

```text
coverage-repair/tooling-review/attempt-11/
```

Attempt 7, 8, 9, and 10 artifacts become immutable historical prerequisites.
Attempt 11 must:

- publish a canonical prior-attempt handoff binding the failed receipt and all
  absent Attempt-10 formal artifacts;
- preserve the two retained candidates and their logical order;
- copy the exact Attempt-10 runner into the historical handoff before adapting
  the live runner;
- replace current-attempt literals, schemas, process terms, paths, and tests
  consistently;
- derive a new candidate ID from fresh build/input identities;
- regenerate generated-scope and static-topology contracts from actual paths;
- keep historical review readers explicit and current readers strict;
- retain every prior test ID and add the diagnostic regression tests; and
- require every source file below 400 physical lines.

No string replacement is allowed as the implementation mechanism. Structured
JSON and Python/PowerShell parsers must own structured data changes.

## Fresh build, review, and launch

Attempt 11 uses fresh A22OR16 build and qualification inputs. It may not reuse
the A22OR15 build result as current evidence. The exact source ledger, build
result, qualification inputs, fixed Orca identity, four runtime closures,
candidate derivation, and parent topology must receive independent build/input
review.

The tooling reviewer must be independent from document and build/input
reviewers. The six review axes are:

```text
requirements_completeness
logical_correctness
boundary_cases
code_quality
test_coverage
actual_execution_results
```

Any `REVISE` result returns an ordered repair list to the main thread. The main
thread fixes only those items and returns identical subjects to the same review
thread until all axes approve.

Only a final approved tooling envelope may publish prelaunch evidence and an
Attempt-11 launch approval. The launch remains non-authorizing until the exact
diagnostic repair, generated-scope watchdog, source ledger, build/input
evidence, runner, and parent topology all match.

## Qualification, collection, and publication

Only the exact independently validated launch approval authorizes the live
watchdog and campaign. After launch approval:

- run the live Attempt-11 generated-scope watchdog exactly once;
- require a passed terminal receipt with exact sorted paths and empty stderr;
- invoke one atomic 71-process campaign with no resume, retry, overwrite,
  selection, old-leaf reuse, or fallback;
- require 69 CLI leaves and two direct leaves with paired equality;
- validate postclaim generated scope in a fresh process;
- invoke the strict collector exactly once with explicitly bound Attempt-11
  claim and receipt paths; and
- stage publication once, obtain two independent sidecar approvals over
  identical bytes, publish once, make the stage unavailable, and run deep
  verification.

Any watchdog, campaign, collector, sidecar, publication, or deep-verification
failure consumes Attempt 11 and requires a separately reviewed recovery
attempt.

## Exit and inherited KSR requirements

Attempt 11 exits only when Package A0 is released with:

- a durable diagnostic that identifies the Attempt-10 failure boundary;
- a focused TDD repair for that boundary;
- full current and historical tooling tests;
- fresh A22OR16 build/input evidence;
- independent six-axis tooling approval;
- a passed one-shot watchdog and atomic campaign;
- complete strict collection;
- dual-approved sidecar publication; and
- deep approved verification.

Then Task 22O resumes at Package A. The parent KSR task remains open until Ares
meets every governing parent contract:

- all option values come only from `ksr_fdmtest_v4.project.3mf`;
- neither production nor tests hardcode fixture identity, option values,
  reference G-code bytes/lines/hashes, candidate IDs, or generated path sets;
- the reference G-code is read only at the final comparison boundary;
- each slicing slice cites its owning Orca source boundary and has no legacy
  fallback;
- obsolete Orca source-level pinning tests are removed only after equivalent
  Ares behavior tests exist;
- real Rust source and separate test modules stay below 400 physical lines and
  use normal `mod`, never `include!` or `include_bytes!`, for splitting;
- ordered findings return to the main thread and identical repaired subjects
  return to the same independent six-axis review task until approval or an
  exact blocker; and
- Ares emits G-code exactly equal to `ksr_fdmtest_v4.gcode` except the allowed
  Ares generator/timestamp metadata.
