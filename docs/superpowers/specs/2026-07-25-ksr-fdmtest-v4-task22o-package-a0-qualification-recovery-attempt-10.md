# Task 22O Package A0 Qualification Recovery Attempt 10 Specification

## Status

Proposed, revision 2. Before two independent document reviewers approve the
same exact specification and plan bytes, only read-only inspection and
create-once document-review artifacts below
`C:\a22or15-evi\document-reviews` are
authorized. No timing runner, A0 source edit, build, launch, claim, candidate,
OrcaSlicer process, or formal campaign is authorized.

One reviewer owns governance and requirement completeness. The other owns
technical correctness and executable verification. They must be distinct from
each other and from the later six-axis tooling reviewer. Any document finding
is repaired in a new create-once review iteration over newly hashed document
bytes.

## Purpose

Attempt 9 repaired the producer-side JSON timestamp representation and passed
all 122 tooling tests, fresh A22OR14 build/input validation, independent
build/input review, and six-axis tooling review. Its formal campaign never
started. Both mandatory pre-formal `generated-scope` watchdog invocations were
boundedly terminated at the fixed 180-second limit before the validator
emitted output:

```text
v1 elapsed = 180050 ms
v2 elapsed = 180052 ms
```

Both failures are non-formal, but the sole independently authorized recovery
invocation has been consumed. Attempt 9 has no further watchdog retry path and
its formal wrapper is forbidden. Attempt 10 first measures the unchanged
validator to completion under one extended finite non-formal bound. It then
derives a new finite watchdog threshold from the measured duration, rebuilds
the current-attempt domain and every launch-bound subject, and authorizes one
fresh campaign.

This remains ignored Package A0 oracle-tooling work. It does not change Ares
production behavior, fixed Orca behavior, the 11-case matrix, the 23 invalid
controls, the 69 CLI plus two direct role order, or expected KSR G-code bytes.

## Fixed upstream and Ares boundaries

The Task 22O rewrite boundary remains OrcaSlicer tag `v2.4.2`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, specifically:

- `src/libslic3r/PerimeterGenerator.cpp`,
  `PerimeterGenerator::process_classic()` at fixed-source lines 1144-1692;
- `src/libslic3r/LayerRegion.cpp:82-142` as the caller/output seam; and
- the fixed Voronoi dependencies already cited by Package A0.

The eventual Ares destination remains `crates/ares-core` over
`PreparedPostPerimeterInputs`, producing ordered Classic perimeter islands,
loop/path metadata, gap-fill entities, `fill_surfaces`, and
`fill_no_overlap`. Existing rectangle-oriented Ares code is not a fallback.
Attempt 10 changes only ignored qualification tooling and evidence until
Package A0 is approved.

## Frozen Attempt-9 evidence

The following exact subjects are immutable:

- specification SHA-256
  `a46c2f313d8bd862ab69f9720cbd75d27bc0d587e8997dc1f047234ce86f4e5a`;
- plan SHA-256
  `c1aaca08360a4e6e43febf83d5d49cbb085a35c409b7b28daff149571c3404a0`;
- final document envelope SHA-256
  `05132a32e49fe20bf8081c32bfffa54f3c05b80a27ae723750786434c132bd33`;
- final tooling envelope SHA-256
  `a4437e46dc2b84ba4f1a6b363e25a604d9f788a90dec08b925537ccf4a8e7c69`;
- final source ledger SHA-256
  `eeaa235e1a63e4cf399df3f8f7812e9dc69c34a7d0d75c8b12dad72b9e880ef7`;
- full 122-test transcript SHA-256
  `9415ce4b1aae682af6a2b2528b844d7ac73013b2de9b84f081ab4de12d8e71af`;
- A22OR14 build-result SHA-256
  `1d123f29ff72d0e589cd38e157ad8f331fd9aba58f8a7675d73622fc1b4f4f4a`;
- qualification-inputs SHA-256
  `7b43c869001dd1e049bd5a493cac963abc4e335d0a280590f7ff5f998debf22e`;
