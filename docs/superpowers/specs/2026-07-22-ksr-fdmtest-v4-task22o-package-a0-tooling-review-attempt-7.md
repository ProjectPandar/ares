# Task 22O Package A0 Qualification Recovery Attempt 7 Specification

## Status

Proposed, revision 11. No further Attempt-7 tooling source edit, fresh build, or formal
process is authorized until both independent document reviewers approve these
exact specification and plan bytes. Reviewer feedback is iterative unless a
review explicitly declares a terminal rejection; only a terminal rejection or
a consumed formal launch ends Attempt 7.

## Purpose

Attempt 7 repairs three independently reproduced tooling defects:

1. terminal candidate `8a5aab7461b877b1` stopped at ordinal 70 because the
   direct fixed probe could not resolve adjacent OpenCASCADE runtime DLLs; and
2. the postqualification closed-set validator rejects the exact corpus and
   detached review paths required by its own publication workflow; and
3. CTest 3.31.6-msvc6 on Windows reports `BAD_COMMAND` for the first absolute
   test executable when its process environment contains a literal empty
   `PATH`, even though the executable exists and direct launch succeeds.

This is an ignored oracle-tooling and evidence-contract repair. It does not
change Ares production behavior, the fixed Orca derivative source bytes,
Package 0, Voronoi semantics, or the frozen 71-role order.

## Fixed upstream and Ares boundaries

The complete Task 22O rewrite boundary remains OrcaSlicer tag `v2.4.2`,
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, specifically:

- `src/libslic3r/PerimeterGenerator.cpp`,
  `PerimeterGenerator::process_classic()` at fixed-source lines 1144-1692,
  and the KSR-reached helpers named by the parent Task 22O specification;
- `src/libslic3r/LayerRegion.cpp:82-142` as the caller/output seam; and
- the fixed Voronoi dependencies already named by Package A0.

The eventual Ares destination remains the project slicing path in
`crates/ares-core` over `PreparedPostPerimeterInputs`, producing ordered
Classic perimeter islands, loop/path metadata, gap-fill entities,
`fill_surfaces`, and `fill_no_overlap`. The old rectangle-oriented
`crates/ares-core/src/perimeters.rs` path is not a fallback.

Attempt 7 itself touches only the narrower observational seam in
`src/libslic3r/PrintObject.cpp`,
`src/libslic3r/Geometry/MedialAxis.cpp`, and the already approved A0
record-session/wire files. Included behavior is runtime qualification and exact
capture of the frozen A0 records. All Classic implementation, option handling,
Arachne, infill, motion planning, and G-code production remain deferred to the
approved tracked packages. The twelve-file A0 source ledger, source status,
Package-0 patch, and A0 patch must remain byte-identical.

## Immutable failed campaign

The following Attempt-6 candidate is terminal evidence:

```text
.superpowers/sdd/task22o-oracle/voronoi-a0/runs/qualified/8a5aab7461b877b1
```

It contains 70 run specs and 70 run directories: 69 passed CLI leaves and one
failed direct leaf at ordinal 70. It has root `FAILED.json`, no root
`result.json`, no ordinal-71 spec or directory, and no successful direct
leaf. Ordinal 70 exited `-1073741515` (`0xC0000135`,
`STATUS_DLL_NOT_FOUND`) without timeout, output, temporary output, root-log
residue, or surviving process.

Attempt 7 creates these exact create-once handoff subjects outside the failed
candidate:

```text
coverage-repair/tooling-review/attempt-7/failure-handoff/
  failed-candidate-tree-v1.json
  qualification-inputs-v1.json
  fixed-probe-attempt6.exe
  fixed-probe-imports-v1.txt
  failure-handoff-v1.json
```

`failed-candidate-tree-v1.json` is a canonical no-follow whole-tree ledger.
It records the root-relative literal path, kind (`directory`, `file`, or
`reparse`), file byte count and SHA-256, and reparse metadata for every entry,
plus aggregate file/directory/reparse counts and the exact 70/70/69/1 run
structure. It follows no reparse point and rejects case-fold, short-name,
trailing-dot, trailing-space, or junction aliases. The other two binary
snapshots preserve the external-only Attempt-6 qualification input and probe
bytes; the handoff binds them, the tree ledger, PE import output, Attempt-6
approval ledger/transcript/report/envelope, and the observed failure facts.

