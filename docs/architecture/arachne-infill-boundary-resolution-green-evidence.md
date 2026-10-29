# Arachne infill-resolution prerequisite — GREEN evidence

Original combined-branch production commit: `36938b864` on main `96328f269`.
Mergeable prerequisite branch: `worker4/arachne-infill-resolution`, from the same
main, extracts only the public None test (`048cdb619`) and cherry-picks the
approved fix as `c33e781dd`. The eight non-None RED cases remain on
`worker4/arachne-counterbore` at `819f7e1a4`; none are registered on this branch.
The combined-branch results below are historical, not this branch's test verdict.
Approved SPEC: `cf123a796`, operator-adjusted two-file implementation.
Explicit clarification confirmed `config.scaled_resolution`, not a field on
ClassicPreludeRecord. No counterbore activation, guard assertion or comparator
changes. This is **prerequisite CODE REVIEW**, not FULL-READY/merge approval.

## Source and surgical change

`PerimeterGenerator.hpp:135` owns unreduced member resolution; Arachne
`PerimeterGenerator.cpp:2125,2148` owns the reduced wall-input value, while
`:2506` uses unreduced member resolution for infill contour simplification.

Changed only these production files:
- `crates/ares-core/src/project_slice/perimeters/arachne/config.rs` (167 LOC):
  preserve the already-computed `scaled_resolution` in `ArachneRecordConfig`.
- `crates/ares-core/src/project_slice/perimeters/arachne/materialize.rs` (269 LOC):
  pass `config.scaled_resolution` to the existing infill-boundary helper.

Diff: three insertions, one deletion. No repeated floor/scale logic, inverse
wall-factor calculation, new stage/API, regions[0], Unsupported, fallback or
Native substitution. No geometry/Fill, ArcFitter, W3/W5/W6 file edits.

## Directed complete-output verification

Root: `/tmp/worker4-arachne-resolution-green-QeDz`.
Launcher: `/tmp/worker4-arachne-resolution-green.sh`, outer `taskset -c 15`.
Independent W4 target retained from the paired diagnosis:
`/tmp/worker4-arc-paired-eyIt/target`.
Environment: jobs1, `ARES_PARITY_THREADS=1`, test-threads1, fresh mode-separated
Native cache/artifacts. Official then seeded sequential execution.
Filter: `test(arachne) | test(counterbore_)` on `ares-cli --test orca_parity`.
START/END were recorded in inbox. No full/workspace test run.

| Producer | Directed result | Runtime |
| --- | --- | --- |
| Official | 94 run: 86 PASS, 8 unchanged non-None guard RED | 124.529s |
| Seeded | 94 run: 86 PASS, 8 unchanged non-None guard RED | 151.579s |

Both commands exit100 because the separately frozen non-None activation tests
still expect eventual PASS and currently reject the unsupported feature. They
are **not** skipped/ignored/weakened or reported as GREEN. The eight failures
are exactly the prior Bridges/Filled primary, first-layer, straight-hole and
independent-two-island guard cases; no extra failure remains.
1086 tests are excluded by this targeted filter; no full coverage is claimed.
The additional 72 Arachne controls all PASS in each producer.

The existing Arachne None primary now PASSes **complete ordered bytes**, keeping
enabled arcs, fixture, profiles, placement, and strict generator-only comparison:
- official: `official/artifacts/orca-runner-7aRZki/counterbore-result.json`
- seeded: `seeded/artifacts/orca-runner-yNFZVh/counterbore-result.json`

Both Ares raw SHA:
`5f55844348571cd26b85fecde5bdc0ae8d8994b14bd569885bdeb0e6170da972`.
All 13 formerly PASS counterbore controls (11 classic partial/None/Filled and
2 Arachne solid cases) remain PASS with **unchanged raw Ares SHA** versus the
frozen RED run in each mode. Receipt:
`control-invariance-and-none-green.json` preserves all hashes and record paths.

Official setarch-R wrapper SHA:
`1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`.
Shared Native before/after unchanged:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.

## Gates and supplemental source witness