- prelaunch evidence SHA-256
  `8279556c178aa60c44d0331f30d8c2b9c78cfe8a1134aea40a3742a5a288bee8`;
- launch approval SHA-256
  `2c34def7b69d6cba4ffbcf9bbd437899d7b91fab37668d131934c06e736b21da`;
- candidate ID `f81050680415514f`, which remains absent;
- v1 script
  `C:\a22or14-driver\run_attempt9_generated_scope_watchdog.ps1`, 10,080
  bytes, SHA-256
  `ebc755cf558647f6e7177ce9508e974ff9b45d9be57aae10bcaf26efd9fb0e3d`;
- repaired v2 script
  `C:\a22or14-driver\run_attempt9_generated_scope_watchdog_v2.ps1`, 11,434
  bytes, SHA-256
  `6606d174d54884a4c9ade05101340884485a86cbbd7c6222d80d3ebd19f35767`;
- Attempt-9 formal runner
  `tooling/run_fixed_qualification.ps1`, 20,014 bytes, 391 physical lines,
  SHA-256
  `987829e95dce3a335a0cfdf73b5b7727ca6cdc1a0496a6e04f40d4b54caa078a`,
  whose terminal source-ledger row is exactly
  `987829e95dce3a335a0cfdf73b5b7727ca6cdc1a0496a6e04f40d4b54caa078a  391  tooling/run_fixed_qualification.ps1`;
- v1 watchdog transcript
  `C:\a22or14-driver\attempt9-generated-scope-watchdog-v1.txt`, 502 bytes,
  SHA-256
  `57b435ca7a196b956d2ec0f43f0007d15c9da6fb109876a6f5f77b893d0cf4a5`;
  and
- v2 watchdog transcript
  `C:\a22or14-driver\attempt9-generated-scope-watchdog-v2.txt`, 627 bytes,
  SHA-256
  `d6b60f60146bf34909225bd56cac189ad2aee578fa640462ca9ec929e9019a6b`.

The v2 transcript records `MODE=non-formal`, `TIMEOUT_MS=180000`,
`ELAPSED_MS=180052`, `EXIT_CODE=-1`, `PATH_COUNT=-1`, empty stdout/stderr,
and absent claim, candidate, and formal transcript. Attempt-9 claim
`coverage-repair/tooling-review/attempt-9/launch-claim-v1.json`, candidate
root, and formal transcript
`C:\a22or14-driver\attempt9-campaign-formal-run-v1.txt` remain absent.

Never rerun either Attempt-9 watchdog, invoke the Attempt-9 formal wrapper,
overwrite either transcript, create the Attempt-9 claim/candidate, or mutate
any Attempt-9 launch/review/build/input subject.

## Runtime diagnosis boundary

`ELAPSED_MS=180052` is the timeout threshold plus recursive termination
overhead, not a completed validator duration. Empty stdout does not identify a
slow stage because `a0_tooling_approval.py` emits its JSON only after all
validation succeeds.

The unchanged validator deliberately:

- walks the complete A0 tree without following reparses;
- stats every entry and SHA-256 hashes every ordinary file;
- validates the complete Attempt-7 and Attempt-8 retained history;
- validates build evidence and all four runtime closures; and
- reconstructs the exact prelaunch closed set.

The two retained histories alone contain 427,784 files and 6,845,817,303
logical bytes. Static inspection found bounded traversal and no retry or
nontermination path. Therefore Attempt 10 must measure completion before
changing validator source. The two calls to `_validated_core` are only a
possible later optimization; they are not evidence of a defect.

## Document approval location

Before the timing run, document evidence lives outside A0 so the unchanged
Attempt-9 exact-scope validator sees no new A0 paths:

```text
C:\a22or15-evi\document-reviews\iteration-0001\  (rejected revision 1)
C:\a22or15-evi\document-reviews\iteration-0002\  (revision 2 gate)
C:\a22or15-evi\document-reviews\registry-v1.json
C:\a22or15-evi\document-reviews\final-envelope.json
```

