# Task 22O Package A0 Plan: Fixed Voronoi Sidecar Qualification

## Objective and gate

Implement only the detached qualification amendment defined in
`2026-07-21-ksr-fdmtest-v4-task22o-package-a0-voronoi-sidecar-amendment.md`.
Produce an immutable fixed-Orca MedialAxis/Voronoi sidecar, qualify its corpus
and controls, compare `boostvoronoi` 0.12.1 exactly on native, compile-check the
adapter for WASM, and freeze an approved selection or explicit rejection. Do
not edit tracked Rust or begin Package A geometry implementation in this plan.

The original Task 22O spec and plan remain byte-identical at SHA-256
`78c44972e284eb615bf96228cbc5d0fe3a5c731a853c3b1cf518f92219b95674`
and `94c361d0d4c89eb5019f07f3a3e4101b8d89857d02c06629e3c794920f645e80`.
Before A0 execution, freeze this amendment pair and obtain two independent
document approvals. Any `REJECT` returns a concrete repair list to the main
thread; refreeze and resubmit the same complete frame after repair.

## Working rules

Use TDD for ignored tooling: corruption and comparison-mutation REDs precede
exporter/adapter GREEN. Use `apply_patch` for manual source edits. All C++, Rust,
Python, PowerShell, and test source created for A0 remains below 400 physical
LOC and is split into real modules, never `include!` or `include_bytes!` source
fragments. Rust tests use Cargo Nextest, never `cargo test` as the default.

Every run is single-attempt, bounded, fresh-path, and archived. Never sort,
deduplicate, normalize, apply tolerances, select a preferred run, or derive
expected bytes from the KSR reference G-code. Keep the user's untracked
`main.obj` and all unrelated files untouched.

The Package 0 tree and its approved artifacts are read-only inputs. A0 uses a
new fixed worktree, build tree, run tree, and detached review reports. It never
regenerates the Package 0 manifest, KSR aggregate, verifier result, reviews, or
approval envelope.

## Ignored path manifest

All A0 implementation/evidence lives under this ignored tree and is never
staged:

```text
.superpowers/sdd/task22o-oracle/voronoi-a0/
  README.md
  WIRE.md
  document-reviews/fixed-source-seam.md
  document-reviews/ares-plan-platform.md
  document-reviews/approval-envelope.json
  corpus/source-case-inventory-v1.json
  corpus/corpus-v1.bin
  corpus/corpus-v1.json
  tooling/pack_voronoi.py
  tooling/parse_voronoi.py
  tooling/compare_voronoi.py
  tooling/verify_manifest.py
  tooling/tests/test_wire.py
  tooling/tests/test_compare.py
  tooling/tests/test_manifest.py
  fixed-source/instrumentation.patch
  fixed-source/source-files.sha256
  fixed-source/dependency-files.sha256
  fixed-source/build/configure.log
  fixed-source/build/build.log
  fixed-source/build/install.log
  fixed-probe/main.cpp
  runs/env-off/
  runs/payload-only/
  runs/sidecar-only/
  runs/qualified/
  runs/fixed-probe/
  boostvoronoi-probe/Cargo.toml
  boostvoronoi-probe/Cargo.lock
  boostvoronoi-probe/src/lib.rs
  boostvoronoi-probe/src/main.rs
  boostvoronoi-probe/tests/wire.rs
  boostvoronoi-probe/tests/corpus.rs
  sidecar-manifest-v1.json
  engine-selection-manifest-v1.json
  reviews/sidecar-fixed-source.md
  reviews/sidecar-qualification.md
  reviews/sidecar-approval-envelope.json
  reviews/engine-semantics.md
  reviews/engine-platform-license.md
  reviews/engine-approval-envelope.json
```

The out-of-tree fixed worktree and build locations are recorded by absolute
path and hash in the manifest but are not staged. No directory wildcard
authorizes another source or evidence path.

## Package A0.0: document freeze and preflight

