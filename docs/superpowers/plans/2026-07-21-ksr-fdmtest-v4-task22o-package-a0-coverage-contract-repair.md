# Task 22O Package A0 Coverage Contract Repair Plan

## Objective

Implement only the repair amendment defined in
`docs/superpowers/specs/2026-07-21-ksr-fdmtest-v4-task22o-package-a0-coverage-contract-repair.md`.
Add one direct MedialAxis input that exercises validation branch 1, define and
test the reachable coverage contract, and implement the exact-copy
`corpus-v1.bin` container. Do not change the reviewed fixed Orca derivative,
the 71-process role plan, or tracked Rust production code.

The immutable parent A0 spec, plan, and document envelope remain SHA-256
`f40807bd9d891f8d38a7fb82bb2c2db74294ab67e38c02fd8e6a903224221200`,
`9f84c95dc9a2dbf4c55f4b3d381455921c13f40c180e50230dd76830310538f5`,
and `b234da531b0e1a9d9b681d059717a6c5fb564e57beefec257d4109665d894890`.

## Working rules

- Freeze this spec/plan pair and obtain two independent approvals before source
  or corpus edits.
- Preserve the fixed Orca commit, reviewed twelve-path status, Package 0 bytes,
  approved eleven-case identities, environment names, and process ordering.
- Treat the existing development runs and branch search as discovery evidence,
  never as formal qualification.
- Use RED then GREEN at the fixed-probe and ignored tooling public interfaces.
- Publish evidence create-once. Do not retry, overwrite, select a preferred run,
  or start formal qualification before tooling review approves.
- Keep every modified C++, Python, PowerShell, and Rust source below 400 physical
  lines. Do not use `include!` or `include_bytes!` to split Rust source.

## Exact ignored-path manifest amendment

The parent manifest remains exact. Relative to
`.superpowers/sdd/task22o-oracle/voronoi-a0/`, this repair authorizes only the
following additional authored source, test, review, and retained evidence paths:

```text
coverage-repair/direct-probe/green-test.txt
coverage-repair/direct-probe/green.orca22v
coverage-repair/direct-probe/green.stderr.txt
coverage-repair/direct-probe/green.stdout.txt
coverage-repair/direct-probe/red-test.txt
coverage-repair/document-reviews/approval-envelope.json
coverage-repair/document-reviews/corpus-qualification.md
coverage-repair/document-reviews/source-reachability.md
coverage-repair/exploration/branch1_search.cpp
coverage-repair/exploration/branch1_search.exe
coverage-repair/exploration/branch1_search.orca22v.bin
coverage-repair/exploration/branch1_search.stderr.txt
coverage-repair/exploration/branch1_search.stdout.txt
coverage-repair/exploration/discovery-result.json
coverage-repair/exploration/eleven-case-campaign-summary.identity.json
coverage-repair/exploration/eleven-case-campaign-summary.json
coverage-repair/exploration/eleven-case-coverage.json
coverage-repair/fixed-source/a0-package0-relative.patch
coverage-repair/fixed-source/source-files.sha256
coverage-repair/fixed-source/source-freeze-approval-envelope.json
coverage-repair/fixed-source/source-freeze-review.md
coverage-repair/fixed-source/source-status-v1.json
coverage-repair/tooling-review/approval-envelope.json
coverage-repair/tooling-review/mock-test-results.txt
coverage-repair/tooling-review/six-axis-review.md
coverage-repair/tooling-review/source-files.sha256
fixed-probe/CMakeLists.txt
fixed-probe/corpus/direct-medial-cases-v1.json
fixed-probe/corpus/fixed-source-inputs-v1.json
fixed-probe/evidence/post-review-final/run-1.orca22v
fixed-probe/evidence/post-review-final/run-2.orca22v
fixed-probe/src/fixed_cases.cpp
fixed-probe/src/fixed_cases.hpp
fixed-probe/src/fixed_cases_basic.cpp
fixed-probe/src/fixed_cases_edge_collapse.cpp
fixed-probe/src/fixed_cases_internal.hpp
fixed-probe/src/fixed_cases_regressions.cpp
fixed-probe/src/medial_cases.cpp
fixed-probe/src/medial_cases.hpp
fixed-probe/tests/test_probe.py
tooling/a0_cases.py
tooling/a0_common.py
tooling/a0_corpus.py
tooling/a0_corpus_container.py
tooling/a0_coverage.py
tooling/a0_qualified.py
tooling/a0_run_validation.py
tooling/a0_source_build.py
tooling/assemble_sidecar_manifest.py
tooling/hash_tree_no_follow.ps1
tooling/run_controls.psm1
tooling/run_fixed_qualification.ps1
tooling/run_io.psm1
tooling/run_one.ps1
tooling/run_postprocess.psm1
tooling/run_process.psm1
tooling/tests/fixtures.py
tooling/tests/mock_qualification_process.py
tooling/tests/qualification_fixture.py
tooling/tests/runner_fixtures.py
tooling/tests/test_a0_cases.py
tooling/tests/test_a0_corpus.py
tooling/tests/test_a0_corpus_container.py
tooling/tests/test_a0_coverage.py
tooling/tests/test_a0_source_build.py
tooling/tests/test_assembly.py
tooling/tests/test_qualification_contract.py
tooling/tests/test_run_catalog.py
tooling/tests/test_run_one.py
tooling/tests/test_runner_postprocess.py
tooling/tests/test_workspace_integrity.py
```