The retained evidence does not record the historical inherited `PATH`.
Therefore the loader-path diagnosis is explicitly an inference from the PE
imports, absent adjacent DLLs, `0xC0000135`, and the CTest-only PATH
modification; it is not represented as an observed historical PATH fact.

Every gate and the immediate prelaunch check must regenerate the whole-tree
ledger in memory and require byte equality. Nothing beneath the failed
candidate may be added, removed, renamed, or changed. No leaf from it may be
selected, copied, promoted, or used as corpus input.

## Review lifecycle and exact evidence

Document review evidence uses:

```text
coverage-repair/tooling-review/attempt-7/document-reviews/iteration-NNNN/
  specification.md
  plan.md
  governance.md
  technical.md
  review-envelope.json
coverage-repair/tooling-review/attempt-7/document-reviews/registry-v1.json
coverage-repair/tooling-review/attempt-7/document-reviews/final-envelope.json
```

Iterations are consecutive from `0001`, allocated create-once before review.
Each iteration stores the exact reviewed document copies, both reports, and a
canonical envelope with exactly `schema_version`, `kind`, `iteration`,
`subjects`, `reviews`, `state`, and `implementation_authorized`.
Reports contain reviewer identity, both subject identities, findings, and one
literal verdict. `REVISE` feedback permits a new iteration; it never mutates
an old one. After the final iteration, each registry row binds the current
iteration envelope and the preceding envelope identity (null for `0001`).
Thus immutable seven-field envelopes never predict a later iteration. The
registry enumerates every consecutive iteration path, identity, and hash link.
The final envelope binds that registry and one final common document pair, and
authorizes implementation only when both final verdicts are `APPROVE` and the
governance and technical reviewer identities differ.

Tooling reviews use the equivalent create-once topology:

```text
coverage-repair/tooling-review/attempt-7/tooling-reviews/iteration-NNNN/
  source-files.sha256
  test-results.txt
  six-axis-review.md
  review-envelope.json
coverage-repair/tooling-review/attempt-7/tooling-reviews/registry-v1.json
coverage-repair/tooling-review/attempt-7/tooling-reviews/final-envelope.json
```

Each report covers requirements, logic, edge cases, code quality, test
coverage, and actual results. A repair creates a new source ledger, transcript,
and iteration. A published terminal `REJECT` consumes Attempt 7; ordinary
`REVISE` feedback does not. After the last iteration, each registry row binds
the current and preceding envelope identities (null predecessor for `0001`);
the canonical registry enumerates the consecutive literal children and
complete hash chain without mutating any iteration.
The final tooling envelope has exactly `schema_version`, `kind`, `attempt`,
`state`, `reviewer`, `final_iteration`, `registry`, `subjects`,
`axis_verdicts`, and `tooling_approved`. Its reviewer must differ from both
document reviewers. The later stage/final sidecar validator, which has all
three identities, requires its two sidecar reviewers to differ from each other
and from this tooling reviewer.

## Runtime closure and build-result v2

Attempt 7 must make CTest and formal direct roles use the same adjacent-DLL
loader model:

1. build all targets;
2. install `orca-slicer.exe`, the Orca DLL, the fixed probe, and runtime DLLs
   into a fresh install root;
3. enumerate every ordinary, non-reparse top-level installed `.dll`;
4. copy that exact closed DLL set beside the build-tree probe;
5. compare the build and installed sets by literal basename, bytes, and SHA-256;
6. run CTest with the copied adjacent DLLs and no probe-specific PATH change;
7. compare the build closure again after CTest; and
8. compare the installed closure again immediately before launch approval and
   in memory immediately before formal process 1.

The build and installed probe bytes and SHA-256 must also be identical.
Missing, extra, renamed, mutated, or reparse entries fail. Both sets must
contain `TKLCAF.dll` and `TKernel.dll`.

CTest is launched by absolute executable paths. The evidence driver constructs
its environment from scratch without merging the ambient environment. Its
exact serialized key set is `SystemRoot`, `TEMP`, `TMP`, `LANG`, `LC_ALL`, and
`LC_CTYPE`; the first three values are nonempty absolute paths and the locale
values are exactly `C`. Environment names are compared with Windows
case-insensitive semantics, so any key case-folding to `path`, every extra key,
or any missing/invalid required value fails.