Each iteration contains exact copies of the specification and plan, both
review reports, and a canonical review envelope. All files are ASCII/LF,
create-once, hash-bound, and immutable. The final envelope authorizes only the
measurement phase. Attempt-10 A0 tooling later imports and binds these exact
subjects without rewriting them.

## One finite timing measurement

After document approval, create an external timing tool and focused tests under
`C:\a22or15-driver`. The production timing script and every test source must be
ASCII/LF and below 400 physical lines. The tests use a synthetic child and
cover success, timeout with recursive kill/wait, nonzero exit, stderr,
create-once collision, forbidden-path detection, environment clearing, and
failure cleanup. They must not execute the live validator.

The production measurement is exactly one invocation of unchanged:

```text
bundled-python -B a0_tooling_approval.py generated-scope
```

It uses the exact A0 and repository roots, tooling working directory,
`PYTHONDONTWRITEBYTECODE=1`, and a PATH limited to the bound PowerShell and Git
directories. It binds the exact Python, PowerShell, Git, Attempt-9 launch,
v1/v2 script, v1/v2 transcript, and approved document identities.

The measurement hard timeout is 900,000 ms. Attempt 8 completed the same
contract with one retained tree in 163,019 ms; 900,000 ms is therefore a
finite calibration ceiling with more than five times that observed baseline,
not a replacement watchdog threshold. Before and after execution it
requires:

- no matching validator, formal runner, OrcaSlicer, or fixed-probe process;
- no recursive `__pycache__`, `.pyc`, or `.pyo`;
- no Attempt-9 or Attempt-10 claim/candidate;
- no Attempt-9 or Attempt-10 formal transcript; and
- no repository-root runner temporary.

Timeout handling recursively terminates and waits for the complete process
tree. The create-once canonical timing receipt is:

```text
C:\a22or15-evi\timing\measurement-v1.json
```

Success requires exit zero, empty stderr, the exact sorted 155-path JSON
contract, unchanged Attempt-9 launch subjects, and all absence checks. The
receipt records exact command/executable/argv/working-directory/environment
identities, bound subjects, child and wall elapsed milliseconds, process
result, normalized stdout identity and exact path list, and pre/post
cleanliness. It is written with create-new semantics and forced flush.
Its exact top-level keys are:

```text
schema_version
kind
state
consumed_attempt
recovery_attempt
command
subjects
timing
process
output
cleanliness
failure
```

Success uses `state="passed"` and `failure=null`. Failure uses
`state="failed"`, records the bounded diagnostic, and cannot authorize a
threshold.

Failure consumes the one measurement and forbids an in-place retry. A timeout
or operationally unacceptable completion requires a separately reviewed
Attempt 11 before validator instrumentation or source optimization.

## Derived watchdog threshold

Let `M` be the successful measurement's child elapsed milliseconds. Attempt 10
derives:

```text
W = ceil_to_60000(max(360000, 2 * M))
```

`W` is recorded at `C:\a22or15-evi\timing\threshold-v1.json` in a create-once
canonical threshold envelope that binds the timing script, measurement
receipt, normalized stdout hash, Attempt-9 launch, and document final. The
formula record contains multiplier 2, minimum 360,000 ms, quantum 60,000 ms,
and maximum 1,800,000 ms. The derived threshold must be greater than `M`, no
greater than 1,800,000 ms, and independently recomputable. If the formula
yields more than 1,800,000 ms, Attempt 10 stops and requires a separately
reviewed performance-repair attempt.

The threshold envelope has exactly `schema_version`, `kind`, `state`,
`measurement`, `formula`, `measured_elapsed_ms`, `raw_timeout_ms`, and
`generated_scope_timeout_ms`. It has no override or manual-adjustment field.
Python and PowerShell independently recompute `W`; the exact create-once
envelope then receives an independent read-only review. Any receipt parse,
formula, bounds, dual-recomputation, canonical-schema, create-new write, or
threshold-review failure consumes Attempt 10. The threshold remains
non-authorizing and recovery requires a separately reviewed Attempt 11.