The exact tracked repair documents are:

```text
docs/superpowers/plans/2026-07-21-ksr-fdmtest-v4-task22o-package-a0-coverage-contract-repair.md
docs/superpowers/specs/2026-07-21-ksr-fdmtest-v4-task22o-package-a0-coverage-contract-repair.md
```

Add exactly those two paths to `ALLOWED_AMENDMENT_DOCUMENTS`; the final sidecar
manifest binds their frozen identities. The combined formal `CandidateRoot` is
exactly `runs/qualified/<candidate-id>/`, beneath the parent-authorized root.
Generated `fixed-probe/build/**`, `**/__pycache__/**`, formal out-of-tree
build/install trees, and children of parent-authorized run roots are not authored
manifest additions. Exactly seven current files, with no directory wildcard,
are excluded development residue:

```text
fixed-probe/evidence/final-smoke/run-1.orca22v
fixed-probe/evidence/final-smoke/run-2.orca22v
fixed-probe/evidence/post-review-final/run-1.stderr.log
fixed-probe/evidence/post-review-final/run-1.stdout.log
fixed-probe/evidence/post-review-final/run-2.stderr.log
fixed-probe/evidence/post-review-final/run-2.stdout.log
fixed-probe/runs/explore-1.orca22v
```

They are excluded from every subject while the two `.orca22v` files are
authorized frozen inputs. Add a closed-set test that accepts only the parent
manifest, the exact additions above, these exact exclusions, and generated
non-subject trees; every other authored path fails. No other authored path is
authorized.

## A0R.1: freeze and review the repair frame

1. Rehash the parent documents, envelope, Package 0 manifest/result, fixed Orca
   commit/tree, fixed Ares commit/tree, eleven-case development coverage, direct
   probe wire, and branch-search evidence.
2. Rehash the exact frozen A0 patch, source ledger, source status, source review,
   and source approval envelope at SHA-256
   `269841a4842970cb2046b048bece3fcf416b7230b25854a7051e1b35354ad5df`,
   `d85b2b35fd788f332a1a7e29ba7f94c9be8c085195f5e5016d21d8969a69c5c4`,
   `8e8cca81ba0494a0d0e6e853a8bd562d2fb676e282a72cdc93675114d362971a`,
   `f0bf0de7f3c9b56fab569424fb3d8445393a927c8039e55219525eeb921bec2f`,
   and `5fb414d7e09ea188ca78da54800a5a89fa662e3a31d779381255e391b7f9f9ef`.
   Verify the envelope binds the complete identities and `APPROVE`;
   independently regenerate the patch, ledger, and status without changing
   them.