Hash this amendment pair. Reviewer 1 independently reads fixed Git objects and
checks blobs, Boost archive identity, MedialAxis/PrintObject seams, nine-path
parent delta, twelve-path full status, activation guard, and capture fields.
Reviewer 2 checks the current Ares baseline, ignored-only boundary, corpus,
TDD/LOC/mod rules, two-stage immutability, dependency/license outcome, and
default/WASM gates. Both return literal `APPROVE` against the same exact hashes.
Freeze a detached document approval envelope; do not edit the approved docs or
reports afterward.

Before implementation, run the Package 0 verifier and verify its final manifest,
KSR aggregate, two reviews, and approval envelope hashes. Verify the fixed
commit/tree, every cited source blob, Boost 1.84 archive, candidate crate archive,
candidate VCS commit, Rust minimum, and license. Stop on any mismatch.

## Package A0.1: wire and comparator REDs

Freeze `WIRE.md` before exporter code. Define `ORCA22V` version 1 and `ARES22V`
exactly as the amendment specifies, including integer widths, float bits,
nullable sentinel, reference domains, nested counts, EOFs, and atomic-publish
rules.

Write ignored parser REDs for bad magic/version, every truncation boundary,
unknown record tag, variant-field substitution, missing inner or outer EOF,
trailing bytes, count overflow, out-of-range and inconsistent cell/edge/vertex
references, invalid twin/next/previous/rotation links, width-cardinality
mismatch, and parent-`ARES22O` mismatch.

Write comparator mutation REDs that independently detect cell reorder, directed
edge reorder, source-category change, one-bit float change, repair/closing-state
change, rotation-neighbor change, validation-decision change, endpoint-width
change, and ThickPolyline reorder. GREEN only the strict packer, recursive
parser, comparator, and self-tests; do not inspect fixed runs yet.

## Package A0.2: fixed derivative and build

Create a fresh detached worktree at fixed Orca commit `8500fcd...` and a fresh
out-of-tree build. Reapply the exact approved Package 0 patch. Verify its seven
source paths and parent hashes before adding A0.

Implement the sidecar with the exact nine-path delta:

1. extend the first-statement guard/runtime with the paired
   `ORCA22O_VORONOI_PATH` contract; normalize O/O-temp/V/V-temp and reject unless
   all four absolute paths are pairwise distinct and fresh;
2. resolve fixed object order before the perimeter parallel loop, preallocate
   object/layer slots, and add a thread-local RAII token inside the TBB worker
   lambda around each `m_layers[layer_idx]->make_perimeters()` call;
3. instrument only `Geometry/MedialAxis.cpp` at wrapper-return, annotation,
   validation, rotation/chaining, and return seams; and
4. add the four `Ares22OVoronoi*` capture/serialization files and register them
   in `src/libslic3r/CMakeLists.txt`.

Immediately after the unchanged `export_ares22o_payload(*this)` returns in
`PrintObject::make_perimeters()` and before `set_done(posPerimeters)`, invoke the
V finalizer. It aggregates exact object slots and writes only `ORCA22V` after
the last object and after verifying O final exists. It publishes through the
fresh V temp path and propagates failure before completion. Do not finalize in
the noexcept runtime destructor. Build `ARES22V` only in ignored tooling by
concatenating the exact parsed O and V subjects under the specified frame.

Do not edit `Voronoi.cpp/.hpp`, `VoronoiUtils.cpp/.hpp`, scheduler files, or any
other fixed source. Append MedialAxis invocations only to their preallocated
object/layer slot, then serialize object/layer/invocation order without sorting.
Do not serialize pointers or internal per-angle repair attempts. Keep all added
instrumentation files below 400 physical LOC.

Freeze the patch, nine-path parent diff, twelve-path fixed-commit status, parent
byte-equality ledger, fixed-source byte-equality ledger, configure/build/install
logs, compiler/toolchain identities, derivative executable/DLL identities, and
the exact Package 0 parent manifest/source-manifest identities.

## Package A0.3: fixed direct corpus

Create `fixed-probe/main.cpp` outside the derivative source tree. It links the
derivative libslic3r and explicitly enters a probe-only record session. Its raw
point and raw segment variants call unchanged Boost constructors; its wrapped
segment variant calls the unchanged Orca `VoronoiDiagram` constructor; only its
direct MedialAxis variant enters the same layer token used by slicer workers.
All four tagged variants use the production sidecar serializer and atomic
publisher. The probe may not mutate environment state after startup or bypass
the CLI guard for normal slicer runs.