All exit0, logs in the directed root:
- `cargo clippy -p ares-cli --test orca_parity -- -D warnings`.
- `cargo clippy -p ares-core -- -D warnings`.
- `cargo fmt` (targeted existing RED file), `cargo fmt --check`; final fmt-check.
- `cargo check --target wasm32-unknown-unknown -p ares-core -p ares-wasm -p ares-vgcode`.
These are scoped compile/lint checks, not native Windows/macOS runtime tests.

Approved standing read-only GDB probe after production change:
`/tmp/worker4-resolution-probe-FHMW` with new artifacts, outer CPU15/jobs1/threads1.
The same public None test PASSes, and captured simplifier calls show:
- wall input contour4/hole4 tolerance2400 remains unchanged;
- first infill contour12/hole32 now tolerance12000;
- second infill contour4/hole52 now tolerance12000.
No debugger writes, set var/calls, source instrumentation or probe commit.
This source witness supplements, never substitutes for full-stream parity.

## Requested read-only classic audit

At the pinned upstream, `PerimeterGenerator.cpp:1655` is the `process_classic`
infill contour simplification using member `m_scaled_resolution`.
Rust `classic/infill_boundary/preflight.rs:36` computes
`resolution.max(EPSILON) / scale.factor()` without an arc/fuzzy0.2 reduction;
this flows into the validated overlap and
`classic/infill_boundary/geometry.rs:113` simplification. The classic path
already uses the full value. No classic change is needed or made.

Correction to the audit question: upstream `:1720` is inside
`add_infill_contour_for_arachne` (`:1695–1725`), **not process_classic**. That
helper also uses member `m_scaled_resolution`, but the pinned source has no
call site (only its definition/declaration). Ares has no independently reached
port of that unused helper; do not claim it has tested classic runtime coverage.

## Mergeable isolated-branch verification

Root: `/tmp/worker4-arachne-resolution-isolated-lnaB`.
Launcher: `/tmp/worker4-arachne-resolution-isolated.sh`, outer CPU15,
jobs1/test-threads1, same W4 independent target, fresh mode caches/artifacts.
Filter retained `test(arachne) | test(counterbore_)`.

| Producer | Result | Runtime | Exit |
| --- | --- | --- | --- |
| Official | 84/84 PASS, 0 failures | 115.068s | 0 |
| Seeded | 84/84 PASS, 0 failures | 144.974s | 0 |

The one newly added None regression PASSes complete ordered bytes in both modes;
72 existing Arachne controls and11 classic counterbore controls PASS. All11
classic Ares raw hashes match the frozen original RED controls. The eight
non-None REDs and their two new solid tests are not added/registered on this
branch, rather than ignored or altered; they remain on the counterbore branch.
1086 unrelated tests were excluded by the filter, not claimed tested.
`isolated-review-receipt.json` stores record paths, hashes and summaries.

None record paths relative to the isolated root:
- `official/artifacts/orca-runner-4XYB5R/counterbore-result.json`
- `seeded/artifacts/orca-runner-4GxBW8/counterbore-result.json`
Both retain Ares SHA `5f55844348571cd26b85fecde5bdc0ae8d8994b14bd569885bdeb0e6170da972`.

CLI-test/core strict clippy, fmt and core/wasm/vgcode WASM checks were rerun on
this isolated branch and all exit0. Shared Native SHA is unchanged. Public
helper/None assertions reconstruct exactly from the frozen original test after
reversing only the matrix-path relocation; rustfmt adds a single path line wrap.
No comparator/assertion weakening or production change beyond the approved
cherry-pick occurred. New matrix has only `none-primary`; its original geometry,
profile-chain hashes, effective overrides and expected full-output PASS remain.

This branch is ready for coordinator review and subsequent CI/exclusive-full
scheduling, **not yet FULL-READY or merge-approved**. The coordinator, not W4,
will push CI. The full matrix remains required to assess other Arachne arc cases.

## Residual scope

The original counterbore non-None slice still needs its approved shared geometry
handoffs and a separately approved exact diff for ten existing guard assertions.
Nested-island-in-hole and compatible-multiregion public reachability gaps remain
open. The old ArcFitter/Circle numeric proposal was not implemented. No FULL-GO,
full run, merge request or push is claimed by this prerequisite evidence.