The bound CTest 3.31.6-msvc6 process adds exactly
`CMAKE_CONFIG_TYPE=Release`, `CTEST_INTERACTIVE_DEBUG_MODE=1`, and
`VSCONSOLEOUTPUT=1` to the generated test and normalizes `SystemRoot` to
`SYSTEMROOT`. No resource specification or resource group is used. Therefore
the Python wrapper's exact effective environment has nine keys: those four
CTest/Windows keys plus `TEMP`, `TMP`, `LANG`, `LC_ALL`, and `LC_CTYPE`.
`fixed-probe/tests/test_probe.py` validates that exact closed set before any
subprocess, rejects every key case-folding to `path`, and create-once writes a
canonical observation to
`<build-root>/ares22o-ctest-child-environment-v1.json`. CMake passes that exact
absolute path through required `--environment-observation`; generated CTest
has neither `ENVIRONMENT` nor `ENVIRONMENT_MODIFICATION`.

Build evidence binds the absolute Git executable identity; CMake passes it as
required `--git <absolute-path>` to the wrapper, and all three wrapper Git calls
use that argument. The CTest record binds common argv, working directory, Git
and generated-property identities; a canonical RED subrecord binds the exact
literal-empty-`PATH` environment, exit 8, and `BAD_COMMAND` transcript; the
GREEN fields bind the exact six-key PATH-absent environment, exit 0, success
transcript, and canonical child-observation identity/content. That one build
result is transitively bound by tooling review, prelaunch evidence, and launch
approval. Formal evidence also binds the
absolute PowerShell executable identity. The production runner invokes each
`run_one.ps1` through that absolute executable with a recorded base
`PATH=""`; the two ORCA activation variables are the only per-role
environment deltas. The formal runner never searches an ambient PATH, prepends
an unbound directory, or copies a DLL at role time.

The exact `ctest` object keys are `sequence`, `argv`, `working_directory`,
`environment`, `git`, `exit_code`, `log`, `generated_properties`, `red`, and
`child_environment_observation`. `red` has exactly `environment`, `exit_code`,
and `log`. Its log is `<evidence-root>/ctest-red.log`; the GREEN log is
`<evidence-root>/ctest.log`; both are canonical ASCII/LF transcripts. The
observation has the exact build-root path above and canonical JSON bytes.

Canonical `build-result-v2.json` has the existing source, patch, dependency,
command, cache, and toolchain evidence plus:

- `built.fixed_probe`;
- `installed.executable`, `installed.dll`, and
  `installed.fixed_probe`;
- `runtime_closure.build_before_ctest`;
- `runtime_closure.build_after_ctest`;
- `runtime_closure.installed_after_install`;
- `runtime_closure.installed_prelaunch`; and
- `ctest` common command/generated-property evidence, canonical RED and GREEN
  environment/log results, and child-environment observation.

Each closure uses root-relative top-level paths and binds the probe plus every
DLL. The build-result validator constrains artifacts to the corresponding
fresh roots, proves the four closure records regenerate and compare as
required, and is the only source of the installed probe identity.
Qualification inputs, runner, collector, assembly, deep verification, tooling
approval, and candidate-root checks must consume that same build result.

## Qualification inputs and framed candidate identity

`qualification-inputs-v2.json` uses named roots and root-relative artifact
paths; changing only source/build/install directory names cannot change the
candidate ID. The runner receives the separately bound build result to resolve
those roots. All semantic fields, the complete 71-role plan, runtime closure,
and artifact identities remain in the qualification input.

The candidate ID is the first 16 lowercase hexadecimal characters of SHA-256
over this exact framed preimage:

```text
ASCII "A0CID2\0"
for each field below, in order:
  u16le(label byte length)
  ASCII label bytes
  u64le(payload byte length)
  raw payload bytes

fields:
  domain = ASCII "task22o-a0-attempt-7-candidate-v2"
  source_ledger = raw source-files.sha256 bytes
  installed_orca = raw installed orca-slicer.exe bytes
  installed_probe = raw installed fixed-probe bytes
  installed_runtime_closure = canonical installed_prelaunch JSON bytes
  role_plan = canonical role_plan JSON bytes
  qualification_inputs = canonical qualification-inputs-v2 JSON bytes with
                         only the top-level candidate_id field removed
```

Python and PowerShell independently recompute the preimage and ID. Tests must
reject domain drift, field reordering, coordinated boundary shifts, one-byte
mutation of every payload, or any qualification field change.