3. Verify the exact ignored-path amendment and reject any unlisted authored
   source, test, review, or evidence path.
4. Freeze this spec/plan pair as one exact frame.
5. Request two fresh read-only reviews:
   - source/reachability review of validation branch 1, chaining action 3, the
     new exact input, and absence of a fixed-Orca source change;
   - corpus/qualification review of the binary grammar, predetermined run-1
     policy, parent binding, coverage minimums, and unchanged 71 roles.
6. Store detached reports at the exact three `coverage-repair/document-reviews/`
   paths named above. On any rejection, repair the
   documents, refreeze, and repeat both reviews before implementation.

Verification: exact SHA ledgers, literal `APPROVE` verdicts, and an envelope that
binds both document bytes and both report bytes. The document envelope also
binds the five frozen source-package subjects as immutable prerequisites.

## A0R.2: add the branch-1 direct case test first

Extend the fixed-probe test seam before production probe code:

1. change the direct-case contract RED from five to six ordered MedialAxis cases;
2. require the sixth input's exact width bits, contour, empty holes, first
   construction `0:0`, no closing, and at least one branch-1 decision with false
   accepted/active values;
3. parse the approved
   `b267e41b6788de1bf8d1dcba427f7a3ed57eaf3f0593e36a39452c9dd6470ed1`
   wire and require the first 16 amended record
   bodies, tags, and order to remain byte-identical, with exactly one appended
   `endpoint_epsilon_notch` record;
4. require aggregate validation branches `{0,1,2,3}`, aggregate chaining actions
   `{0,1,2}`, the existing closing transition, and no action-3 coverage claim;
5. run the focused probe test and retain the expected RED at
   `coverage-repair/direct-probe/red-test.txt` against the five-case probe.

Then minimally update `fixed-probe/src/medial_cases.hpp`,
`fixed-probe/src/medial_cases.cpp`, and
`fixed-probe/corpus/direct-medial-cases-v1.json` with the exact sixth case. Do
not edit the fixed Orca source or runtime wire. Rebuild only the development
probe, run it once to establish GREEN, and re-run parser corruption tests. Write
only the exact `green-test.txt`, `green.orca22v`, `green.stdout.txt`, and
`green.stderr.txt` retained development subjects named in the path manifest.

Verification: RED evidence, GREEN evidence, six exact MedialAxis cases in order,
the original 16 record bodies preserved, branch 1 present, action 3 absent,
closing transition preserved, strict EOF, and all modified source files below
400 LOC.

## A0R.3: make coverage validation explicit

At the ignored tooling public seam, add one synthetic aggregate fixture for each
missing minimum category and one complete fixture. REDs must prove rejection of:

- each missing record tag and each unresolved source-inventory claim;
- each missing construction pair or closing transition;
- missing cell, edge, validation, chaining, or ThickPolyline minimums;
- missing vertex annotation 0, 1, or 2, and action-0 coverage missing either
  `chosen_reversed` value;
- validation branch sets that omit 1;
- a checker that incorrectly requires unreachable action 3;
- unknown categories that are dropped instead of retained.

Add a valid synthetic action-3 wire record. Prove strict parse succeeds, the
comparator rejects an action-0/action-3 substitution, observation retains the
actual action-3 count, and the minimum validator still requires only actions
0, 1, and 2.

Implement the minimum-set validator from the repair spec. Load and rehash
`source-case-inventory-v1.json`; its ten `coverage_summary.required_claims` keys
are authoritative. Keep actual counts and additional valid categories in the
result. Do not manufacture branch records or normalize category IDs.

Verification: mutation-sensitive focused tests plus the complete ignored Python
and PowerShell suites.

## A0R.4: implement the exact-copy corpus container