Freeze direct inputs from fixed `test_voronoi.cpp` for raw point, raw/open
segment, edge-collapse, duplicate-vertex, intersecting-edge, missing-vertex, and
wrapper-repair behavior. Add MedialAxis rectangle, concave/T-junction, one-hole,
two-hole, and near-degenerate closing cases. Record exact coordinates and width
bits only in the ignored corpus. Run the strict tagged-record parser and report
actual constructor, repair, branch, and category coverage before qualification.

Before copying coordinates or running the probe, freeze
`corpus/source-case-inventory-v1.json`. For every selected constructor call it
records the exact fixed `TEST_CASE` name, fixed source blob and line range,
ordered input variable/range, raw-points/raw-segments/wrapped-segments tag, and
coverage reason. It also lists every inspected but deferred call with a reason.
Verify that ordinary, hole, multiple-hole, edge-collapse, duplicate,
intersecting, missing-vertex, and repair claims each resolve to one or more exact
entries. The sidecar manifest binds this inventory and its extraction audit.

## Package A0.4: controls and paired qualification

Use a bounded supervisor and fresh absolute targets for every process. Archive
the first attempt, root stdout/stderr, command/environment ledger, hashed fresh
datadir clone, output hashes, parser result, process tree, and residue check.
Never retry or overwrite.

Run in order:

1. env-off control: both variables absent; require successful G-code and no
   O/V/temp;
2. payload-only control: two processes each for KSR and A1-A7/B1/C1/D1; require
   exact approved per-case O bytes and no V;
3. path controls: sidecar-only plus each empty/relative/existing and every
   O/O-temp/V/V-temp alias class; require first-statement rejection and no
   G-code/O/V/temp mutation;
4. qualified mode: two processes each for the same eleven inputs; require exact
   approved O, byte-identical V/composite pairs, complete recursive parse,
   parent binding, EOFs, index consistency, no temp, and no residual process;
5. fixed direct probe: two processes; require byte-identical complete V wires.

A4 and A6 zero-call results are retained but cannot satisfy topology coverage.
Any O mismatch invalidates the derivative. Any V difference, parser failure,
coverage gap, timeout, residue, or nonzero unexpected exit blocks the package.

## Package A0.5: sidecar manifest and review

After all prior gates pass, generate `sidecar-manifest-v1.json` deterministically
twice in memory and require byte equality. It binds document approvals, parent
Package 0 artifacts, runtime/tool identities, dependency identities, source
ledgers, patch/build outputs, complete corpus, every control/run output, coverage,
wire/tool hashes, parser/comparator tests, and absence of retries/residue.
Do not include its own hash or a mutable review verdict in the reviewed subject.

Freeze it once. Reviewer 1 checks fixed source, allowed diff, guard/token/capture,
wire, and exact parent O preservation. Reviewer 2 checks provenance, corpus,
two-run isolation, complete references/EOFs, coverage, and no normalization or
selection. Both inspect the same bytes and return literal `APPROVE`. Freeze a
detached approval envelope that binds the manifest and both report hashes.
Never mutate the subject or reports after review.

## Package A0.6: candidate adapter RED/GREEN

Only after A0.5 approval, create the ignored `boostvoronoi-probe` pinned to exact
0.12.1 archive `077839...`. Its manifest contains a local `[workspace]` so Cargo
cannot attach it to the root workspace, and pins `boostvoronoi = "=0.12.1"`.
Freeze its own lockfile before execution and use only its own explicit target
directory. First write adapter REDs from the approved sidecar corpus. Compare
every accessible native-order field and exact bit named by the specification;
no graph canonicalization, tolerance, deduplication, or reordered comparison is
allowed.

Implement only enough ignored adapter code to run the exact comparison. Run
Nextest, default `cargo check`, and `cargo check --target
wasm32-unknown-unknown` with the Tier-1 Rust 1.91.0 toolchain. Record and verify
the exact rustc/Cargo executable identities and versions, the 1.91.0 WASM target,
exact lock and transitive dependency graph/checksums/licenses, crate archive,
VCS commit, adapter source hashes, results, and every unmatched or inaccessible
field. WASM is a compile qualification in A0; browser execution remains Package
H of the parent plan.