The exact candidate is `runs/qualified/<candidate-id>`. Before approval the
qualified parent must contain only the immutable failed Attempt-6 child. The
new path must be absent. A collision with `8a5aab7461b877b1` or any existing
child fails closed without salt, alternate path, or rebuild-root workaround and
requires a new attempt/domain.

## Single-use launch authorization

Fresh build/install, all four runtime-closure records, CTest, version-2 inputs,
candidate-ID computation, and tooling review occur before final launch
authorization. These exact create-once files close the prelaunch evidence:

```text
coverage-repair/tooling-review/attempt-7/prelaunch-evidence-v1.json
coverage-repair/tooling-review/attempt-7/launch-approval-envelope.json
```

`prelaunch-evidence-v1.json` is the finalized generated-scope ledger for the
completed document/tooling registries, failure handoff, build, runtime, CTest,
and qualification evidence. `launch-approval-envelope.json` is the sole path
the production runner may consume. It has exactly `schema_version`, `kind`,
`attempt`, `state`, `candidate_id`, `candidate_root`,
`authorized_process_count`, `formal_orca_execution_authorized`,
`launch_claim_path`, `qualified_parent_children`, `tooling_reviewer`, and
`subjects`. It binds:

- final document and tooling review envelopes;
- the failure handoff and regenerated failed-candidate tree identity;
- build-result v2 and qualification-input v2 identities;
- exact candidate ID and absolute candidate path;
- the qualified-parent child list;
- all runtime closure and CTest evidence; and
- the static source/test topology, prelaunch evidence ledger, and frozen
  publication-registry policy.

The launch validator requires the tooling reviewer to differ from both
document reviewers and rejects any envelope path other than the literal one
above. Every nested identity has exactly `root`, `path`, `bytes`, and
`sha256`; root/path pairs are validated without aliases.

It authorizes one candidate and one full campaign. Its launch claim path is:

```text
coverage-repair/tooling-review/attempt-7/launch-claim-v1.json
```

The runner validates the envelope and candidate absence, then atomically
creates the canonical claim with create-new semantics before creating the
candidate or launching process 1. The claim binds the envelope, candidate, and
input identities and remains after success or failure. An existing claim,
candidate collision, or changed parent prevents launch. Every campaign runs all
71 roles from ordinal 1. A first failure is terminal and requires Attempt 8;
there is no resume, retry, overwrite, third paired run, or preferred selection.

Any source, test, CMake, runner, build-result, qualification-input, closure, or
candidate-identity change after fresh build preparation invalidates every fresh
root and requires the full tooling verification and build/input preparation to
run again before another tooling review. After the launch claim exists, such a
change requires Attempt 8. A later sidecar `REVISE` may create another stage
only from the unchanged launch-bound sources, candidate, and evidence.

## Closed-set and publication phases

Attempt 7 replaces prefix exclusions with explicit static and generated scopes:

- `coverage-repair/tooling-review/attempt-7/static-topology-v1.json` freezes
  the Attempt-6 126 A0 paths plus exactly five new source and four new test
  modules: 135 fixed A0 paths total. It also freezes 14 repository documents
  (the prior 12 plus this spec and plan).
- Its source role total is 69: 49 Python, eight PowerShell, and twelve other
  CMake/C++/header/data sources. Its 28 test-role paths are an exact subset.
  Test discovery is exactly 122 tests, including exactly 26 required IDs. The
  immutable historical prerequisite count remains 31.
- New or removed source/test/document paths require another document review.
  Evidence-only review iterations do not.
- Each document iteration has exactly five files; each tooling iteration has
  exactly four; each publication iteration has exactly six. Their registries
  require consecutive `iteration-NNNN` children, exact per-iteration schemas,
  a previous-envelope hash chain, and literal file counts.
- The launch envelope binds the finalized prelaunch generated-scope ledger.
  The final sidecar envelope binds the separately created postlaunch
  publication registry.
- `coverage-repair/tooling-review/attempt-7/publication/policy-v1.json`
  freezes the publication registry, iteration, and root-relative record schemas
  before launch without predicting any later iteration path.
- `runs/qualified` is independently closed by exact child names and complete
  candidate topology. Repository-local `fixed-probe/build` and every
  `__pycache__` are forbidden. No directory-wide exclusion remains.

Publication uses create-once staging iterations:

```text
coverage-repair/tooling-review/attempt-7/publication/iteration-NNNN/
  corpus/corpus-v1.bin
  corpus/corpus-v1.json
  sidecar-manifest-v1.json
  reviews/sidecar-fixed-source.md
  reviews/sidecar-qualification.md
  reviews/sidecar-approval-envelope.json
coverage-repair/tooling-review/attempt-7/publication/registry-v1.json
```

Manifest corpus records and sidecar-envelope manifest/report records contain
only canonical paths relative to an explicit logical `publication_root`.
Stage and final roots have identical relative topology. Verifiers reject
absolute paths, empty/dot/dot-dot segments, case/short-name/reparse aliases, and
owner-relative fallback; they resolve every publication record only beneath
the caller-supplied root.

The real closed-set validator has these explicit states:

- `prequalification`: final corpus, manifest, and detached reviews absent;
  only the failed candidate exists under `runs/qualified`;
- `stage-prepublish`: one literal stage has exactly the two corpus files and
  requires its manifest and reviews absent; the assembler invokes this state;
- `stage-published`: the same stage has exactly the two corpus files and
  manifest, with reviews absent; subject verification invokes this state;
- `stage-revise`: the same stage adds two ordered reports and one rejected
  envelope; accepted verdict vectors are `APPROVE/REVISE`,
  `REVISE/APPROVE`, and `REVISE/REVISE`, so at least one verdict is
  `REVISE`;
- `stage-reject`: the same six-file topology records either ordered vector
  containing `REJECT`; validation must then create and validate the complete
  postlaunch registry before this terminal state forbids another stage;
- `stage-approved`: the same stage adds exactly two reports and one envelope;
- `final-published`: the final two corpus files and manifest equal the
  approved stage bytes, while final detached reviews are absent; and
- `approved`: final publication adds exactly
  `reviews/sidecar-fixed-source.md`,
  `reviews/sidecar-qualification.md`, and
  `reviews/sidecar-approval-envelope.json`.

The staged manifest binds its two corpus files in literal wire/JSON order and
does not bind itself or detached reviews. Rejected staged bytes and reports are
immutable and a new hash-chained staging iteration is used. The postlaunch
registry enumerates all stages and verdicts. Only approved staged
corpus/manifest/report bytes are copied create-once to their identical final
relative paths. A separate canonical final sidecar envelope binds the
postlaunch registry, approved stage envelope, final manifest, and final report
identities. After final publication, any rejection or byte drift requires
Attempt 8; final paths are never overwritten. Integration tests remove the
stage before final deep verification to prove no staged or owner-relative path
is consulted.

Document-final validation rejects equal document reviewer identities. Launch
validation rejects a tooling reviewer equal to either document reviewer.
Stage-revise, stage-reject, stage-approved, and approved validation reject
equal sidecar reviewers or a sidecar reviewer equal to the tooling reviewer.
Mutation tests cover every equality case.

## Test-driven repair

Public tests first reproduce each defect, then the minimum implementation makes
them pass. Required new test IDs are a closed list:

```text
candidate_v2_is_framed_and_path_independent
candidate_v2_rejects_each_subject_mutation
candidate_parent_accepts_only_bound_failed_history
failed_candidate_tree_detects_any_entry_mutation
failed_candidate_tree_rejects_result_or_ordinal_71_injection
runtime_closure_matches_build_and_install_sets
runtime_closure_rejects_missing_extra_renamed_mutated_or_reparse_dll
ctest_uses_adjacent_runtime_and_absolute_git_with_path_absent
build_result_v2_binds_installed_probe_closures_and_ctest
qualification_v2_selects_only_installed_probe
runner_requires_attempt7_envelope_and_atomic_launch_claim
mock_campaign_runs_fresh_69_cli_plus_2_direct_roles
stage_prepublication_uses_real_closed_set
stage_publication_rejects_corpus_substitution_or_adjacent_injection
approved_publication_is_root_relative_and_survives_stage_removal
```

The existing mock-campaign test also requires the bound absolute PowerShell
launcher, `PATH=""` on every `run_one.ps1` launch and formal child, and only
the two ORCA variables as per-role deltas. Ambient-PATH PowerShell lookup or a
nonempty base PATH must fail; this adds no sixteenth test ID.