Write REDs for bad outer magic/version/count, every truncation boundary,
identifier order or encoding drift, unknown kind, wrong hash/length, wrong inner
magic, inner trailing bytes, wrong `ARES22V` parent, changed approved
`ARES22O`, bad outer trailer, outer trailing bytes, missing run-2 equality, and
attempted overwrite.

For row `i`, compute the exact later reserve as the eight-byte trailer plus
`sum(2 + len(expected_literal_identifier[j]) + 1 + 32 + 8)` for every later row
`j`, with zero payload bytes. Before identifier access, reserve that sum plus the
current `u16`, known literal identifier, and `1 + 32 + 8` suffix, then require
the `u16` to equal the expected literal length. The boundary REDs include
identifier length `0xffff`, unequal literal length, and lengths that consume the
current suffix, even one byte of a later literal identifier, any later fixed
field, or the trailer. Also test `payload_length = 0xffffffffffffffff`, every
length larger than the bounded remainder, checked-offset overflow, invalid and
truncated UTF-8, cross-case parent/payload swaps, kind swaps, identifier
permutations, and substituting valid run 2 for predetermined run 1. No variable
length may cause allocation, copy, slicing, decoding, or hashing before its
complete bounded region is proved present.

Implement the smallest assembler/parser pair that:

1. reads the frozen qualification result in the canonical eleven-case order;
2. requires each qualified run-1/run-2 V and composite pair to be byte-equal;
3. enforces the repair spec's explicit twelve-row identifier/role/kind table and
   requires each kind-1 parent to equal that same case's run-1
   `expected_parent_ares22o`;
4. copies the eleven run-1 `ARES22V` composites and direct run-1 `ORCA22V`
   byte-for-byte under the specified 12-entry framing;
5. builds canonical `corpus-v1.json` from independently rehashed inputs;
6. assembles both subjects twice in memory and requires byte equality;
7. publishes each path create-once only after full validation.

Keep qualification orchestration in `a0_corpus.py`; put the new bounded binary
container implementation in the pre-authorized real module
`a0_corpus_container.py`, with focused tests in
`tests/test_a0_corpus_container.py`. Put semantic minimum-set tests in
`tests/test_a0_coverage.py`. No file may reach 400 physical LOC.

Do not parse/re-encode inner records while assembling the binary. Parsing is a
separate verification pass over each exact embedded payload.

Verification: all framing REDs/GREENs, an input-to-embedded-byte equality test
for every entry, deterministic two-pass bytes, and overwrite rejection.

At the runner boundary, add REDs before changing orchestration. The candidate ID
is exactly `sha256(raw source-files.sha256 bytes || installed executable
bytes)[:16]` in lowercase hexadecimal, and the only resolved root is
`.superpowers/sdd/task22o-oracle/voronoi-a0/runs/qualified/<candidate-id>`.
Wrong ID, sibling root, path alias, and pre-existing root must fail before root
creation or process execution with zero filesystem mutation. Implement the same
independent formula/root validation in the result collector and sidecar-manifest
assembler before either publishes any artifact. At each of the runner,
collector, and assembler boundaries, every wrong-ID/root/alias/pre-existing
case records byte-for-byte before/after filesystem snapshots and requires zero
mutation. Mutation tests prove neither downstream component trusts runner output.

## A0R.5: independent tooling review

After all mock tests pass, start a fresh read-only review turn. It inspects:

1. requirements completeness;
2. logical correctness;
3. boundary cases;
4. code quality and the 400-LOC rule;
5. test coverage and mutation sensitivity;
6. actual mock execution results.

The reviewer returns a concrete repair list. The main thread applies repairs,
reruns every affected test, and resubmits until the exact tooling bytes receive
`APPROVE`. Persist the complete mock output and exact tooling/test source ledger
as `coverage-repair/tooling-review/mock-test-results.txt` and
`source-files.sha256`; persist the final review and its detached envelope as
`six-axis-review.md` and `approval-envelope.json`. The envelope binds the exact
tooling ledger, test output, and literal verdict. No real formal Orca process
starts before that approval.

## A0R.6: fresh build and formal qualification