If any semantic or platform gate fails, stop with `selected_engine: null`. Do
not try another crate, port a fixed Boost subset, edit workspace Cargo/lock or
notices, or create tracked REDs under this amendment.

## Package A0.7: engine manifest and review

Generate `engine-selection-manifest-v1.json` deterministically twice and require
byte equality. It references the exact approved sidecar manifest and envelope,
candidate identity, adapter and corpus hashes, complete exact-comparison result,
default/WASM result, license/Rust minimum, and either the selected engine or
`null` with the first blocking reason.

Freeze it once. Two new review turns, distinct from A0.5, inspect the same bytes.
Reviewer 1 verifies semantic field/bit completeness and every mismatch result.
Reviewer 2 verifies native/WASM builds, dependency provenance, BSL-1.0 handling,
tool/runtime identity, and absence of normalization or unapproved fallback.
Freeze their reports and detached approval envelope without mutating the engine
manifest.

## Package A0.8: Package A handoff

If the approved engine manifest selects `boostvoronoi` 0.12.1, return its exact
manifest/envelope identities to the main Task 22O plan. Package A must still
freeze a separate tracked leaf manifest and write ordinary behavioral REDs
without fixed source IDs, ignored paths, pointer indices, or oracle tooling
dependencies before adding the dependency, adapter, lock update, and BSL notice.

If the approved result is `selected_engine: null`, stop. Return the exact first
failure and repair/coverage evidence to the main thread. A further approved doc
amendment is required before any fixed Boost subset or alternate path begins.

## Focused commands

Record the exact `$A0_PYTHON` absolute path, size, SHA-256, and version in both
manifests. The minimum focused gates are:

```text
& $A0_PYTHON -m unittest discover -s .superpowers/sdd/task22o-oracle/voronoi-a0/tooling/tests -p "test_*.py"
cargo +1.91.0 nextest run --manifest-path .superpowers/sdd/task22o-oracle/voronoi-a0/boostvoronoi-probe/Cargo.toml --locked --target-dir .superpowers/sdd/task22o-oracle/voronoi-a0/boostvoronoi-probe/target
cargo +1.91.0 check --manifest-path .superpowers/sdd/task22o-oracle/voronoi-a0/boostvoronoi-probe/Cargo.toml --locked --target-dir .superpowers/sdd/task22o-oracle/voronoi-a0/boostvoronoi-probe/target
cargo +1.91.0 check --manifest-path .superpowers/sdd/task22o-oracle/voronoi-a0/boostvoronoi-probe/Cargo.toml --locked --target-dir .superpowers/sdd/task22o-oracle/voronoi-a0/boostvoronoi-probe/target --target wasm32-unknown-unknown
& $A0_PYTHON .superpowers/sdd/task22o-oracle/voronoi-a0/tooling/verify_manifest.py .superpowers/sdd/task22o-oracle/voronoi-a0/sidecar-manifest-v1.json
& $A0_PYTHON .superpowers/sdd/task22o-oracle/voronoi-a0/tooling/verify_manifest.py .superpowers/sdd/task22o-oracle/voronoi-a0/engine-selection-manifest-v1.json
```

The manifest verifiers rehash every referenced file, regenerate each manifest
twice in memory, validate every mandatory group and count, and reject self-
reference, post-review mutation, missing reports, non-APPROVE verdicts, trailing
bytes, residue, retry evidence, root Cargo/lock drift, target output outside the
probe tree, a compiler other than the recorded Tier-1 Rust 1.91.0 toolchain, or
tracked A0 implementation changes.

## Completion and review loop

After each immutable stage, independent reviewers return findings grouped by
requirements completeness, logical correctness, boundary cases, code quality,
test coverage, and actual run evidence. Reviewers are read-only. The main thread
applies the repair list, reruns all affected gates, freezes a new subject, and
requests fresh review. Continue until both A0 stages approve or the exact fixed
source/candidate/platform blocker is documented with `selected_engine: null`.