The CTest-only correction does not weaken that formal-runner rule. It replaces
the required test ID above one-for-one, changes only existing source/test
paths, keeps 122 discovered tests and 26 required IDs, and authorizes the
minimal fixture and validator changes needed to reject every CTest evidence
record containing a `PATH` key. The generated CTest file must remain free of
both `ENVIRONMENT` and `ENVIRONMENT_MODIFICATION`.

The same public test mutates every CTest process-environment entry by removal,
extra insertion, invalid value, and case-variant/duplicate PATH alias; mutates
both generated environment properties under case variants; mutates the RED
status/transcript/environment; and mutates every child-observation key/value.
It also invokes the real wrapper with injected `PATH`, `Path`, and `path` so a
validator-only check cannot stand in for actual child observation.

The late full-suite regression repair may also modify
`tooling/tests/test_run_one.py`, solely to make its direct test launcher use
the same bound absolute PowerShell and literal `run_one.ps1` invocation as the
production launcher. The child process environment sets `PATH=""`, and the
PowerShell command itself first executes `$env:PATH=''` before invoking the
literal runner and propagating `$LASTEXITCODE`. The retained failed-process
case asserts `process.json["exit_code"] == 9`, so a preceding environment
failure cannot pass accidentally. This scope correction adds no discovered
test or required test ID and does not authorize a `run_one.ps1` production
change.

The same correction removes fixture order dependence without changing
production behavior: `tooling/tests/runner_fixtures.py` writes its derived
`build-result-v2.json` and `qualification-inputs-v2.json` beneath a dedicated
`runner-evidence` sibling, leaving the process-wide `formal_bundle()` bytes
untouched. `source`, `runner-build`, `runner-install`, and `runner-evidence`
remain pairwise tree-disjoint.

Because revision 7 was retained with a technical `REVISE`, the post-revision-8
registry audit found the repository-document hash fixture outside its exact
modified-path list, and the first fresh build exposed the CTest empty-`PATH`
launch defect after revision 9. Revision 10 was then retained with a governance
and technical `REVISE` because it did not close the exact environment key set,
bind RED/GREEN and child-observation evidence, cover the complete mutation
matrix, or authorize the child-observation producer. The final approved
document history therefore has eleven consecutive five-file iterations. The
already listed `fixed-probe/CMakeLists.txt`,
`fixed-probe/tests/test_probe.py`,
`tooling/a0_qualified.py`, `tooling/a0_tooling_approval.py`,
`tooling/a0_runtime_closure.py`,
`tooling/a0_repair_contract.py`,
`tooling/tests/a0_manifest_fixture.py`,
`tooling/tests/a0_tooling_approval_fixture.py`,
`tooling/tests/test_a0_corpus.py`,
`tooling/tests/test_a0_runtime_closure.py`,
`tooling/tests/test_a0_source_build.py`,
`tooling/tests/test_manifest.py`,
`tooling/tests/test_manifest_deep.py`, and
`tooling/tests/test_tooling_approval.py` may receive only the minimal literal
six-to-eleven document-cardinality, generated-scope, final revision-11
specification/plan identity, CTest RED/GREEN/child-observation producer,
fixture, validator, one-for-one required-ID replacement, and mutation
expectation updates needed to validate that immutable history. The production
and corpus-test document identity maps
bind the final revision-11 repository document hashes, not an intermediate
approved revision. This changes no static source/test path count, discovered
test count, required test count, or review schema.

Tests use real public validators and filesystem fixtures. Closed-set validation
may not be mocked. Development RED transcripts are retained separately, while
the fresh-build CTest RED is also bound in `build-result-v2.json`. The complete
tooling suite, fixed-probe CTest, source/LOC and static gates, and fresh build
evidence must pass before formal authorization. Every edited source or test
file remains below 400 physical lines.

## Approval gates and exit

Attempt 7 has three independent gates:

1. two document reviewers approve the same exact final spec/plan bytes;
2. after fresh build and input preparation, a fresh six-axis tooling reviewer
   approves the exact source ledger, full transcript, failure handoff,
   build/runtime evidence, candidate identity, and launch contract; and
3. after a successful 71-role campaign, two sidecar reviewers approve the same
   staged corpus/manifest bytes and a detached envelope verifies them.

Attempt 7 exits only when the failed candidate remains byte-identical, all
review gates approve, one fresh campaign passes 71/71 with no retry or residue,
approved staged bytes are published once, deep approved verification passes,
and Package A0 releases the separately reviewed adapter/engine work. Tracked
Rust production work remains blocked until then.