Create a brand-new detached worktree at fixed Orca commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Apply the approved Package 0 patch,
verify its seven source identities, then apply only the 62,560-byte A0 patch at
SHA-256 `269841a4842970cb2046b048bece3fcf416b7230b25854a7051e1b35354ad5df`.
Regenerate and require byte equality with the frozen twelve-path ledger
`d85b2b35fd788f332a1a7e29ba7f94c9be8c085195f5e5016d21d8969a69c5c4`
and status `8e8cca81ba0494a0d0e6e853a8bd562d2fb676e282a72cdc93675114d362971a`.
Any same-path byte drift fails before configure. Configure, build
`ALL_BUILD` plus the amended fixed probe, test the probe, and install into fresh
paths using the fixed dependency tree and toolchain. Do not use any development
binary or build directory.

After the build evidence validates, execute the frozen 71 roles once in order:
one env-off, one `o-empty-v-absent`, 22 payload-only, 23 invalid path controls,
22 qualified, and two direct probe processes. Stop and retain the first failure.
Never retry a role or overwrite an artifact. The single combined
`CandidateRoot` is exactly `runs/qualified/<candidate-id>/`, where the ID is
recomputed from the frozen source-ledger bytes concatenated with the installed
executable bytes; it must be absent before process 1 and all 71 leaves remain
beneath it. Runner, collector, and manifest checks must each pass independently.

Verification: exact source/build identity, all controls, exact approved O and
parent, paired V/composite equality, six-case direct wires containing branch 1,
no action-3 requirement, bounded process-tree exit, no temp/root-log residue,
and explicit datadir before/after evidence.

## A0R.7: corpus, manifest, and review

Aggregate the formal run-1 union and require every minimum semantic category.
Create `corpus-v1.bin` and `.json` twice in memory, require byte equality, and
publish once. Generate the sidecar manifest only after both corpus subjects and
all 71 leaf results verify.

Extend the manifest's closed groups to bind by exact path, bytes, and SHA-256
every non-excluded authored repair addition listed above. This explicitly
includes both repair documents, both document reviews and envelope, all five
source-package subjects, all nine retained `coverage-repair/exploration/`
subjects, all five retained `coverage-repair/direct-probe/` subjects, every
tooling-review subject, and every repaired tooling/test source. For every bound
subject, individual removal, substitution, and one-byte mutation tests fail.

Verify the exhaustive correspondence: parent-manifest paths remain
parent-bound; every authorized repair addition is final-manifest-bound;
generated build/cache trees are non-subjects containing no authorized authored
path; and only the exact seven listed files are excluded residue. Injecting any
additional authored file under the two former residue directories must fail.
Self-reference remains forbidden.

Resume the parent A0 two-review protocol without collapsing or weakening it:

- reviewer 1 checks fixed source, allowed diff, capture/wire, exact parent O,
  branch reachability, and the new direct input;
- reviewer 2 checks provenance, corpus framing, paired isolation, complete
  references/EOFs, coverage, and absence of normalization or run selection;
- a detached envelope binds the manifest and both approval reports.

Any finding returns to the main thread for repair and complete affected-gate
rerun. Only an approved sidecar manifest may unblock candidate engine selection.

## Focused commands

Use the bundled Python identity recorded by the parent A0 amendment. Minimum
development gates are:

```text
& $A0_PYTHON -m unittest discover -s .superpowers/sdd/task22o-oracle/voronoi-a0/tooling/tests -p "test_*.py"
ctest --test-dir <fresh-build> -C Release -R ares22o_voronoi_fixed_probe_contract --output-on-failure
& $A0_PYTHON .superpowers/sdd/task22o-oracle/voronoi-a0/tooling/verify_manifest.py .superpowers/sdd/task22o-oracle/voronoi-a0/sidecar-manifest-v1.json
```

PowerShell runner mock tests and static no-`$args`/LOC audits remain mandatory
even though their exact invocation is recorded by the approved tooling result.