No threshold may be selected directly from either 180-second failure. No
validator source is changed before a successful measurement and accepted
threshold.

## Attempt-10 current-attempt adaptation

After the threshold envelope is frozen:

- Python and PowerShell candidate domains use a new framed Attempt-10 domain;
- current schema kinds, attempt numbers, runner messages, review, launch,
  claim, publication, and formal transcript paths say Attempt 10;
- historical Attempt-7, Attempt-8, and Attempt-9 paths, kinds, identities,
  failures, and absent-execution facts remain unchanged;
- a canonical Attempt-10 prior-attempt handoff binds the complete Attempt-9
  launch state, both watchdog scripts/transcripts, the timing evidence, and
  the absence of Attempt-9 claim/candidate/formal execution;
- before any live-runner mutation, the exact Attempt-9 runner is copied
  create-once to
  `coverage-repair/tooling-review/attempt-10/prior-attempt/attempt9-runner.ps1`;
  the handoff binds its rooted 20,014-byte identity and the exact terminal
  Attempt-9 source-ledger row;
- the nested Attempt-8 handoff continues to bind the two retained candidates;
- the candidate parent prelaunch logical order remains
  `["8a5aab7461b877b1","11d07a6fd071c211"]`;
- postclaim logical order appends exactly the fresh Attempt-10 candidate ID;
  and
- physical membership always equals the sorted set of the active logical
  binding.

Preserve all 122 existing A0 test IDs and all 26 existing required IDs without
rename or deletion. Add a small `a0_watchdog_contract.py` module and a separate
`tests/test_a0_watchdog_contract.py` module for threshold and launch binding
behavior because the existing approval, handoff, candidate, and approval-test
files are already at 395-399 LOC. Static topology derives and freezes the new
exact source-path, test-path, discovered-ID, required-ID, and A0-path counts
from independent literal sets, and expands the exact repository document set
by only this specification and plan. Do not guess or copy topology counts from
Attempt 9. The Attempt-10 static-topology subject also freezes the independently
derived sorted current-attempt generated-scope path set, its count, and the
SHA-256 of its normalized compact JSON. This is a new Attempt-10 contract; the
155-path Attempt-9 measurement remains only timing and frozen-history evidence.

The Attempt-10 launch envelope has exactly:

```text
document_final
tooling_final
prelaunch_evidence
prior_attempt
timing_evidence
build_result
qualification_inputs
powershell
failure_handoff
failed_candidate_tree
static_topology
publication_policy
```

The launch envelope also records `generated_scope_timeout_ms=W` and binds the
threshold envelope as `timing_evidence`. Python validation, the external
pre-formal watchdog, and the new exact formal wrapper
`tooling/run_attempt10_fixed_qualification.ps1` independently recompute and
require the same value. The `powershell` launch subject binds that new path and
its final reviewed hash. The old live
`tooling/run_fixed_qualification.ps1` is removed only after its create-once
history copy is validated; the history copy is never executable. The new
formal runner passes `W` explicitly only to its preclaim
`a0_tooling_approval.py generated-scope` invocation. Its later bounded
build/input hydration call keeps the existing 180,000 ms timeout. Focused
tests must prove the two timeout call sites cannot be conflated and that the
old runner, its history copy, omission, or path substitution cannot be invoked
or silently accepted.

Any source, test, CMake, runner, candidate-domain, build-result,
qualification-input, runtime-closure, policy, topology, threshold, or
document-final change invalidates prepared current roots and requires new
roots before launch.

## Fresh A22OR15 build and launch

Use new literal, ordinary, canonical, pairwise tree-disjoint roots under the
A22OR15 family for source, build, install, and evidence. They must not reuse
A22OR8 through A22OR14 or any Attempt-6 root. Recreate and independently
validate fixed source, configure/build/install, CTest RED/GREEN, child
environment, all four adjacent-DLL runtime closures, build-result v2,
qualification-inputs v2, candidate agreement, and candidate absence.

Run the complete newly discovered A0 test set and every static check after
adaptation. Every edited source/test file remains ASCII and below 400 physical
lines. Rust source splitting with `include!` or `include_bytes!` remains
forbidden.

A six-axis tooling reviewer distinct from both document reviewers approves the
exact final ledger, test results, threshold, prior-attempt handoff, fresh
build/input evidence, topology, policy, and launch contract. Only then publish
the sole Attempt-10 launch approval create-once.

Before formal invocation, run the Attempt-10 live `generated-scope` validator
under the derived threshold `W`, with the same environment and recursive
cleanup contract as the timing measurement. This watchdog is exactly once and
must pass. Its distinct canonical receipt is create-once at
`C:\a22or15-evi\watchdog\attempt10-generated-scope-v1.json`; it binds the exact
launch approval and static-topology identities. Its sorted path set, count, and
normalized stdout hash must equal the independently derived Attempt-10
contract frozen in that launch-bound static-topology subject, not the 155-path
Attempt-9 timing receipt. `measurement-v1.json` remains immutable timing
evidence and is never reused or overwritten by the watchdog.

The sole formal wrapper and transcript targets are:

```text
tooling/run_attempt10_fixed_qualification.ps1
C:\a22or15-driver\attempt10-campaign-formal-run-v1.txt
```

Immediately before invocation require transcript, claim, candidate, and runner
temporaries absent. Invoke the formal wrapper exactly once, create the claim
atomically before the candidate, and execute ordinals 1-71 without resume,
retry, overwrite, selection, or old-leaf reuse.

## Collection and publication

Immediately after successful execution, a fresh process validates the
postclaim generated scope. Then invoke the strict existing collector exactly
once over only the fresh candidate. It validates all leaves, standalone and
embedded datadir equality for all 69 CLI roles, the two direct-role absence
contract, and unchanged candidate-tree bytes before any stage exists.

A formal, postclaim, or first-collector failure consumes Attempt 10. Never
repair or replace Attempt-10 evidence in place; recovery requires a separately
reviewed Attempt 11.

Only a candidate that passes 71/71 execution and strict collection may enter
the existing create-once stage/review/publish state machine. Two independent
sidecar reviewers, distinct from the tooling reviewer, inspect identical
staged corpus/manifest bytes. Any `REVISE`, `REJECT`, disagreement, or byte
drift consumes Attempt 10 and requires Attempt 11. Dual approval permits one
final publication followed by deep approved verification.

## Exit and inherited KSR requirements

Attempt 10 exits only when the timing measurement and derived watchdog are
frozen, one fresh campaign passes all 71 roles and strict collection, dual
sidecar review approves identical bytes, deep verification passes, and Package
A0 releases the separately reviewed Task 22O adapter/engine work.

The governing Task 22O specification remains 29,472 bytes with SHA-256
`78c44972e284eb615bf96228cbc5d0fe3a5c731a853c3b1cf518f92219b95674`.
The governing plan remains 35,729 bytes with SHA-256
`94c361d0d4c89eb5019f07f3a3e4101b8d89857d02c06629e3c794920f645e80`.

After Package A0, tracked implementation must derive every Option only from
the supplied 3MF; must not inspect fixture names or reference G-code; must
remove obsolete source-pinning tests in touched scope; must use real Rust
modules and separate test modules below 400 physical lines without `include!`
or `include_bytes!` splitting; must provide no legacy fallback; and must match
the supplied KSR G-code except the allowed Ares/timestamp metadata difference.

One reviewer independent of the implementing main thread reviews requirement
completeness, logical correctness, boundary cases, code quality, test
coverage, and actual execution results. It returns an ordered fix list to the
main thread. The main thread repairs findings and the same reviewer identity
revalidates until all six axes approve or a precise blocker is recorded.
